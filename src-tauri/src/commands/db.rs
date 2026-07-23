//! SQLite (SQLCipher on cipher build) pool 초기화 + 마이그레이션 실행 (T9).
//!
//! ## lifecycle
//!
//! 1. 앱 시작: pool 미초기화 상태 — 잠금 화면에서 비밀번호 입력 대기.
//! 2. `unlock_db` 통과 후: T10 시작 시퀀스가 [`initialize`] 호출 → keyring 에서 키 조회 →
//!    SqlitePool 생성 → SQLCipher 빌드에서는 `PRAGMA key` 적용 → `sqlx::migrate!` 실행.
//! 3. 이후 audit / sync / 도메인 모듈이 [`pool`] 헬퍼로 pool 참조.
//!
//! ## Feature 게이트
//!
//! - cipher off: 평문 SQLite (개발 빌드) — PRAGMA key 적용 단계 건너뛰기. 마이그레이션은 동일 적용.
//! - cipher on: SQLCipher AES-256, 첫 connection 에서 PRAGMA key 적용 후 마이그레이션.
//!
//! T9 에서는 IPC 노출 없음 — pool 초기화는 T10 startup IPC 가 호출하며, 다른 모듈은 `pool()`
//! 만 호출한다. 본 sprint 단계에서는 audit/sync 모듈이 직접 호출하는 테스트 경로만 활성.
//!
//! T10 통합 전까지 `initialize`/`build_pool`/`configure_connection` 등이 호출되지
//! 않으므로 모듈 전체에 `#[allow(dead_code)]` 를 적용한다.

#![allow(dead_code)]

use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::OnceCell;

/// 전역 SqlitePool — unlock 후 lazy 초기화. 미초기화 상태에서 호출 시 `AppError::Config` 반환.
static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

/// SQLite max_connections — SQLite 는 단일 writer 제한이라 1 로 고정.
///
/// 다중 reader 허용을 위해 WAL 모드를 켜는 것은 T10 startup PRAGMA 설정에서 처리.
const MAX_CONNECTIONS: u32 = 1;

/// connection acquire 대기 최대 시간 — pool 이 busy 할 때 이 시간 내 acquire 못하면 에러.
const ACQUIRE_TIMEOUT_SECS: u64 = 30;
// max_lifetime/idle_timeout 미설정 — SQLite 는 로컬 파일이라 network stale 없음.
// T1(A3) 이전에는 커넥션 교체 시 PRAGMA key(cipher)·busy_timeout 등이 재적용되지 않아
// cipher 빌드에서 idle 후 모든 쿼리가 실패하는 심각한 버그(C3)가 있었다. 이제 after_connect
// 훅([`configure_connection`])이 새 커넥션마다 key + pragma 를 재적용하므로 커넥션 교체가
// 안전하다. 실제 유휴 close/재연결 풀 라이프사이클 관리는 T6 에서 도입.

/// 전역 pool 참조. 미초기화 시 `AppError::Config` — 호출자가 unlock 흐름을 안내한다.
pub(crate) fn pool() -> Result<&'static SqlitePool, AppError> {
    POOL.get()
        .ok_or_else(|| AppError::Config("DB가 아직 잠금 해제되지 않았습니다.".to_string()))
}

/// 본 모듈 외부에서 pool 이 이미 초기화되었는지 확인 — 테스트 / 재진입 방지용.
#[allow(dead_code)]
pub(crate) fn is_initialized() -> bool {
    POOL.get().is_some()
}

/// SQLite 파일 URL 을 생성한다 — `sqlite:///abs/path.db` 형식 (sqlx 요구).
///
/// 절대 경로 변환은 사용자 작업 디렉토리 변경에 안전한 동작을 보장한다.
fn db_url(path: &PathBuf) -> Result<String, AppError> {
    let abs = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| AppError::Config(format!("작업 디렉토리 확인 실패: {}", e)))?
            .join(path)
    };
    let abs_str = abs.to_string_lossy().replace('\\', "/");
    Ok(format!("sqlite:///{}", abs_str.trim_start_matches('/')))
}

/// DB pool 을 초기화한다 — unlock 통과 직후 1회 호출.
///
/// cipher on 빌드는 `PRAGMA key` 적용 후 마이그레이션. off 는 평문 SQLite + 동일 마이그레이션.
/// 이미 초기화되었으면 기존 pool 을 그대로 반환 — 재호출 idempotent.
pub(crate) async fn initialize(db_path: PathBuf) -> Result<&'static SqlitePool, AppError> {
    POOL.get_or_try_init(|| async move { build_pool(db_path).await })
        .await
}

