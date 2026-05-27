//! 수업 관리 캘린더 도메인 IPC (Sprint 10 T8, PRD §4.6).
//!
//! Phase 3 — 캘린더 뷰 + 보강 관리 뷰의 백엔드 집계.
//!
//! ## IPC 목록
//! - `get_calendar_data(year_month)` — 일자별 정규/보강 수업 raw 데이터
//! - `get_makeup_management_data(year_month)` — 보강 필요 원생 목록 (소멸기한 임박 순)
//!
//! ## 시간대별 합산 책임
//! - 백엔드: 일자별 수업 리스트 (start_time + class_minutes) raw 제공
//! - 프론트엔드: 시간대(1시간 단위 등) 합산 — AC-4.6-1 시작 + 진행 중 원생 합산
//!
//! ## 보강 수업 시작 시간
//! - 현재 `makeup_attendances` 스키마에 `start_time` 컬럼 없음 (Sprint 9 폐기)
//! - 보강은 시간대 없이 일자별 목록만 제공. 캘린더 UI 에서 별도 표시 (일자 셀 하단 등).

use crate::commands::attendance::validate_year_month;
use crate::commands::db;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

// ────────────────────────────────────────────────────────────────────
// 응답 구조체
// ────────────────────────────────────────────────────────────────────

/// 캘린더 한 달 데이터 — 일자별 수업 목록.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarMonth {
    pub year_month: String,
    pub days: Vec<CalendarDay>,
}

/// 캘린더 한 일자 — 정규 수업 + 보강 수업 분리.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarDay {
    pub event_date: String,
    pub regular_sessions: Vec<CalendarSession>,
    pub makeup_sessions: Vec<CalendarSession>,
}

/// 캘린더 수업 1건 — 정규는 `start_time` Some, 보강은 None.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CalendarSession {
    pub student_id: i64,
    pub student_name: String,
    pub start_time: Option<String>,
    pub class_minutes: i64,
}

/// 보강 관리 뷰 한 원생 — PRD §4.6.3.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MakeupManagementStudent {
    pub student_id: i64,
    pub student_name: String,
    pub serial_no: String,
    /// 잔여 보강필요시간 (분 단위).
    pub remaining_minutes: i64,
    /// 가장 임박한 makeup_deadline (YYYY-MM) — 없으면 None.
    pub earliest_deadline: Option<String>,
    /// 소멸 임박 플래그 — deadline 월의 `study_periods.end_date - 7일` 이내 도래.
    pub is_imminent: bool,
}

// ────────────────────────────────────────────────────────────────────
// IPC: 캘린더 데이터 조회
// ────────────────────────────────────────────────────────────────────

/// 해당 월의 일자별 정규/보강 수업 데이터 — PRD §4.6.1.
#[tauri::command]
pub async fn get_calendar_data(year_month: String) -> Result<CalendarMonth, String> {
    validate_year_month(&year_month)?;
    let pool = db::pool().map_err(String::from)?;
    get_calendar_data_impl(pool, &year_month).await
}

