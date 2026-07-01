//! app.lock 동시성 제어 (ADR-002, PRD §5.3).
//!
//! 양 PC(Windows 교습소 + macOS 자택) 시점 분리 사용을 강제하기 위해 클라우드 동기화 폴더에
//! `app.lock` JSON 파일을 두고 heartbeat 메커니즘으로 점유 상태를 관리한다.
//!
//! ## 흐름
//!
//! 1. `acquire_lock(force=false)`: 락 파일 없음 / 본 디바이스 점유 / **stale 락(5분 미갱신)
//!    자동 점유** 시 성공. 다른 디바이스가 fresh 락 점유 중이면 실패.
//!    `force=true` 는 fresh 락도 강제 점유 (사용자가 의식적으로 다른 PC 사용 차단할 때).
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
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// 락 파일명.
const LOCK_FILENAME: &str = "app.lock";

/// 24시간 미갱신 시 stale 판정 — heartbeat 제거(Sprint 17) 후 비정상 종료 기준을 24h로 완화.
const STALE_THRESHOLD_SECONDS: i64 = 86400;

/// device.id 파일 경로 — `lib.rs::setup` 에서 OS `app_config_dir/device.id` 로 1회 초기화.
///
/// Sprint 7 T3 (R37): 클라우드 동기화 폴더가 아닌 OS 로컬 경로 — 양 PC 가 각자 다른 device.id 를
/// 보유해야 stale lock 자동 점유가 "본 디바이스" 락을 올바르게 식별한다. 클라우드 폴더에 두면
/// 양 PC 가 동일 UUID 로 sync 되어 식별 불가.
static DEVICE_ID_PATH: OnceLock<PathBuf> = OnceLock::new();

/// `lib.rs::setup` 에서 1회 호출 — `{app_config_dir}/device.id` 경로를 모듈에 전달.
///
/// 미설정 상태에서 `device_id()` 가 호출되면 메모리-only fallback (임시 UUID v4). 테스트 환경에서
/// 발생할 수 있는 분기로, production 에서는 setup 이 정상 실행되어 항상 set 된다.
pub fn init_device_id_path(path: PathBuf) {
    let _ = DEVICE_ID_PATH.set(path);
}

/// 본 디바이스의 고유 ID — 앱 프로세스 시작 시 파일에서 로드 또는 신규 생성 후 영속화.
///
/// Sprint 7 T3 (Issue 8): 매 프로세스 새 UUID 생성 → 파일 영속화로 변경. 정상 종료 → 재시작
/// 시 동일 ID 유지, 비정상 종료 후 stale lock 자동 점유가 "본 디바이스" 락으로 올바르게 판정.
///
/// MAC 주소/하드웨어 시리얼 사용 금지 (PRD §5.3 보안 정책). 파일 손상 (UUID 파싱 실패) 시 새
/// UUID 재생성 + 파일 재기록 (graceful fallback). 부재 시 새 UUID 생성 후 atomic write.
fn device_id() -> Uuid {
    static DEVICE_ID: OnceLock<Uuid> = OnceLock::new();
    *DEVICE_ID.get_or_init(load_or_create_device_id)
}

fn load_or_create_device_id() -> Uuid {
    let Some(path) = DEVICE_ID_PATH.get() else {
        // setup 진입 전 호출 (테스트 환경 등) — 메모리-only fallback.
        return Uuid::new_v4();
    };
    load_or_create_device_id_at(path)
}

fn load_or_create_device_id_at(path: &Path) -> Uuid {
    match std::fs::read_to_string(path) {
        Ok(content) => match Uuid::parse_str(content.trim()) {
            Ok(uuid) => uuid,
            Err(_) => {
                eprintln!(
                    "[lock] device.id 손상 감지 ({} 바이트). 새 UUID 재생성.",
                    content.len()
                );
                regenerate_device_id(path)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => regenerate_device_id(path),
        Err(e) => {
            eprintln!("[lock] device.id 읽기 실패 ({}): {} — 메모리-only fallback", path.display(), e);
            Uuid::new_v4()
        }
    }
}

/// 새 UUID 를 생성하여 파일에 atomic 저장 — 실패 시에도 메모리 UUID 반환 (graceful).
///
/// T2 `store_salt_to` 패턴 답습: tmp → rename + sync_all + 부모 디렉토리 best-effort fsync.
fn regenerate_device_id(path: &Path) -> Uuid {
    let new_id = Uuid::new_v4();
    if let Err(e) = write_device_id_atomic(path, &new_id) {
        eprintln!(
            "[lock] device.id 파일 저장 실패 ({}): {} — 본 프로세스는 메모리 UUID 사용",
            path.display(),
            e
        );
    }
    new_id
}

fn write_device_id_atomic(path: &Path, uuid: &Uuid) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("id.tmp");
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)?;
        use std::io::Write as _;
        f.write_all(uuid.to_string().as_bytes())?;
        f.sync_all()?;
    }
    // Sprint 8 T8 (R48-a / I-S2-10): 소유자 read/write 만 허용 (0o600). Unix 전용 — Windows 는
    // NTFS ACL 모델이라 별도 처리 불필요. set_permissions 실패는 best-effort 로 무시 (파일
    // 자체는 정상 저장된 후의 보강 단계).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Some(parent) = path.parent() {
        if let Ok(dir) = std::fs::File::open(parent) {
            let _ = dir.sync_all();
        }
    }
    Ok(())
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
    OwnedBySelf { last_heartbeat_seconds_ago: i64 },
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

