//! app.lock 동시성 제어 (ADR-002, PRD §5.3).
//!
//! 양 PC(Windows 교습소 + macOS 자택) 시점 분리 사용을 강제하기 위해 클라우드 동기화 폴더에
//! `app.lock` JSON 파일을 두고 heartbeat 메커니즘으로 점유 상태를 관리한다.
//!
//! ## 흐름
//!
//! 1. `acquire_lock(force=false)`: 락 파일 없음 또는 우리 점유면 성공 — heartbeat 갱신.
//!    다른 디바이스 점유 중이면 실패 (5분 stale 시 force=true 로 강제 점유 가능).
//! 2. `check_lock_status()`: 현재 락 상태 (Free / OwnedBySelf / OwnedByOther{stale}) 반환.
//! 3. `release_lock()`: 우리 점유일 때만 파일 삭제 (다른 디바이스 락 보호).
//!
//! heartbeat 백그라운드 task 통합은 T10 시작 시퀀스에서 추가된다.
//!
//! ## 락 파일 위치
//!
//! - T6 (현재): `./SmartHB-data/app.lock` 임시 위치 (dev).
//! - T9 (마법사 통합): 클라우드 동기화 폴더 하위 `smarthb/app.lock` 으로 이전.

use crate::app_err;
use crate::commands::audit::{self, AuditEventType};
use crate::commands::paths;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use uuid::Uuid;

/// 락 파일명.
const LOCK_FILENAME: &str = "app.lock";

/// 5분 미갱신 시 strale 판정 (PRD §5.3).
const STALE_THRESHOLD_SECONDS: i64 = 300;

/// 본 디바이스의 고유 ID — 앱 프로세스 시작 시 1회 OsRng UUIDv4 생성.
///
/// MAC 주소나 하드웨어 시리얼 사용 금지 (PRD §5.3 보안 정책). 프로세스 재시작 시 새 ID 가
/// 생성되므로, 동일 PC 의 두 SmartHB 인스턴스는 서로 다른 디바이스로 인식된다 (단일 사용자
/// 모델이라 발생하지 않을 시나리오).
fn device_id() -> Uuid {
    static DEVICE_ID: OnceLock<Uuid> = OnceLock::new();
    *DEVICE_ID.get_or_init(Uuid::new_v4)
}

/// 락 파일 본문 — JSON 직렬화.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct LockInfo {
    device_id: Uuid,
    last_heartbeat: DateTime<Utc>,
}

impl LockInfo {
    fn new_for_self() -> Self {
        Self {
            device_id: device_id(),
            last_heartbeat: Utc::now(),
        }
    }

    fn seconds_since_heartbeat(&self) -> i64 {
        (Utc::now() - self.last_heartbeat).num_seconds()
    }

    fn is_stale(&self) -> bool {
        self.seconds_since_heartbeat() >= STALE_THRESHOLD_SECONDS
    }

    fn is_self(&self) -> bool {
        self.device_id == device_id()
    }
}

/// 현재 락 상태 — IPC 응답.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum LockStatus {
    /// 락 파일 없음 또는 비어있음.
    Free,
    /// 본 디바이스가 점유 중.
    OwnedBySelf {
        last_heartbeat_seconds_ago: i64,
    },
    /// 다른 디바이스가 점유 중.
    ///
    /// `stale=true` 면 5분 이상 heartbeat 미갱신 — 사용자에게 강제 점유 옵션 제공.
    OwnedByOther {
        stale: bool,
        last_heartbeat_seconds_ago: i64,
    },
}

/// 락 파일 경로 — `paths::data_root()` 와 단일 데이터 루트 공유. sync 모듈이 mtime 감시에 재사용.
pub(crate) fn lock_path() -> PathBuf {
    paths::data_root().join(LOCK_FILENAME)
}

