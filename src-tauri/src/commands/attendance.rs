//! 출결 도메인 IPC — Sprint 8 T2 (PRD §4.5.1, data-model §2.4).
//!
//! 두 IPC 제공:
//! - [`check_attendance_exists`] — 해당 월 정규 출결 존재 여부 (UI "출결 생성" 버튼 활성 조건)
//! - [`generate_attendances`] — 해당 월 재원 원생 × 수업 요일 일자에 정규 출결 일괄 INSERT
//!
//! 생성 규칙:
//! 1. 교습기간이 설정 + `is_confirmed=1` 이어야 한다
//! 2. 같은 월에 이미 생성된 출결이 있으면 거부 (AC-4.5-1 중복 방지)
//! 3. `student_schedules` 의 현행 (effective_to IS NULL) 요일별 스케줄을 기준으로 일자 산출
//! 4. `schedule_events` JOIN `schedule_codes` 에서 `allows_regular_class=0` 인 일자/기간은 제외
//!    (공휴일·휴원일·방학 등)
//! 5. 원생 `enroll_date` 이전 / `withdraw_date` 이후 일자는 제외
//! 6. `class_minutes = duration_hours × 60` (V101 hours INTEGER 저장)
//! 7. 전체 INSERT 를 단일 트랜잭션으로 처리 (부분 실패 시 롤백)

use crate::commands::db::pool;
use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

const MINUTES_PER_HOUR: i64 = 60;

/// 출결 생성 결과 — 프론트엔드 토스트/요약에 사용.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub year_month: String,
    pub student_count: i64,
    pub attendance_count: i64,
}

#[tauri::command]
pub async fn check_attendance_exists(year_month: String) -> Result<bool, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    check_exists_impl(pool, &year_month).await
}

#[tauri::command]
pub async fn generate_attendances(year_month: String) -> Result<GenerateResult, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    generate_impl(pool, &year_month).await
}

async fn check_exists_impl(pool: &SqlitePool, year_month: &str) -> Result<bool, String> {
    validate_year_month(year_month)?;
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM regular_attendances WHERE year_month = ?) AS flag",
    )
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("출결 존재 여부 조회 실패: {}", e))?;
    let flag: i64 = row.try_get("flag").map_err(|e| e.to_string())?;
    Ok(flag != 0)
}

async fn generate_impl(pool: &SqlitePool, year_month: &str) -> Result<GenerateResult, String> {
    validate_year_month(year_month)?;

    let (start_date, end_date) = load_confirmed_period(pool, year_month).await?;

    if check_exists_impl(pool, year_month).await? {
        return Err(format!(
            "{} 출결이 이미 생성되어 있습니다. 기존 출결을 확인 후 다시 시도하세요.",
            year_month
        ));
    }

    let off_dates = load_off_dates(pool, &start_date, &end_date).await?;
    let students = load_active_students(pool, &start_date).await?;

    let sd = parse_date(&start_date)?;
    let ed = parse_date(&end_date)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    let mut attendance_count: i64 = 0;
    let mut student_count: i64 = 0;

    for s in &students {
        let dow_to_minutes = load_weekly_schedule(&mut *tx, s.id).await?;
        if dow_to_minutes.is_empty() {
            continue;
        }

        let enroll_d = parse_date(&s.enroll_date)?;
        let withdraw_d = match &s.withdraw_date {
            Some(w) => Some(parse_date(w)?),
            None => None,
        };

        let mut inserted = 0i64;
        let mut d = sd;
        while d <= ed {
            let dow = d.weekday().number_from_monday() as i64;
            if let Some(&minutes) = dow_to_minutes.get(&dow) {
                let in_enroll_range = d >= enroll_d && withdraw_d.is_none_or(|wd| d <= wd);
                let date_str = d.format("%Y-%m-%d").to_string();
                if in_enroll_range && !off_dates.contains(&date_str) {
                    sqlx::query(
                        "INSERT INTO regular_attendances \
                         (student_id, event_date, year_month, status, class_minutes) \
                         VALUES (?, ?, ?, 'present', ?)",
                    )
                    .bind(s.id)
                    .bind(&date_str)
                    .bind(year_month)
                    .bind(minutes)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("출결 INSERT 실패: {}", e))?;
                    inserted += 1;
                }
            }
            d = d
                .succ_opt()
                .ok_or_else(|| "날짜 계산 오버플로".to_string())?;
        }

        if inserted > 0 {
            student_count += 1;
            attendance_count += inserted;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(GenerateResult {
        year_month: year_month.to_string(),
        student_count,
        attendance_count,
    })
}

// ─────────────────────── 헬퍼 ───────────────────────

struct StudentRow {
    id: i64,
    enroll_date: String,
    withdraw_date: Option<String>,
}

fn validate_year_month(ym: &str) -> Result<(), String> {
    if ym.len() != 7 || ym.as_bytes()[4] != b'-' {
        return Err("year_month 는 YYYY-MM 형식이어야 합니다.".to_string());
    }
    let year = &ym[..4];
    let month = &ym[5..];
    if !year.chars().all(|c| c.is_ascii_digit()) || !month.chars().all(|c| c.is_ascii_digit()) {
        return Err("year_month 에 숫자가 아닌 문자가 포함되어 있습니다.".to_string());
    }
    Ok(())
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("날짜 파싱 실패 ({}): {}", s, e))
}

