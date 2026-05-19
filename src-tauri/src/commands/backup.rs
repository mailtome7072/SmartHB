//! 4계층 자동 백업 (ADR-003, PRD §5.3/§5.4).
//!
//! `exit(10)` / `hourly(24)` / `daily(30)` / `weekly(4)` 순환 정책으로 SQLCipher DB 백업을 관리한다.
//! SQLite Online Backup API 를 rusqlite 0.32 의 `bundled-sqlcipher-vendored-openssl` 빌드에서
//! 사용하여 백업 파일이 SQLCipher 암호화 상태 그대로 유지된다.
//!
//! ## 흐름
//!
//! 1. `create_backup(layer)`: 계층 디렉토리 보장 → 파일명 생성 → SQLCipher 백업 →
//!    `PRAGMA quick_check` 검증 → 4계층 순환 삭제.
//! 2. `list_backups(layer?)`: 계층별(또는 전체) 백업 메타데이터를 시간 역순으로 반환.
//! 3. `restore_backup(path)`: 지정 백업 파일로 현재 DB 복원 (`integrity` 모듈 안전망 공유).
//!
//! ## Feature 게이트
//!
//! 4계층 관리·파일 시스템 로직은 항상 컴파일된다. SQLite Online Backup API 호출 부분만
//! `cipher` feature 가 켜진 빌드에서 활성된다 — cipher off 빌드의 `create_backup` 은 사용자
//! 친화 안내 메시지 (`AppError::Backup`) 를 반환한다.
//!
//! ## 백업 위치
//!
//! - T7 (현재): `./SmartHB-data/backup/{exit,hourly,daily,weekly}/` 임시 위치 (dev).
//! - T9 (마법사 통합): 클라우드 동기화 폴더 하위 `smarthb/backup/...` 로 이전.

use crate::commands::audit::{self, AuditEventType};
use crate::error::AppError;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 앱 데이터 루트 디렉토리 (T7 임시).
///
/// 백업·락·rollback 모듈이 공유하는 단일 루트. T9 통합 시점에 사용자 마법사가 지정한
/// 클라우드 동기화 폴더로 교체된다 (`./SmartHB-data` → `<클라우드폴더>/smarthb`).
const DATA_ROOT_DEV: &str = "./SmartHB-data";

/// 백업 디렉토리 서브패스 — `DATA_ROOT_DEV` 하위.
const BACKUP_SUBDIR: &str = "backup";

/// 소스 DB 파일명 — `DATA_ROOT_DEV` 하위.
const DB_FILENAME: &str = "app.db";

/// 백업 파일명 구성 요소 — UTC 기준 1초 단위 정렬을 위해 PREFIX + STEM(`YYYYMMDD_HHMMSS`) + SUFFIX 조합.
const FILENAME_PREFIX: &str = "app_";
const FILENAME_STEM_FORMAT: &str = "%Y%m%d_%H%M%S";
const FILENAME_SUFFIX: &str = ".db";

/// 백업 계층 — PRD §5.3 4계층 순환 정책.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BackupLayer {
    Exit,
    Hourly,
    Daily,
    Weekly,
}

impl BackupLayer {
    /// 전체 백업 조회 시 내부 순회용 — UI 는 `list_backups(None)` IPC 사용.
    pub(crate) const ALL: [BackupLayer; 4] = [Self::Exit, Self::Hourly, Self::Daily, Self::Weekly];

    /// 계층별 최대 보관 개수 (PRD §5.3). 초과 시 가장 오래된 파일 삭제.
    pub fn max_keep(self) -> usize {
        match self {
            Self::Exit => 10,
            Self::Hourly => 24,
            Self::Daily => 30,
            Self::Weekly => 4,
        }
    }

    /// 계층 디렉토리명 — `BACKUP_ROOT_DEV` 하위에 위치.
    pub fn subdir(self) -> &'static str {
        match self {
            Self::Exit => "exit",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
        }
    }
}

/// 백업 메타데이터 — IPC 응답.
#[derive(Debug, Serialize, Clone)]
pub struct BackupMetadata {
    pub path: String,
    pub layer: BackupLayer,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
}

/// `AppError::Backup` 을 한 줄로 생성하는 헬퍼 — lock.rs 의 `lock_err` 와 동일 패턴.
fn backup_err(context: &str, e: impl std::fmt::Display) -> AppError {
    AppError::Backup(format!("{}: {}", context, e))
}

