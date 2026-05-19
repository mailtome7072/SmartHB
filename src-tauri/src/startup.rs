//! 앱 시작 시퀀스 + 성능 검증 (T10, PRD §5.6 < 3초).
//!
//! ## 흐름
//!
//! UI 가 별도로 [`crate::commands::sync::check_sync_status`] 로 동기화 대기를 처리한 후
//! 사용자 비밀번호를 받아 본 IPC [`app_startup_sequence`] 를 호출한다.
//!
//! ```text
//! app_startup_sequence(password, force_lock)
//!     ├── 측정 시작 (Instant::now)
//!     ├── [tokio::join!] acquire_lock(force) + check_integrity(quick) 병렬
//!     ├── 비밀번호 검증 (auth::verify_password)
//!     ├── db::initialize → PRAGMA key (cipher build) + WAL + cache_size + migrate
//!     ├── audit::cleanup_older_than(365)  (best-effort)
//!     ├── 백그라운드 spawn: heartbeat (60초) + hourly 백업 (1시간)
//!     └── 측정 종료 → StartupResult 반환
//! ```
//!
//! 측정값은 PRD §5.6 의 3초 예산 검증에 사용된다 — 동기화 대기와 사용자 비밀번호 입력
//! 시간은 예산 외이므로 제외.
//!
//! ## 백그라운드 task lifecycle
//!
//! 본 모듈은 `OnceLock<BackgroundHandles>` 로 spawn 결과를 1회 보관 — 재진입(unlock 재호출)
//! 시 중복 spawn 을 방지한다. 앱 종료는 [`exit_hook`] 에서 exit 백업을 동기 실행한 후
//! 락을 해제한다. 백그라운드 task 는 OS 프로세스 종료와 함께 정리되므로 명시적 abort 는
//! 본 sprint 에서 생략 (lib.rs RunEvent::Exit 가 즉시 프로세스 종료).
//!
//! ## Feature 게이트
//!
//! cipher off 빌드에서는 무결성 검증과 backup 이 모두 stub 안내를 반환한다. startup
//! 자체는 fail-soft — 무결성/백업 실패는 startup 실패가 아니라 결과 필드(`integrity_ok=false`,
//! `backup_available=false`) 로 보고하여 개발 빌드에서도 메인 진입을 허용한다.

use crate::commands::{audit, auth, backup, db, integrity, lock};
use crate::error::AppError;
use serde::Serialize;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use zeroize::Zeroizing;

/// audit_logs 보관 기간 (PRD §6.6 — 최근 1년).
const AUDIT_RETENTION_DAYS: i64 = 365;

/// heartbeat 갱신 간격 (PRD §5.3 — 60초).
const HEARTBEAT_INTERVAL_SECS: u64 = 60;

/// hourly 백업 간격.
const HOURLY_BACKUP_INTERVAL_SECS: u64 = 3600;

/// 시작 시퀀스 결과 — IPC 응답.
///
/// `elapsed_ms` 는 startup IPC 진입부터 종료까지의 wall-clock 시간 (PRD §5.6 < 3000ms 목표).
/// `integrity_ok` 와 `backup_available` 은 cipher off 개발 빌드에서 stub 결과를 의미하는
/// 정보 필드 — startup 성공/실패 결정에는 사용되지 않는다.
#[derive(Debug, Serialize)]
pub struct StartupResult {
    pub elapsed_ms: u128,
    pub lock_force_used: bool,
    pub integrity_ok: bool,
    pub audit_cleaned: u64,
}

/// 백그라운드 task 핸들 묶음 — `OnceLock` 으로 1회 spawn 보장.
struct BackgroundHandles {
    _heartbeat: JoinHandle<()>,
    _hourly_backup: JoinHandle<()>,
}

static BACKGROUND: OnceLock<BackgroundHandles> = OnceLock::new();