/// 락 파일 디렉토리를 보장한다 (없으면 생성). heartbeat 호출 시 idempotent.
fn ensure_lock_dir() -> Result<(), AppError> {
    if let Some(parent) = lock_path().parent() {
        std::fs::create_dir_all(parent).map_err(|e| app_err!(Lock, "락 디렉토리 생성 실패", e))?;
    }
    Ok(())
}

/// 락 파일 내용을 파싱한다. 빈 문자열이면 `None`.
fn parse_lock_info(content: &str) -> Result<Option<LockInfo>, AppError> {
    if content.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(content)
        .map(Some)
        .map_err(|e| app_err!(Lock, "락 파일 파싱 실패", e))
}

/// 락 파일을 (열고 → 읽고) 한 번에 수행하는 read-only 헬퍼.
///
/// `check_lock_status` 처럼 단순 조회 시 사용. acquire 흐름은 별도 atomic 함수로 분리.
fn read_lock_info() -> Result<Option<LockInfo>, AppError> {
    let path = lock_path();
    if !path.exists() {
        return Ok(None);
    }
    let mut file = File::open(&path).map_err(|e| app_err!(Lock, "락 파일 열기 실패", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| app_err!(Lock, "락 파일 읽기 실패", e))?;
    parse_lock_info(&content)
}

/// **atomic acquire** — fs2 advisory lock 을 보유한 채 read → 판정 → write 를 단일 파일
/// 핸들에서 수행하여 read/write 사이 race window 를 제거한다.
///
/// 동작:
/// 1. `OpenOptions::create+read+write` 로 파일 열기 (truncate 안 함 — 기존 내용 보존)
/// 2. `try_lock_exclusive()` — 다른 프로세스가 이미 락 보유 중이면 즉시 실패
/// 3. 파일 내용 읽기 → `LockInfo` 파싱
/// 4. force/self/other 판정 → 점유 가능하면 새 `LockInfo` 직렬화 + 파일 truncate + write
/// 5. 함수 종료 시 file drop → fs2 락 자동 해제
///
/// fs2 락 보유 구간이 read→판정→write 전체를 감싸므로 동시 acquire 가 직렬화된다.
pub(crate) fn acquire_lock_atomic(force: bool) -> Result<(), AppError> {
    ensure_lock_dir()?;
    let path = lock_path();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)
        .map_err(|e| app_err!(Lock, "락 파일 생성 실패", e))?;
    file.try_lock_exclusive()
        .map_err(|e| app_err!(Lock, "락 획득 실패 — 다른 프로세스가 락 파일 점유 중", e))?;

    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| app_err!(Lock, "락 파일 읽기 실패", e))?;
    let current = parse_lock_info(&content)?;

    match current {
        None => {}
        Some(info) if info.is_self() => {}
        Some(info) if force && info.is_stale() => {}
        Some(info) => {
            return Err(AppError::Lock(format!(
                "다른 컴퓨터에서 사용 중입니다. (마지막 활동: {}초 전)",
                info.seconds_since_heartbeat()
            )));
        }
    }

    let new_info = LockInfo::new_for_self();
    let json = serde_json::to_string_pretty(&new_info)
        .map_err(|e| app_err!(Lock, "락 JSON 직렬화 실패", e))?;
    file.set_len(0).map_err(|e| app_err!(Lock, "락 파일 truncate 실패", e))?;
    file.seek(SeekFrom::Start(0)).map_err(|e| app_err!(Lock, "락 파일 seek 실패", e))?;
    file.write_all(json.as_bytes()).map_err(|e| app_err!(Lock, "락 파일 쓰기 실패", e))?;
    Ok(())
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// 현재 락 상태를 반환한다.
#[tauri::command]
pub async fn check_lock_status() -> Result<LockStatus, String> {
    let info = read_lock_info().map_err(String::from)?;
    let status = match info {
        None => LockStatus::Free,
        Some(info) if info.is_self() => LockStatus::OwnedBySelf {
            last_heartbeat_seconds_ago: info.seconds_since_heartbeat(),
        },
        Some(info) => LockStatus::OwnedByOther {
            stale: info.is_stale(),
            last_heartbeat_seconds_ago: info.seconds_since_heartbeat(),
        },
    };
    Ok(status)
}

