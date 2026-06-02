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

use crate::commands::{audit, auth, backup, db, expiration, integrity, lock, paths};
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
/// 각 단계별 `*_ms` 필드는 T4 R8 cipher on 실측 디버깅을 위한 timing breakdown — 3초 초과 시
/// 어느 단계가 병목인지 즉시 식별 가능. PBKDF2 600K iter (`password_verify_ms`) 가 보통 가장
/// 큰 비중을 차지한다 (~500ms 예상).
///
/// `integrity_ok` 는 cipher off 개발 빌드에서 stub 결과를 의미하는 정보 필드 — startup
/// 성공/실패 결정에는 사용되지 않는다.
#[derive(Debug, Serialize)]
pub struct StartupResult {
    pub elapsed_ms: u128,
    pub parallel_phase_ms: u128,
    pub password_verify_ms: u128,
    pub db_init_ms: u128,
    pub audit_cleanup_ms: u128,
    pub lock_force_used: bool,
    pub integrity_ok: bool,
    pub audit_cleaned: u64,
    /// Sprint 10 T4 (PI-05/PI-09): 앱 시작 직후 소멸 자동 전이 결과.
    /// `transitioned_count > 0` 시 프론트엔드에서 토스트 표시.
    /// ExpirationReport 내부는 camelCase, 본 필드명은 snake_case (기존 StartupResult 패턴 유지).
    pub expiration_report: expiration::ExpirationReport,
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
/// 인증 방식 — PIN 검증 또는 키체인 자동 로드(스킵, ADR-008).
enum AuthStep {
    Pin(Zeroizing<String>),
    Keychain,
}

/// PIN 입력 잠금 해제 (기존 동작 유지).
#[tauri::command]
pub async fn app_startup_sequence(
    password: String,
    force_lock: bool,
) -> Result<StartupResult, String> {
    run_startup(force_lock, AuthStep::Pin(Zeroizing::new(password))).await
}

/// 키체인 자동 잠금 해제 (ADR-008, skip_pin_on_launch=true).
///
/// PIN 비교 없이 OS Keychain 의 기존 유도키를 로드하여 진입한다. 키체인에 키가 없으면
/// (이 PC 에서 한 번도 PIN 인증 안 함 = 새 PC) Err 를 반환 → 프론트가 LockScreen 으로 폴백.
#[tauri::command]
pub async fn auto_unlock_with_keychain(force_lock: bool) -> Result<StartupResult, String> {
    run_startup(force_lock, AuthStep::Keychain).await
}

/// startup 공통 시퀀스 — 인증 단계(AuthStep)만 분기하고, 이후 락/무결성/DB/audit/소멸/백그라운드는
/// 양 경로가 공유한다 (R91: PIN 경로 로직 불변 유지).
async fn run_startup(force_lock: bool, auth: AuthStep) -> Result<StartupResult, String> {
    let started = Instant::now();

    // 1. 락 + 무결성 quick_check 병렬 — PRD §5.6 시작 < 3초 예산을 위해 join!
    let parallel_start = Instant::now();
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
    let parallel_phase_ms = parallel_start.elapsed().as_millis();
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

    // 2. 인증 — PIN: keyring salt+key 비교 / Keychain: 비교 없이 키 로드(스킵, ADR-008).
    //    PBKDF2 600K iter(PIN 경로) 가 보통 가장 큰 비중.
    let verify_start = Instant::now();
    match auth {
        AuthStep::Pin(password) => auth::verify_password(&password).await.map_err(String::from)?,
        AuthStep::Keychain => {
            // 키체인의 기존 유도키 로드 — 비교 단계 생략. 키 부재(새 PC) 시 에러 → LockScreen 폴백.
            //
            // cipher on/off 공통 동작 (A91, ADR-008 정합):
            // - cipher on: 로드한 키가 이후 db::initialize 의 PRAGMA key 에 사용됨.
            // - cipher off: db 는 평문이라 키를 PRAGMA 에 쓰지 않지만, get_cached_or_load_key 는
            //   salt(파일)+key(keyring) 로드를 동일하게 수행한다. 즉 "stub/즉시성공"이 아니라
            //   양 빌드 모두 키체인 접근이 일어나며, 키 존재 여부가 곧 "이 PC 인증 키 보유" 판정이 된다.
            //   (개발 모드(Tauri 미동작)에서는 프론트 래퍼가 먼저 reject 하여 이 경로에 도달하지 않음)
            auth::get_cached_or_load_key()
                .map_err(|_| "KeyNotFound: 이 PC에 저장된 인증 키가 없습니다.".to_string())?;
        }
    }
    let password_verify_ms = verify_start.elapsed().as_millis();

    // 3. DB pool 초기화 — PRAGMA key (cipher build) + WAL + cache_size + migrate.
    let db_init_start = Instant::now();
    db::initialize(paths::db_path())
        .await
        .map_err(String::from)?;
    let db_init_ms = db_init_start.elapsed().as_millis();

    // 4. audit_logs 1년 롤링 정리 (best-effort).
    let audit_start = Instant::now();
    let audit_cleaned = audit::cleanup_older_than(AUDIT_RETENTION_DAYS)
        .await
        .unwrap_or_else(|e| {
            eprintln!("[audit] 1년 정리 실패 (무시): {}", e);
            0
        });
    let audit_cleanup_ms = audit_start.elapsed().as_millis();

    // 5. Sprint 10 T4 (PI-05): 소멸 자동 전이 — 앱 시작 트리거.
    //    db::initialize 완료 후 pool 사용 가능. fail-soft — 실패해도 startup 자체는 성공.
    let expiration_report = match expiration::expire_overdue_absences_impl(
        db::pool().expect("pool initialized above"),
        None,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[startup] 소멸 자동 전이 실패 (무시): {}", e);
            expiration::ExpirationReport {
                transitioned_count: 0,
                details: vec![],
            }
        }
    };