/// 앱 시작 시퀀스를 실행한다.
///
/// 사용자 비밀번호는 IPC 호출 직후 `Zeroizing<String>` 으로 감싸 메모리 폐기를 보장한다.
/// force_lock=true 는 사용자가 이전 화면에서 stale 락 강제 점유를 결정한 후에만 호출된다 —
/// IPC 자체는 사전 확인을 강제하지 않으므로 UI 가 사용자 동의를 받았다고 가정한다.
#[tauri::command]
pub async fn app_startup_sequence(
    password: String,
    force_lock: bool,
) -> Result<StartupResult, String> {
    let password = Zeroizing::new(password);
    let started = Instant::now();

    // 1. 락 + 무결성 quick_check 병렬 — PRD §5.6 시작 < 3초 예산을 위해 join!
    let (lock_result, integrity_result) = tokio::join!(
        async {
            tokio::task::spawn_blocking(move || lock::acquire_lock_atomic(force_lock))
                .await
                .map_err(|e| AppError::Lock(format!("락 작업 실패: {}", e)))
                .and_then(|r| r)
        },
        async {
            tokio::task::spawn_blocking(integrity::check_integrity_quick_for_startup)
                .await
                .map_err(|e| AppError::Integrity(format!("무결성 작업 실패: {}", e)))
                .and_then(|r| r)
        },
    );
    lock_result.map_err(String::from)?;
    if force_lock {
        audit::try_record(audit::AuditEventType::LockForced, None, None).await;
    }
    let integrity_ok = matches!(integrity_result, Ok(integrity::IntegrityCheckResult::Ok));
    if let Ok(integrity::IntegrityCheckResult::Failed { detail }) = &integrity_result {
        let first_line = detail.lines().next().unwrap_or("(no detail)");
        audit::try_record(
            audit::AuditEventType::IntegrityCheckFailed,
            None,
            Some(first_line),
        )
        .await;
    }

    // 2. 비밀번호 검증 — keyring salt + key 비교.
    auth::verify_password(&password).await.map_err(String::from)?;

    // 3. DB pool 초기화 — PRAGMA key (cipher build) + WAL + cache_size + migrate.
    db::initialize(backup::db_path())
        .await
        .map_err(String::from)?;

    // 4. audit_logs 1년 롤링 정리 (best-effort).
    let audit_cleaned = audit::cleanup_older_than(AUDIT_RETENTION_DAYS)
        .await
        .unwrap_or_else(|e| {
            eprintln!("[audit] 1년 정리 실패 (무시): {}", e);
            0
        });

    // 5. 백그라운드 task spawn — 재진입 시 중복 spawn 방지.
    spawn_background_tasks();

    Ok(StartupResult {
        elapsed_ms: started.elapsed().as_millis(),
        lock_force_used: force_lock,
        integrity_ok,
        audit_cleaned,
    })
}

/// heartbeat + hourly 백업 백그라운드 task 를 1회 spawn 한다.
fn spawn_background_tasks() {
    BACKGROUND.get_or_init(|| {
        let heartbeat = tokio::spawn(async {
            let mut ticker = tokio::time::interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
            // 첫 tick 은 즉시 발생 — 이를 소비하여 다음 tick 부터 60초 간격이 되도록 한다.
            ticker.tick().await;
            loop {
                ticker.tick().await;
                lock::heartbeat_tick().await;
            }
        });
        let hourly_backup = tokio::spawn(async {
            let mut ticker = tokio::time::interval(Duration::from_secs(HOURLY_BACKUP_INTERVAL_SECS));
            ticker.tick().await; // 첫 tick 즉시 소비
            loop {
                ticker.tick().await;
                backup::try_create_backup(backup::BackupLayer::Hourly).await;
            }
        });
        BackgroundHandles {
            _heartbeat: heartbeat,
            _hourly_backup: hourly_backup,
        }
    });
}

/// 앱 종료 hook — `RunEvent::ExitRequested` 에서 호출.
///
/// exit 백업을 동기 실행한 후 락을 해제한다. async runtime 이 살아있을 때만 동작하므로
/// `tauri::async_runtime::block_on` 으로 호출된다 (lib.rs).
pub async fn exit_hook() {
    backup::try_create_backup(backup::BackupLayer::Exit).await;
    // 락 해제 — 본 디바이스 점유일 때만 동작 (다른 디바이스 락은 자동 보호).
    if let Err(e) = tokio::task::spawn_blocking(|| {
        std::fs::remove_file(lock::lock_path()).ok();
    })
    .await
    {
        eprintln!("[startup] exit 락 정리 실패 (무시): {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_constants_match_prd() {
        assert_eq!(AUDIT_RETENTION_DAYS, 365);
        assert_eq!(HEARTBEAT_INTERVAL_SECS, 60);
        assert_eq!(HOURLY_BACKUP_INTERVAL_SECS, 3600);
    }

    #[test]
    fn startup_result_serializes_with_camel_snake_fields() {
        let r = StartupResult {
            elapsed_ms: 1234,
            lock_force_used: false,
            integrity_ok: true,
            audit_cleaned: 0,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""elapsed_ms":1234"#));
        assert!(json.contains(r#""lock_force_used":false"#));
        assert!(json.contains(r#""integrity_ok":true"#));
        assert!(json.contains(r#""audit_cleaned":0"#));
    }
}