/// 락 파일 내용을 파싱한다.
///
/// **손상 복구 정책 (2026-05-21 사고 대응)**: NTFS power-loss 패턴으로 락 파일이 NULL
/// 바이트 전체로 손상되는 사고가 setup.rs 와 동일하게 발생한다 (실측: 111 바이트 0x00).
/// `String::trim()` 은 NULL(`\0`) 을 공백으로 인식하지 않아 빈 문자열 분기를 통과시키고,
/// 파싱 실패가 `AppError::Lock` 으로 wrap 되어 사용자에게는 "다른 컴퓨터에서 사용 중" 메시지가
/// 잘못 표시된다. 본 함수는 손상 패턴을 감지하면 `Ok(None)` 을 반환한다 — 호출자
/// `acquire_lock_atomic` 가 truncate + write 로 즉시 새 락을 작성하므로 자동 복구된다.
fn parse_lock_info(content: &str) -> Result<Option<LockInfo>, AppError> {
    if is_lock_corrupted(content) {
        eprintln!("[lock] app.lock 손상 감지 (빈 파일 또는 all-zero). free 로 fallback.");
        return Ok(None);
    }
    match serde_json::from_str(content) {
        Ok(info) => Ok(Some(info)),
        Err(e) => {
            eprintln!(
                "[lock] app.lock 파싱 실패 (손상 의심, free 로 fallback): {}",
                e
            );
            Ok(None)
        }
    }
}

/// 빈 문자열 또는 NULL 바이트만 있는 락 파일을 손상으로 간주한다.
///
/// `String::trim()` 은 NULL 을 공백으로 보지 않으므로 별도 검사가 필요. JSON 파싱 단계 전 빠른 컷.
fn is_lock_corrupted(content: &str) -> bool {
    content.is_empty() || content.bytes().all(|b| b == 0)
}