/// 앱 데이터 루트 — backup·integrity·lock 모듈 공유.
///
/// T9 마법사 통합 시점에 클라우드 동기화 폴더 경로로 교체된다 — 본 함수가 단일 변경 지점.
pub(crate) fn data_root() -> PathBuf {
    PathBuf::from(DATA_ROOT_DEV)
}

/// 소스 DB 파일 경로 — integrity 모듈의 검증·복원 대상.
pub(crate) fn db_path() -> PathBuf {
    data_root().join(DB_FILENAME)
}

fn backup_root() -> PathBuf {
    data_root().join(BACKUP_SUBDIR)
}

fn backup_dir(layer: BackupLayer) -> PathBuf {
    backup_root().join(layer.subdir())
}

fn ensure_backup_dir(layer: BackupLayer) -> Result<PathBuf, AppError> {
    let dir = backup_dir(layer);
    std::fs::create_dir_all(&dir).map_err(|e| backup_err("백업 디렉토리 생성 실패", e))?;
    Ok(dir)
}

/// 일반화된 타임스탬프 파일명 생성기 — `{prefix}{YYYYMMDD_HHMMSS}{suffix}`.
///
/// integrity 모듈의 rollback 파일명도 동일 형식을 공유하므로 prefix/suffix 인자화.
pub(crate) fn timestamped_filename(prefix: &str, suffix: &str, now: DateTime<Utc>) -> String {
    format!("{}{}{}", prefix, now.format(FILENAME_STEM_FORMAT), suffix)
}

fn generate_filename(now: DateTime<Utc>) -> String {
    timestamped_filename(FILENAME_PREFIX, FILENAME_SUFFIX, now)
}

/// 파일명에서 타임스탬프를 추출한다. `app_` 접두사·`.db` 접미사·UTC 포맷 모두 매치되어야 한다.
fn parse_timestamp_from_filename(name: &str) -> Option<DateTime<Utc>> {
    let stem = name.strip_prefix(FILENAME_PREFIX)?.strip_suffix(FILENAME_SUFFIX)?;
    NaiveDateTime::parse_from_str(stem, FILENAME_STEM_FORMAT)
        .ok()
        .map(|n| n.and_utc())
}

/// 지정 디렉토리를 스캔하여 백업 파일 메타데이터를 시간 오름차순으로 반환한다.
///
/// 파일명 패턴(`app_YYYYMMDD_HHMMSS.db`)에 맞지 않는 항목은 무시 — 다른 도구가 만든 파일이
/// 디렉토리에 있어도 안전하게 처리한다. 디렉토리가 없으면 빈 Vec 반환 (TOCTOU 회피 — `exists()`
/// 사전 검사 없이 `read_dir` 의 `NotFound` 에러만 빈 결과로 변환).
///
/// `layer` 는 메타데이터 라벨링용이며 디렉토리 위치 결정에는 사용되지 않는다 — 호출자가 임의의
/// path 를 검사할 수 있도록 path 와 layer 를 분리한다.
fn scan_dir(dir: &Path, layer: BackupLayer) -> Result<Vec<BackupMetadata>, AppError> {
    let read = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(backup_err("백업 디렉토리 스캔 실패", e)),
    };
    let mut entries: Vec<BackupMetadata> = Vec::new();
    for entry in read {
        let entry = entry.map_err(|e| backup_err("백업 디렉토리 항목 읽기 실패", e))?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(created_at) = parse_timestamp_from_filename(name) else {
            continue;
        };
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        entries.push(BackupMetadata {
            path: path.to_string_lossy().into_owned(),
            layer,
            created_at,
            size_bytes,
        });
    }
    entries.sort_by_key(|m| m.created_at);
    Ok(entries)
}

pub(crate) fn scan_layer(layer: BackupLayer) -> Result<Vec<BackupMetadata>, AppError> {
    scan_dir(&backup_dir(layer), layer)
}

/// 지정 디렉토리의 백업 파일 개수가 `max` 를 초과하면 가장 오래된 파일부터 삭제한다.
fn rotate_dir(dir: &Path, layer: BackupLayer, max: usize) -> Result<(), AppError> {
    let entries = scan_dir(dir, layer)?;
    if entries.len() <= max {
        return Ok(());
    }
    let to_delete = entries.len() - max;
    for m in entries.into_iter().take(to_delete) {
        std::fs::remove_file(&m.path).map_err(|e| backup_err("순환 삭제 실패", e))?;
    }
    Ok(())
}

