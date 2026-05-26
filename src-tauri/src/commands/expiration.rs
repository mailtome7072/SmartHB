//! 보강 소멸 자동 전이 도메인 (Sprint 10 T3, PRD §4.5.7).
//!
//! Phase 3 — 미보강 결석의 소멸기한 도래 자동 전이.
//!
//! ## 규칙 (PRD §4.5.7)
//! - `regular_attendances.makeup_deadline` 은 년월(YYYY-MM) 단위 저장 (결석 발생 월 + 1)
//! - 실제 소멸일 = `makeup_deadline` 년월의 `study_periods.end_date`
//! - 다음 월 교습기간 미등록 상태에서는 "소멸기한 미확정" — 자동 전이 보류
//! - 교습기간 등록 시 즉시 확정 (T4 트리거에서 호출)
//!
//! ## 사용자 결정 (Sprint 10 T2, 2026-05-26)
//! - **PI-05**: 트리거 3개소 (앱 시작 + 출결 생성 + 교습기간 등록) — T4에서 통합
//! - **PI-06**: 소멸 판정 기준일 = `chrono::Local::now()`. 테스트는 `Option<NaiveDate>` 주입
//! - **PI-09**: 자동 전이 알림 = 토스트 (건수 > 0일 때만)
//!
//! ## 핵심 SQL
//! ```ignore
//! UPDATE regular_attendances
//! SET status = 'makeup_expired'
//! WHERE status = 'absent'
//!   AND makeup_attendance_id IS NULL
//!   AND makeup_deadline IS NOT NULL
//!   AND makeup_deadline IN (
//!     SELECT year_month FROM study_periods WHERE end_date <= ?  -- as_of
//!   )
//! RETURNING id, student_id, event_date, makeup_deadline;
//! ```
//!
//! `study_periods` 미등록 월은 서브쿼리에 매칭 안 되어 자연스럽게 제외됨 (PI-05 정책).

use crate::commands::audit::{self, AuditEventType};
use crate::commands::db;
use chrono::{Local, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// 자동 전이 결과 — IPC 응답.
///
/// `transitioned_count == 0` 인 경우 UI 알림 생략 (PI-09).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExpirationReport {
    pub transitioned_count: usize,
    pub details: Vec<ExpiredAbsenceDetail>,
}

/// 소멸 전이된 결석 1건 — UI 토스트/audit 로그 메타데이터.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExpiredAbsenceDetail {
    pub student_id: i64,
    pub student_name: String,
    pub event_date: String,
    pub makeup_deadline: String,
}

/// 소멸기한 도래 미보강 결석 일괄 전이.
///
/// 트리거 3개소(앱 시작 / 출결 생성 / 교습기간 등록) 모두 동일 IPC 호출 (T4 통합).
#[tauri::command]
pub async fn expire_overdue_absences() -> Result<ExpirationReport, String> {
    let pool = db::pool().map_err(String::from)?;
    let report = expire_overdue_absences_impl(pool, None).await?;
    if report.transitioned_count > 0 {
        // audit: 전이된 결석마다 1건씩 기록. fire-and-forget.
        for d in &report.details {
            audit::try_record(
                AuditEventType::MakeupExpired,
                Some(&d.student_id.to_string()),
                Some(&format!(
                    r#"{{"eventDate":"{}","makeupDeadline":"{}"}}"#,
                    d.event_date, d.makeup_deadline
                )),
            )
            .await;
        }
    }
    Ok(report)
}

