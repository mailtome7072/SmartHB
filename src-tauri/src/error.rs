//! 애플리케이션 전역 에러 타입.
//!
//! PRD §6.4 준수:
//! - 사용자 친화 한국어 메시지만 IPC 로 노출 (`From<AppError> for String`)
//! - 기술 상세(원본 에러, 디버그 정보)는 `Debug` trait 으로만 접근 — 향후 tracing crate
//!   통합 시 로그로 기록 (Sprint 1 T9 audit log 와는 별개 — audit 은 비즈니스 이벤트, 본 로그는 에러)
//! - 스택 트레이스·에러 코드 IPC 직접 노출 금지
//!
//! 사용 패턴:
//! ```ignore
//! pub async fn unlock_db(password: String) -> Result<(), String> {
//!     do_unlock(&password).await.map_err(AppError::from).map_err(String::from)
//! }
//! ```

use thiserror::Error;

/// 모든 Tauri 커맨드·내부 로직의 공통 에러 타입.
///
/// IPC 반환 시 [`String`] 으로 직렬화되며, 직렬화 결과는 사용자 친화 한국어 메시지다.
/// 기술 상세를 보존하기 위해 `#[from]` 으로 원본 에러 타입을 wrapping 한다.
#[derive(Debug, Error)]
pub enum AppError {
    /// 인증 실패 — 비밀번호 불일치, 복구 코드 무효, 키 유도 실패 등.
    #[error("인증 실패: {0}")]
    Auth(String),

    /// 데이터베이스 접근/쿼리 실패. sqlx 에러를 래핑.
    #[error("DB 오류: {0}")]
    Db(#[from] sqlx::Error),

    /// 동시성 락 충돌 — 다른 디바이스가 점유 중이거나 락 파일 손상.
    #[error("락 오류: {0}")]
    Lock(String),

    /// 백업 생성·복원·순환 삭제 실패.
    #[error("백업 오류: {0}")]
    Backup(String),

    /// 무결성 검증 실패 — `PRAGMA integrity_check` 또는 `quick_check` 비통과.
    #[error("무결성 오류: {0}")]
    Integrity(String),

    /// 파일 시스템 접근 실패 (디스크 가득 참, 권한 부족 등).
    #[error("파일 시스템 오류: {0}")]
    Io(#[from] std::io::Error),

    /// 설정값 누락·형식 오류 — 초기 마법사 미완료, 환경변수 누락 등.
    #[error("설정 오류: {0}")]
    Config(String),

    /// 사용자 친화 한국어 메시지를 그대로 노출 — 도메인 검증 오류, 비즈니스 규칙 위반 등.
    ///
    /// 다른 variant 는 `user_message()` 가 정형 메시지로 변환하지만, 본 variant 는 inner string
    /// 을 그대로 IPC 응답에 사용한다. 호출자가 이미 사용자 친화 한국어 메시지를 작성한 경우 사용.
    #[error("{0}")]
    UserFacing(String),
}

impl AppError {
    /// IPC 응답으로 사용할 사용자 친화 한국어 메시지.
    ///
    /// 50대 사용자 기준으로 작성: 짧고 명확하며 "어떻게 해야 하는지" 안내를 포함.
    /// 기술 용어(SQL, mutex, errno) 사용 금지.
    pub fn user_message(&self) -> String {
        match self {
            AppError::Auth(_) => "비밀번호가 올바르지 않거나 인증에 실패했습니다.".to_string(),
            AppError::Db(_) => {
                "데이터를 처리하는 중 오류가 발생했습니다. 잠시 후 다시 시도해주세요.".to_string()
            }
            AppError::Lock(_) => {
                "다른 컴퓨터에서 프로그램을 사용 중입니다. 종료 후 다시 시도해주세요.".to_string()
            }
            AppError::Backup(_) => {
                "백업을 처리하는 중 오류가 발생했습니다. 디스크 여유 공간을 확인해주세요.".to_string()
            }
            AppError::Integrity(_) => {
                "데이터 검증에 실패했습니다. 최근 백업으로 자동 복원을 시도합니다.".to_string()
            }
            AppError::Io(_) => {
                "파일을 읽거나 쓰는 중 오류가 발생했습니다. 폴더 접근 권한을 확인해주세요.".to_string()
            }
            AppError::Config(_) => {
                "설정 정보를 불러오는 중 오류가 발생했습니다. 초기 설정 마법사를 다시 실행해주세요.".to_string()
            }
            AppError::UserFacing(msg) => msg.clone(),
        }
    }
}

/// Tauri IPC 직렬화 — 사용자 친화 한국어 메시지만 노출.
///
/// 기술 상세(원본 에러, 스택)는 사용자 화면에는 노출되지 않지만, **dev 콘솔/stderr 에는
/// `Display` trait 으로 보존된다** (2026-05-21 진단 인프라 추가). PRD §6.4 "사용자 화면에는
/// 친화 메시지, 콘솔/로그에는 기술 상세" 정책 준수. tracing crate 통합 시 본 stderr 호출을
/// `tracing::error!` 로 교체한다.
impl From<AppError> for String {
    fn from(err: AppError) -> Self {
        eprintln!("[error] {}", err);
        err.user_message()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_user_message_is_korean() {
        let err = AppError::Auth("invalid password hash".to_string());
        let msg: String = err.into();
        assert!(msg.contains("비밀번호"));
        assert!(!msg.contains("invalid password hash"), "기술 디테일이 IPC 메시지에 누출됨");
    }

    #[test]
    fn db_error_hides_sql_details() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err: AppError = sqlx_err.into();
        let msg: String = app_err.into();
        assert!(msg.contains("데이터"));
        assert!(!msg.contains("RowNotFound"), "sqlx 내부 타입명이 IPC 메시지에 누출됨");
    }

    #[test]
    fn io_error_from_implements_correctly() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let app_err: AppError = io_err.into();
        let msg: String = app_err.into();
        assert!(msg.contains("파일"));
        assert!(!msg.contains("access denied"));
    }

    #[test]
    fn all_variants_return_non_empty_korean_message() {
        let variants = [
            AppError::Auth("x".into()),
            AppError::Lock("x".into()),
            AppError::Backup("x".into()),
            AppError::Integrity("x".into()),
            AppError::Config("x".into()),
        ];
        for v in variants {
            let msg = v.user_message();
            assert!(!msg.is_empty(), "user_message 가 빈 문자열을 반환함");
            assert!(
                msg.chars().any(|c| ('\u{AC00}'..='\u{D7A3}').contains(&c)),
                "user_message 에 한글이 포함되지 않음: {}",
                msg
            );
        }
    }

    #[test]
    fn display_trait_preserves_technical_details_for_logs() {
        // user_message 와 달리, Display trait 은 기술 디테일을 보존해야 한다 (로그용)
        let err = AppError::Auth("PBKDF2 derivation failed".to_string());
        let display = format!("{}", err);
        assert!(display.contains("PBKDF2 derivation failed"));
    }
}