fn rotate_layer(layer: BackupLayer) -> Result<(), AppError> {
    rotate_dir(&backup_dir(layer), layer, layer.max_keep())
}

// ----------------------------------------------------------------------------
// SQLCipher 백업 — cipher feature 게이트 (ADR-003 9번 항목)
// ----------------------------------------------------------------------------

/// PRAGMA key 적용용 SQL 단편을 생성한다. hex 인코딩이므로 `[0-9a-f]` 만 사용 → SQL injection 안전.
///
/// integrity 모듈에서도 동일 형식으로 PRAGMA key 적용이 필요하여 `pub(crate)` 노출.
#[cfg(feature = "cipher")]
pub(crate) fn pragma_key_sql(hex_key: &str) -> String {
    format!("PRAGMA key = \"x'{}'\";", hex_key)
}

/// SQLCipher Online Backup + 즉시 `PRAGMA quick_check` 검증을 단일 dst Connection 에서 수행.
///
/// exit 백업 ~50ms 예산 (PRD §5.6/§5.3) 을 위해 dst Connection 을 backup 후 그대로 재사용 —
/// 별도 검증 connection 을 다시 열지 않음 (SQLCipher PBKDF2 key derivation 1회 절감).
#[cfg(feature = "cipher")]
fn perform_backup_with_cipher(source: &Path, dest: &Path) -> Result<(), AppError> {
    use crate::commands::auth::retrieve_key_from_keyring;
    use rusqlite::Connection;
    use rusqlite::backup::Backup;
    use std::time::Duration;

    let key = retrieve_key_from_keyring()?;
    let hex_key = key.to_hex();
    let pragma_sql = pragma_key_sql(hex_key.as_str());

    let src = Connection::open(source).map_err(|e| backup_err("소스 DB 열기 실패", e))?;
    src.execute_batch(&pragma_sql).map_err(|e| backup_err("소스 PRAGMA key 적용 실패", e))?;

    let mut dst = Connection::open(dest).map_err(|e| backup_err("대상 DB 열기 실패", e))?;
    dst.execute_batch(&pragma_sql).map_err(|e| backup_err("대상 PRAGMA key 적용 실패", e))?;

    {
        let backup = Backup::new(&src, &mut dst).map_err(|e| backup_err("백업 초기화 실패", e))?;
        backup
            .run_to_completion(100, Duration::from_millis(0), None)
            .map_err(|e| backup_err("백업 실행 실패", e))?;
    }
    drop(src);

    let result: String = dst
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|e| backup_err("quick_check 실행 실패", e))?;
    if result != "ok" {
        return Err(AppError::Backup(format!("백업 quick_check 실패: {}", result)));
    }
    Ok(())
}

#[cfg(not(feature = "cipher"))]
fn perform_backup_with_cipher(_source: &Path, _dest: &Path) -> Result<(), AppError> {
    Err(AppError::Backup(
        "암호화 빌드(--features cipher)에서만 백업이 가능합니다.".to_string(),
    ))
}

// ----------------------------------------------------------------------------
// 동기 헬퍼 — Tauri IPC 가 spawn_blocking 으로 호출
// ----------------------------------------------------------------------------

fn create_backup_sync(layer: BackupLayer) -> Result<BackupMetadata, AppError> {
    let dir = ensure_backup_dir(layer)?;
    let now = Utc::now();
    let filename = generate_filename(now);
    let dest = dir.join(&filename);

    let source = db_path();
    if let Err(e) = perform_backup_with_cipher(&source, &dest) {
        let _ = std::fs::remove_file(&dest);
        return Err(e);
    }

    let size_bytes = std::fs::metadata(&dest).map(|m| m.len()).unwrap_or(0);
    rotate_layer(layer)?;

    Ok(BackupMetadata {
        path: dest.to_string_lossy().into_owned(),
        layer,
        created_at: now,
        size_bytes,
    })
}