/// 락을 획득한다.
///
/// `force=false`: 락 파일 없음 또는 본 디바이스 점유면 성공. 다른 디바이스 점유 중이면 실패.
/// `force=true`: stale(5분 미갱신) 락만 강제 점유 가능 — 정상 동작 중인 다른 디바이스 락은
/// 보호한다. UI 가 사전에 stale 여부를 사용자에게 확인 후 force=true 호출.
///
/// `acquire_lock_atomic` 가 fs2 advisory lock 보유 중 read→판정→write 를 수행하므로
/// 동시 acquire race 가 직렬화된다.
#[tauri::command]
pub async fn acquire_lock(force: bool) -> Result<(), String> {
    acquire_lock_atomic(force).map_err(String::from)?;
    // force=true 호출은 사용자가 명시적으로 강제 점유를 결정한 시점 — 사실로 기록.
    // pool 미초기화 (startup 전) 일 수 있으므로 try_record (silent fail).
    if force {
        audit::try_record(AuditEventType::LockForced, None, None).await;
    }
    Ok(())
}

/// 백그라운드 heartbeat task — 60초마다 호출되어 mtime 갱신.
///
/// `acquire_lock_atomic(false)` 는 본 디바이스 점유 중일 때도 성공하므로 heartbeat 효과와 동등.
/// 실패는 stderr 로만 기록 — 백그라운드 task 가 panic 으로 죽지 않도록 한다.
pub(crate) async fn heartbeat_tick() {
    if let Err(e) = tokio::task::spawn_blocking(|| acquire_lock_atomic(false))
        .await
        .map_err(|e| app_err!(Lock, "heartbeat 작업 실패", e))
        .and_then(|r| r)
    {
        eprintln!("[lock] heartbeat 실패 (재시도 60초 후): {}", e);
    }
}

/// 락을 해제한다 (본 디바이스 점유일 때만, T11 R7 advisory lock 적용).
///
/// `acquire_lock_atomic` 와 동일한 advisory lock 보호 수준을 제공한다 — fs2 의
/// `try_lock_exclusive` 로 다른 프로세스가 동시에 락 파일을 조작하지 못하게 한 상태에서
/// 본 디바이스 점유 여부를 재확인하고 삭제한다. 다른 디바이스 점유 락은 보호.
#[tauri::command]
pub async fn release_lock() -> Result<(), String> {
    release_lock_atomic().map_err(String::from)
}