async fn build_pool(db_path: PathBuf) -> Result<SqlitePool, AppError> {
    // T2(C1): 셋업 완료(config.json setup_completed=true) 상태인데 app.db 가 없으면 = 클라우드
    // 동기화 폴더의 DB 가 일시 부재(dehydration)하거나 유실된 것. 빈 DB 를 새로 날조하지 않고
    // 명확한 안내 에러로 중단한다(RCA C1). 최초 설정(마법사)은 DB 생성이 setup_completed=true
    // 전환보다 먼저 일어나므로(setup/page.tsx Step3<Step4) 이 가드에 걸리지 않는다.
    // salt.bin 도 함께 확인하되, setup_completed 가 주 판별자다(salt 가 함께 dehydrate 돼도 안전).
    let setup_done = crate::commands::paths::setup_completed();
    if setup_done && !db_path.exists() {
        let salt_hint = if crate::commands::paths::salt_path().exists() {
            ""
        } else {
            " (설정 파일 salt.bin 도 확인되지 않습니다)"
        };
        return Err(AppError::Config(format!(
            "DB 파일(app.db)이 없습니다. 클라우드 동기화가 완료된 후 다시 시도해 주세요.{}",
            salt_hint
        )));
    }

    if let Some(parent) = db_path.parent() {
        // 셋업 완료 상태에서 데이터 폴더 자체가 없으면(전체 dehydration) 생성하지 않는다.
        if setup_done && !parent.exists() {
            return Err(AppError::Config(
                "데이터 폴더가 없습니다. 클라우드 동기화가 완료된 후 다시 시도해 주세요.".to_string(),
            ));
        }
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Config(format!("DB 디렉토리 생성 실패: {}", e)))?;
    }

    let url = db_url(&db_path)?;
    // 셋업 완료 상태에서는 절대 새 DB 를 만들지 않는다 (위 가드로 부재는 이미 차단됨 — 이중 방어).
    let connect_options = SqliteConnectOptions::from_str(&url)
        .map_err(|e| AppError::Config(format!("DB URL 파싱 실패: {}", e)))?
        .create_if_missing(!setup_done);

    // T1(A3): PRAGMA key(cipher) + startup PRAGMA 를 after_connect 훅으로 이전한다.
    // 풀이 새 커넥션을 열 때마다(유휴 close 후 재연결 포함) 자동 재적용되어 C3(키 유실)·H5 근절.
    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .acquire_timeout(Duration::from_secs(ACQUIRE_TIMEOUT_SECS))
        .after_connect(|conn, _meta| Box::pin(configure_connection(conn)))
        .connect_with(connect_options)
        .await?;

    // 사전 스냅샷 (ADR-011 R140): 기존 DB 에 미적용 마이그레이션이 있으면 migrate 직전에 백업.
    // 백필(V312) 등 데이터 변형 마이그레이션이 커밋 성공 후 오작동해도, 직전 상태 복구본을
    // 남긴다(정상 경로는 마이그레이션 직전 백업이 없었음). cipher off 빌드는 백업이 stub →
    // try_create_backup 이 조용히 no-op(stderr 안내). 실패해도 startup 은 계속 진행(fail-soft).
    let migrator = sqlx::migrate!("./migrations");
    if has_pending_migrations(&migrator, &pool).await {
        crate::commands::backup::try_create_backup(crate::commands::backup::BackupLayer::Exit).await;
    }
    migrator
        .run(&pool)
        .await
        .map_err(|e| AppError::Config(format!("마이그레이션 실행 실패: {}", e)))?;

    Ok(pool)
}

/// 임베드된 마이그레이션 최신 버전이 DB 에 적용된 최신 버전보다 높은지 판정한다 (기존 DB 한정).
///
/// 첫 실행(신규 DB)은 `_sqlx_migrations` 테이블이 없거나 비어 있어 `applied_max=None` →
/// `false`(빈 DB 백업은 무의미). 기존 DB 에 신규 마이그레이션이 대기 중일 때만 `true`.
async fn has_pending_migrations(
    migrator: &sqlx::migrate::Migrator,
    pool: &SqlitePool,
) -> bool {
    let embedded_max = migrator.iter().map(|m| m.version).max().unwrap_or(0);
    let applied_max: Option<i64> = sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .unwrap_or(None);
    matches!(applied_max, Some(v) if embedded_max > v)
}