/// 핵심 구현 — `as_of` 가 `None` 이면 `Local::now()`. 테스트는 명시 일자 주입.
pub(crate) async fn expire_overdue_absences_impl(
    pool: &SqlitePool,
    as_of: Option<NaiveDate>,
) -> Result<ExpirationReport, String> {
    let cutoff = as_of.unwrap_or_else(|| Local::now().date_naive());
    let cutoff_str = cutoff.to_string();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    // RETURNING 으로 전이된 레코드 메타데이터 조회 + UPDATE 원자성 확보.
    // student 이름 JOIN 은 RETURNING 직후 별도 쿼리로 — SQLite RETURNING 은 JOIN 미지원.
    let rows = sqlx::query(
        "UPDATE regular_attendances \
         SET status = 'makeup_expired', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE status = 'absent' \
           AND makeup_attendance_id IS NULL \
           AND makeup_deadline IS NOT NULL \
           AND makeup_deadline IN ( \
             SELECT year_month FROM study_periods WHERE end_date <= ? \
           ) \
         RETURNING student_id, event_date, makeup_deadline",
    )
    .bind(&cutoff_str)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("소멸 전이 UPDATE 실패: {}", e))?;

    let mut details: Vec<ExpiredAbsenceDetail> = Vec::with_capacity(rows.len());
    for r in rows {
        let student_id: i64 = r.try_get("student_id").map_err(|e| e.to_string())?;
        let event_date: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let makeup_deadline: String = r.try_get("makeup_deadline").map_err(|e| e.to_string())?;
        let student_name: String =
            sqlx::query_scalar("SELECT name FROM students WHERE id = ?")
                .bind(student_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| format!("원생 이름 조회 실패: {}", e))?;
        details.push(ExpiredAbsenceDetail {
            student_id,
            student_name,
            event_date,
            makeup_deadline,
        });
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(ExpirationReport {
        transitioned_count: details.len(),
        details,
    })
}

// ────────────────────────────────────────────────────────────────────
// T6: 퇴교 시 미사용 보강 처리 (PRD §4.5.9)
// ────────────────────────────────────────────────────────────────────
//
// 사용자 결정 (Sprint 10 T2, 2026-05-26):
// - PI-11: 본 모듈(expiration.rs) 에 추가 — 소멸 도메인 일치
// - PI-12: external_expire memo = 결석별 absence_memo 일괄 저장
//
// `defer_withdrawal` 선택지는 UI 에서 다이얼로그 닫기로 처리 — IPC 호출 없음.

/// 퇴교 시 미보강 결석 1건 — UI 다이얼로그에 표시.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingAbsenceForWithdrawal {
    pub id: i64,
    pub event_date: String,
    pub class_minutes: i64,
    pub makeup_deadline: Option<String>,
}

/// 퇴교 시 미사용 보강 조회 응답.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawalPendingMakeup {
    pub student_id: i64,
    pub remaining_minutes: i64,
    pub absences: Vec<PendingAbsenceForWithdrawal>,
}

/// 퇴교 처리 선택지 — PRD §4.5.9. `defer_withdrawal` 은 UI 처리 (IPC 호출 안 함).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum WithdrawalChoice {
    /// 즉시 소멸 — 전체 미보강 결석 → makeup_expired 전이.
    ImmediateExpire,
    /// 외부 처리 후 소멸 — 동일 memo 를 모든 미보강 결석 absence_memo 에 일괄 저장 + makeup_expired 전이.
    ExternalExpire { memo: String },
}

/// 퇴교 대상 원생의 미보강 결석 조회.
///
/// 빈 리스트(`absences.is_empty() && remaining_minutes == 0`) 면 UI 는 다이얼로그 미표시.
#[tauri::command]
pub async fn get_pending_makeup_for_withdrawal(
    student_id: i64,
) -> Result<WithdrawalPendingMakeup, String> {
    let pool = db::pool().map_err(String::from)?;
    get_pending_makeup_for_withdrawal_impl(pool, student_id).await
}

