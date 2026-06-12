//! 감사 로그 (T9, PRD §6.6).
//!
//! ## 기록 대상 (우선순위)
//!
//! - 비밀번호 변경 (`PasswordChange`)
//! - 백업 생성·복원 (`BackupCreated`, `BackupRestored`)
//! - 락 강제 점유 (`LockForced`)
//! - 무결성 검증 실패 (`IntegrityCheckFailed`)
//! - 자가 진단 결과 (`SelfDiagnostic` — 후속 sprint)
//!
//! ## 보안 원칙
//!
//! - 민감 데이터(비밀번호, hex key, SQLCipher salt)는 절대 미기록
//! - 호출자가 `details` 직렬화 시점에 사전 마스킹 — 본 모듈은 입력을 그대로 저장한다
//! - 1년 롤링 보관 — `cleanup_old` 가 T10 시작 시퀀스에서 호출됨
//!
//! ## T10 통합 예정
//!
//! 본 sprint(T9) 에서는 인프라(테이블·헬퍼·IPC)만 제공. backup/lock/auth 모듈이
//! 실제로 [`record`] 를 호출하는 통합은 T10 (시작 시퀀스 + 모듈 lifecycle 결정) 으로 미룬다.

use crate::commands::db;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 감사 이벤트 종류 — 신규 이벤트 추가 시 본 enum 에 variant 만 추가하면 된다.
///
/// 보안 이벤트(Sprint 1) + 도메인 이벤트(Sprint 2~) 를 모두 포함. 도메인 이벤트는 PRD §6.6
/// 자가 진단 + 운영 추적에 사용된다.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditEventType {
    // Sprint 1 — 보안 이벤트
    PasswordChange,
    BackupCreated,
    BackupRestored,
    LockForced,
    IntegrityCheckFailed,
    // Sprint 2 — 원생 도메인 이벤트
    StudentCreated,
    StudentUpdated,
    StudentWithdrawn,
    // Sprint 4 T8 — 퇴교 번복
    StudentReinstated,
    // Sprint 8 T3 — 출결 도메인
    AttendanceToggled,
    // Sprint 8 T8 (R47 / I-S2-9) — 보안 일반 (예: salt 마이그레이션, 권한 변경 등)
    SecurityEvent,
    // Sprint 9 T3/T4 — 보강 도메인 (PRD §4.5.4~6)
    MakeupCreated,
    MakeupCancelled,
    // Sprint 10 T3 — 보강 소멸 자동 전이 (PRD §4.5.7)
    MakeupExpired,
    // Sprint 11 T3 — 청구 상태 머신 (PRD §4.9.3). 마감(month-closed/closed-modified)은 V111 폐기.
    BillConfirmed,
    // Sprint 16 T0 — 수업일 변경 (사용자 이슈 2026-06-08)
    // 케이스1: 특정일 1회성 수업일 이동 (출결 행 event_date 변경)
    AttendanceRescheduled,
    // 케이스2: 특정일 이후 영구 스케줄 변경 + 변경일 이후 출결 재생성
    ScheduleChangedWithRegen,
}

impl AuditEventType {
    /// DB 저장용 string code — enum variant 추가 시 본 매핑도 갱신.
    fn as_code(self) -> &'static str {
        match self {
            Self::PasswordChange => "password-change",
            Self::BackupCreated => "backup-created",
            Self::BackupRestored => "backup-restored",
            Self::LockForced => "lock-forced",
            Self::IntegrityCheckFailed => "integrity-check-failed",
            Self::StudentCreated => "student-created",
            Self::StudentUpdated => "student-updated",
            Self::StudentWithdrawn => "student-withdrawn",
            Self::StudentReinstated => "student-reinstated",
            Self::AttendanceToggled => "attendance-toggled",
            Self::SecurityEvent => "security-event",
            Self::MakeupCreated => "makeup-created",
            Self::MakeupCancelled => "makeup-cancelled",
            Self::MakeupExpired => "makeup-expired",
            Self::BillConfirmed => "bill-confirmed",
            Self::AttendanceRescheduled => "attendance-rescheduled",
            Self::ScheduleChangedWithRegen => "schedule-changed-with-regen",
        }
    }
}