/// T1(A3) after_connect 훅 — 풀이 새 커넥션을 열 때마다 호출되어 PRAGMA key(cipher) +
/// startup PRAGMA 를 재적용한다. 유휴 후 커넥션 교체(T6) 시에도 키·pragma 가 유지된다.
///
/// PRAGMA 설명 (PRD §5.6 시작 < 3초 예산):
/// - `journal_mode=WAL`: 동시 reader 허용 + 쓰기 latency 감소
/// - `cache_size=-8000`: 8MB 페이지 캐시 (음수는 KiB 단위)
/// - `foreign_keys=ON`: SQLite 기본값 OFF — 외래키 제약 강제
/// - `busy_timeout=30000`: 클라우드 동기화/백업 lock 충돌 시 최대 30초 재시도
/// - `journal_size_limit`: WAL 파일 상한 64MB — 클라우드 동기화 부하 제어
///
/// 반환 타입이 `sqlx::Error` 인 이유: after_connect 콜백 시그니처 요구. 키 로드 실패(AppError)는
/// `sqlx::Error::Configuration` 으로 매핑한다.
async fn configure_connection(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    // PRAGMA key 는 커넥션 확립 직후, 다른 어떤 데이터 접근 쿼리보다 먼저 적용되어야 한다 (SQLCipher).
    apply_cipher_key_conn(conn).await?;
    sqlx::query("PRAGMA journal_mode=WAL").execute(&mut *conn).await?;
    sqlx::query("PRAGMA cache_size=-8000").execute(&mut *conn).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&mut *conn).await?;
    sqlx::query("PRAGMA busy_timeout=30000").execute(&mut *conn).await?;
    sqlx::query("PRAGMA journal_size_limit=67108864").execute(&mut *conn).await?;
    Ok(())
}

#[cfg(feature = "cipher")]
async fn apply_cipher_key_conn(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    use crate::commands::auth::get_cached_or_load_key;
    use crate::commands::paths::pragma_key_sql;

    // Sprint 7 T1: 캐시 경유로 keyring 호출 1회로 통합 (verify_password 가 이미 채워둔 캐시 hit).
    let key = get_cached_or_load_key()
        .map_err(|e| sqlx::Error::Configuration(String::from(e).into()))?;
    let hex_key = key.to_hex();
    sqlx::query(&pragma_key_sql(hex_key.as_str())).execute(&mut *conn).await?;
    Ok(())
}

#[cfg(not(feature = "cipher"))]
async fn apply_cipher_key_conn(_conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    // 평문 SQLite (개발 빌드) — PRAGMA key 적용 없음.
    Ok(())
}

/// 테스트 전용 — 인메모리 SqlitePool 을 만들고 마이그레이션을 적용한다.
///
/// 전역 `POOL` 을 건드리지 않으므로 테스트 간 격리. cipher feature off 에서만 의미 있음
/// (인메모리 DB 는 SQLCipher 적용 불가).
#[cfg(all(test, not(feature = "cipher")))]
pub(crate) async fn test_pool_in_memory() -> Result<SqlitePool, AppError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Config(format!("테스트 마이그레이션 실패: {}", e)))?;
    Ok(pool)
}