pub(crate) async fn get_pending_makeup_for_withdrawal_impl(
    pool: &SqlitePool,
    student_id: i64,
) -> Result<WithdrawalPendingMakeup, String> {
    let rows = sqlx::query(
        "SELECT id, event_date, class_minutes, makeup_deadline \
         FROM regular_attendances \
         WHERE student_id = ? AND status = 'absent' AND makeup_attendance_id IS NULL \
         ORDER BY event_date",
    )
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("미보강 결석 조회 실패: {}", e))?;

    let mut absences: Vec<PendingAbsenceForWithdrawal> = Vec::with_capacity(rows.len());
    let mut remaining_minutes: i64 = 0;
    for r in rows {
        let cm: i64 = r.try_get("class_minutes").map_err(|e| e.to_string())?;
        remaining_minutes += cm;
        absences.push(PendingAbsenceForWithdrawal {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            event_date: r.try_get("event_date").map_err(|e| e.to_string())?,
            class_minutes: cm,
            makeup_deadline: r.try_get("makeup_deadline").map_err(|e| e.to_string())?,
        });
    }
    Ok(WithdrawalPendingMakeup {
        student_id,
        remaining_minutes,
        absences,
    })
}

/// 퇴교 처리 — 선택지에 따라 미보강 결석 일괄 전이 + 학생 withdraw_date 설정.
///
/// 단일 트랜잭션 — 보강 전이/메모 갱신/퇴교 설정이 원자적으로 적용.
#[tauri::command]
pub async fn process_withdrawal_makeup(
    student_id: i64,
    choice: WithdrawalChoice,
    withdraw_date: String,
) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let expired_ids =
        process_withdrawal_makeup_impl(pool, student_id, &choice, &withdraw_date).await?;
    // audit fire-and-forget — 전이된 결석마다 1건 + 학생 퇴교 1건.
    for aid in &expired_ids {
        audit::try_record(
            AuditEventType::MakeupExpired,
            Some(&student_id.to_string()),
            Some(&format!(
                r#"{{"attendanceId":{},"trigger":"withdrawal"}}"#,
                aid
            )),
        )
        .await;
    }
    audit::try_record(
        AuditEventType::StudentWithdrawn,
        Some(&student_id.to_string()),
        None,
    )
    .await;
    Ok(())
}

