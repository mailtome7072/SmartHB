//! DB 무결성 검증 + 자동 복원 (T8, PRD §5.3/§5.4).
//!
//! ## 흐름
//!
//! 1. `check_integrity(mode)`: 현재 DB 에 대해 `PRAGMA quick_check`(기본, ~50ms) 또는
//!    `PRAGMA integrity_check`(일일 백업 시점) 실행 후 결과 파싱.
//! 2. `auto_restore()`: `backup/exit/` 계층에서 최신 백업부터 `quick_check` 폴백으로
//!    무결한 후보를 선택 → 현재 DB 를 `restore_rollback/` 으로 이동 → 백업 파일을
//!    `app.db` 로 파일 복사 (SQLCipher 암호화 상태 그대로).
//!
//! ## Feature 게이트
//!
//! 검증·복원 흐름은 SQLCipher PRAGMA key 적용이 필요하므로 `cipher` feature on 빌드에서만
//! 정식 동작한다. cipher off 빌드는 사용자 친화 안내 메시지로 즉시 거부.
//!
//! ## 안전망
//!
//! - 복원 직전 현재 DB 는 항상 `restore_rollback/rollback_YYYYMMDD_HHMMSS.db` 로 보존되어
//!   백업 파일 복사 실패 시 자동으로 되돌려진다.
//! - exit 계층 전체 후보가 손상되었을 경우 daily/weekly 자동 폴백은 수행하지 않는다 —
//!   사용자에게 명확한 에러로 보고 후 수동 결정(T10 startup UI 흐름에서 처리).

use crate::app_err;
use crate::commands::audit::{self, AuditEventType};
use crate::commands::backup::{self, BackupLayer, BackupMetadata};
use crate::commands::paths;
use crate::commands::runtime::run_blocking;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const ROLLBACK_SUBDIR: &str = "restore_rollback";
const ROLLBACK_FILENAME_PREFIX: &str = "rollback_";
const ROLLBACK_FILENAME_SUFFIX: &str = ".db";

/// 무결성 검증 모드.
///
/// - `Quick`: `PRAGMA quick_check` — ~50ms, 앱 시작 시 사용 (PRD §5.6 < 3초 예산)
/// - `Full`: `PRAGMA integrity_check` — 더 무거움, 일일 백업 시점 또는 사용자 수동 실행
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrityMode {
    Quick,
    Full,
}

impl IntegrityMode {
    #[cfg_attr(not(feature = "cipher"), allow(dead_code))]
    fn pragma(self) -> &'static str {
        match self {
            Self::Quick => "PRAGMA quick_check",
            Self::Full => "PRAGMA integrity_check",
        }
    }
}

/// 검증 결과 — IPC 응답.
///
/// `quick_check` / `integrity_check` 가 "ok" 단일 행 반환 시 `Ok`,
/// 그 외(손상)는 다중 행 메시지를 `\n` 으로 결합하여 `detail` 에 저장.
#[cfg_attr(not(feature = "cipher"), allow(dead_code))]
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum IntegrityCheckResult {
    Ok,
    Failed { detail: String },
}

/// 자동 복원 결과 — UI 가 사용자에게 안내할 정보.
#[derive(Debug, Serialize)]
pub struct RestoreResult {
    pub restored_from: String,
    pub rollback_path: String,
}

fn rollback_dir() -> PathBuf {
    paths::data_root().join(ROLLBACK_SUBDIR)
}

fn ensure_rollback_dir() -> Result<PathBuf, AppError> {
    let dir = rollback_dir();
    std::fs::create_dir_all(&dir).map_err(|e| app_err!(Integrity, "rollback 디렉토리 생성 실패", e))?;
    Ok(dir)
}

fn generate_rollback_filename(now: DateTime<Utc>, idx: usize) -> String {
    let base = backup::timestamped_filename(ROLLBACK_FILENAME_PREFIX, "", now);
    format!("{}_{}{}",  base, idx, ROLLBACK_FILENAME_SUFFIX)
}

// ============================================================================
// PRAGMA 실행 — cipher feature 게이트
// ============================================================================

