//! Tauri IPC 비동기 어댑터 + 에러 컨텍스트 매크로 (T11 통합).
//!
//! ## `run_blocking` — IPC 동기 작업 어댑터
//!
//! SQLite Online Backup API / rusqlite Connection / fs2 advisory lock 등 동기 API 호출을
//! tokio blocking worker pool 에서 실행하여 async runtime event loop 를 차단하지 않는다
//! (PRD §5.6 시작 < 3초 예산 영향 방지). T10 이전에 backup/integrity 모듈이 동일 시그니처를
//! 별도 보유하던 중복을 본 모듈로 통합한다.
//!
//! ## `app_err!` — 에러 컨텍스트 매크로
//!
//! backup/integrity/lock 모듈이 각자 `*_err` private 헬퍼로 보유하던 `AppError::Variant(format!(...))`
//! 패턴을 단일 매크로로 통합한다. AppError variant 가 다른 모듈마다 다르므로 enum 분기는
//! 매크로 인자로 받는다.

use crate::error::AppError;

/// blocking 작업을 spawn_blocking 으로 실행하고 IPC 응답용 `Result<T, String>` 으로 변환한다.
///
/// `error_variant` 는 spawn join 실패(panic) 시 사용할 `AppError` 생성자. 백업/무결성/락
/// 등 호출 도메인에 맞춰 사용자 친화 메시지 매핑이 변경되어야 하므로 매크로처럼 분기한다.
///
/// 본 함수는 IPC 커맨드(`#[tauri::command]`) 안에서만 호출되어야 한다 — 내부 비동기 함수가
/// 다른 IPC 가 부르는 `run_blocking` 결과의 String 에러를 다시 받아 처리하기 어렵기 때문.
pub(crate) async fn run_blocking<T, F>(
    error_variant: fn(String) -> AppError,
    join_ctx: &'static str,
    f: F,
) -> Result<T, String>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| error_variant(format!("{}: {}", join_ctx, e)))
        .and_then(|r| r)
        .map_err(String::from)
}

/// `AppError::Variant(format!("{ctx}: {err}"))` 패턴을 단일 매크로로 통합.
///
/// - `app_err!(Lock, "락 파일 생성 실패", e)` → `AppError::Lock("락 파일 생성 실패: <e>".into())`
/// - `app_err!(Backup, ...)`, `app_err!(Integrity, ...)` 등 모든 variant 공통 사용.
///
/// `Display` trait 가 구현된 모든 에러 타입을 받는다.
#[macro_export]
macro_rules! app_err {
    ($variant:ident, $ctx:expr, $e:expr) => {
        $crate::error::AppError::$variant(format!("{}: {}", $ctx, $e))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_blocking_propagates_success() {
        let result: Result<i32, String> =
            run_blocking(AppError::Lock, "테스트", || Ok::<i32, AppError>(42)).await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn run_blocking_translates_app_error_to_user_message() {
        let result: Result<(), String> = run_blocking(AppError::Lock, "테스트", || {
            Err(AppError::Backup("내부 사유".to_string()))
        })
        .await;
        let err = result.expect_err("AppError 가 String 으로 변환되어야 함");
        // user_message 가 한국어 안내를 반환 — 기술 디테일 노출 없음
        assert!(err.contains("백업"));
        assert!(!err.contains("내부 사유"), "기술 디테일이 IPC 메시지에 누출됨");
    }

    #[test]
    fn app_err_macro_formats_context_and_error() {
        let err = app_err!(Lock, "락 파일 생성 실패", "permission denied");
        match err {
            AppError::Lock(msg) => {
                assert!(msg.contains("락 파일 생성 실패"));
                assert!(msg.contains("permission denied"));
                assert!(msg.contains(":"));
            }
            _ => panic!("Lock variant 기대"),
        }
    }

    #[test]
    fn app_err_macro_supports_all_variants() {
        // 컴파일이 되면 통과 — 각 variant 에서 매크로 사용 가능 검증.
        let _ = app_err!(Auth, "ctx", "e");
        let _ = app_err!(Lock, "ctx", "e");
        let _ = app_err!(Backup, "ctx", "e");
        let _ = app_err!(Integrity, "ctx", "e");
        let _ = app_err!(Config, "ctx", "e");
    }
}