pub(crate) async fn get_calendar_data_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<CalendarMonth, String> {
    // 1. 정규 수업: regular_attendances WHERE year_month = ? + 학생 정보 JOIN.
    //    학생 스케줄에서 start_time 추출 (effective_to IS NULL — 현행).
    //    status 무관 — 출결 그리드에 모든 상태 표시.
    let regular_rows = sqlx::query(
        "SELECT \
            ra.event_date, \
            ra.class_minutes, \
            ra.student_id, \
            s.name AS student_name, \
            ss.start_time \
         FROM regular_attendances ra \
         JOIN students s ON s.id = ra.student_id \
         LEFT JOIN student_schedules ss \
           ON ss.student_id = ra.student_id \
          AND ss.effective_to IS NULL \
          AND ss.day_of_week = ((CAST(strftime('%w', ra.event_date) AS INTEGER) + 6) % 7) + 1 \
         WHERE ra.year_month = ? \
         ORDER BY ra.event_date, ss.start_time, s.name",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("정규 수업 조회 실패: {}", e))?;

    // 2. 보강 수업: makeup_attendances WHERE year_month = ?.
    let makeup_rows = sqlx::query(
        "SELECT \
            m.event_date, \
            m.class_minutes, \
            m.student_id, \
            s.name AS student_name \
         FROM makeup_attendances m \
         JOIN students s ON s.id = m.student_id \
         WHERE m.year_month = ? \
         ORDER BY m.event_date, s.name",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강 수업 조회 실패: {}", e))?;

    // 3. 일자별 그룹화 — BTreeMap 으로 정렬 유지.
    use std::collections::BTreeMap;
    let mut by_day: BTreeMap<String, (Vec<CalendarSession>, Vec<CalendarSession>)> =
        BTreeMap::new();

    for r in regular_rows {
        let event_date: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let entry = by_day.entry(event_date).or_default();
        entry.0.push(CalendarSession {
            student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
            student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
            start_time: r.try_get("start_time").ok(),
            class_minutes: r.try_get("class_minutes").map_err(|e| e.to_string())?,
        });
    }
    for r in makeup_rows {
        let event_date: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let entry = by_day.entry(event_date).or_default();
        entry.1.push(CalendarSession {
            student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
            student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
            start_time: None,
            class_minutes: r.try_get("class_minutes").map_err(|e| e.to_string())?,
        });
    }

    let days: Vec<CalendarDay> = by_day
        .into_iter()
        .map(|(event_date, (regular, makeup))| CalendarDay {
            event_date,
            regular_sessions: regular,
            makeup_sessions: makeup,
        })
        .collect();

    Ok(CalendarMonth {
        year_month: year_month.to_string(),
        days,
    })
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 관리 뷰 데이터 조회
// ────────────────────────────────────────────────────────────────────

/// 보강 필요 원생 리스트 — PRD §4.6.3 보강 관리 뷰.
///
/// 정렬: `earliest_deadline ASC` (소멸기한 임박 순), NULL 은 마지막.
/// 소멸 임박: `study_periods.end_date - 7일` 이내 (AC-4.6-2).
#[tauri::command]
pub async fn get_makeup_management_data(
    year_month: String,
) -> Result<Vec<MakeupManagementStudent>, String> {
    validate_year_month(&year_month)?;
    let pool = db::pool().map_err(String::from)?;
    get_makeup_management_data_impl(pool, &year_month, None).await
}

pub(crate) async fn get_makeup_management_data_impl(
    pool: &SqlitePool,
    _year_month: &str,
    as_of: Option<NaiveDate>,
) -> Result<Vec<MakeupManagementStudent>, String> {
    let today = as_of.unwrap_or_else(|| chrono::Local::now().date_naive());
    let imminent_threshold = today + chrono::Duration::days(7);

    // 학생별 미보강 결석 집계: 잔여 시간 + 가장 임박한 deadline.
    let rows = sqlx::query(
        "SELECT \
            s.id AS student_id, \
            s.name AS student_name, \
            s.serial_no, \
            COALESCE(SUM(ra.class_minutes), 0) AS remaining_minutes, \
            MIN(ra.makeup_deadline) AS earliest_deadline \
         FROM students s \
         JOIN regular_attendances ra ON ra.student_id = s.id \
         WHERE ra.status = 'absent' AND ra.makeup_attendance_id IS NULL \
         GROUP BY s.id, s.name, s.serial_no \
         HAVING remaining_minutes > 0 \
         ORDER BY earliest_deadline IS NULL, earliest_deadline ASC, s.name",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강 관리 데이터 조회 실패: {}", e))?;

    let mut result: Vec<MakeupManagementStudent> = Vec::with_capacity(rows.len());
    for r in rows {
        let earliest_deadline: Option<String> =
            r.try_get("earliest_deadline").map_err(|e| e.to_string())?;
        // 소멸 임박 판정 — deadline 월의 study_periods.end_date 가 (today + 7일) 이내인지.
        let is_imminent = if let Some(ref ym) = earliest_deadline {
            let period_end: Option<String> = sqlx::query_scalar(
                "SELECT end_date FROM study_periods WHERE year_month = ?",
            )
            .bind(ym)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("교습기간 조회 실패: {}", e))?
            .flatten();
            match period_end {
                Some(end_str) => {
                    let end = NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")
                        .map_err(|e| format!("교습기간 종료일 파싱 실패: {}", e))?;
                    end >= today && end <= imminent_threshold
                }
                None => false, // 교습기간 미등록 — 소멸기한 미확정
            }
        } else {
            false
        };

        result.push(MakeupManagementStudent {
            student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
            student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
            serial_no: r.try_get("serial_no").map_err(|e| e.to_string())?,
            remaining_minutes: r.try_get("remaining_minutes").map_err(|e| e.to_string())?,
            earliest_deadline,
            is_imminent,
        });
    }

    Ok(result)
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    async fn seed_student_with_schedule(
        pool: &SqlitePool,
        serial_no: &str,
        name: &str,
        schedules: &[(i64, &str, i64)], // (dow, start_time, duration_hours)
    ) -> i64 {
        let sid: i64 = sqlx::query_scalar(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES (?, ?, 'male', 'elementary', 3, '2026-01-01') RETURNING id",
        )
        .bind(serial_no)
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("seed student");

        for (dow, start_time, duration_hours) in schedules {
            sqlx::query(
                "INSERT INTO student_schedules \
                    (student_id, day_of_week, start_time, duration_hours, effective_from) \
                 VALUES (?, ?, ?, ?, '2026-01-01')",
            )
            .bind(sid)
            .bind(dow)
            .bind(start_time)
            .bind(duration_hours)
            .execute(pool)
            .await
            .expect("seed schedule");
        }
        sid
    }

    async fn seed_period(pool: &SqlitePool, ym: &str, start: &str, end: &str) {
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES (?, ?, ?, 1)",
        )
        .bind(ym)
        .bind(start)
        .bind(end)
        .execute(pool)
        .await
        .expect("seed period");
    }

    async fn seed_attendance(
        pool: &SqlitePool,
        sid: i64,
        event_date: &str,
        ym: &str,
        minutes: i64,
        status: &str,
        deadline: Option<&str>,
    ) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(sid)
        .bind(event_date)
        .bind(ym)
        .bind(status)
        .bind(minutes)
        .bind(deadline)
        .fetch_one(pool)
        .await
        .expect("seed attendance")
    }

    async fn seed_makeup(
        pool: &SqlitePool,
        sid: i64,
        event_date: &str,
        ym: &str,
        minutes: i64,
    ) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "INSERT INTO makeup_attendances \
                (student_id, event_date, year_month, class_minutes) \
             VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(sid)
        .bind(event_date)
        .bind(ym)
        .bind(minutes)
        .fetch_one(pool)
        .await
        .expect("seed makeup")
    }

    // ─────────────── get_calendar_data ───────────────

    /// 정규 수업 + 보강 수업이 일자별로 분리 + 학생 스케줄 start_time 정상 매핑.
    #[tokio::test]
    async fn calendar_groups_regular_and_makeup_by_day() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 월(1)요일 16:00 1시간 수업.
        let sid = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        // 2026-06-01 (월) 정규 출석.
        seed_attendance(&pool, sid, "2026-06-01", "2026-06", 60, "present", None).await;
        // 2026-06-13 (토) 보강 시드.
        seed_makeup(&pool, sid, "2026-06-13", "2026-06", 60).await;

        let result = get_calendar_data_impl(&pool, "2026-06").await.expect("ok");
        assert_eq!(result.year_month, "2026-06");
        assert_eq!(result.days.len(), 2);

        let day1 = &result.days[0];
        assert_eq!(day1.event_date, "2026-06-01");
        assert_eq!(day1.regular_sessions.len(), 1);
        assert_eq!(day1.regular_sessions[0].student_name, "원생A");
        assert_eq!(day1.regular_sessions[0].start_time.as_deref(), Some("16:00"));
        assert_eq!(day1.regular_sessions[0].class_minutes, 60);
        assert!(day1.makeup_sessions.is_empty());

        let day2 = &result.days[1];
        assert_eq!(day2.event_date, "2026-06-13");
        assert!(day2.regular_sessions.is_empty());
        assert_eq!(day2.makeup_sessions.len(), 1);
        assert!(day2.makeup_sessions[0].start_time.is_none(), "보강은 시작 시간 없음");
        assert_eq!(day2.makeup_sessions[0].class_minutes, 60);
    }

    /// year_month 필터 — 다른 월 데이터 제외.
    #[tokio::test]
    async fn calendar_filters_by_year_month() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        seed_attendance(&pool, sid, "2026-05-26", "2026-05", 60, "present", None).await;
        seed_attendance(&pool, sid, "2026-06-01", "2026-06", 60, "present", None).await;

        let jun = get_calendar_data_impl(&pool, "2026-06").await.expect("ok");
        assert_eq!(jun.days.len(), 1);
        assert_eq!(jun.days[0].event_date, "2026-06-01");
    }

    // ─────────────── get_makeup_management_data ───────────────

    /// 보강 필요 원생만 표시 + earliest_deadline 오름차순 정렬 + 잔여 시간 합산.
    #[tokio::test]
    async fn makeup_management_lists_pending_in_deadline_order() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s1 = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        let s2 = seed_student_with_schedule(&pool, "S002", "원생B", &[(1, "17:00", 1)]).await;
        let s3 = seed_student_with_schedule(&pool, "S003", "원생C", &[(1, "18:00", 1)]).await;

        // s1: deadline=2026-07 결석 1건 (60분)
        seed_attendance(&pool, s1, "2026-06-15", "2026-06", 60, "absent", Some("2026-07")).await;
        // s2: deadline=2026-08 결석 2건 (90+30=120분)
        seed_attendance(&pool, s2, "2026-07-05", "2026-07", 90, "absent", Some("2026-08")).await;
        seed_attendance(&pool, s2, "2026-07-10", "2026-07", 30, "absent", Some("2026-08")).await;
        // s3: 결석 0건 (목록 제외)
        let _ = s3;

        let result = get_makeup_management_data_impl(&pool, "2026-06", None).await.expect("ok");
        assert_eq!(result.len(), 2);
        // earliest_deadline ASC 정렬 — 2026-07 먼저, 2026-08 다음.
        assert_eq!(result[0].student_id, s1);
        assert_eq!(result[0].remaining_minutes, 60);
        assert_eq!(result[0].earliest_deadline.as_deref(), Some("2026-07"));
        assert_eq!(result[1].student_id, s2);
        assert_eq!(result[1].remaining_minutes, 120);
        assert_eq!(result[1].earliest_deadline.as_deref(), Some("2026-08"));
    }

    /// 소멸 임박 판정 — deadline 월의 study_periods.end_date 가 (today + 7일) 이내.
    #[tokio::test]
    async fn makeup_management_marks_imminent_within_seven_days() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s1 = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        let s2 = seed_student_with_schedule(&pool, "S002", "원생B", &[(1, "17:00", 1)]).await;
        // s1: deadline=2026-07, 7월 교습기간 종료 2026-07-31 — 기준일 2026-07-26 → 5일 이내 → 임박
        seed_attendance(&pool, s1, "2026-06-15", "2026-06", 60, "absent", Some("2026-07")).await;
        seed_period(&pool, "2026-07", "2026-07-01", "2026-07-31").await;
        // s2: deadline=2026-08, 8월 교습기간 종료 2026-08-31 — 기준일 2026-07-26 → 36일 후 → 임박 아님
        seed_attendance(&pool, s2, "2026-07-05", "2026-07", 60, "absent", Some("2026-08")).await;
        seed_period(&pool, "2026-08", "2026-08-01", "2026-08-31").await;

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 26);
        let result = get_makeup_management_data_impl(&pool, "2026-07", as_of).await.expect("ok");
        assert_eq!(result.len(), 2);
        assert!(result[0].is_imminent, "s1: 7/31 - 7/26 = 5일 → 임박");
        assert!(!result[1].is_imminent, "s2: 8/31 - 7/26 = 36일 → 임박 아님");
    }

    /// 교습기간 미등록 deadline → 소멸 임박 false (PRD §4.5.7 미확정).
    #[tokio::test]
    async fn makeup_management_imminent_false_when_period_missing() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        seed_attendance(&pool, sid, "2026-06-15", "2026-06", 60, "absent", Some("2026-07")).await;
        // 7월 교습기간 시드 없음.

        let as_of = NaiveDate::from_ymd_opt(2026, 7, 1);
        let result = get_makeup_management_data_impl(&pool, "2026-07", as_of).await.expect("ok");
        assert_eq!(result.len(), 1);
        assert!(!result[0].is_imminent, "교습기간 미등록 → 임박 false");
    }

    /// 보강완료/소멸 결석은 카운트 제외 — status='absent' AND makeup_attendance_id IS NULL 필터.
    #[tokio::test]
    async fn makeup_management_excludes_resolved_absences() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student_with_schedule(&pool, "S001", "원생A", &[(1, "16:00", 1)]).await;
        // makeup_done + makeup_expired 시드 — 모두 제외 대상
        sqlx::query(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-06-01', '2026-06', 'makeup_done', 60, '2026-07'), \
                    (?, '2026-06-02', '2026-06', 'makeup_expired', 60, '2026-07')",
        )
        .bind(sid)
        .bind(sid)
        .execute(&pool)
        .await
        .expect("seed resolved");

        let result = get_makeup_management_data_impl(&pool, "2026-06", None).await.expect("ok");
        assert!(result.is_empty(), "미보강 결석 없음 → 목록 빈 결과");
    }
}