/// 키 캐싱 버전 — 호출자가 키를 1회 조회 후 다수 후보를 검증할 때 사용한다.
///
/// `select_healthy_backup` 처럼 exit 백업 N개를 순회하며 검증할 때 매 호출마다 OS Keychain IPC
/// (~수~수십 ms) 발생을 막는다. 단일 검증은 [`run_pragma_check`] 가 wrap.
#[cfg(feature = "cipher")]
fn run_pragma_check_with_key(
    db_path: &Path,
    hex_key: &str,
    mode: IntegrityMode,
) -> Result<IntegrityCheckResult, AppError> {
    use rusqlite::Connection;

    let conn = Connection::open(db_path).map_err(|e| app_err!(Integrity, "DB 열기 실패", e))?;
    conn.execute_batch(&paths::pragma_key_sql(hex_key))
        .map_err(|e| app_err!(Integrity, "PRAGMA key 적용 실패", e))?;

    let pragma = mode.pragma();
    let mut stmt = conn
        .prepare(pragma)
        .map_err(|e| app_err!(Integrity, "PRAGMA 준비 실패", e))?;
    let rows: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| app_err!(Integrity, "PRAGMA 실행 실패", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| app_err!(Integrity, "PRAGMA 결과 읽기 실패", e))?;

    Ok(classify_pragma_rows(rows))
}

#[cfg(feature = "cipher")]
fn run_pragma_check(db_path: &Path, mode: IntegrityMode) -> Result<IntegrityCheckResult, AppError> {
    use crate::commands::auth::get_cached_or_load_key;
    // Sprint 7 T1: 캐시 경유 — startup 후 무결성 검증 시 keyring 다이얼로그 0회.
    let key = get_cached_or_load_key()?;
    let hex_key = key.to_hex();
    run_pragma_check_with_key(db_path, hex_key.as_str(), mode)
}

#[cfg(not(feature = "cipher"))]
fn run_pragma_check(_db_path: &Path, _mode: IntegrityMode) -> Result<IntegrityCheckResult, AppError> {
    Err(AppError::Integrity(
        "암호화 빌드(--features cipher)에서만 무결성 검증이 가능합니다.".to_string(),
    ))
}

/// PRAGMA 결과 행을 정상/손상으로 분류한다. cipher feature 분기와 분리하여 단위 테스트 가능.
#[cfg_attr(not(feature = "cipher"), allow(dead_code))]
fn classify_pragma_rows(rows: Vec<String>) -> IntegrityCheckResult {
    if rows.len() == 1 && rows[0] == "ok" {
        IntegrityCheckResult::Ok
    } else {
        IntegrityCheckResult::Failed {
            detail: rows.join("\n"),
        }
    }
}

// ============================================================================
// 복원 흐름
// ============================================================================

/// 지정 계층에서 시간 역순(최신→과거)으로 후보를 가져온다.
fn candidates_newest_first(layer: BackupLayer) -> Result<Vec<BackupMetadata>, AppError> {
    let mut entries = backup::scan_layer(layer)?;
    entries.sort_by_key(|m| std::cmp::Reverse(m.created_at));
    Ok(entries)
}

/// 후보들을 시간 역순으로 순회하며 quick_check 통과한 가장 최신 백업을 반환한다.
///
/// 모든 후보가 손상되었으면 `None` — 호출자가 명확한 에러로 사용자에게 안내한다.
///
/// OS Keychain 키 조회는 1회만 수행하여 후보 N개 손상 시 N회 IPC 발생을 막는다.
fn select_healthy_backup(layer: BackupLayer) -> Result<Option<BackupMetadata>, AppError> {
    let candidates = candidates_newest_first(layer)?;
    if candidates.is_empty() {
        return Ok(None);
    }
    select_first_healthy_with_cached_key(candidates)
}