pub(crate) async fn process_withdrawal_makeup_impl(
    pool: &SqlitePool,
    student_id: i64,
    choice: &WithdrawalChoice,
    withdraw_date: &str,
) -> Result<Vec<i64>, String> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    // external_expire 인 경우 모든 미보강 결석에 동일 memo 일괄 저장.
    if let WithdrawalChoice::ExternalExpire { memo } = choice {
        sqlx::query(
            "UPDATE regular_attendances \
             SET absence_memo = ?, \
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE student_id = ? AND status = 'absent' AND makeup_attendance_id IS NULL",
        )
        .bind(memo)
        .bind(student_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("결석 메모 일괄 저장 실패: {}", e))?;
    }

    // 미보강 결석 → makeup_expired 일괄 전이 (deadline 무관).
    let rows = sqlx::query(
        "UPDATE regular_attendances \
         SET status = 'makeup_expired', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE student_id = ? AND status = 'absent' AND makeup_attendance_id IS NULL \
         RETURNING id",
    )
    .bind(student_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("퇴교 보강 전이 실패: {}", e))?;
    let expired_ids: Vec<i64> = rows
        .into_iter()
        .map(|r| r.try_get::<i64, _>("id").map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    // 학생 withdraw_date 설정.
    let result = sqlx::query(
        "UPDATE students SET \
            withdraw_date = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?",
    )
    .bind(withdraw_date)
    .bind(student_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("퇴교 처리 실패: {}", e))?;
    if result.rows_affected() == 0 {
        return Err(format!("원생을 찾을 수 없습니다 (id={}).", student_id));
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(expired_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    /// 테스트 헬퍼 — 학생 1명 시드 (attendance.rs 패턴, NOT NULL 컬럼 모두 채움).
    async fn seed_student(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES (?, ?, 'male', 'elementary', 3, '2026-01-01') RETURNING id",
        )
        .bind(name)
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed student")
    }

    /// 테스트 헬퍼 — 교습기간 시드.
    async fn seed_period(pool: &SqlitePool, year_month: &str, start: &str, end: &str) {
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES (?, ?, ?, 1)",
        )
        .bind(year_month)
        .bind(start)
        .bind(end)
        .execute(pool)
        .await
        .expect("seed period");
    }

    /// 테스트 헬퍼 — 미보강 결석 1건 시드.
    async fn seed_absence(
        pool: &SqlitePool,
        student_id: i64,
        event_date: &str,
        year_month: &str,
        makeup_deadline: Option<&str>,
    ) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, ?, ?, 'absent', 60, ?) RETURNING id",
        )
        .bind(student_id)
        .bind(event_date)
        .bind(year_month)
        .bind(makeup_deadline)
        .fetch_one(pool)
        .await
        .expect("seed absence")
    }

    async fn fetch_status(pool: &SqlitePool, attendance_id: i64) -> String {
        sqlx::query_scalar("SELECT status FROM regular_attendances WHERE id = ?")
            .bind(attendance_id)
            .fetch_one(pool)
            .await
            .expect("fetch status")
    }

    // ─────────────── T3 — 소멸 자동 전이 ───────────────

    /// 소멸기한 도래 + 미보강 결석 → makeup_expired 전이 성공.
    /// study_periods.end_date 가 as_of 이전 → 매칭 → 전이.
    #[tokio::test]
    async fn expires_overdue_unmatched_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        // 5월 결석 → makeup_deadline = 2026-06
        let aid = seed_absence(&pool, sid, "2026-05-15", "2026-05", Some("2026-06")).await;
        // 6월 교습기간 종료일 = 2026-06-30
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        // 기준일 2026-07-01 → 6월 종료 후 → 전이 대상
        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 1);
        assert_eq!(report.details[0].student_id, sid);
        assert_eq!(report.details[0].student_name, "S001");
        assert_eq!(report.details[0].event_date, "2026-05-15");
        assert_eq!(report.details[0].makeup_deadline, "2026-06");
        assert_eq!(fetch_status(&pool, aid).await, "makeup_expired");
    }

    /// 소멸기한 미도래 (study_periods.end_date 가 미래) → 전이 없음.
    #[tokio::test]
    async fn does_not_expire_when_deadline_not_reached() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let aid = seed_absence(&pool, sid, "2026-05-15", "2026-05", Some("2026-06")).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        // 기준일 2026-06-15 → 종료일 2026-06-30 미도래 → 전이 없음
        let as_of = NaiveDate::from_ymd_opt(2026, 6, 15);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 0);
        assert!(report.details.is_empty());
        assert_eq!(fetch_status(&pool, aid).await, "absent");
    }

    /// 이미 makeup_done 상태 → 전이 대상 아님 (status='absent' 조건 위반).
    #[tokio::test]
    async fn skips_already_matched_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        // makeup_done 상태로 직접 시드
        let aid: i64 = sqlx::query_scalar(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-05-15', '2026-05', 'makeup_done', 60, '2026-06') RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("seed makeup_done");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 0);
        assert_eq!(fetch_status(&pool, aid).await, "makeup_done");
    }

    /// 이미 makeup_expired 상태 → 중복 전이 없음 (status='absent' 조건 위반).
    #[tokio::test]
    async fn skips_already_expired_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let aid: i64 = sqlx::query_scalar(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-05-15', '2026-05', 'makeup_expired', 60, '2026-06') RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("seed makeup_expired");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 0);
        assert_eq!(fetch_status(&pool, aid).await, "makeup_expired");
    }

    /// 교습기간 미등록 월 → 소멸 전이 보류 (PI-05 정책 / PRD §4.5.7 "소멸기한 미확정").
    /// makeup_deadline = '2026-06' 인데 study_periods 에 2026-06 행 없음 → 서브쿼리 매칭 실패.
    #[tokio::test]
    async fn defers_when_study_period_not_registered() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let aid = seed_absence(&pool, sid, "2026-05-15", "2026-05", Some("2026-06")).await;
        // study_periods 미등록 — 시드 생략

        let as_of = NaiveDate::from_ymd_opt(2027, 1, 1); // 충분히 미래
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 0, "교습기간 미등록 → 전이 보류");
        assert_eq!(fetch_status(&pool, aid).await, "absent");
    }

    /// 복수 원생 + 복수 결석 batch 전이 + details 정확성 검증.
    #[tokio::test]
    async fn batch_expires_multiple_students() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s1 = seed_student(&pool, "원생A").await;
        let s2 = seed_student(&pool, "원생B").await;
        let a1 = seed_absence(&pool, s1, "2026-05-10", "2026-05", Some("2026-06")).await;
        let a2 = seed_absence(&pool, s1, "2026-05-20", "2026-05", Some("2026-06")).await;
        let a3 = seed_absence(&pool, s2, "2026-05-25", "2026-05", Some("2026-06")).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 3);
        assert_eq!(report.details.len(), 3);
        // 학생 이름 매핑 검증 — student_id → name JOIN
        let names: Vec<&str> = report.details.iter().map(|d| d.student_name.as_str()).collect();
        assert!(names.iter().filter(|n| **n == "원생A").count() == 2);
        assert!(names.iter().filter(|n| **n == "원생B").count() == 1);
        // 모두 makeup_expired 전이 확인
        for aid in [a1, a2, a3] {
            assert_eq!(fetch_status(&pool, aid).await, "makeup_expired");
        }
    }

    /// makeup_deadline 이 NULL 인 경우 → 전이 대상 아님 (출결 토글 안 된 결석).
    #[tokio::test]
    async fn skips_absence_without_deadline() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let aid = seed_absence(&pool, sid, "2026-05-15", "2026-05", None).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30").await;

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let report = expire_overdue_absences_impl(&pool, as_of).await.expect("ok");

        assert_eq!(report.transitioned_count, 0);
        assert_eq!(fetch_status(&pool, aid).await, "absent");
    }

    // ─────────────── T6 — 퇴교 시 미사용 보강 처리 (PRD §4.5.9) ───────────────

    async fn fetch_withdraw_date(pool: &SqlitePool, sid: i64) -> Option<String> {
        sqlx::query_scalar("SELECT withdraw_date FROM students WHERE id = ?")
            .bind(sid)
            .fetch_one(pool)
            .await
            .expect("fetch withdraw_date")
    }

    async fn fetch_memo(pool: &SqlitePool, aid: i64) -> Option<String> {
        sqlx::query_scalar("SELECT absence_memo FROM regular_attendances WHERE id = ?")
            .bind(aid)
            .fetch_one(pool)
            .await
            .expect("fetch memo")
    }

    /// 미보강 결석 조회 — 잔여 시간 합 + event_date 오름차순 정렬.
    #[tokio::test]
    async fn withdrawal_lists_pending_absences_with_remaining_minutes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        // 3건 결석 (60+90+30 = 180분)
        seed_absence(&pool, sid, "2026-05-20", "2026-05", Some("2026-06")).await;
        let aid_mid = sqlx::query_scalar::<_, i64>(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-05-10', '2026-05', 'absent', 90, '2026-06') RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("seed 90m");
        sqlx::query(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-05-05', '2026-05', 'absent', 30, '2026-06')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("seed 30m");

        let result = get_pending_makeup_for_withdrawal_impl(&pool, sid).await.expect("ok");
        assert_eq!(result.student_id, sid);
        assert_eq!(result.remaining_minutes, 60 + 90 + 30);
        assert_eq!(result.absences.len(), 3);
        // 정렬: event_date 오름차순
        assert_eq!(result.absences[0].event_date, "2026-05-05");
        assert_eq!(result.absences[1].event_date, "2026-05-10");
        assert_eq!(result.absences[1].id, aid_mid);
        assert_eq!(result.absences[2].event_date, "2026-05-20");
    }

    /// 미보강 결석 0건 → 빈 리스트 + remaining_minutes=0 (UI 다이얼로그 미표시 조건).
    #[tokio::test]
    async fn withdrawal_returns_empty_when_no_pending_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        // 출석만 시드 (제외 대상)
        sqlx::query(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes) \
             VALUES (?, '2026-05-10', '2026-05', 'present', 60)",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("seed present");

        let result = get_pending_makeup_for_withdrawal_impl(&pool, sid).await.expect("ok");
        assert_eq!(result.remaining_minutes, 0);
        assert!(result.absences.is_empty());
    }

    /// immediate_expire → 전체 미보강 결석 makeup_expired + withdraw_date 설정.
    #[tokio::test]
    async fn withdrawal_immediate_expire_transitions_all_and_sets_withdraw_date() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let a1 = seed_absence(&pool, sid, "2026-05-05", "2026-05", Some("2026-06")).await;
        let a2 = seed_absence(&pool, sid, "2026-05-10", "2026-05", Some("2026-06")).await;

        let expired = process_withdrawal_makeup_impl(
            &pool,
            sid,
            &WithdrawalChoice::ImmediateExpire,
            "2026-05-31",
        )
        .await
        .expect("ok");

        assert_eq!(expired.len(), 2);
        assert_eq!(fetch_status(&pool, a1).await, "makeup_expired");
        assert_eq!(fetch_status(&pool, a2).await, "makeup_expired");
        assert_eq!(fetch_withdraw_date(&pool, sid).await.as_deref(), Some("2026-05-31"));
        // memo 는 변경 없음
        assert!(fetch_memo(&pool, a1).await.is_none());
    }

    /// external_expire → memo 일괄 저장 + 전체 makeup_expired + withdraw_date 설정.
    #[tokio::test]
    async fn withdrawal_external_expire_saves_memo_and_transitions_all() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        let a1 = seed_absence(&pool, sid, "2026-05-05", "2026-05", Some("2026-06")).await;
        let a2 = seed_absence(&pool, sid, "2026-05-10", "2026-05", Some("2026-06")).await;

        let memo = "환불 처리됨 (외부 정산)".to_string();
        let expired = process_withdrawal_makeup_impl(
            &pool,
            sid,
            &WithdrawalChoice::ExternalExpire { memo: memo.clone() },
            "2026-05-31",
        )
        .await
        .expect("ok");

        assert_eq!(expired.len(), 2);
        assert_eq!(fetch_status(&pool, a1).await, "makeup_expired");
        assert_eq!(fetch_status(&pool, a2).await, "makeup_expired");
        assert_eq!(fetch_memo(&pool, a1).await.as_deref(), Some(memo.as_str()));
        assert_eq!(fetch_memo(&pool, a2).await.as_deref(), Some(memo.as_str()));
        assert_eq!(fetch_withdraw_date(&pool, sid).await.as_deref(), Some("2026-05-31"));
    }

    /// 미보강 결석 0건 → 전이 0건이지만 withdraw_date 는 정상 설정.
    #[tokio::test]
    async fn withdrawal_zero_absences_still_sets_withdraw_date() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001").await;
        // 결석 시드 없음.

        let expired = process_withdrawal_makeup_impl(
            &pool,
            sid,
            &WithdrawalChoice::ImmediateExpire,
            "2026-05-31",
        )
        .await
        .expect("ok");

        assert!(expired.is_empty());
        assert_eq!(fetch_withdraw_date(&pool, sid).await.as_deref(), Some("2026-05-31"));
    }

    /// 미존재 학생 id → 에러.
    #[tokio::test]
    async fn withdrawal_rejects_missing_student() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let err = process_withdrawal_makeup_impl(
            &pool,
            99999,
            &WithdrawalChoice::ImmediateExpire,
            "2026-05-31",
        )
        .await
        .expect_err("학생 없음");
        assert!(err.contains("찾을 수 없습니다"));
    }
}