/// 감사 로그 항목 — IPC 응답.
#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub created_at: String,
    pub event_type: String,
    pub event_subject: Option<String>,
    pub details: Option<String>,
}

/// sqlx query_as 반환 튜플 별칭 — `(id, created_at, event_type, event_subject, details)`.
type AuditLogRow = (i64, String, String, Option<String>, Option<String>);

/// 감사 이벤트 1건을 기록한다. `details` 는 JSON 문자열 권장 (마스킹된 상태).
///
/// T10 호출자 (auth/lock/backup/integrity) 는 [`try_record`] 를 통해 호출하여
/// pool 미초기화 (unlock 전) 상태에서도 startup 흐름을 차단하지 않는다.
pub(crate) async fn record(
    event_type: AuditEventType,
    event_subject: Option<&str>,
    details: Option<&str>,
) -> Result<(), AppError> {
    let pool = db::pool()?;
    sqlx::query(
        "INSERT INTO audit_logs (event_type, event_subject, details) VALUES (?, ?, ?)",
    )
    .bind(event_type.as_code())
    .bind(event_subject)
    .bind(details)
    .execute(pool)
    .await?;
    Ok(())
}

/// best-effort 기록 — pool 미초기화 또는 DB 오류 시 silent fail.
///
/// startup 시퀀스 / set_password / acquire_lock 등 pool 이 아직 초기화되지 않은 시점에서
/// 호출될 수 있는 위치에서 사용한다. 실패는 stderr 로만 기록되어 사용자 흐름에 영향을 주지 않는다.
pub(crate) async fn try_record(
    event_type: AuditEventType,
    event_subject: Option<&str>,
    details: Option<&str>,
) {
    if let Err(e) = record(event_type, event_subject, details).await {
        eprintln!("[audit] 기록 생략 ({:?}): {}", event_type, e);
    }
}

/// 시간 역순 페이지네이션 — `since` 이후 항목 중 `limit` 개 반환.
///
/// `limit` 기본 100, 최대 1000 (메모리 보호). UI 는 무한 스크롤 또는 페이지 단위로 호출.
async fn list_logs(
    since: Option<DateTime<Utc>>,
    limit: Option<u32>,
) -> Result<Vec<AuditLogEntry>, AppError> {
    let pool = db::pool()?;
    let limit = limit.unwrap_or(100).min(1000);

    // since 유무에 따라 SQL 문자열만 분기 — bind 흐름과 query_as 호출은 단일화하여 중복 제거.
    let sql = if since.is_some() {
        "SELECT id, created_at, event_type, event_subject, details \
         FROM audit_logs WHERE created_at >= ? ORDER BY created_at DESC LIMIT ?"
    } else {
        "SELECT id, created_at, event_type, event_subject, details \
         FROM audit_logs ORDER BY created_at DESC LIMIT ?"
    };
    let mut q = sqlx::query_as::<_, AuditLogRow>(sql);
    if let Some(t) = since {
        q = q.bind(t.to_rfc3339());
    }
    let rows: Vec<AuditLogRow> = q.bind(limit).fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|(id, created_at, event_type, event_subject, details)| AuditLogEntry {
            id,
            created_at,
            event_type,
            event_subject,
            details,
        })
        .collect())
}