    // 6. 백그라운드 task spawn — 재진입 시 중복 spawn 방지.
    spawn_background_tasks();

    let elapsed_ms = started.elapsed().as_millis();

    // R8 cipher on 실측 디버깅용 timing breakdown 로그.
    // 3초 초과 시 어느 단계가 병목인지 즉시 식별 가능.
    eprintln!(
        "[startup] total={elapsed_ms}ms parallel={parallel_phase_ms}ms password={password_verify_ms}ms db_init={db_init_ms}ms audit={audit_cleanup_ms}ms (PRD §5.6 < 3000ms)"
    );
    if elapsed_ms > 3000 {
        eprintln!(
            "[startup] ⚠️ 3초 예산 초과 ({elapsed_ms}ms) — PRAGMA cache_size 튜닝 검토 필요"
        );
    }

    Ok(StartupResult {
        elapsed_ms,
        parallel_phase_ms,
        password_verify_ms,
        db_init_ms,
        audit_cleanup_ms,
        lock_force_used: force_lock,
        integrity_ok,
        audit_cleaned,
        expiration_report,
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

/// 앱 종료 hook — `RunEvent::ExitRequested` + `WindowEvent::CloseRequested` 양쪽에서 호출.
///
/// exit 백업을 동기 실행한 후 락을 해제한다. async runtime 이 살아있을 때만 동작하므로
/// `tauri::async_runtime::block_on` 으로 호출된다 (lib.rs).
///
/// R15 (sprint-review Medium): `release_lock_atomic` 을 호출하여 advisory lock 보유 + 본
/// 디바이스 점유 재확인 후 삭제. `std::fs::remove_file` 직접 호출 시 다른 디바이스 락을
/// 손상시키는 엣지 케이스를 방지한다.
///
/// V24 (Sprint 7 post-review): idempotent 가드 — 윈도우 닫기 + 앱 종료 두 이벤트가 순차
/// 발생해도 백업·락 해제 작업은 1회만 수행되도록 `AtomicBool` 플래그.
pub async fn exit_hook() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static RAN: AtomicBool = AtomicBool::new(false);
    if RAN.swap(true, Ordering::SeqCst) {
        return;
    }
    backup::try_create_backup(backup::BackupLayer::Exit).await;
    match tokio::task::spawn_blocking(lock::release_lock_atomic).await {
        Ok(Ok(())) => eprintln!("[startup] exit 락 해제 완료"),
        Ok(Err(e)) => eprintln!("[startup] exit 락 해제 실패 (무시): {}", e),
        Err(e) => eprintln!("[startup] exit 락 작업 실패 (무시): {}", e),
    }
    // Sprint 8 T6 (I-S2-4 / R42): CRED_CACHE 명시적 무효화.
    // ZeroizeOnDrop 으로 메모리 영(0) 처리되어 종료 시점의 키 잔류 시간을 최소화한다.
    // backup/lock 해제는 cipher feature on 빌드에서 캐시 키를 필요로 할 수 있으므로 그 이후에 호출.
    auth::invalidate_credential_cache();
    eprintln!("[startup] exit 자격증명 캐시 무효화 완료");
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
            parallel_phase_ms: 50,
            password_verify_ms: 500,
            db_init_ms: 600,
            audit_cleanup_ms: 30,
            lock_force_used: false,
            integrity_ok: true,
            audit_cleaned: 0,
            expiration_report: expiration::ExpirationReport {
                transitioned_count: 0,
                details: vec![],
            },
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""elapsed_ms":1234"#));
        assert!(json.contains(r#""parallel_phase_ms":50"#));
        assert!(json.contains(r#""password_verify_ms":500"#));
        assert!(json.contains(r#""db_init_ms":600"#));
        assert!(json.contains(r#""audit_cleanup_ms":30"#));
        assert!(json.contains(r#""lock_force_used":false"#));
        assert!(json.contains(r#""integrity_ok":true"#));
        assert!(json.contains(r#""audit_cleaned":0"#));
    }

    #[test]
    fn startup_result_timing_breakdown_sum_approximates_total() {
        // breakdown 합 ≤ total (병렬 단계가 wall-clock 효과 반영하므로 정확히 같지 않을 수 있음).
        let r = StartupResult {
            elapsed_ms: 1230,
            parallel_phase_ms: 100,
            password_verify_ms: 500,
            db_init_ms: 600,
            audit_cleanup_ms: 30,
            lock_force_used: false,
            integrity_ok: true,
            audit_cleaned: 0,
            expiration_report: expiration::ExpirationReport {
                transitioned_count: 0,
                details: vec![],
            },
        };
        let sum = r.parallel_phase_ms + r.password_verify_ms + r.db_init_ms + r.audit_cleanup_ms;
        assert!(sum <= r.elapsed_ms, "breakdown 합 ≤ 총 elapsed");
    }

    /// cipher off 환경에서 무결성 검증 fail-soft 동작을 검증한다.
    ///
    /// startup IPC 자체는 keyring 의존 (verify_password) + DB 파일 의존 (db::initialize) 이라
    /// 전체 통합은 사용자 환경 테스트에서만 가능. 본 단위 테스트는 fail-soft 분기만 확인.
    #[cfg(not(feature = "cipher"))]
    #[test]
    fn integrity_quick_check_for_startup_is_fail_soft_when_db_missing() {
        // 정확한 db_path 가 존재하지 않을 때 quick_check 가 Ok 로 fail-soft 동작.
        let result = integrity::check_integrity_quick_for_startup();
        // db_path 존재 여부에 따라 Ok 또는 Failed — 둘 다 panic 없이 반환.
        assert!(result.is_ok(), "fail-soft: cipher off 에서 Ok 반환 기대");
    }
}