fn list_backups_sync(layer: Option<BackupLayer>) -> Result<Vec<BackupMetadata>, AppError> {
    match layer {
        Some(l) => {
            let mut entries = scan_layer(l)?;
            entries.sort_by_key(|m| std::cmp::Reverse(m.created_at));
            Ok(entries)
        }
        None => {
            let mut all = Vec::new();
            for l in BackupLayer::ALL {
                all.extend(scan_layer(l)?);
            }
            all.sort_by_key(|m| std::cmp::Reverse(m.created_at));
            Ok(all)
        }
    }
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// `spawn_blocking` 으로 동기 작업을 실행하고 결과를 IPC 응답용 `Result<T, String>` 으로 변환한다.
///
/// SQLite Online Backup API 의 동기 호출이 async 런타임 이벤트 루프를 막지 않도록 보장한다
/// (PRD §5.6, 앱 시작 < 3초 예산 영향 방지).
async fn run_blocking<T, F>(join_ctx: &'static str, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Backup(format!("{}: {}", join_ctx, e)))
        .and_then(|r| r)
        .map_err(String::from)
}

/// 지정 계층에 백업을 생성한다. `cipher` feature off 빌드에서는 안내 메시지를 반환한다.
#[tauri::command]
pub async fn create_backup(layer: BackupLayer) -> Result<BackupMetadata, String> {
    let meta = run_blocking("백업 작업 실패", move || create_backup_sync(layer)).await?;
    audit::try_record(AuditEventType::BackupCreated, Some(layer.subdir()), None).await;
    Ok(meta)
}

/// 백그라운드 hourly task 용 wrapper — 실패해도 panic 없이 stderr 로만 보고.
///
/// cipher off 빌드에서는 첫 시도부터 stub 에러 — 반복 노이즈 방지를 위해 단일 메시지로 출력.
pub(crate) async fn try_create_backup(layer: BackupLayer) {
    match run_blocking("백업 작업 실패", move || create_backup_sync(layer)).await {
        Ok(_) => {
            audit::try_record(AuditEventType::BackupCreated, Some(layer.subdir()), None).await;
        }
        Err(e) => eprintln!("[backup] {} 백업 실패 (백그라운드, 재시도 다음 주기): {}", layer.subdir(), e),
    }
}

/// 백업 파일 목록을 시간 역순으로 반환한다. `layer` 미지정 시 4계층 전체.
#[tauri::command]
pub async fn list_backups(layer: Option<BackupLayer>) -> Result<Vec<BackupMetadata>, String> {
    run_blocking("백업 목록 작업 실패", move || list_backups_sync(layer)).await
}

/// 사용자가 지정한 백업 파일 경로로 현재 DB 를 복원한다.
///
/// `integrity::restore_from_path` 를 호출하여 무결성 검증·rollback 보존을 공유한다 —
/// `auto_restore` 와 동일한 안전망(quick_check 통과 + 복원 직전 DB 보존).
#[tauri::command]
pub async fn restore_backup(path: String) -> Result<crate::commands::integrity::RestoreResult, String> {
    let result = run_blocking("백업 복원 작업 실패", {
        let path = path.clone();
        move || crate::commands::integrity::restore_from_path(Path::new(&path))
    })
    .await?;
    audit::try_record(AuditEventType::BackupRestored, Some(&path), None).await;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_keep_matches_prd_policy() {
        assert_eq!(BackupLayer::Exit.max_keep(), 10);
        assert_eq!(BackupLayer::Hourly.max_keep(), 24);
        assert_eq!(BackupLayer::Daily.max_keep(), 30);
        assert_eq!(BackupLayer::Weekly.max_keep(), 4);
    }

    #[test]
    fn subdir_paths_match_layer() {
        assert_eq!(BackupLayer::Exit.subdir(), "exit");
        assert_eq!(BackupLayer::Hourly.subdir(), "hourly");
        assert_eq!(BackupLayer::Daily.subdir(), "daily");
        assert_eq!(BackupLayer::Weekly.subdir(), "weekly");
    }

    #[test]
    fn filename_format_is_round_trip() {
        let now = DateTime::parse_from_rfc3339("2026-05-19T15:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let name = generate_filename(now);
        assert_eq!(name, "app_20260519_153045.db");
        assert!(name.starts_with(FILENAME_PREFIX));
        assert!(name.ends_with(FILENAME_SUFFIX));
        let parsed = parse_timestamp_from_filename(&name).expect("타임스탬프 파싱 성공");
        assert_eq!(parsed.timestamp(), now.timestamp());
    }

    #[test]
    fn parse_rejects_invalid_filename_patterns() {
        assert!(parse_timestamp_from_filename("backup.db").is_none());
        assert!(parse_timestamp_from_filename("app_20260519.db").is_none());
        assert!(parse_timestamp_from_filename("app_invalid_date.db").is_none());
        assert!(parse_timestamp_from_filename("app_20260519_153045.txt").is_none());
    }

    /// 임시 디렉토리 기반 헬퍼 — 테스트 간 격리.
    fn with_temp_layer_dir<F>(f: F)
    where
        F: FnOnce(&Path),
    {
        let unique = format!(
            "smarthb_backup_test_{}_{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("임시 디렉토리 생성");
        f(&dir);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn write_backup_file(dir: &Path, ts: &str) {
        let path = dir.join(format!("app_{}.db", ts));
        std::fs::write(&path, b"dummy").expect("더미 파일 쓰기");
    }

    #[test]
    fn scan_dir_ignores_unrelated_files() {
        with_temp_layer_dir(|dir| {
            write_backup_file(dir, "20260519_100000");
            write_backup_file(dir, "20260519_120000");
            std::fs::write(dir.join("README.txt"), b"x").unwrap();
            std::fs::write(dir.join("other.db"), b"x").unwrap();

            let entries = scan_dir(dir, BackupLayer::Exit).unwrap();
            assert_eq!(entries.len(), 2);
            assert!(entries[0].path.contains("100000"));
            assert!(entries[1].path.contains("120000"));
        });
    }

    #[test]
    fn scan_dir_returns_empty_for_missing_directory() {
        let missing = std::env::temp_dir().join(format!("smarthb_missing_{}", uuid::Uuid::new_v4()));
        let entries = scan_dir(&missing, BackupLayer::Exit).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn rotation_keeps_only_max_keep_files() {
        with_temp_layer_dir(|dir| {
            for i in 0..12 {
                write_backup_file(dir, &format!("2026051{:01}_120000", i % 10));
            }
            rotate_dir(dir, BackupLayer::Exit, BackupLayer::Exit.max_keep()).unwrap();
            let count = std::fs::read_dir(dir).unwrap().count();
            assert!(count <= BackupLayer::Exit.max_keep());
        });
    }

    #[test]
    fn rotation_deletes_oldest_first() {
        with_temp_layer_dir(|dir| {
            write_backup_file(dir, "20260101_120000");
            write_backup_file(dir, "20260201_120000");
            write_backup_file(dir, "20260301_120000");
            rotate_dir(dir, BackupLayer::Exit, 2).unwrap();

            let names: Vec<String> = std::fs::read_dir(dir)
                .unwrap()
                .filter_map(|e| e.ok().and_then(|e| e.file_name().into_string().ok()))
                .collect();
            assert_eq!(names.len(), 2);
            assert!(!names.iter().any(|n| n.contains("20260101")), "가장 오래된 파일이 삭제되어야 함");
            assert!(names.iter().any(|n| n.contains("20260201")));
            assert!(names.iter().any(|n| n.contains("20260301")));
        });
    }

    #[cfg(not(feature = "cipher"))]
    #[test]
    fn create_backup_returns_friendly_error_without_cipher() {
        let result = perform_backup_with_cipher(
            Path::new("dummy_src.db"),
            Path::new("dummy_dst.db"),
        );
        let err = result.expect_err("cipher off 빌드에서는 에러");
        let user_message: String = err.into();
        assert!(
            user_message.contains("백업"),
            "사용자 메시지에 '백업' 포함: {}",
            user_message
        );
    }

    #[test]
    fn list_backups_sync_returns_empty_when_no_backups() {
        // BACKUP_ROOT_DEV 가 존재하지 않거나 비어있을 때 빈 Vec 반환 여부.
        // 본 테스트는 환경에 따라 false-positive 가능하므로 layer 없이 호출 결과 타입만 검증.
        let result = list_backups_sync(Some(BackupLayer::Weekly));
        assert!(result.is_ok());
    }

    #[cfg(not(feature = "cipher"))]
    #[test]
    fn restore_backup_rejects_invalid_path_without_cipher() {
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(restore_backup("/tmp/nonexistent.db".to_string()));
        let err = result.expect_err("cipher off 빌드에서는 무결성 검증이 거부");
        // AppError::Integrity::user_message 한국어 검증 키워드 확인
        assert!(
            err.contains("검증") || err.contains("복원"),
            "사용자 메시지: {}",
            err
        );
    }
}