/// 1년(또는 임의 days) 이상 된 audit_logs 를 삭제한다.
///
/// T10 시작 시퀀스 [`crate::startup::app_startup_sequence`] 에서 호출되어 보관 기간 정책을 강제한다.
pub(crate) async fn cleanup_older_than(days: i64) -> Result<u64, AppError> {
    let pool = db::pool()?;
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < ?")
        .bind(cutoff.to_rfc3339())
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// 감사 로그를 시간 역순으로 조회한다.
///
/// 미초기화 pool (unlock 미수행) 호출 시 `AppError::Config` 의 사용자 친화 메시지 반환.
#[tauri::command]
pub async fn get_audit_logs(
    since: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<AuditLogEntry>, String> {
    let since_dt = match since {
        Some(s) => Some(
            DateTime::parse_from_rfc3339(&s)
                .map_err(|e| AppError::Config(format!("since 파싱 실패: {}", e)))
                .map_err(String::from)?
                .with_timezone(&Utc),
        ),
        None => None,
    };
    list_logs(since_dt, limit).await.map_err(String::from)
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    #[test]
    fn audit_event_type_codes_are_kebab_case() {
        assert_eq!(AuditEventType::PasswordChange.as_code(), "password-change");
        assert_eq!(AuditEventType::BackupRestored.as_code(), "backup-restored");
        assert_eq!(AuditEventType::LockForced.as_code(), "lock-forced");
    }

    #[test]
    fn audit_event_type_serde_round_trip() {
        let types = [
            AuditEventType::PasswordChange,
            AuditEventType::BackupCreated,
            AuditEventType::BackupRestored,
            AuditEventType::LockForced,
            AuditEventType::IntegrityCheckFailed,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let parsed: AuditEventType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, t);
        }
        // kebab-case 직렬화 정합
        assert_eq!(
            serde_json::to_string(&AuditEventType::PasswordChange).unwrap(),
            r#""password-change""#
        );
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn record_and_list_logs_round_trip() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        // record 가 전역 POOL 을 쓰는 대신 테스트 pool 을 직접 호출하기 위해 SQL 인라인.
        sqlx::query("INSERT INTO audit_logs (event_type, event_subject, details) VALUES (?, ?, ?)")
            .bind("password-change")
            .bind(Some("device-abc"))
            .bind(Option::<&str>::None)
            .execute(&pool)
            .await
            .expect("INSERT 성공");

        sqlx::query("INSERT INTO audit_logs (event_type, event_subject, details) VALUES (?, ?, ?)")
            .bind("backup-restored")
            .bind(Option::<&str>::None)
            .bind(Some(r#"{"layer":"exit"}"#))
            .execute(&pool)
            .await
            .expect("INSERT 성공");

        let rows: Vec<AuditLogRow> = sqlx::query_as(
            "SELECT id, created_at, event_type, event_subject, details FROM audit_logs ORDER BY id DESC",
        )
        .fetch_all(&pool)
        .await
        .expect("조회 성공");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].2, "backup-restored");
        assert_eq!(rows[1].2, "password-change");
        assert_eq!(rows[1].3.as_deref(), Some("device-abc"));
        assert_eq!(rows[0].4.as_deref(), Some(r#"{"layer":"exit"}"#));
    }

    #[tokio::test]
    async fn try_record_silent_fails_when_pool_uninitialized() {
        // pool 미초기화 상태 — record 는 Err 반환, try_record 는 silent fail.
        // 본 테스트는 try_record 가 panic 없이 즉시 반환하는지만 검증.
        if !crate::commands::db::is_initialized() {
            // 호출 자체가 panic 없이 완료되어야 한다.
            try_record(AuditEventType::PasswordChange, None, None).await;
            try_record(AuditEventType::BackupCreated, Some("test"), Some(r#"{"k":"v"}"#)).await;
            // 통과 — silent fail 정상 동작
        }
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn cleanup_deletes_only_old_rows() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        // 2년 전 로그
        sqlx::query("INSERT INTO audit_logs (created_at, event_type) VALUES (?, ?)")
            .bind((Utc::now() - chrono::Duration::days(730)).to_rfc3339())
            .bind("password-change")
            .execute(&pool)
            .await
            .expect("old INSERT");

        // 어제 로그
        sqlx::query("INSERT INTO audit_logs (created_at, event_type) VALUES (?, ?)")
            .bind((Utc::now() - chrono::Duration::days(1)).to_rfc3339())
            .bind("backup-created")
            .execute(&pool)
            .await
            .expect("recent INSERT");

        // 1년(365일) 이전 삭제 시뮬레이션 — cleanup_older_than 본체는 전역 POOL 사용이라
        // 본 테스트는 동일 쿼리를 직접 실행하여 cutoff 동작만 검증한다.
        let cutoff = (Utc::now() - chrono::Duration::days(365)).to_rfc3339();
        let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < ?")
            .bind(cutoff)
            .execute(&pool)
            .await
            .expect("DELETE 성공");

        assert_eq!(result.rows_affected(), 1, "2년 전 로그만 삭제");

        let remaining: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining.0, 1);
    }
}
