//! 보강 도메인 IPC (Sprint 9~10, PRD §4.5.4~6, §4.5.10).
//!
//! Phase 3 — 보강 등록(개별/일괄) + 매칭 + 취소/미등원 + 결석 이력.
//! 본 모듈은 attendance.rs 와 별개 도메인 — V107 FK
//! (`regular_attendances.makeup_attendance_id → makeup_attendances.id`) 를 통해 정규 출결과
//! 연결되지만, 보강 등록/취소/미등원 트랜잭션은 본 모듈이 담당한다.
//!
//! ## Sprint 9 진입점
//! - T2 (본 세션): `get_pending_absences`, `get_makeup_eligible_dates` IPC 2종
//! - T3: `create_makeup_with_absences` (트랜잭션 매칭)
//! - T4: `cancel_makeup`, `mark_makeup_absent`, `batch_create_makeups`
//!
//! ## PI-02 결정 (사용자, 2026-05-24)
//! - 옵션 A 일 단위 매칭 — 보강 1일 = 결석 N일 충당. 시간값 비교 없음.
//! - 분 단위 전환은 T3 검증 3 활성/비활성 1줄 토글로 가능 (R58 추적).

use crate::commands::attendance::validate_year_month;
use crate::commands::db;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::collections::BTreeMap;

// ────────────────────────────────────────────────────────────────────
// 응답 구조체 (camelCase serde)
// ────────────────────────────────────────────────────────────────────

/// 원생의 미처리 결석 1건 — `status='absent' AND makeup_attendance_id IS NULL`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAbsence {
    pub id: i64,
    pub event_date: String,
    pub year_month: String,
    pub class_minutes: i64,
    pub makeup_deadline: Option<String>,
    pub absence_memo: Option<String>,
}

/// 보강 가능 일자 1건 — `schedule_codes.allows_makeup_class=1` 인 학사일정 일자.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibleDate {
    pub event_date: String,
    pub schedule_code_name: String,
}

// ────────────────────────────────────────────────────────────────────
// IPC: 미처리 결석 조회
// ────────────────────────────────────────────────────────────────────

/// 원생의 미처리 결석 목록 — 소멸기한 임박 순 정렬 (NULL 은 마지막).
///
/// PRD §4.5.4 보강 등록 다이얼로그가 충당 결석 선택지를 표시하기 위해 호출.
#[tauri::command]
pub async fn get_pending_absences(student_id: i64) -> Result<Vec<PendingAbsence>, String> {
    let pool = db::pool().map_err(String::from)?;
    get_pending_absences_impl(pool, student_id).await
}