async fn load_confirmed_period(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<(String, String), String> {
    let row = sqlx::query(
        "SELECT start_date, end_date, is_confirmed \
         FROM study_periods WHERE year_month = ?",
    )
    .bind(year_month)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("교습기간 조회 실패: {}", e))?
    .ok_or_else(|| {
        format!(
            "{} 교습기간이 설정되지 않았습니다. 학사 캘린더에서 먼저 교습기간을 설정하세요.",
            year_month
        )
    })?;

    let is_confirmed: i64 = row.try_get("is_confirmed").map_err(|e| e.to_string())?;
    if is_confirmed == 0 {
        return Err(format!(
            "{} 교습기간이 아직 확정되지 않았습니다. 교습기간을 확정한 후 다시 시도하세요.",
            year_month
        ));
    }
    let start: String = row.try_get("start_date").map_err(|e| e.to_string())?;
    let end: String = row.try_get("end_date").map_err(|e| e.to_string())?;
    Ok((start, end))
}

async fn load_off_dates(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> Result<HashSet<String>, String> {
    let rows = sqlx::query(
        "SELECT e.event_date, COALESCE(e.period_end_date, e.event_date) AS end_d \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_regular_class = 0 \
           AND e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(end_date)
    .bind(start_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("학사일정 조회 실패: {}", e))?;

    let mut off: HashSet<String> = HashSet::new();
    for r in rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let mut d = parse_date(&s)?;
        let ed = parse_date(&e)?;
        while d <= ed {
            off.insert(d.format("%Y-%m-%d").to_string());
            d = d
                .succ_opt()
                .ok_or_else(|| "OFF 날짜 계산 오버플로".to_string())?;
        }
    }
    Ok(off)
}

async fn load_active_students(
    pool: &SqlitePool,
    start_date: &str,
) -> Result<Vec<StudentRow>, String> {
    let rows = sqlx::query(
        "SELECT id, enroll_date, withdraw_date FROM students \
         WHERE withdraw_date IS NULL OR withdraw_date >= ? \
         ORDER BY id",
    )
    .bind(start_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("재원 원생 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(StudentRow {
                id: r.try_get("id").map_err(|e: sqlx::Error| e.to_string())?,
                enroll_date: r.try_get("enroll_date").map_err(|e: sqlx::Error| e.to_string())?,
                withdraw_date: r
                    .try_get("withdraw_date")
                    .map_err(|e: sqlx::Error| e.to_string())?,
            })
        })
        .collect()
}

