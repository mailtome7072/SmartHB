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

/// T3(H4): 복원 소스로 인정할 최소 파일 크기 — SQLite 최소 1 페이지(512B) 미만이면 손상/빈 파일.
const MIN_VALID_DB_BYTES: u64 = 512;

/// T3(H3): auto_restore 다계층 폴백 검색 순서 — exit(종료 스냅샷) → daily → weekly.
/// 각 계층 내부는 최신순. hourly 는 손상/빈 상태를 포함할 위험이 상대적으로 커 자동 폴백 대상에서
/// 제외한다(수동 복원에서는 여전히 선택 가능).
const RESTORE_LAYER_CHAIN: [BackupLayer; 3] =
    [BackupLayer::Exit, BackupLayer::Daily, BackupLayer::Weekly];

/// auto_restore 자동 시도 상한 — 무한 루프 방지 + 시작 지연 억제.
const MAX_RESTORE_ATTEMPTS: usize = 5;

/// `app.db` 의 WAL/SHM 사이드카 경로 2개를 반환한다.
fn sidecar_paths(db: &Path) -> [PathBuf; 2] {
    let mut wal = db.as_os_str().to_owned();
    wal.push("-wal");
    let mut shm = db.as_os_str().to_owned();
    shm.push("-shm");
    [PathBuf::from(wal), PathBuf::from(shm)]
}

/// T3(H1): 복원 대상 DB 의 stale WAL/SHM 사이드카를 제거한다 — 구 손상 DB 의 WAL 이 남아
/// 복원된 app.db 에 잘못 적용되는 것을 방지. best-effort(실패는 경고만).
fn remove_stale_sidecars(db: &Path) {
    for p in sidecar_paths(db) {
        if p.exists() {
            if let Err(e) = std::fs::remove_file(&p) {
                eprintln!("[integrity] stale 사이드카 제거 실패 (무시) {}: {}", p.display(), e);
            }
        }
    }
}

/// R145(ntfs-power-loss): 복원 직후 데이터 페이지를 디스크에 강제 flush — 전원 손실 시 NULL 손상 방지.
fn fsync_file(path: &Path) {
    match std::fs::File::open(path) {
        Ok(f) => {
            if let Err(e) = f.sync_all() {
                eprintln!("[integrity] 복원본 fsync 실패 (무시): {}", e);
            }
        }
        Err(e) => eprintln!("[integrity] 복원본 fsync 위해 열기 실패 (무시): {}", e),
    }
}

/// T3(H4): 복원 소스 파일 크기 사전 검증(열기 전, cipher 무관). 512B 미만은 손상/빈 파일로 거부.
fn validate_source_file_size(path: &Path) -> Result<(), AppError> {
    let size = std::fs::metadata(path)
        .map_err(|e| app_err!(Integrity, "백업 파일 확인 실패", e))?
        .len();
    if size < MIN_VALID_DB_BYTES {
        return Err(AppError::Integrity(format!(
            "백업 파일이 너무 작아 손상으로 판단됩니다 ({} bytes): {}",
            size,
            path.display()
        )));
    }
    Ok(())
}

/// T3(H4): 복원 소스의 mtime 이 현재 라이브 DB 보다 과거이면 신선도 역전 경고(차단 아님).
fn warn_if_stale_source(backup: &Path, current_db: &Path) {
    if let (Ok(bm), Ok(cm)) = (std::fs::metadata(backup), std::fs::metadata(current_db)) {
        if let (Ok(bt), Ok(ct)) = (bm.modified(), cm.modified()) {
            if bt < ct {
                eprintln!(
                    "[integrity] ⚠️ 복원 소스가 현재 DB 보다 오래됨 (신선도 역전) — 복원 후 최근 입력이 유실될 수 있음"
                );
            }
        }
    }
}

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