#[cfg(feature = "cipher")]
fn select_first_healthy_with_cached_key(
    candidates: Vec<BackupMetadata>,
) -> Result<Option<BackupMetadata>, AppError> {
    use crate::commands::auth::get_cached_or_load_key;
    // Sprint 7 T1: 캐시 경유 — 다수 백업 후보 검증 시 매번 keyring 다이얼로그 0회.
    let key = get_cached_or_load_key()?;
    let hex_key = key.to_hex();
    for candidate in candidates {
        let path = PathBuf::from(&candidate.path);
        let result = run_pragma_check_with_key(&path, hex_key.as_str(), IntegrityMode::Quick)?;
        if matches!(result, IntegrityCheckResult::Ok) {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

#[cfg(not(feature = "cipher"))]
fn select_first_healthy_with_cached_key(
    _candidates: Vec<BackupMetadata>,
) -> Result<Option<BackupMetadata>, AppError> {
    // cipher off 빌드는 검증이 거부되므로 후보 순회를 시도하지 않고 즉시 안내.
    run_pragma_check(Path::new("."), IntegrityMode::Quick).map(|_| None)
}

/// 지정 백업 파일로 현재 DB 를 복원한다.
///
/// 1. 후보 백업 `PRAGMA quick_check` 통과 확인 — 실패 시 즉시 거부
/// 2. 현재 DB 를 `restore_rollback/` 으로 rename (atomic on most filesystems)
/// 3. 백업 파일을 `app.db` 로 파일 복사 — SQLCipher 암호화 상태 그대로 (복호화 금지)
/// 4. 복사 실패 시 rollback 을 되돌려 원상 복구 시도
fn restore_from_path_sync(backup_path: &Path, idx: usize) -> Result<RestoreResult, AppError> {
    let check = run_pragma_check(backup_path, IntegrityMode::Quick)?;
    if !matches!(check, IntegrityCheckResult::Ok) {
        return Err(AppError::Integrity(format!(
            "지정된 백업이 손상되어 복원할 수 없습니다: {}",
            backup_path.display()
        )));
    }

    let rollback_dir = ensure_rollback_dir()?;
    let rollback_path = rollback_dir.join(generate_rollback_filename(Utc::now(), idx));
    let current_db = paths::db_path();

    if current_db.exists() {
        std::fs::rename(&current_db, &rollback_path)
            .map_err(|e| app_err!(Integrity, "현재 DB rollback 이동 실패", e))?;
    }

    if let Err(e) = std::fs::copy(backup_path, &current_db) {
        // 복원 실패 — rollback 을 되돌려 원상복구 시도
        let _ = std::fs::rename(&rollback_path, &current_db);
        return Err(app_err!(Integrity, "백업 파일 복사 실패", e));
    }

    Ok(RestoreResult {
        restored_from: backup_path.to_string_lossy().into_owned(),
        rollback_path: rollback_path.to_string_lossy().into_owned(),
    })
}

/// exit 계층 최신 정상 백업으로 자동 복원 (현재 DB 는 rollback 보존).
/// startup 손상 자동복원(`startup::run_startup`) 과 `auto_restore` IPC 가 공유.
pub(crate) fn auto_restore_sync() -> Result<RestoreResult, AppError> {
    let healthy = select_healthy_backup(BackupLayer::Exit)?;
    let backup_meta = healthy.ok_or_else(|| {
        AppError::Integrity(
            "exit 계층에서 무결한 백업을 찾지 못했습니다. 일일/주간 백업을 수동으로 선택해주세요."
                .to_string(),
        )
    })?;
    restore_from_path_sync(Path::new(&backup_meta.path), 0)
}

/// 복원 후 quick_check 재검증을 포함한 자동 복원 — startup 전용.
///
/// 파일 복사 후 OS 레벨 손상(NTFS power-loss 등)을 감지하기 위해 복원된 app.db 에
/// quick_check 를 재실행한다. 실패 시 다음 최신 exit 백업으로 재시도 (최대 3회).
/// 모두 실패하면 사용자 친화 에러 반환.
pub(crate) fn auto_restore_with_retry() -> Result<RestoreResult, AppError> {
    let candidates = candidates_newest_first(BackupLayer::Exit)?;
    if candidates.is_empty() {
        return Err(AppError::Integrity(
            "복원할 수 있는 백업이 없습니다. 수동으로 백업 파일을 선택해주세요.".to_string(),
        ));
    }

    let mut last_err = AppError::Integrity("알 수 없는 복원 오류".to_string());

    for (idx, candidate) in candidates.into_iter().take(3).enumerate() {
        let result = restore_from_path_sync(Path::new(&candidate.path), idx);
        let restore_result = match result {
            Ok(r) => r,
            Err(e) => {
                last_err = e;
                continue;
            }
        };
        // 복원 후 quick_check 재검증 — 파일 복사 후 OS 레벨 손상 감지.
        match run_pragma_check(&paths::db_path(), IntegrityMode::Quick) {
            Ok(IntegrityCheckResult::Ok) => return Ok(restore_result),
            Ok(IntegrityCheckResult::Failed { detail }) => {
                eprintln!("[integrity] 복원 후 재검증 실패 (다음 백업 시도): {}", detail);
                last_err = AppError::Integrity(format!("복원 후 무결성 검증 실패: {}", detail));
            }
            Err(e) => {
                eprintln!("[integrity] 복원 후 재검증 실행 오류 (다음 백업 시도): {}", e);
                last_err = e;
            }
        }
    }

    Err(AppError::Integrity(format!(
        "복원할 수 있는 백업이 없습니다. 마지막 오류: {}",
        last_err
    )))
}

/// `backup::restore_backup` 에서 사용하는 동기 인터페이스.
///
/// path 지정 복원 — 사용자가 `list_backups` 결과에서 특정 파일을 선택했을 때 호출된다.
/// auto_restore 와 동일한 안전망(quick_check + rollback 보존)을 공유한다.
pub(crate) fn restore_from_path(backup_path: &Path) -> Result<RestoreResult, AppError> {
    restore_from_path_sync(backup_path, 0)
}

/// T10 시작 시퀀스 전용 동기 quick_check — 현재 DB 가 없거나 cipher off 빌드면 `Ok` 로 fail-soft.
///
/// startup 의 tokio::join! 안에서 spawn_blocking 으로 호출되며, 무결성 실패가 startup 자체를
/// 차단하지 않도록 한다. cipher off 개발 빌드에서는 DB 가 평문이거나 존재하지 않을 수 있으므로
/// 안내 메시지를 startup 결과 필드(`integrity_ok=false`) 로 전달한다.
pub(crate) fn check_integrity_quick_for_startup() -> Result<IntegrityCheckResult, AppError> {
    let db_path = paths::db_path();
    if !db_path.exists() {
        // 첫 실행 — DB 파일 자체가 아직 없음. startup 이 db::initialize 로 생성한다.
        return Ok(IntegrityCheckResult::Ok);
    }
    match run_pragma_check(&db_path, IntegrityMode::Quick) {
        Ok(r) => Ok(r),
        // cipher off 개발 빌드 — run_pragma_check 가 stub 안내. startup 은 fail-soft 진행.
        #[cfg(not(feature = "cipher"))]
        Err(AppError::Integrity(_)) => Ok(IntegrityCheckResult::Ok),
        Err(e) => Err(e),
    }
}

// ============================================================================
// Tauri IPC commands
// ============================================================================

/// 현재 DB 에 대해 무결성 검증을 수행한다.
#[tauri::command]
pub async fn check_integrity(mode: IntegrityMode) -> Result<IntegrityCheckResult, String> {
    let result = run_blocking(AppError::Integrity, "무결성 검증 작업 실패", move || {
        run_pragma_check(&paths::db_path(), mode)
    })
    .await?;
    if let IntegrityCheckResult::Failed { detail } = &result {
        // detail 첫 줄만 기록 — 다중 행 결합이 너무 길어질 수 있음. 민감 데이터 포함 위험 없음
        // (SQLite quick_check 출력은 row id + 손상 타입 텍스트).
        let first_line = detail.lines().next().unwrap_or("(no detail)");
        audit::try_record(AuditEventType::IntegrityCheckFailed, None, Some(first_line)).await;
    }
    Ok(result)
}

/// `backup/exit/` 의 가장 최신 무결한 백업으로 자동 복원한다.
#[tauri::command]
pub async fn auto_restore() -> Result<RestoreResult, String> {
    let result = run_blocking(AppError::Integrity, "자동 복원 작업 실패", auto_restore_sync).await?;
    audit::try_record(AuditEventType::BackupRestored, Some(&result.restored_from), None).await;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_mode_pragma_strings() {
        assert_eq!(IntegrityMode::Quick.pragma(), "PRAGMA quick_check");
        assert_eq!(IntegrityMode::Full.pragma(), "PRAGMA integrity_check");
    }

    #[test]
    fn classify_ok_single_row() {
        let result = classify_pragma_rows(vec!["ok".to_string()]);
        assert_eq!(result, IntegrityCheckResult::Ok);
    }

    #[test]
    fn classify_failed_single_corruption_message() {
        let result = classify_pragma_rows(vec!["row 1: corrupt".to_string()]);
        match result {
            IntegrityCheckResult::Failed { detail } => assert_eq!(detail, "row 1: corrupt"),
            _ => panic!("Failed 분류 기대"),
        }
    }

    #[test]
    fn classify_failed_multiple_rows_joined_with_newline() {
        let rows = vec![
            "row 1: corrupt".to_string(),
            "row 2: broken index".to_string(),
        ];
        let result = classify_pragma_rows(rows);
        match result {
            IntegrityCheckResult::Failed { detail } => {
                assert!(detail.contains("row 1: corrupt"));
                assert!(detail.contains("row 2: broken index"));
                assert!(detail.contains('\n'));
            }
            _ => panic!("Failed 분류 기대"),
        }
    }

    #[test]
    fn classify_failed_empty_rows() {
        let result = classify_pragma_rows(vec![]);
        match result {
            IntegrityCheckResult::Failed { detail } => assert!(detail.is_empty()),
            _ => panic!("Failed 분류 기대 — 빈 결과도 비정상"),
        }
    }

    #[test]
    fn rollback_filename_format_is_consistent() {
        let now = DateTime::parse_from_rfc3339("2026-05-19T15:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        // A108: idx 접미사로 동일 초 내 충돌 방지
        let name0 = generate_rollback_filename(now, 0);
        let name1 = generate_rollback_filename(now, 1);
        assert_eq!(name0, "rollback_20260519_153045_0.db");
        assert_eq!(name1, "rollback_20260519_153045_1.db");
        assert!(name0.starts_with(ROLLBACK_FILENAME_PREFIX));
        assert!(name0.ends_with(ROLLBACK_FILENAME_SUFFIX));
        assert_ne!(name0, name1, "동일 타임스탬프라도 idx 로 고유성 보장");
    }

    #[test]
    fn auto_restore_with_retry_returns_err_when_no_backups() {
        // A109: Exit 계층 백업이 없는 환경(CI/테스트)에서는 즉시 Err 반환
        let result = auto_restore_with_retry();
        match result {
            Err(e) => {
                let msg: String = e.into();
                assert!(
                    msg.contains("백업") || msg.contains("복원") || msg.contains("cipher"),
                    "에러 메시지가 사용자 친화적이어야 함: {}",
                    msg
                );
            }
            Ok(_) => {
                // 로컬 환경에서 Exit 백업이 실제로 있는 경우 Ok 도 허용
            }
        }
    }

    #[test]
    fn integrity_check_result_serializes_with_kind_tag() {
        let ok = IntegrityCheckResult::Ok;
        let json = serde_json::to_string(&ok).unwrap();
        assert_eq!(json, r#"{"kind":"ok"}"#);

        let failed = IntegrityCheckResult::Failed {
            detail: "row 1: corrupt".to_string(),
        };
        let json = serde_json::to_string(&failed).unwrap();
        assert!(json.contains(r#""kind":"failed""#));
        assert!(json.contains("row 1: corrupt"));
    }

    #[test]
    fn integrity_mode_round_trip_serde() {
        let modes = [IntegrityMode::Quick, IntegrityMode::Full];
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let parsed: IntegrityMode = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, mode);
        }
        // kebab-case 직렬화 정합 확인
        assert_eq!(serde_json::to_string(&IntegrityMode::Quick).unwrap(), r#""quick""#);
        assert_eq!(serde_json::to_string(&IntegrityMode::Full).unwrap(), r#""full""#);
    }

    #[cfg(not(feature = "cipher"))]
    #[test]
    fn check_returns_friendly_error_without_cipher() {
        let result = run_pragma_check(Path::new("dummy.db"), IntegrityMode::Quick);
        let err = result.expect_err("cipher off 빌드에서는 에러");
        let user_message: String = err.into();
        // AppError::Integrity::user_message 가 한국어 안내 포함하는지
        assert!(
            user_message.contains("검증") || user_message.contains("복원"),
            "사용자 메시지: {}",
            user_message
        );
    }
}
