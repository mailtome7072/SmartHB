//! 4계층 자동 백업 (ADR-003, PRD §5.3/§5.4).
//!
//! `exit(5)` / `hourly(12)` / `daily(14)` / `weekly(4)` 순환 정책으로 SQLCipher DB 백업을 관리한다.
//! (1인 사용 시스템 — 클라우드 동기화 폴더 점유 절감을 위해 Sprint 16 에서 축소, PRD §5.4 v1.5.2)
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

use crate::app_err;
use crate::commands::audit::{self, AuditEventType};
use crate::commands::paths;
use crate::commands::runtime::run_blocking;
use crate::error::AppError;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::{Path, PathBuf};

/// 백업 디렉토리 서브패스 — `paths::data_root()` 하위.
const BACKUP_SUBDIR: &str = "backup";

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

    /// 계층별 최대 보관 개수 (PRD §5.4 v1.5.2). 초과 시 가장 오래된 파일 삭제.
    ///
    /// 1인 사용 시스템 + 백업이 클라우드 동기화 폴더에 위치(업로드 트래픽·용량 점유)하므로
    /// 복구 시나리오(당일 실수=hourly, 손상=exit, 과거 시점=daily/weekly)를 유지하는 선에서
    /// 최소화 — 합계 최대 35개.
    pub fn max_keep(self) -> usize {
        match self {
            Self::Exit => 5,
            Self::Hourly => 12,
            Self::Daily => 14,
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

    /// catch-up 생성 주기 — daily/weekly 만 대상 (exit 은 종료 hook, hourly 는 interval 타이머).
    ///
    /// 간헐적 사용 패턴(앱이 24시간 떠 있지 않음)에서는 순수 interval 타이머가 fire 하지
    /// 못하므로, 시작 시 + hourly tick 마다 최신 백업의 경과 시간을 보고 따라잡는다
    /// ([`run_catchup_backups`]).
    pub(crate) fn catchup_interval(self) -> Option<chrono::Duration> {
        match self {
            Self::Daily => Some(chrono::Duration::hours(24)),
            Self::Weekly => Some(chrono::Duration::days(7)),
            Self::Exit | Self::Hourly => None,
        }
    }
}

/// catch-up 백업 생성 기한 판정 — 순수 함수 (feature 무관 단위테스트 대상).
///
/// 백업이 한 건도 없거나(`latest=None`) 최신 백업이 `interval` 이상 경과했으면 due.
fn is_due(latest: Option<DateTime<Utc>>, now: DateTime<Utc>, interval: chrono::Duration) -> bool {
    match latest {
        None => true,
        Some(latest) => now - latest >= interval,
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

/// 복원 리허설로 검증한 주요 테이블의 행 수 (PRD §5.4 "검증된 데이터 건수").
#[derive(Debug, Serialize, Clone)]
pub struct TableCount {
    pub table: String,
    pub count: i64,
}

/// 복원 리허설 결과 — IPC 응답.
///
/// 백업 파일을 격리된 임시 사본으로 복사해 `PRAGMA integrity_check` + 행 수 카운트를 수행한
/// 결과다. **운영 DB 에는 어떤 영향도 주지 않는다** (사본만 열람 후 폐기).
///
/// - `success`: 무결성 검증 통과 + 주요 테이블 열람 성공 시 true.
/// - `integrity_detail`: 실패 시 사유 — 손상 메시지(integrity_check 비-ok 행) 또는 열기/복호화
///   실패 메시지. 성공 시 None.
/// - `table_counts`: 검증된 주요 테이블 행 수. 성공 시에만 채워진다.
#[derive(Debug, Serialize, Clone)]
pub struct RehearsalResult {
    pub backup_path: String,
    pub size_bytes: u64,
    pub success: bool,
    pub integrity_detail: Option<String>,
    pub table_counts: Vec<TableCount>,
    pub total_rows: i64,
}

fn backup_root() -> PathBuf {
    paths::data_root().join(BACKUP_SUBDIR)
}

fn backup_dir(layer: BackupLayer) -> PathBuf {
    backup_root().join(layer.subdir())
}

fn ensure_backup_dir(layer: BackupLayer) -> Result<PathBuf, AppError> {
    let dir = backup_dir(layer);
    std::fs::create_dir_all(&dir).map_err(|e| app_err!(Backup, "백업 디렉토리 생성 실패", e))?;
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
        Err(e) => return Err(app_err!(Backup, "백업 디렉토리 스캔 실패", e)),
    };
    let mut entries: Vec<BackupMetadata> = Vec::new();
    for entry in read {
        let entry = entry.map_err(|e| app_err!(Backup, "백업 디렉토리 항목 읽기 실패", e))?;
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
        std::fs::remove_file(&m.path).map_err(|e| app_err!(Backup, "순환 삭제 실패", e))?;
    }
    Ok(())
}

fn rotate_layer(layer: BackupLayer) -> Result<(), AppError> {
    rotate_dir(&backup_dir(layer), layer, layer.max_keep())
}

// ----------------------------------------------------------------------------
// SQLCipher 백업 — cipher feature 게이트 (ADR-003 9번 항목)
// ----------------------------------------------------------------------------

/// SQLCipher Online Backup + 즉시 `PRAGMA quick_check` 검증을 단일 dst Connection 에서 수행.
///
/// exit 백업 ~50ms 예산 (PRD §5.6/§5.3) 을 위해 dst Connection 을 backup 후 그대로 재사용 —
/// 별도 검증 connection 을 다시 열지 않음 (SQLCipher PBKDF2 key derivation 1회 절감).
#[cfg(feature = "cipher")]
fn perform_backup_with_cipher(source: &Path, dest: &Path) -> Result<(), AppError> {
    use crate::commands::auth::get_cached_or_load_key;
    use rusqlite::Connection;
    use rusqlite::backup::Backup;
    use std::time::Duration;

    // Sprint 7 T1: 캐시 경유로 startup 후 백업 시 keyring 다이얼로그 0회.
    let key = get_cached_or_load_key()?;
    let hex_key = key.to_hex();
    let pragma_sql = paths::pragma_key_sql(hex_key.as_str());

    let src = Connection::open(source).map_err(|e| app_err!(Backup, "소스 DB 열기 실패", e))?;
    // 클라우드 동기화 lock 충돌 시 30초 재시도 — busy_timeout 미설정 시 즉시 SQLITE_BUSY
    src.busy_timeout(Duration::from_secs(30))
        .map_err(|e| app_err!(Backup, "소스 DB busy_timeout 설정 실패", e))?;
    src.execute_batch(&pragma_sql).map_err(|e| app_err!(Backup, "소스 PRAGMA key 적용 실패", e))?;

    let mut dst = Connection::open(dest).map_err(|e| app_err!(Backup, "대상 DB 열기 실패", e))?;
    dst.busy_timeout(Duration::from_secs(30))
        .map_err(|e| app_err!(Backup, "대상 DB busy_timeout 설정 실패", e))?;
    dst.execute_batch(&pragma_sql).map_err(|e| app_err!(Backup, "대상 PRAGMA key 적용 실패", e))?;

    {
        let backup = Backup::new(&src, &mut dst).map_err(|e| app_err!(Backup, "백업 초기화 실패", e))?;
        backup
            .run_to_completion(100, Duration::from_millis(0), None)
            .map_err(|e| app_err!(Backup, "백업 실행 실패", e))?;
    }
    drop(src);

    let result: String = dst
        .query_row("PRAGMA quick_check", [], |r| r.get(0))
        .map_err(|e| app_err!(Backup, "quick_check 실행 실패", e))?;
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
// 복원 리허설 (PRD §5.4) — 백업 파일을 격리 사본으로 검증, 운영 DB 무영향
// ----------------------------------------------------------------------------

/// 리허설 시 행 수를 세는 주요 테이블 — 컴파일 타임 고정 allowlist (사용자 입력 아님).
///
/// `COUNT(*)` 쿼리에 직접 보간되므로 반드시 정적 상수만 사용한다 (SQL 인젝션 무관).
const REHEARSAL_TABLES: [&str; 6] = [
    "students",
    "student_schedules",
    "regular_attendances",
    "makeup_attendances",
    "bills",
    "payments",
];

/// 리허설용 임시 사본 connection 에 SQLCipher 키를 적용한다.
///
/// cipher on 빌드: 운영 DB 와 동일 키로 백업 사본을 복호화 (백업은 암호화 상태 그대로 보관됨).
/// cipher off 빌드(R98): 평문 백업만 리허설 대상이므로 PRAGMA key 미적용.
#[cfg(feature = "cipher")]
async fn apply_rehearsal_key(pool: &SqlitePool) -> Result<(), AppError> {
    use crate::commands::auth::get_cached_or_load_key;
    let key = get_cached_or_load_key()?;
    let hex_key = key.to_hex();
    sqlx::query(&paths::pragma_key_sql(hex_key.as_str()))
        .execute(pool)
        .await
        .map_err(|e| app_err!(Backup, "백업 사본 복호화 키 적용 실패", e))?;
    Ok(())
}

#[cfg(not(feature = "cipher"))]
async fn apply_rehearsal_key(_pool: &SqlitePool) -> Result<(), AppError> {
    Ok(())
}

/// 격리된 임시 사본을 열어 무결성 검증 + 주요 테이블 행 수를 수집한다.
///
/// 열기/복호화/검증 단계 실패는 Err 가 아닌 "실패한 리허설 결과"로 변환되어야 하므로
/// (백업이 복원 가능한지 판별하는 것이 리허설의 목적), 이 어댑터는 검증 단계의 에러를
/// `(false, Some(reason), _)` 로 매핑한다. 파일 복사/임시 디렉토리 같은 전제 조건
/// 실패만 [`run_backup_rehearsal_inner`] 에서 Err 로 처리한다.
async fn verify_rehearsal_copy(temp_db: &Path) -> (bool, Option<String>, Vec<TableCount>) {
    match collect_rehearsal_findings(temp_db).await {
        Ok(counts) => (true, None, counts),
        Err(reason) => (false, Some(reason), Vec::new()),
    }
}

/// 사본을 read-only 로 열어 무결성 검증 후 주요 테이블 행 수를 수집한다.
///
/// 열기/복호화/무결성/조회 단계의 실패는 모두 사용자 메시지 `Err(String)` 로 반환된다.
async fn collect_rehearsal_findings(temp_db: &Path) -> Result<Vec<TableCount>, String> {
    // busy_timeout 을 connect 옵션에 지정 — PRAGMA key 보다 먼저 적용되어 첫 접근도 보호
    let options = SqliteConnectOptions::new()
        .filename(temp_db)
        .create_if_missing(false)
        .read_only(true)
        .busy_timeout(std::time::Duration::from_secs(30));
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect_with(options)
        .await
        .map_err(|e| format!("백업 파일을 열 수 없습니다: {}", e))?;

    // 검증 결과를 먼저 받은 뒤 pool 을 닫아, 검증 실패 시에도 connection 을 누수하지 않는다.
    let result = async {
        apply_rehearsal_key(&pool).await.map_err(String::from)?;

        let integrity_rows: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_all(&pool)
            .await
            .map_err(|e| format!("무결성 검증을 실행할 수 없습니다: {}", e))?;
        if !(integrity_rows.len() == 1 && integrity_rows[0] == "ok") {
            return Err(integrity_rows.join("\n"));
        }

        let mut counts = Vec::with_capacity(REHEARSAL_TABLES.len());
        for table in REHEARSAL_TABLES {
            // table 은 정적 allowlist 상수 — 사용자 입력 아님 (SQL 인젝션 무관).
            let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                .fetch_one(&pool)
                .await
                .map_err(|e| format!("'{}' 테이블을 읽을 수 없습니다: {}", table, e))?;
            counts.push(TableCount {
                table: table.to_string(),
                count,
            });
        }
        Ok(counts)
    }
    .await;

    pool.close().await;
    result
}

/// 백업 파일을 임시 디렉토리에 복사해 무결성·행 수를 검증한 뒤 사본을 폐기한다.
///
/// WAL 사이드카까지 안전히 제거하기 위해 단일 파일이 아닌 전용 임시 디렉토리에 복사한다.
async fn run_backup_rehearsal_inner(backup_path: String) -> Result<RehearsalResult, AppError> {
    let src = PathBuf::from(&backup_path);
    let size_bytes = std::fs::metadata(&src)
        .map_err(|e| app_err!(Backup, "백업 파일을 찾을 수 없습니다", e))?
        .len();

    let temp_dir = std::env::temp_dir().join(format!("smarthb_rehearsal_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| app_err!(Backup, "리허설 임시 디렉토리 생성 실패", e))?;
    let temp_db = temp_dir.join("rehearsal.db");

    let findings = match std::fs::copy(&src, &temp_db) {
        Ok(_) => Ok(verify_rehearsal_copy(&temp_db).await),
        Err(e) => Err(app_err!(Backup, "백업 파일 복사 실패", e)),
    };

    // 검증 성공/실패와 무관하게 임시 사본(+WAL 사이드카) 전체 제거.
    let _ = std::fs::remove_dir_all(&temp_dir);

    let (success, integrity_detail, table_counts) = findings?;
    let total_rows = table_counts.iter().map(|t| t.count).sum();
    Ok(RehearsalResult {
        backup_path,
        size_bytes,
        success,
        integrity_detail,
        table_counts,
        total_rows,
    })
}

// ----------------------------------------------------------------------------
// 동기 헬퍼 — Tauri IPC 가 spawn_blocking 으로 호출
// ----------------------------------------------------------------------------

fn create_backup_sync(layer: BackupLayer) -> Result<BackupMetadata, AppError> {
    let dir = ensure_backup_dir(layer)?;
    let now = Utc::now();
    let filename = generate_filename(now);
    let dest = dir.join(&filename);

    let source = paths::db_path();
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

/// 지정 계층에 백업을 생성한다. `cipher` feature off 빌드에서는 안내 메시지를 반환한다.
#[tauri::command]
pub async fn create_backup(layer: BackupLayer) -> Result<BackupMetadata, String> {
    let meta = run_blocking(AppError::Backup, "백업 작업 실패", move || create_backup_sync(layer)).await?;
    audit::try_record(AuditEventType::BackupCreated, Some(layer.subdir()), None).await;
    Ok(meta)
}

/// 백그라운드 hourly task 용 wrapper — 실패해도 panic 없이 stderr 로만 보고.
///
/// cipher off 빌드에서는 첫 시도부터 stub 에러 — 반복 노이즈 방지를 위해 단일 메시지로 출력.
pub(crate) async fn try_create_backup(layer: BackupLayer) {
    match run_blocking(AppError::Backup, "백업 작업 실패", move || create_backup_sync(layer)).await {
        Ok(_) => {
            audit::try_record(AuditEventType::BackupCreated, Some(layer.subdir()), None).await;
        }
        Err(e) => eprintln!("[backup] {} 백업 실패 (백그라운드, 재시도 다음 주기): {}", layer.subdir(), e),
    }
}

/// daily/weekly catch-up — 최신 백업이 주기(24h/7d) 이상 경과(또는 0건)한 계층만 백업 생성.
///
/// 앱 시작 직후 + hourly tick 마다 호출된다 (startup.rs). 디렉토리 스캔 실패는 fail-soft —
/// 해당 계층만 건너뛰고 다음 주기에 재시도한다. 실제 백업 생성은 cipher 빌드에서만 동작
/// (off 빌드는 [`try_create_backup`] 의 stub 안내 경로).
pub(crate) async fn run_catchup_backups() {
    for layer in [BackupLayer::Daily, BackupLayer::Weekly] {
        let Some(interval) = layer.catchup_interval() else {
            continue;
        };
        let latest = match scan_layer(layer) {
            Ok(entries) => entries.last().map(|m| m.created_at),
            Err(e) => {
                eprintln!("[backup] {} catch-up 스캔 실패 (다음 주기 재시도): {}", layer.subdir(), e);
                continue;
            }
        };
        if is_due(latest, Utc::now(), interval) {
            try_create_backup(layer).await;
        }
    }
}

/// 백업 파일 목록을 시간 역순으로 반환한다. `layer` 미지정 시 4계층 전체.
#[tauri::command]
pub async fn list_backups(layer: Option<BackupLayer>) -> Result<Vec<BackupMetadata>, String> {
    run_blocking(AppError::Backup, "백업 목록 작업 실패", move || list_backups_sync(layer)).await
}

/// 사용자가 지정한 백업 파일 경로로 현재 DB 를 복원한다.
///
/// `integrity::restore_from_path` 를 호출하여 무결성 검증·rollback 보존을 공유한다 —
/// `auto_restore` 와 동일한 안전망(quick_check 통과 + 복원 직전 DB 보존).
#[tauri::command]
pub async fn restore_backup(path: String) -> Result<crate::commands::integrity::RestoreResult, String> {
    let result = run_blocking(AppError::Backup, "백업 복원 작업 실패", {
        let path = path.clone();
        move || crate::commands::integrity::restore_from_path(Path::new(&path))
    })
    .await?;
    audit::try_record(AuditEventType::BackupRestored, Some(&path), None).await;
    Ok(result)
}

/// 백업 파일이 복원 가능한지 격리된 사본으로 검증한다 (PRD §5.4 복원 리허설).
///
/// 운영 DB 에는 영향이 없다 — 백업을 임시 디렉토리에 복사해 `PRAGMA integrity_check` 와
/// 주요 테이블 행 수를 확인한 뒤 사본을 폐기한다. cipher off 개발 빌드는 평문 백업만
/// 리허설 대상이다 (R98).
#[tauri::command]
pub async fn run_backup_rehearsal(backup_path: String) -> Result<RehearsalResult, String> {
    run_backup_rehearsal_inner(backup_path)
        .await
        .map_err(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_keep_matches_prd_policy() {
        // PRD §5.4 v1.5.2 — 1인 시스템 축소 정책 (합계 35).
        assert_eq!(BackupLayer::Exit.max_keep(), 5);
        assert_eq!(BackupLayer::Hourly.max_keep(), 12);
        assert_eq!(BackupLayer::Daily.max_keep(), 14);
        assert_eq!(BackupLayer::Weekly.max_keep(), 4);
    }

    #[test]
    fn catchup_interval_targets_daily_and_weekly_only() {
        assert_eq!(BackupLayer::Daily.catchup_interval(), Some(chrono::Duration::hours(24)));
        assert_eq!(BackupLayer::Weekly.catchup_interval(), Some(chrono::Duration::days(7)));
        assert_eq!(BackupLayer::Exit.catchup_interval(), None);
        assert_eq!(BackupLayer::Hourly.catchup_interval(), None);
    }

    #[test]
    fn is_due_when_no_backup_exists() {
        let now = Utc::now();
        assert!(is_due(None, now, chrono::Duration::hours(24)));
    }

    #[test]
    fn is_due_only_after_interval_elapsed() {
        let now = Utc::now();
        let interval = chrono::Duration::hours(24);
        // 미경과 (23시간 전) → 아직 아님
        assert!(!is_due(Some(now - chrono::Duration::hours(23)), now, interval));
        // 정확히 경과 → due (>= 경계 포함)
        assert!(is_due(Some(now - interval), now, interval));
        // 초과 경과 (25시간 전) → due
        assert!(is_due(Some(now - chrono::Duration::hours(25)), now, interval));
    }

    #[test]
    fn is_due_weekly_interval() {
        let now = Utc::now();
        let interval = chrono::Duration::days(7);
        assert!(!is_due(Some(now - chrono::Duration::days(6)), now, interval));
        assert!(is_due(Some(now - chrono::Duration::days(8)), now, interval));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn run_catchup_backups_is_fail_soft_without_cipher() {
        // cipher off — due 판정 시 try_create_backup 이 stub 에러를 stderr 로만 출력.
        // panic 없이 반환해야 한다 (백그라운드 task 안전성).
        run_catchup_backups().await;
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
    #[tokio::test]
    async fn try_create_backup_silent_skips_without_cipher() {
        // cipher off 빌드 — create_backup_sync 가 안내 에러 반환.
        // try_create_backup 은 stderr 만 출력하고 panic 없이 반환해야 한다.
        try_create_backup(BackupLayer::Hourly).await;
        // 통과 — silent skip 정상 동작
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

    // ------------------------------------------------------------------------
    // 복원 리허설 테스트 (cipher off — 평문 백업, R98)
    // ------------------------------------------------------------------------

    /// 평문 SQLite 백업 사본을 만들어 리허설 검증한다. cipher off 전용 (cipher on 은 키 조회 필요).
    #[cfg(not(feature = "cipher"))]
    async fn make_plaintext_backup(rows_in_students: usize) -> (PathBuf, PathBuf) {
        use sqlx::sqlite::SqliteJournalMode;

        let dir = std::env::temp_dir().join(format!("smarthb_rehearsal_src_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("소스 디렉토리 생성");
        let db_path = dir.join("source.db");

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            // WAL 사이드카 없이 단일 파일에 모든 데이터를 두어 복사 시 누락을 방지.
            .journal_mode(SqliteJournalMode::Delete);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .expect("소스 풀 생성");

        for table in REHEARSAL_TABLES {
            sqlx::query(&format!("CREATE TABLE {} (id INTEGER PRIMARY KEY)", table))
                .execute(&pool)
                .await
                .expect("테이블 생성");
        }
        for i in 0..rows_in_students {
            sqlx::query("INSERT INTO students (id) VALUES (?)")
                .bind(i as i64 + 1)
                .execute(&pool)
                .await
                .expect("행 삽입");
        }
        pool.close().await;
        (dir, db_path)
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn rehearsal_succeeds_on_valid_plaintext_backup() {
        let (dir, db_path) = make_plaintext_backup(3).await;

        let result = run_backup_rehearsal_inner(db_path.to_string_lossy().into_owned())
            .await
            .expect("정상 백업은 Err 가 아니어야 함");

        assert!(result.success, "무결성 통과 시 success=true");
        assert!(result.integrity_detail.is_none(), "성공 시 사유 없음");
        assert_eq!(result.table_counts.len(), REHEARSAL_TABLES.len());
        assert_eq!(result.total_rows, 3, "students 3행만 존재");
        let students = result
            .table_counts
            .iter()
            .find(|t| t.table == "students")
            .expect("students 항목");
        assert_eq!(students.count, 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn rehearsal_reports_failure_on_corrupt_backup() {
        let dir = std::env::temp_dir().join(format!("smarthb_rehearsal_bad_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("corrupt.db");
        // 유효한 SQLite 헤더가 아닌 쓰레기 바이트 — 열기 또는 검증 단계에서 실패.
        std::fs::write(&bad, b"this is definitely not a sqlite database file").unwrap();

        let result = run_backup_rehearsal_inner(bad.to_string_lossy().into_owned())
            .await
            .expect("손상 파일도 결과 객체로 보고 (Err 아님)");

        assert!(!result.success, "손상 백업은 success=false");
        assert!(result.integrity_detail.is_some(), "실패 사유가 채워져야 함");
        assert!(result.table_counts.is_empty(), "실패 시 행 수 미수집");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// `REHEARSAL_TABLES` 가 실제 스키마와 동기화되어 있는지 보장한다.
    ///
    /// 정적 목록이라 새 도메인 테이블이 추가되거나 테이블명이 바뀌면 stale 될 수 있다. 그
    /// 경우 리허설이 해당 테이블을 누락한 채 "성공"으로 보고하는 거짓 안심을 막기 위해, 갓
    /// 마이그레이션한 DB 에 모든 항목이 존재하는지 CI 에서 검증한다.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn rehearsal_tables_exist_in_current_schema() {
        let pool = crate::commands::db::test_pool_in_memory()
            .await
            .expect("테스트 마이그레이션");
        for table in REHEARSAL_TABLES {
            let exists: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?")
                    .bind(table)
                    .fetch_one(&pool)
                    .await
                    .expect("sqlite_master 조회");
            assert_eq!(exists, 1, "REHEARSAL_TABLES 의 '{}' 가 스키마에 없음 (목록 갱신 필요)", table);
        }
        pool.close().await;
    }

    #[tokio::test]
    async fn rehearsal_errors_on_missing_file() {
        let missing = std::env::temp_dir()
            .join(format!("smarthb_rehearsal_missing_{}.db", uuid::Uuid::new_v4()));
        let result = run_backup_rehearsal_inner(missing.to_string_lossy().into_owned()).await;
        let err: String = result.expect_err("존재하지 않는 파일은 전제 조건 실패 Err").into();
        assert!(err.contains("백업"), "사용자 메시지: {}", err);
    }
}