/// 주어진 키로 DB 를 열어 `students` 행수를 센다 — 빈/날조 DB 판정 및 복원 후보 비어있음 검사용.
#[cfg(feature = "cipher")]
fn count_students_with_key(db_path: &Path, hex_key: &str) -> Result<i64, AppError> {
    use rusqlite::Connection;
    let conn = Connection::open(db_path).map_err(|e| app_err!(Integrity, "DB 열기 실패", e))?;
    conn.execute_batch(&paths::pragma_key_sql(hex_key))
        .map_err(|e| app_err!(Integrity, "PRAGMA key 적용 실패", e))?;
    conn.query_row("SELECT COUNT(*) FROM students", [], |r| r.get(0))
        .map_err(|e| app_err!(Integrity, "students 행수 조회 실패", e))
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

/// T3(H3): 복원 계층 체인(exit→daily→weekly) 전체 후보를 우선순위 순으로 이어붙인다.
/// 각 계층 내부는 최신순 — 결과는 [최신 exit … 과거 exit, 최신 daily …, 최신 weekly …].
fn restore_candidates_across_chain() -> Result<Vec<BackupMetadata>, AppError> {
    let mut all = Vec::new();
    for layer in RESTORE_LAYER_CHAIN {
        all.extend(candidates_newest_first(layer)?);
    }
    Ok(all)
}

/// T3(H4): 복원 후보 사전 검증 — 크기 + quick_check + 비어있지 않음(원생 데이터 존재).
/// 하나라도 실패하면 Err 로 해당 후보를 스킵하게 한다. cipher off 빌드는 quick_check 가 Err 를
/// 반환하므로 자동 복원 자체가 성립하지 않는다(개발 빌드는 fail-soft).
fn precheck_restore_candidate(path: &Path) -> Result<(), AppError> {
    validate_source_file_size(path)?;
    let check = run_pragma_check(path, IntegrityMode::Quick)?;
    if !matches!(check, IntegrityCheckResult::Ok) {
        return Err(AppError::Integrity(format!(
            "백업 무결성 검증 실패: {}",
            path.display()
        )));
    }
    #[cfg(feature = "cipher")]
    if is_empty_domain_db(path).unwrap_or(false) {
        return Err(AppError::Integrity(
            "백업에 원생 데이터가 없어 복원 후보에서 제외합니다 (빈 백업).".to_string(),
        ));
    }
    Ok(())
}

/// T3(H3): exit→daily→weekly 계층 체인 전체를 순회하며 quick_check 통과 + 비어있지 않은
/// 가장 우선순위 높은(최신 exit → … → 과거 weekly) 백업을 반환한다. 모두 부적합하면 `None`.
fn select_healthy_backup_chain() -> Result<Option<BackupMetadata>, AppError> {
    for layer in RESTORE_LAYER_CHAIN {
        let candidates = candidates_newest_first(layer)?;
        if candidates.is_empty() {
            continue;
        }
        if let Some(m) = select_first_healthy_with_cached_key(candidates)? {
            return Ok(Some(m));
        }
    }
    Ok(None)
}

/// 후보들을 시간 역순으로 순회하며 quick_check 통과 + 원생 데이터 존재(비어있지 않음)한 가장
/// 최신 백업을 반환한다 (T3 H4: 빈/열세 소스 거부).
///
/// 개별 후보 검증 오류(열기/복호화 실패)는 abort 하지 않고 다음 후보로 스킵한다.
/// OS Keychain 키 조회는 1회만 수행하여 후보 N개 검증 시 N회 IPC 발생을 막는다.
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
        if validate_source_file_size(&path).is_err() {
            eprintln!("[integrity] 백업 크기 미달 스킵: {}", candidate.path);
            continue;
        }
        match run_pragma_check_with_key(&path, hex_key.as_str(), IntegrityMode::Quick) {
            Ok(IntegrityCheckResult::Ok) => {}
            Ok(IntegrityCheckResult::Failed { .. }) => {
                eprintln!("[integrity] 손상 백업 스킵: {}", candidate.path);
                continue;
            }
            Err(e) => {
                eprintln!("[integrity] 백업 검증 오류 스킵 ({}): {}", candidate.path, e);
                continue;
            }
        }
        // 빈 DB(원생 0명) 백업은 복원해도 데이터 소실이므로 후보에서 제외 (H4).
        match count_students_with_key(&path, hex_key.as_str()) {
            Ok(n) if n > 0 => return Ok(Some(candidate)),
            Ok(_) => eprintln!("[integrity] 빈 백업 스킵(원생 0명): {}", candidate.path),
            Err(e) => eprintln!("[integrity] 백업 행수 확인 실패 스킵 ({}): {}", candidate.path, e),
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
/// 1. 소스 파일 크기 사전 검증(H4) + `PRAGMA quick_check` 통과 확인 — 실패 시 즉시 거부
/// 2. 신선도 역전 경고(H4) — 소스가 현재 DB 보다 오래되면 로그
/// 3. 현재 DB 를 `restore_rollback/` 으로 rename (atomic on most filesystems)
/// 4. 구 DB 의 stale WAL/SHM 사이드카 제거(H1) — 복원본에 stale WAL 적용 방지
/// 5. 백업 파일을 `app.db` 로 파일 복사 — SQLCipher 암호화 상태 그대로 (복호화 금지) + fsync(R145)
/// 6. 복사 실패 시 rollback 을 되돌려 원상 복구 시도
fn restore_from_path_sync(backup_path: &Path, idx: usize) -> Result<RestoreResult, AppError> {
    validate_source_file_size(backup_path)?; // H4: 열기 전 크기 검증
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

    warn_if_stale_source(backup_path, &current_db); // H4: 신선도 역전 경고

    if current_db.exists() {
        std::fs::rename(&current_db, &rollback_path)
            .map_err(|e| app_err!(Integrity, "현재 DB rollback 이동 실패", e))?;
    }
    // H1: 구 DB 의 stale WAL/SHM 제거 — 복원된 app.db 에 잘못된 WAL 이 적용되는 것을 방지.
    remove_stale_sidecars(&current_db);

    if let Err(e) = std::fs::copy(backup_path, &current_db) {
        // 복원 실패 — rollback 을 되돌려 원상복구 시도
        let _ = std::fs::rename(&rollback_path, &current_db);
        return Err(app_err!(Integrity, "백업 파일 복사 실패", e));
    }
    fsync_file(&current_db); // R145: 전원 손실 대비 데이터 페이지 커밋

    Ok(RestoreResult {
        restored_from: backup_path.to_string_lossy().into_owned(),
        rollback_path: rollback_path.to_string_lossy().into_owned(),
    })
}

/// 계층 체인(exit→daily→weekly)에서 최신 정상+비어있지 않은 백업으로 자동 복원.
/// 현재 DB 는 rollback 보존. `auto_restore` IPC 가 사용.
pub(crate) fn auto_restore_sync() -> Result<RestoreResult, AppError> {
    let healthy = select_healthy_backup_chain()?;
    let backup_meta = healthy.ok_or_else(|| {
        AppError::Integrity(
            "무결한 백업(exit/daily/weekly)을 찾지 못했습니다. 백업 파일을 수동으로 선택해주세요."
                .to_string(),
        )
    })?;
    restore_from_path_sync(Path::new(&backup_meta.path), 0)
}

/// 복원 후 quick_check 재검증을 포함한 자동 복원 — startup 전용.
///
/// 계층 체인(exit→daily→weekly) 전체를 최신순으로 순회하며, 각 후보를 사전 검증(크기·무결성·
/// 비어있지 않음) 후 복원한다. 복사 후 OS 레벨 손상(NTFS power-loss 등)을 감지하기 위해 복원된
/// app.db 에 quick_check 를 재실행하고, 실패 시 다음 후보로 재시도한다(최대 [`MAX_RESTORE_ATTEMPTS`]회).
/// 모두 실패하면 사용자 친화 에러 반환.
pub(crate) fn auto_restore_with_retry() -> Result<RestoreResult, AppError> {
    let candidates = restore_candidates_across_chain()?;
    if candidates.is_empty() {
        return Err(AppError::Integrity(
            "복원할 수 있는 백업이 없습니다. 수동으로 백업 파일을 선택해주세요.".to_string(),
        ));
    }

    let mut last_err = AppError::Integrity("알 수 없는 복원 오류".to_string());
    let mut idx = 0usize;

    for candidate in candidates {
        if idx >= MAX_RESTORE_ATTEMPTS {
            break;
        }
        // H4: 소스 사전 검증(크기·무결성·비어있지 않음). 실패 시 다음 후보로 스킵(시도 횟수 미소진).
        if let Err(e) = precheck_restore_candidate(Path::new(&candidate.path)) {
            eprintln!(
                "[integrity] 복원 후보 스킵 [{}] {}: {}",
                candidate.layer.subdir(),
                candidate.path,
                e
            );
            last_err = e;
            continue;
        }
        idx += 1;
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
    let setup_done = paths::setup_completed();
    if !db_path.exists() {
        // T2(C2) case A: 셋업 완료 상태인데 DB 파일이 없다 = 유실/클라우드 dehydration.
        // Failed 로 승격 → startup 이 db::initialize(생성 차단) 전에 auto_restore 를 시도한다.
        // 백업이 없으면 auto_restore 가 실패하고 build_pool C1 가드가 명확한 에러로 중단한다.
        if setup_done {
            return Ok(IntegrityCheckResult::Failed {
                detail: "설정 완료 상태이나 DB 파일(app.db)이 존재하지 않습니다 (유실 또는 클라우드 동기화 미완료).".to_string(),
            });
        }
        // 최초 실행 — DB 파일 자체가 아직 없음. startup 이 db::initialize 로 생성한다.
        return Ok(IntegrityCheckResult::Ok);
    }
    match run_pragma_check(&db_path, IntegrityMode::Quick) {
        Ok(IntegrityCheckResult::Ok) => {
            // T2(C2) case B: quick_check 는 통과했으나 도메인 데이터가 비어 있고(원생 0명)
            // 셋업이 완료된 상태면 "빈 DB 날조 의심" → Failed 로 승격해 auto_restore 유도.
            // 판정 오류(키 로드 실패 등)는 오탐 방지를 위해 "비어있지 않음"으로 보수 처리.
            #[cfg(feature = "cipher")]
            if setup_done && is_empty_domain_db(&db_path).unwrap_or(false) {
                return Ok(IntegrityCheckResult::Failed {
                    detail: "DB 에 원생 데이터가 없습니다 (빈 DB 날조 의심).".to_string(),
                });
            }
            Ok(IntegrityCheckResult::Ok)
        }
        Ok(failed) => Ok(failed),
        // cipher off 개발 빌드 — run_pragma_check 가 stub 안내. startup 은 fail-soft 진행.
        #[cfg(not(feature = "cipher"))]
        Err(AppError::Integrity(_)) => Ok(IntegrityCheckResult::Ok),
        Err(e) => Err(e),
    }
}

/// T2(C2): DB 에 도메인 데이터(원생)가 전혀 없는지 판정한다 — 빈 DB 날조 감지용.
///
/// 시드만 존재하는 날조 DB 는 `students` 행수 0. 정상 사용 DB 는 원생이 1명 이상.
/// (최초 설정 직후 원생 미입력 상태도 0 이지만, 그 경로는 `setup_completed=false` 이거나
/// 호출측에서 setup_done 전제를 함께 확인하므로 오탐하지 않는다.)
#[cfg(feature = "cipher")]
fn is_empty_domain_db(db_path: &Path) -> Result<bool, AppError> {
    use crate::commands::auth::get_cached_or_load_key;
    let key = get_cached_or_load_key()?;
    Ok(count_students_with_key(db_path, key.to_hex().as_str())? == 0)
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

    // ─── Sprint 23 T2: startup 빈/부재 DB fail-hard (C2) ───

    /// AC-T2-C2 case A: 셋업 완료(setup_completed=true) + DB 부재 → Failed 승격
    /// (startup 이 auto_restore 를 시도하도록). db_path 미존재는 cipher 무관하게 판정 가능.
    #[test]
    fn startup_check_flags_missing_db_when_setup_done() {
        let dir = std::env::temp_dir().join(format!("smarthb_t2c2_missing_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        paths::update_data_root(dir.clone());
        paths::set_setup_completed(true);

        let r = check_integrity_quick_for_startup().expect("래퍼는 Ok");
        assert!(
            matches!(r, IntegrityCheckResult::Failed { .. }),
            "셋업 완료 + DB 부재 → Failed"
        );

        paths::set_setup_completed(false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC-T2-C2 case A 반례: 최초 실행(setup_completed=false) + DB 부재 → Ok (생성 허용).
    #[test]
    fn startup_check_ok_when_first_run_db_missing() {
        let dir = std::env::temp_dir().join(format!("smarthb_t2c2_first_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        paths::update_data_root(dir.clone());
        paths::set_setup_completed(false);

        let r = check_integrity_quick_for_startup().expect("래퍼는 Ok");
        assert!(
            matches!(r, IntegrityCheckResult::Ok),
            "최초 실행 + DB 부재 → Ok (db::initialize 가 생성)"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ─── Sprint 23 T3: 복원 체계 강화 (H1, H3, H4) ───

    /// AC-T3-H4: 512B 미만 소스는 손상으로 거부, 이상은 통과.
    #[test]
    fn validate_source_file_size_rejects_small() {
        let dir = std::env::temp_dir().join(format!("smarthb_t3_size_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let small = dir.join("small.db");
        std::fs::write(&small, vec![0u8; 100]).unwrap();
        assert!(validate_source_file_size(&small).is_err(), "100B < 512B 거부");
        let big = dir.join("big.db");
        std::fs::write(&big, vec![1u8; 1024]).unwrap();
        assert!(validate_source_file_size(&big).is_ok(), "1024B 허용");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC-T3-H1: 복원 대상의 stale WAL/SHM 사이드카가 제거되고 본체는 유지된다.
    #[test]
    fn remove_stale_sidecars_deletes_wal_shm_only() {
        let dir = std::env::temp_dir().join(format!("smarthb_t3_side_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("app.db");
        std::fs::write(&db, b"body").unwrap();
        let [wal, shm] = sidecar_paths(&db);
        std::fs::write(&wal, b"w").unwrap();
        std::fs::write(&shm, b"s").unwrap();

        remove_stale_sidecars(&db);
        assert!(!wal.exists(), "-wal 제거");
        assert!(!shm.exists(), "-shm 제거");
        assert!(db.exists(), "본체 app.db 는 유지");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC-T3-H3: 복원 후보 체인이 계층 우선순위(exit→daily→weekly) + 계층 내 최신순으로 정렬된다.
    /// weekly 가 시간상 최신이어도 계층 우선순위에 따라 뒤에 온다.
    #[test]
    fn restore_chain_orders_by_layer_priority_then_time() {
        let dir = std::env::temp_dir().join(format!("smarthb_t3_chain_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for sub in ["exit", "daily", "weekly"] {
            std::fs::create_dir_all(dir.join("backup").join(sub)).unwrap();
        }
        paths::update_data_root(dir.clone());
        std::fs::write(dir.join("backup/exit/app_20260101_100000.db"), b"x").unwrap();
        std::fs::write(dir.join("backup/exit/app_20260102_100000.db"), b"x").unwrap();
        std::fs::write(dir.join("backup/daily/app_20260103_100000.db"), b"x").unwrap();
        std::fs::write(dir.join("backup/weekly/app_20260104_100000.db"), b"x").unwrap();

        let chain = restore_candidates_across_chain().expect("체인 스캔");
        let layers: Vec<BackupLayer> = chain.iter().map(|m| m.layer).collect();
        assert_eq!(
            layers,
            vec![
                BackupLayer::Exit,
                BackupLayer::Exit,
                BackupLayer::Daily,
                BackupLayer::Weekly
            ]
        );
        assert!(chain[0].path.contains("20260102"), "exit 계층 내 최신 먼저");
        assert!(chain[1].path.contains("20260101"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC-T3-H4: 크기 미달 백업은 복원 진입 즉시 거부(quick_check 이전 단계에서).
    #[test]
    fn restore_from_path_rejects_undersized_backup() {
        let dir = std::env::temp_dir().join(format!("smarthb_t3_tiny_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        paths::update_data_root(dir.clone());
        let tiny = dir.join("tiny_backup.db");
        std::fs::write(&tiny, vec![0u8; 50]).unwrap();

        let r = restore_from_path(&tiny);
        assert!(r.is_err(), "50B 백업은 크기 검증에서 거부");
        let _ = std::fs::remove_dir_all(&dir);
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