/// 손상된 락 파일을 `app.lock.corrupted-{unix_ts}` 로 rename. best-effort — 실패는 무시.
///
/// `read_lock_info` (read-only 조회) 에서만 호출된다. `acquire_lock_atomic` 은 어차피 곧
/// truncate + write 로 덮어쓰므로 백업하지 않는다 (fs2 advisory lock 보유 중 rename 시 race).
fn backup_corrupted_lock(path: &Path) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = path.with_extension(format!("lock.corrupted-{}", ts));
    if let Err(e) = std::fs::rename(path, &backup) {
        eprintln!("[lock] 손상본 백업 실패 ({}): {}", backup.display(), e);
    } else {
        eprintln!("[lock] 손상본 백업 완료: {}", backup.display());
    }
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
    // read_to_string 은 UTF-8 invalid 시 실패하지만 NULL 바이트는 valid UTF-8 이라 통과한다.
    // 손상은 parse_lock_info 가 감지한다.
    file.read_to_string(&mut content)
        .map_err(|e| app_err!(Lock, "락 파일 읽기 실패", e))?;
    let parsed = parse_lock_info(&content)?;
    // 손상 감지 시 (parsed=None 이면서 content 가 비어있지 않음) 분석용 백업.
    // acquire_lock_atomic 은 fs2 락 보유 race 회피로 백업 안 함 — 본 read-only 경로에서만 백업.
    if parsed.is_none() && !content.is_empty() {
        backup_corrupted_lock(&path);
    }
    Ok(parsed)
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
    file.read_to_string(&mut content)
        .map_err(|e| app_err!(Lock, "락 파일 읽기 실패", e))?;
    let current = parse_lock_info(&content)?;

    match current {
        None => {}
        Some(info) if info.is_self() => {}
        // stale 락(5분 미갱신)은 항상 자동 점유 — 점유한 프로세스가 비정상 종료된 신호.
        // 단일 사용자 모델(PRD §1)에서 stale 락이 잔존하면 사용자가 의식적으로 강제 점유
        // 옵션을 매번 확인하는 것보다 자동 정리가 UX 친화적이고 안전.
        // `force` 옵션은 이제 *fresh* 락(다른 PC 정상 사용 중) 도 강제 점유할 때만 의미.
        Some(info) if info.is_stale() => {
            eprintln!(
                "[lock] stale lock 자동 점유 ({}초 미갱신, 이전 device_id={})",
                info.seconds_since_heartbeat(),
                info.device_id
            );
        }
        Some(info) if force => {
            eprintln!(
                "[lock] force=true 로 fresh lock 강제 점유 ({}초 전, 이전 device_id={})",
                info.seconds_since_heartbeat(),
                info.device_id
            );
        }
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
    file.set_len(0)
        .map_err(|e| app_err!(Lock, "락 파일 truncate 실패", e))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| app_err!(Lock, "락 파일 seek 실패", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| app_err!(Lock, "락 파일 쓰기 실패", e))?;
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
/// `force=false`: 락 없음 / 본 디바이스 점유 / stale 락(자동 점유) 시 성공. fresh 락이면 실패.
/// `force=true`: fresh 락도 강제 점유 — 사용자가 의식적으로 다른 PC 정상 사용 차단할 때.
///
/// **stale 자동 점유 정책 (2026-05-21)**: PRD §5.3 의 "강제 점유 옵션 제공" 은 fresh 락에만
/// 적용. stale 락은 점유 프로세스가 비정상 종료된 신호이므로 자동 정리한다 — 단일 사용자
/// 모델에서 매번 사용자 확인은 UX 마찰만 유발.
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

/// 락을 해제한다 (본 디바이스 점유일 때만, T11 R7 advisory lock 적용).
///
/// `acquire_lock_atomic` 와 동일한 advisory lock 보호 수준을 제공한다 — fs2 의
/// `try_lock_exclusive` 로 다른 프로세스가 동시에 락 파일을 조작하지 못하게 한 상태에서
/// 본 디바이스 점유 여부를 재확인하고 삭제한다. 다른 디바이스 점유 락은 보호.
#[tauri::command]
pub async fn release_lock() -> Result<(), String> {
    release_lock_atomic().map_err(String::from)
}

pub(crate) fn release_lock_atomic() -> Result<(), AppError> {
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
        assert!(
            !info.is_self(),
            "랜덤 UUID 는 본 디바이스 ID 와 충돌 확률 거의 0"
        );
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

    // ─── Sprint 7 T3: device.id 영속화 단위 테스트 ───
    //
    // `load_or_create_device_id_at` 을 직접 호출하여 path 별 동작 검증 — process-wide OnceLock
    // (`DEVICE_ID` / `DEVICE_ID_PATH`) 영향 받지 않음. AC-T3-1/2/4/5 보장.

    fn unique_device_id_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("smarthb-device-id-test-{}-{}", label, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("device.id")
    }

    /// AC-T3-1, AC-T3-2: 재시작 후 동일 device_id 유지 — 파일 1회 생성 후 두 번째 로드가 같은 UUID 반환.
    #[test]
    fn device_id_persists_across_load_calls() {
        let path = unique_device_id_path("persists");
        let first = load_or_create_device_id_at(&path);
        assert!(path.exists(), "device.id 파일이 생성되어야 함 (AC-T3-2)");
        let second = load_or_create_device_id_at(&path);
        assert_eq!(first, second, "재시작 시 동일 UUID 유지 (AC-T3-1)");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// AC-T3-2: 저장된 파일 내용이 UUID 문자열로 파싱 가능.
    #[test]
    fn device_id_file_contains_parseable_uuid() {
        let path = unique_device_id_path("parseable");
        let uuid = load_or_create_device_id_at(&path);
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed = Uuid::parse_str(content.trim()).expect("저장된 파일은 valid UUID");
        assert_eq!(parsed, uuid);
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// AC-T3-4: 두 개의 다른 경로 (PC-A vs PC-B 시뮬레이션) 가 각자 다른 UUID 보유.
    #[test]
    fn device_id_differs_across_app_config_dirs() {
        let path_a = unique_device_id_path("pc-a");
        let path_b = unique_device_id_path("pc-b");
        let id_a = load_or_create_device_id_at(&path_a);
        let id_b = load_or_create_device_id_at(&path_b);
        assert_ne!(id_a, id_b, "다른 app_config_dir 는 다른 device.id 보유");
        std::fs::remove_dir_all(path_a.parent().unwrap()).ok();
        std::fs::remove_dir_all(path_b.parent().unwrap()).ok();
    }

    /// AC-T3-5: 파일이 손상 (UUID 파싱 실패) 되면 새 UUID 생성 + 파일 재기록.
    #[test]
    fn device_id_regenerates_on_corruption() {
        let path = unique_device_id_path("corrupted");
        // 손상 파일: UUID 형식 아님
        std::fs::write(&path, "not-a-uuid-at-all").unwrap();
        let recovered = load_or_create_device_id_at(&path);
        // 재기록 — 파일에 유효 UUID 가 다시 저장됨
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed = Uuid::parse_str(content.trim()).expect("재생성 후 valid UUID");
        assert_eq!(parsed, recovered);
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// 부재 파일 시 새 UUID 생성 + atomic write — tmp 파일 잔존하지 않음.
    #[test]
    fn device_id_atomic_write_no_tmp_leak() {
        let path = unique_device_id_path("no-tmp");
        let _uuid = load_or_create_device_id_at(&path);
        let tmp = path.with_extension("id.tmp");
        assert!(path.exists());
        assert!(!tmp.exists(), "tmp 파일이 rename 후 잔존하면 안 됨");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// Sprint 8 T8 (R48-a / I-S2-10): device.id 권한 0o600 (Unix 전용).
    /// Windows 는 NTFS ACL 모델이라 본 테스트 대상 외.
    #[cfg(unix)]
    #[test]
    fn device_id_file_has_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let path = unique_device_id_path("perm-0600");
        let _uuid = load_or_create_device_id_at(&path);
        let mode = std::fs::metadata(&path)
            .expect("device.id 메타데이터")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "소유자 read/write 전용: {:o}", mode);
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
    }

    /// 빈 파일 (NULL/empty) 도 손상으로 처리하여 재생성 — NTFS power-loss 패턴 방어.
    #[test]
    fn device_id_regenerates_on_empty_file() {
        let path = unique_device_id_path("empty");
        std::fs::write(&path, "").unwrap();
        let _uuid = load_or_create_device_id_at(&path);
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.trim().is_empty(), "재생성 후 비어있지 않아야 함");
        Uuid::parse_str(content.trim()).expect("재생성 후 valid UUID");
        std::fs::remove_dir_all(path.parent().unwrap()).ok();
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
        assert!(
            result.is_ok(),
            "본 디바이스 점유 락 release 성공: {:?}",
            result
        );
    }

    // ------------------------------------------------------------------------
    // 손상 복구 (2026-05-21 사고 대응) — parse_lock_info / is_lock_corrupted 검증.
    // ------------------------------------------------------------------------

    #[test]
    fn is_lock_corrupted_detects_empty_and_all_null() {
        assert!(is_lock_corrupted(""));
        // NULL 바이트 1개 또는 다수 — String::trim 이 잡지 못하는 케이스.
        let null_string = String::from_utf8(vec![0u8; 111]).unwrap();
        assert!(is_lock_corrupted(&null_string));
        // 정상 JSON 은 손상 아님.
        assert!(!is_lock_corrupted(
            r#"{"device_id":"x","last_heartbeat":"2026-05-21T00:00:00Z"}"#
        ));
        // 부분 NULL (앞에만) 은 손상 아님 — 파싱 단계에 위임.
        let partial_null = String::from_utf8(vec![0u8, b'{', b'}']).unwrap();
        assert!(!is_lock_corrupted(&partial_null));
    }

    #[test]
    fn parse_lock_info_returns_none_for_empty_content() {
        let result = parse_lock_info("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_lock_info_returns_none_for_all_null_bytes() {
        // 실제 사고 재현 — 111 바이트 전체 0x00.
        let null_string = String::from_utf8(vec![0u8; 111]).unwrap();
        let result = parse_lock_info(&null_string).unwrap();
        assert!(result.is_none(), "손상 fallback None 반환");
    }

    #[test]
    fn parse_lock_info_returns_none_for_malformed_json() {
        // 파싱 실패도 손상으로 간주 — AppError::Lock 으로 wrap 되어 "다른 컴퓨터 사용 중" 으로
        // 잘못 표시되는 회귀 방지.
        let result = parse_lock_info("{not valid json").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_lock_info_returns_some_for_valid_json() {
        let info = LockInfo::new_for_self();
        let json = serde_json::to_string(&info).unwrap();
        let result = parse_lock_info(&json).unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().is_self());
    }

}