async fn get_pending_absences_impl(
    pool: &SqlitePool,
    student_id: i64,
) -> Result<Vec<PendingAbsence>, String> {
    let rows = sqlx::query(
        "SELECT id, event_date, year_month, class_minutes, makeup_deadline, absence_memo \
         FROM regular_attendances \
         WHERE student_id = ? AND status = 'absent' AND makeup_attendance_id IS NULL \
         ORDER BY (makeup_deadline IS NULL), makeup_deadline ASC, event_date ASC",
    )
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("미처리 결석 조회 실패: {}", e))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        result.push(PendingAbsence {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            event_date: r.try_get("event_date").map_err(|e| e.to_string())?,
            year_month: r.try_get("year_month").map_err(|e| e.to_string())?,
            class_minutes: r.try_get("class_minutes").map_err(|e| e.to_string())?,
            makeup_deadline: r.try_get("makeup_deadline").map_err(|e| e.to_string())?,
            absence_memo: r.try_get("absence_memo").map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 가능 일자 조회
// ────────────────────────────────────────────────────────────────────

/// 원생의 보강 가능 일자 조회 — year_month 내 `allows_makeup_class=1` 학사일정 일자.
///
/// 알고리즘:
/// 1. year_month 의 모든 일자 펼침
/// 2. `schedule_events JOIN schedule_codes WHERE allows_makeup_class=1` 의 단일/기간 일자 펼침
/// 3. 학생 입교일 이전 / 퇴교일 이후 일자 제외
/// 4. 동일 일자 중복 학사일정은 첫 코드명으로 통합 (BTreeMap)
///
/// 정규 수업 요일 필터는 본 IPC 가 아닌 T3 `create_makeup_with_absences` 트랜잭션 검증
/// 단계에서 적용 — 책임 분담 단순화 (학생이 이미 결석한 정규 수업일에 같은 학생의 보강을
/// 등록하는 시나리오는 드물고, 본 IPC 는 "후보 일자" 목록 제공이 책임).
#[tauri::command]
pub async fn get_makeup_eligible_dates(
    student_id: i64,
    year_month: String,
) -> Result<Vec<EligibleDate>, String> {
    validate_year_month(&year_month)?;
    let pool = db::pool().map_err(String::from)?;
    get_makeup_eligible_dates_impl(pool, student_id, &year_month).await
}

async fn get_makeup_eligible_dates_impl(
    pool: &SqlitePool,
    student_id: i64,
    year_month: &str,
) -> Result<Vec<EligibleDate>, String> {
    // 1. 학생 입퇴교 범위 조회 (정규 수업 요일은 본 IPC 책임 외).
    let student_row = sqlx::query("SELECT enroll_date, withdraw_date FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("학생 조회 실패: {}", e))?
        .ok_or_else(|| "학생을 찾을 수 없습니다.".to_string())?;
    let enroll: String = student_row
        .try_get("enroll_date")
        .map_err(|e| e.to_string())?;
    let withdraw: Option<String> = student_row
        .try_get("withdraw_date")
        .map_err(|e| e.to_string())?;
    let enroll_d = NaiveDate::parse_from_str(&enroll, "%Y-%m-%d")
        .map_err(|e| format!("입교일 파싱 실패: {}", e))?;
    let withdraw_d = withdraw
        .as_deref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| format!("퇴교일 파싱 실패: {}", e))?;

    // 2. year_month 범위 (validate 통과 후라 unwrap 안전).
    let parts: Vec<&str> = year_month.split('-').collect();
    let year: i32 = parts[0].parse().expect("validated");
    let month: u32 = parts[1].parse().expect("validated");
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("일자 생성 실패: {}-{:02}-01", year, month))?;
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "다음 월 일자 생성 실패".to_string())?;

    // 3. 학사일정 중 allows_makeup_class=1 인 일자/기간을 month 와 겹치는 범위로 조회.
    // attendance.rs::load_off_dates 와 동일 패턴 — 단일 쿼리 + Rust 측 펼침.
    let makeup_rows = sqlx::query(
        "SELECT e.event_date, COALESCE(e.period_end_date, e.event_date) AS end_d, c.code_name \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_makeup_class = 1 \
           AND e.event_date < ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(next_month_first.to_string())
    .bind(first.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강 가능 학사일정 조회 실패: {}", e))?;

    // 4. 일자 펼침 + month 범위 내만 + 학생 입퇴교 범위 필터.
    // BTreeMap 으로 동일 일자 중복 코드 회피 + event_date 정렬 자동.
    let mut eligible: BTreeMap<String, String> = BTreeMap::new();
    for r in makeup_rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e_str: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let code_name: String = r.try_get("code_name").map_err(|e| e.to_string())?;
        let mut d = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 일자 파싱 실패: {}", e))?;
        let ed = NaiveDate::parse_from_str(&e_str, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 종료일 파싱 실패: {}", e))?;
        while d <= ed {
            if d >= first && d < next_month_first && d >= enroll_d {
                let in_withdraw_range = withdraw_d.is_none_or(|wd| d <= wd);
                if in_withdraw_range {
                    eligible
                        .entry(d.to_string())
                        .or_insert_with(|| code_name.clone());
                }
            }
            d = d.succ_opt().expect("date succ");
        }
    }

    Ok(eligible
        .into_iter()
        .map(|(event_date, schedule_code_name)| EligibleDate {
            event_date,
            schedule_code_name,
        })
        .collect())
}

// ────────────────────────────────────────────────────────────────────
// 단위 테스트 (cipher off 인메모리 풀)
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(not(feature = "cipher"))]
mod tests {
    use super::*;

    /// 테스트 학생 1명 삽입 — attendance.rs::seed_student 의 students 컬럼 규약 동일.
    async fn seed_student(
        pool: &SqlitePool,
        serial_no: &str,
        enroll: &str,
        withdraw: Option<&str>,
    ) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, \
             enroll_date, withdraw_date) \
             VALUES (?, ?, 'male', 'elementary', 3, ?, ?) RETURNING id",
        )
        .bind(serial_no)
        .bind(format!("학생-{}", serial_no))
        .bind(enroll)
        .bind(withdraw)
        .fetch_one(pool)
        .await
        .expect("학생 INSERT");
        row.0
    }

    /// 보강 가능 학사일정 코드 id 조회 (V102 시드 사용).
    async fn schedule_code_id(pool: &SqlitePool, name: &str) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM schedule_codes WHERE code_name = ?")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("코드 id 조회");
        row.0
    }

    async fn insert_schedule_event(
        pool: &SqlitePool,
        code_id: i64,
        event_date: &str,
        period_end_date: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) VALUES (?, ?, ?)",
        )
        .bind(code_id)
        .bind(event_date)
        .bind(period_end_date)
        .execute(pool)
        .await
        .expect("이벤트 INSERT");
    }

    async fn insert_absence(
        pool: &SqlitePool,
        student_id: i64,
        event_date: &str,
        year_month: &str,
        class_minutes: i64,
        makeup_deadline: Option<&str>,
    ) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO regular_attendances \
             (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, ?, ?, 'absent', ?, ?) RETURNING id",
        )
        .bind(student_id)
        .bind(event_date)
        .bind(year_month)
        .bind(class_minutes)
        .bind(makeup_deadline)
        .fetch_one(pool)
        .await
        .expect("결석 INSERT");
        row.0
    }

    // ─────────────── get_pending_absences ───────────────

    /// AC-T2-1: 미처리 결석만 조회 (status='absent' AND makeup_attendance_id IS NULL).
    /// 소멸기한 임박 순 정렬, NULL 은 마지막.
    #[tokio::test]
    async fn pending_absences_sorts_by_makeup_deadline_nulls_last() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;

        // 3건 결석: 6/15(deadline=07), 6/10(deadline=NULL), 6/20(deadline=07)
        // 기대 순서: 6/15(07), 6/20(07), 6/10(NULL)
        insert_absence(&pool, sid, "2026-06-15", "2026-06", 90, Some("2026-07")).await;
        insert_absence(&pool, sid, "2026-06-10", "2026-06", 90, None).await;
        insert_absence(&pool, sid, "2026-06-20", "2026-06", 90, Some("2026-07")).await;

        let list = get_pending_absences_impl(&pool, sid)
            .await
            .expect("미처리 결석 조회");
        let dates: Vec<String> = list.iter().map(|p| p.event_date.clone()).collect();
        assert_eq!(
            dates,
            vec!["2026-06-15", "2026-06-20", "2026-06-10"],
            "deadline 임박순 + NULL 마지막"
        );
    }

    /// AC-T2-1 보강: 매칭된 결석 (makeup_attendance_id NOT NULL) 은 제외.
    #[tokio::test]
    async fn pending_absences_excludes_matched_absences() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;
        let aid = insert_absence(&pool, sid, "2026-06-15", "2026-06", 90, Some("2026-07")).await;

        // 보강 행 + 매칭 설정 — V107 FK 강제로 실제 makeup_id 필요.
        let makeup_id: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-22', '2026-06', 90) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("makeup INSERT");
        sqlx::query(
            "UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=? WHERE id=?",
        )
        .bind(makeup_id.0)
        .bind(aid)
        .execute(&pool)
        .await
        .expect("매칭 설정");

        let list = get_pending_absences_impl(&pool, sid).await.expect("조회");
        assert!(list.is_empty(), "이미 매칭된 결석은 미처리에서 제외");
    }

    /// AC-T2-1 보강: 출석 상태(present) 는 제외.
    #[tokio::test]
    async fn pending_absences_excludes_present_status() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;
        sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, status) \
             VALUES (?, '2026-06-15', '2026-06', 90, 'present')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("출석 INSERT");

        let list = get_pending_absences_impl(&pool, sid).await.expect("조회");
        assert!(list.is_empty(), "출석 상태는 미처리 결석 아님");
    }

    // ─────────────── get_makeup_eligible_dates ───────────────

    /// AC-T2-2: allows_makeup_class=1 인 학사일정이 있는 일자만 반환.
    /// V301 시드 — "공휴수업일" 코드(allows_makeup_class=1).
    #[tokio::test]
    async fn eligible_dates_returns_makeup_class_dates() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-15", "2026-06-22"]);
    }

    /// AC-T2-2 보강: allows_makeup_class=0 인 학사일정은 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_makeup_off_codes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;
        // 방학 코드: allows_makeup_class=0 (V102 시드)
        let vac_code = schedule_code_id(&pool, "방학").await;
        insert_schedule_event(&pool, vac_code, "2026-06-15", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        assert!(list.is_empty(), "방학(allows_makeup_class=0)은 보강 가능 아님");
    }

    /// AC-T2-3: 학생 입교일 이전 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_before_enroll() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 6/20 입교 학생 — 6/15 학사일정은 입교 전이라 제외
        let sid = seed_student(&pool, "S001", "2026-06-20", None).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-22"], "입교 전 6/15 제외");
    }

    /// AC-T2-3 보강: 학생 퇴교일 이후 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_after_withdraw() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", Some("2026-06-18")).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-15"], "퇴교 후 6/22 제외");
    }

    /// AC-T2-2 기간성 코드: period_end_date 가 있으면 시작~종료 모든 일자 펼침.
    /// month 와 겹치는 부분만 반환.
    #[tokio::test]
    async fn eligible_dates_expands_period_codes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        // 6/14 ~ 6/16 (3일) 기간성 보강 가능일
        insert_schedule_event(&pool, makeup_code, "2026-06-14", Some("2026-06-16")).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-14", "2026-06-15", "2026-06-16"]);
    }

    // validate_year_month 자체 검증은 attendance.rs 의 단위 테스트
    // (`validate_year_month_rejects_out_of_range_month`) 에서 보장됨. 본 모듈은 호출만 위임.
}