/// 테스트 전용 — 실제 프로덕션과 동일한 after_connect 훅([`configure_connection`])을 붙인
/// 파일 기반 풀을 만든다. `max_lifetime` 을 짧게 지정하면 acquire 시 커넥션이 교체되어
/// 재연결 시 PRAGMA 재적용을 검증할 수 있다 (cipher off 에서는 key 이외 pragma 만 적용).
#[cfg(all(test, not(feature = "cipher")))]
pub(crate) async fn test_pool_with_after_connect(
    db_path: &std::path::Path,
    max_lifetime: Option<Duration>,
) -> Result<SqlitePool, AppError> {
    let url = db_url(&db_path.to_path_buf())?;
    let connect_options = SqliteConnectOptions::from_str(&url)
        .map_err(|e| AppError::Config(format!("DB URL 파싱 실패: {}", e)))?
        .create_if_missing(true);
    let mut builder = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .min_connections(0);
    if let Some(lifetime) = max_lifetime {
        builder = builder.max_lifetime(lifetime);
    }
    let pool = builder
        .after_connect(|conn, _meta| Box::pin(configure_connection(conn)))
        .connect_with(connect_options)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_url_handles_relative_path() {
        let url = db_url(&PathBuf::from("test.db")).expect("URL 생성");
        assert!(url.starts_with("sqlite:///"));
        assert!(url.ends_with("test.db"));
    }

    #[test]
    fn db_url_handles_absolute_path() {
        #[cfg(windows)]
        let path = PathBuf::from("C:/temp/test.db");
        #[cfg(not(windows))]
        let path = PathBuf::from("/tmp/test.db");

        let url = db_url(&path).expect("URL 생성");
        assert!(url.starts_with("sqlite:///"));
        assert!(url.ends_with("test.db"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn in_memory_pool_runs_migrations() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool 생성");
        // V001 코드 테이블 적용 확인
        let count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM payment_methods")
            .fetch_one(&pool)
            .await
            .expect("payment_methods 테이블 조회 성공");
        assert!(count.0 >= 4, "payment_methods 시드 ≥ 4개: {}", count.0);

        let card_count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM card_companies")
            .fetch_one(&pool)
            .await
            .expect("card_companies 조회 성공");
        assert!(card_count.0 >= 10, "card_companies 시드 ≥ 10개: {}", card_count.0);

        // V104 적용 후 standard_fees 는 주 수업시간별 모델 (V001 학년별 모델 폐기, data-model §5.1)
        // V201(Sprint 5 T3)에서 2시간 행 삭제 → 3~6시간 4건으로 변경
        let fee_count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM standard_fees")
            .fetch_one(&pool)
            .await
            .expect("standard_fees 조회 성공");
        assert_eq!(fee_count.0, 4, "주 3~6시간 시드 4건 (V201: 2시간 삭제)");

        let fee_columns: Vec<(String,)> = sqlx::query_as("PRAGMA table_info(standard_fees)")
            .fetch_all(&pool)
            .await
            .expect("schema 조회 성공")
            .into_iter()
            .map(|r: (i32, String, String, i32, Option<String>, i32)| (r.1,))
            .collect();
        let col_names: Vec<&str> = fee_columns.iter().map(|c| c.0.as_str()).collect();
        assert!(col_names.contains(&"weekly_hours"), "V104 schema 컬럼 weekly_hours");
        assert!(col_names.contains(&"amount"), "V104 schema 컬럼 amount");
        assert!(!col_names.contains(&"grade_code"), "V001 학년별 컬럼은 폐기");

        // V008 audit_logs 적용 확인
        let audit_count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&pool)
            .await
            .expect("audit_logs 테이블 조회 성공");
        assert_eq!(audit_count.0, 0, "audit_logs 초기 상태 빈 테이블");

        // app_settings UNIQUE 제약 검증
        sqlx::query("INSERT INTO app_settings (key, value) VALUES ('k', 'v')")
            .execute(&pool)
            .await
            .expect("첫 삽입 성공");
        let dup = sqlx::query("INSERT INTO app_settings (key, value) VALUES ('k', 'v2')")
            .execute(&pool)
            .await;
        assert!(dup.is_err(), "동일 key 중복 삽입은 UNIQUE 제약 위반");
    }

    #[test]
    fn pool_uninitialized_returns_friendly_error() {
        // 본 테스트는 다른 test 가 POOL 을 초기화하지 않았다는 전제가 필요 — 단위 테스트 격리상
        // 항상 보장되지는 않으므로 미초기화 케이스만 검증.
        if !is_initialized() {
            let result = pool();
            let err = result.expect_err("미초기화 상태에서 에러");
            // user_message 한국어 키워드
            let msg: String = err.into();
            assert!(msg.contains("설정") || msg.contains("잠금"));
        }
    }

    // ─── Sprint 23 T1: after_connect 훅 PRAGMA 재적용 검증 (C3, H5) ───

    /// 테스트 임시 DB 경로 — 프로세스 ID + 태그로 충돌 회피. WAL/SHM 사이드카까지 정리.
    #[cfg(not(feature = "cipher"))]
    fn temp_db_cleanup(path: &std::path::Path) {
        let _ = std::fs::remove_file(path);
        let base = path.to_string_lossy();
        let _ = std::fs::remove_file(format!("{}-wal", base));
        let _ = std::fs::remove_file(format!("{}-shm", base));
    }

    /// AC-T1: after_connect 훅이 커넥션 교체(max_lifetime 만료) 후 재연결 시에도 PRAGMA 를
    /// 재적용하는지 검증. cipher off 이므로 key 이외 3종(WAL/foreign_keys/busy_timeout)을 확인.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn after_connect_reapplies_pragmas_on_reconnect() {
        let path = std::env::temp_dir().join(format!("smarthb_t1_reconnect_{}.db", std::process::id()));
        temp_db_cleanup(&path);
        let pool = test_pool_with_after_connect(&path, Some(Duration::from_millis(1)))
            .await
            .expect("after_connect 풀 생성");

        // 최초 커넥션 — PRAGMA 적용 확인
        let assert_pragmas = |pool: SqlitePool| async move {
            let journal: (String,) = sqlx::query_as("PRAGMA journal_mode")
                .fetch_one(&pool)
                .await
                .expect("journal_mode 조회");
            assert_eq!(journal.0.to_lowercase(), "wal", "journal_mode=WAL 재적용");
            let fk: (i64,) = sqlx::query_as("PRAGMA foreign_keys")
                .fetch_one(&pool)
                .await
                .expect("foreign_keys 조회");
            assert_eq!(fk.0, 1, "foreign_keys=ON 재적용");
            let busy: (i64,) = sqlx::query_as("PRAGMA busy_timeout")
                .fetch_one(&pool)
                .await
                .expect("busy_timeout 조회");
            assert_eq!(busy.0, 30000, "busy_timeout=30000 재적용");
            pool
        };
        let pool = assert_pragmas(pool).await;

        // max_lifetime(1ms) 만료 유도 → 다음 acquire 시 커넥션 교체 → after_connect 재실행
        tokio::time::sleep(Duration::from_millis(20)).await;
        let pool = assert_pragmas(pool).await;

        pool.close().await;
        temp_db_cleanup(&path);
    }

    // ─── Sprint 23 T2: create_if_missing 가드 검증 (C1) ───

    /// AC-T2-C1: 셋업 완료(setup_completed=true) + app.db 부재 → build_pool 이 빈 DB 를
    /// 날조하지 않고 에러로 중단한다 (RCA 전면소실 근절).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn build_pool_blocks_creation_when_setup_done_and_db_missing() {
        let dir = std::env::temp_dir().join(format!("smarthb_t2_block_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        crate::commands::paths::update_data_root(dir.clone());
        crate::commands::paths::set_setup_completed(true);
        std::fs::write(dir.join("salt.bin"), [0u8; 32]).unwrap(); // 셋업 흔적
        let db = dir.join("app.db");

        let result = build_pool(db).await;
        assert!(result.is_err(), "셋업 완료 + DB 부재 → 생성 차단(에러)");

        crate::commands::paths::set_setup_completed(false);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AC-T2-C1: 최초 설정(setup_completed=false) + app.db 부재 → 정상 생성 + 마이그레이션.
    /// 마법사 흐름(DB 생성이 complete_setup 보다 먼저)을 깨지 않음을 보장.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn build_pool_creates_on_first_setup() {
        let dir = std::env::temp_dir().join(format!("smarthb_t2_create_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        crate::commands::paths::update_data_root(dir.clone());
        crate::commands::paths::set_setup_completed(false); // 최초 설정
        let db = dir.join("app.db");

        let pool = build_pool(db).await.expect("최초 설정 DB 생성 성공");
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM students")
            .fetch_one(&pool)
            .await
            .expect("students 조회");
        assert_eq!(count.0, 0, "신규 DB 는 원생 0명 (마이그레이션만 적용)");

        pool.close().await;
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ─── Sprint 8 T1: V106 출결 도메인 마이그레이션 검증 (PRD §6.2) ───

    /// 테스트용 더미 원생 1건 INSERT — FK 충족 후 출결 테스트.
    #[cfg(not(feature = "cipher"))]
    async fn insert_test_student(pool: &SqlitePool) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES ('TEST-001', '테스트', 'male', 'elementary', 3, '2026-01-01') RETURNING id",
        )
        .fetch_one(pool)
        .await
        .expect("students 삽입 성공");
        row.0
    }

    /// AC-T1-1: V106 적용 후 두 테이블 존재 확인.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v106_creates_attendance_tables() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        let regular: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM regular_attendances")
            .fetch_one(&pool)
            .await
            .expect("regular_attendances 테이블 존재");
        assert_eq!(regular.0, 0, "초기 0건");
        let makeup: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances")
            .fetch_one(&pool)
            .await
            .expect("makeup_attendances 테이블 존재");
        assert_eq!(makeup.0, 0);
    }

    /// AC-T1-2: regular_attendances (student_id, event_date) UNIQUE 제약 동작.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn regular_attendances_unique_student_date() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        let sid = insert_test_student(&pool).await;
        sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-03-15', '2026-03', 90)",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("첫 INSERT 성공");

        let dup_result = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-03-15', '2026-03', 90)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(
            dup_result.is_err(),
            "동일 (student_id, event_date) 두 번째 INSERT 는 UNIQUE 위반"
        );
    }

    /// AC-T1-3: makeup_attendances UNIQUE 없음 — 동일 (student_id, event_date) 다중 INSERT 가능.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn makeup_attendances_allows_multiple_same_date() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        let sid = insert_test_student(&pool).await;
        for _ in 0..3 {
            sqlx::query(
                "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
                 VALUES (?, '2026-03-15', '2026-03', 60)",
            )
            .bind(sid)
            .execute(&pool)
            .await
            .expect("makeup 다중 INSERT 허용");
        }
        let count: (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM makeup_attendances WHERE student_id = ? AND event_date = '2026-03-15'",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 3, "동일 (student_id, event_date) 보강 3건 누적");
    }

    /// AC-T1-4: status CHECK 제약 위반 시 INSERT 실패 (regular + makeup 양쪽).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn attendances_status_check_rejects_invalid() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        let sid = insert_test_student(&pool).await;

        let regular_invalid = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) \
             VALUES (?, '2026-03-15', '2026-03', 'invalid', 90)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(
            regular_invalid.is_err(),
            "regular_attendances status='invalid' CHECK 위반"
        );

        let makeup_invalid = sqlx::query(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) \
             VALUES (?, '2026-03-15', '2026-03', 'present', 60)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(
            makeup_invalid.is_err(),
            "makeup_attendances status='present' CHECK 위반 (보강은 makeup_attended/makeup_absent 만 허용)"
        );
    }

    /// AC-T1-4 보강: year_month/event_date GLOB 패턴 위반 + class_minutes 양수 검증.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn attendances_format_checks() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        let sid = insert_test_student(&pool).await;

        // year_month 형식 위반 ('2026-3' — 한 자리 month)
        let bad_ym = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-03-15', '2026-3', 90)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(bad_ym.is_err(), "year_month GLOB 위반");

        // class_minutes 0 또는 음수
        let bad_minutes = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-03-15', '2026-03', 0)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(bad_minutes.is_err(), "class_minutes > 0 CHECK 위반");
    }

    /// V107 (Sprint 8 review F2): regular_attendances.makeup_attendance_id → makeup_attendances(id)
    /// FK 제약. PRAGMA foreign_keys=ON 환경에서 무효 id 참조 시 INSERT 실패해야 한다.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn regular_attendances_makeup_fk_enforced() {
        let pool = test_pool_in_memory().await.expect("인메모리 pool");
        // 인메모리 SQLite 는 기본 foreign_keys=OFF — V107 동작 검증을 위해 명시적으로 ON.
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .expect("foreign_keys ON");
        let sid = insert_test_student(&pool).await;

        // makeup_attendances 에 존재하지 않는 id (9999) 참조 → FK 위반으로 실패해야 함.
        let bad_fk = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, status, makeup_attendance_id) \
             VALUES (?, '2026-03-15', '2026-03', 90, 'makeup_done', 9999)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(
            bad_fk.is_err(),
            "무효 makeup_attendance_id 참조는 FK 위반 — V107 적용 확인"
        );

        // NULL 은 허용 (보강 미매칭 결석 상태).
        let null_ok = sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, makeup_attendance_id) \
             VALUES (?, '2026-03-16', '2026-03', 90, NULL)",
        )
        .bind(sid)
        .execute(&pool)
        .await;
        assert!(null_ok.is_ok(), "NULL makeup_attendance_id 는 허용");
    }
}
