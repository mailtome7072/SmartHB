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
//! T10 통합 전까지 `initialize`/`build_pool`/`apply_cipher_key_if_enabled` 등이 호출되지
//! 않으므로 모듈 전체에 `#[allow(dead_code)]` 를 적용한다.

#![allow(dead_code)]

use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::sync::OnceCell;

/// 전역 SqlitePool — unlock 후 lazy 초기화. 미초기화 상태에서 호출 시 `AppError::Config` 반환.
static POOL: OnceCell<SqlitePool> = OnceCell::const_new();

/// SQLite max_connections — SQLite 는 단일 writer 제한이라 1 로 고정.
///
/// 다중 reader 허용을 위해 WAL 모드를 켜는 것은 T10 startup PRAGMA 설정에서 처리.
const MAX_CONNECTIONS: u32 = 1;

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
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Config(format!("DB 디렉토리 생성 실패: {}", e)))?;
    }

    let url = db_url(&db_path)?;
    let connect_options = SqliteConnectOptions::from_str(&url)
        .map_err(|e| AppError::Config(format!("DB URL 파싱 실패: {}", e)))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(MAX_CONNECTIONS)
        .connect_with(connect_options)
        .await?;

    apply_cipher_key_if_enabled(&pool).await?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Config(format!("마이그레이션 실행 실패: {}", e)))?;

    Ok(pool)
}

#[cfg(feature = "cipher")]
async fn apply_cipher_key_if_enabled(pool: &SqlitePool) -> Result<(), AppError> {
    use crate::commands::auth::retrieve_key_from_keyring;
    use crate::commands::backup::pragma_key_sql;

    let key = retrieve_key_from_keyring()?;
    let hex_key = key.to_hex();
    // PRAGMA key 는 첫 connection 마다 적용되어야 한다. max_connections=1 이므로 1회로 충분.
    sqlx::query(&pragma_key_sql(hex_key.as_str())).execute(pool).await?;
    Ok(())
}

#[cfg(not(feature = "cipher"))]
async fn apply_cipher_key_if_enabled(_pool: &SqlitePool) -> Result<(), AppError> {
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

        let grade_count: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM standard_fees")
            .fetch_one(&pool)
            .await
            .expect("standard_fees 조회 성공");
        assert_eq!(grade_count.0, 12, "초중고 12학년 표준 교습비");

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
}