fn release_lock_atomic() -> Result<(), AppError> {
    let path = lock_path();
    // 락 파일이 없으면 이미 해제 상태로 간주 — idempotent.
    if !path.exists() {
        return Ok(());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(|e| app_err!(Lock, "락 파일 열기 실패", e))?;
    file.try_lock_exclusive()
        .map_err(|e| app_err!(Lock, "락 해제 직전 advisory lock 획득 실패", e))?;

    // advisory lock 보유 상태에서 본 디바이스 점유 여부 재확인 후 삭제.
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| app_err!(Lock, "락 파일 읽기 실패", e))?;
    let current = parse_lock_info(&content)?;
    match current {
        None => {}
        Some(info) if info.is_self() => {}
        Some(_) => {
            return Err(AppError::Lock(
                "다른 디바이스가 점유한 락은 해제할 수 없습니다.".to_string(),
            ));
        }
    }
    // file drop 으로 advisory lock 자동 해제 + Windows file handle close.
    // close 후 remove_file 호출 — Windows 에서 열린 핸들은 삭제 실패 원인.
    drop(file);
    std::fs::remove_file(&path).map_err(|e| app_err!(Lock, "락 파일 삭제 실패", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 단위 테스트는 LOCK_DIR_DEV 를 공유하므로 통합 테스트 모듈은 직렬 실행 필요.
    /// 본 테스트는 LockInfo 의 순수 로직만 검증한다 (파일 I/O 제외).

    #[test]
    fn lock_info_is_self_when_device_id_matches() {
        let info = LockInfo::new_for_self();
        assert!(info.is_self());
    }

    #[test]
    fn lock_info_is_not_self_for_different_device() {
        let info = LockInfo {
            device_id: Uuid::new_v4(),
            last_heartbeat: Utc::now(),
        };
        assert!(!info.is_self(), "랜덤 UUID 는 본 디바이스 ID 와 충돌 확률 거의 0");
    }

    #[test]
    fn lock_info_is_stale_after_threshold() {
        let info = LockInfo {
            device_id: Uuid::new_v4(),
            last_heartbeat: Utc::now() - chrono::Duration::seconds(STALE_THRESHOLD_SECONDS + 1),
        };
        assert!(info.is_stale());
        assert!(info.seconds_since_heartbeat() >= STALE_THRESHOLD_SECONDS);
    }

    #[test]
    fn lock_info_is_fresh_within_threshold() {
        let info = LockInfo {
            device_id: Uuid::new_v4(),
            last_heartbeat: Utc::now() - chrono::Duration::seconds(STALE_THRESHOLD_SECONDS - 60),
        };
        assert!(!info.is_stale());
    }

    #[test]
    fn lock_info_serializes_to_json() {
        let info = LockInfo {
            device_id: Uuid::parse_str("12345678-1234-1234-1234-123456789abc").unwrap(),
            last_heartbeat: Utc::now(),
        };
        let json = serde_json::to_string(&info).expect("직렬화 성공");
        assert!(json.contains("device_id"));
        assert!(json.contains("last_heartbeat"));
        let parsed: LockInfo = serde_json::from_str(&json).expect("역직렬화 성공");
        assert_eq!(parsed.device_id, info.device_id);
    }

    #[test]
    fn device_id_is_stable_within_process() {
        let id1 = device_id();
        let id2 = device_id();
        assert_eq!(id1, id2, "OnceLock 으로 1회 생성된 ID 가 stable 해야 함");
    }

    #[test]
    fn release_lock_atomic_is_idempotent_when_no_file() {
        // 락 파일이 없는 상태에서도 release 가 성공해야 함 (idempotent).
        let path = lock_path();
        let _ = std::fs::remove_file(&path);
        assert!(release_lock_atomic().is_ok());
    }

    #[test]
    fn release_lock_atomic_removes_self_owned_lock() {
        // acquire 직후 즉시 release — 다른 테스트와 lock_path 를 공유하므로 path.exists()
        // 의존성 없이 두 호출의 결과만 검증한다.
        let acquired = acquire_lock_atomic(false);
        if acquired.is_err() {
            return; // 외부 점유 — 본 테스트 skip
        }
        let result = release_lock_atomic();
        assert!(result.is_ok(), "본 디바이스 점유 락 release 성공: {:?}", result);
    }

    #[tokio::test]
    async fn heartbeat_tick_does_not_panic_on_success() {
        // 본 디바이스 락 점유 후 heartbeat 호출 — 락 디렉토리는 SmartHB-data 공유.
        // acquire_lock_atomic 이 idempotent 하므로 결과적으로 mtime 만 갱신.
        let acquired = acquire_lock_atomic(false);
        if acquired.is_err() {
            // 다른 테스트 또는 외부 프로세스가 점유 중 — heartbeat 도 동일하게 거부될 것.
            // 본 테스트는 panic 없는 동작만 검증하므로 skip.
            return;
        }
        // heartbeat 호출이 panic 없이 완료되어야 한다.
        heartbeat_tick().await;
        heartbeat_tick().await; // 2회 호출도 안전 (재진입)

        // 정리 — 다른 테스트와 격리.
        let _ = std::fs::remove_file(lock_path());
    }
}