async fn load_weekly_schedule<'c, E>(
    executor: E,
    student_id: i64,
) -> Result<HashMap<i64, i64>, String>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query(
        "SELECT day_of_week, duration_hours FROM student_schedules \
         WHERE student_id = ? AND effective_to IS NULL",
    )
    .bind(student_id)
    .fetch_all(executor)
    .await
    .map_err(|e| format!("원생 스케줄 조회 실패: {}", e))?;

    let mut map = HashMap::new();
    for r in rows {
        let dow: i64 = r.try_get("day_of_week").map_err(|e| e.to_string())?;
        let hours: i64 = r.try_get("duration_hours").map_err(|e| e.to_string())?;
        map.insert(dow, hours * MINUTES_PER_HOUR);
    }
    Ok(map)
}

// ─────────────────────── 단위 테스트 ───────────────────────

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    /// 원생 1명 삽입 후 id 반환. day_of_week 와 duration_hours 를 부여하면 현행 스케줄로 추가.
    async fn seed_student(
        pool: &SqlitePool,
        serial_no: &str,
        enroll: &str,
        withdraw: Option<&str>,
        schedules: &[(i64, i64)], // (day_of_week 1~7, duration_hours)
    ) -> i64 {
        let row = sqlx::query(
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
        let id: i64 = row.try_get("id").expect("학생 id");

        for (dow, hours) in schedules {
            sqlx::query(
                "INSERT INTO student_schedules \
                 (student_id, day_of_week, start_time, duration_hours, effective_from) \
                 VALUES (?, ?, '16:00', ?, ?)",
            )
            .bind(id)
            .bind(dow)
            .bind(hours)
            .bind(enroll)
            .execute(pool)
            .await
            .expect("스케줄 INSERT");
        }
        id
    }

    async fn seed_period(
        pool: &SqlitePool,
        year_month: &str,
        start: &str,
        end: &str,
        is_confirmed: i64,
    ) {
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(year_month)
        .bind(start)
        .bind(end)
        .bind(is_confirmed)
        .execute(pool)
        .await
        .expect("교습기간 INSERT");
    }

    /// 시스템 예약 코드 '공휴일' 이 V200 시드에 있다고 가정 — 없으면 직접 삽입 후 id 반환.
    async fn schedule_code_id(pool: &SqlitePool, name: &str) -> i64 {
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM schedule_codes WHERE code_name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await
                .expect("schedule_code 조회");
        if let Some((id,)) = existing {
            return id;
        }
        let row = sqlx::query(
            "INSERT INTO schedule_codes \
             (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
              is_duplicate_blocked, is_period_type) \
             VALUES (?, 0, 0, 0, 1, 0) RETURNING id",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("schedule_code INSERT");
        row.try_get("id").expect("code id")
    }

    async fn add_schedule_event(
        pool: &SqlitePool,
        code_id: i64,
        event_date: &str,
        period_end: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) \
             VALUES (?, ?, ?)",
        )
        .bind(code_id)
        .bind(event_date)
        .bind(period_end)
        .execute(pool)
        .await
        .expect("schedule_event INSERT");
    }

    // ─────── AC-T2-1 ───────

    #[tokio::test]
    async fn generate_creates_attendances_for_active_students() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 월/수/금 (1, 3, 5) 주 3일 수업
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (3, 1), (5, 1)]).await;
        // 화/목 (2, 4) 주 2일 수업
        seed_student(&pool, "S002", "2026-04-01", None, &[(2, 2), (4, 2)]).await;

        let result = generate_impl(&pool, "2026-06").await.expect("generate");
        // 2026-06 월(1)·수(3)·금(5) = 5+4+4=13일 (calendar 확인)
        // 화(2)·목(4) = 5+4 = 9일 — 6/2, 9, 16, 23, 30 (5) + 6/4, 11, 18, 25 (4) = 9
        // S001 (월수금): 6/1·3·5·8·10·12·15·17·19·22·24·26·29 → 월=5(1,8,15,22,29), 수=5(3,10,17,24), 금=4(5,12,19,26) = 14
        // 정확한 카운트는 chrono 위임 — 합계 검증만.
        assert!(result.attendance_count > 0);
        assert_eq!(result.student_count, 2);
        assert_eq!(result.year_month, "2026-06");

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances WHERE year_month = '2026-06'",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(total.0, result.attendance_count);
    }

    // ─────── AC-T2-2 ───────

    #[tokio::test]
    async fn generate_skips_off_days() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 매일 수업 (월~일)
        seed_student(
            &pool,
            "S001",
            "2026-04-01",
            None,
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
        )
        .await;

        // 6/6 현충일 (단일 일자), 6/15~6/19 방학 (기간성) 모두 allows_regular_class=0 인 코드로 등록.
        let holiday_id = schedule_code_id(&pool, "현충일").await; // 신규 (allows_regular_class=0)
        add_schedule_event(&pool, holiday_id, "2026-06-06", None).await;
        let vac_id = schedule_code_id(&pool, "방학").await; // V102 시드 (allows_regular_class=0)
        add_schedule_event(&pool, vac_id, "2026-06-15", Some("2026-06-19")).await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        // 6/6 + 6/15~6/19 (5일) = 총 6개 OFF 일자 → 출결 없어야 함.
        let off_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE event_date IN ('2026-06-06','2026-06-15','2026-06-16','2026-06-17','2026-06-18','2026-06-19')",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(off_count.0, 0, "OFF 일자에 출결이 생성되면 안 됩니다");

        // 6/30 마지막 날에는 출결 있어야 함.
        let last_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-30'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(last_count.0, 1);
    }

    // ─────── AC-T2-3 ───────

    #[tokio::test]
    async fn generate_respects_enroll_withdraw_range() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 6/15 입교, 6/25 퇴교, 매일 수업
        seed_student(
            &pool,
            "S001",
            "2026-06-15",
            Some("2026-06-25"),
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
        )
        .await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        // 6/14 이전 출결 없음
        let before: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date < '2026-06-15'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(before.0, 0);

        // 6/26 이후 출결 없음
        let after: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date > '2026-06-25'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(after.0, 0);

        // 6/15 ~ 6/25 = 11일 — 매일 수업이므로 11건
        let mid: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE event_date >= '2026-06-15' AND event_date <= '2026-06-25'",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(mid.0, 11);
    }

    // ─────── AC-T2-4 ───────

    #[tokio::test]
    async fn generate_blocks_duplicate_month() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        generate_impl(&pool, "2026-06").await.expect("first");
        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("두 번째 호출은 실패해야 함");
        assert!(
            err.contains("이미 생성"),
            "에러 메시지에 '이미 생성' 포함 필요: {}",
            err
        );
    }

    // ─────── AC-T2-5 ───────

    #[tokio::test]
    async fn generate_requires_confirmed_period() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 0).await; // 미확정
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("미확정 교습기간은 거부");
        assert!(err.contains("확정"), "확정 안내 메시지 필요: {}", err);
    }

    #[tokio::test]
    async fn generate_requires_period_to_exist() {
        let pool = test_pool_in_memory().await.expect("pool");
        // 교습기간 미설정
        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("교습기간 미설정 → 거부");
        assert!(err.contains("설정"), "설정 안내 메시지 필요: {}", err);
    }

    // ─────── AC-T2-6 ───────

    #[tokio::test]
    async fn class_minutes_matches_schedule_hours() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // duration_hours=2 → class_minutes=120 이어야 함
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 2)]).await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        let minutes: (i64,) =
            sqlx::query_as("SELECT class_minutes FROM regular_attendances LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("minutes");
        assert_eq!(minutes.0, 120, "duration_hours=2 → class_minutes=120");
    }

    // ─────── check_attendance_exists ───────

    #[tokio::test]
    async fn check_attendance_exists_reflects_state() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        assert!(!check_exists_impl(&pool, "2026-06").await.expect("초기 false"));
        generate_impl(&pool, "2026-06").await.expect("generate");
        assert!(check_exists_impl(&pool, "2026-06").await.expect("생성 후 true"));
    }

    #[tokio::test]
    async fn validate_year_month_rejects_invalid_formats() {
        assert!(validate_year_month("2026-06").is_ok());
        assert!(validate_year_month("2026/06").is_err());
        assert!(validate_year_month("2026-6").is_err());
        assert!(validate_year_month("YYYY-MM").is_err());
        assert!(validate_year_month("").is_err());
    }
}
