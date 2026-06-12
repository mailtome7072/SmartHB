//! 수업 스케줄 CRUD IPC (Sprint 2 T10, PRD §4.2·§6.2, data-model §1.2).
//!
//! ## 인터페이스
//!
//! - [`set_schedule`] — 원생의 특정 요일 스케줄 설정/변경. 기존 행 `effective_to` 갱신 +
//!   신규 INSERT 패턴으로 변경 이력 보존 (AC-4.2-2).
//! - [`get_schedules`] — 원생별 현행 스케줄 (`effective_to IS NULL`).
//! - [`get_schedule_history`] — 원생별 전체 변경 이력.
//! - [`get_weekly_hours`] — 원생별 주 총 수업시간 (PI-03 용어 통일 권장).
//!
//! ## UNIQUE 제약
//!
//! 부분 인덱스 `UNIQUE(student_id, day_of_week) WHERE effective_to IS NULL` 가 V101
//! 마이그레이션에서 강제. set_schedule 흐름은 기존 행 effective_to 갱신을 먼저 수행하여
//! UNIQUE 충돌 회피.

use crate::commands::db;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// 현행 또는 과거 스케줄 — IPC 응답.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct StudentSchedule {
    pub id: i64,
    pub student_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub duration_hours: i64,
    pub effective_from: String,
    pub effective_to: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl StudentSchedule {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            student_id: row.try_get("student_id")?,
            day_of_week: row.try_get("day_of_week")?,
            start_time: row.try_get("start_time")?,
            duration_hours: row.try_get("duration_hours")?,
            effective_from: row.try_get("effective_from")?,
            effective_to: row.try_get("effective_to")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// 스케줄 설정/변경 payload.
#[derive(Debug, Deserialize)]
pub struct ScheduleSet {
    pub student_id: i64,
    pub day_of_week: i64,
    pub start_time: String,
    pub duration_hours: i64,
    pub effective_from: String,
}

/// 특정 (원생, 요일) 의 현행 행(`effective_to IS NULL`) 을 `effective_to` 로 마감한다 — tx 내부용.
async fn close_current_row_tx(
    tx: &mut sqlx::SqliteConnection,
    student_id: i64,
    day_of_week: i64,
    effective_to: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE student_schedules SET \
            effective_to = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE student_id = ? AND day_of_week = ? AND effective_to IS NULL",
    )
    .bind(effective_to)
    .bind(student_id)
    .bind(day_of_week)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)?;
    Ok(())
}

/// 신규 현행 행 INSERT — tx 내부용. UNIQUE 부분 인덱스는 effective_to IS NULL 행만
/// 대상이므로 호출 전 [`close_current_row_tx`] 로 마감되어 있어야 한다.
async fn insert_schedule_tx(
    tx: &mut sqlx::SqliteConnection,
    payload: &ScheduleSet,
) -> Result<StudentSchedule, AppError> {
    let row = sqlx::query(
        "INSERT INTO student_schedules \
            (student_id, day_of_week, start_time, duration_hours, effective_from) \
         VALUES (?, ?, ?, ?, ?) \
         RETURNING id, student_id, day_of_week, start_time, duration_hours, \
                   effective_from, effective_to, created_at, updated_at",
    )
    .bind(payload.student_id)
    .bind(payload.day_of_week)
    .bind(&payload.start_time)
    .bind(payload.duration_hours)
    .bind(&payload.effective_from)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Db)?;
    StudentSchedule::from_row(&row)
}

pub(crate) async fn set_schedule_impl(
    pool: &sqlx::SqlitePool,
    payload: ScheduleSet,
) -> Result<StudentSchedule, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Db)?;
    // 기존 현행 스케줄 마감. effective_to 는 신규 effective_from 와 동일 일자로 설정
    // (이전 스케줄이 effective_from 직전까지 유효).
    close_current_row_tx(&mut tx, payload.student_id, payload.day_of_week, &payload.effective_from)
        .await?;
    let schedule = insert_schedule_tx(&mut tx, &payload).await?;
    tx.commit().await.map_err(AppError::Db)?;
    Ok(schedule)
}

/// 요일 변경을 단일 트랜잭션으로 수행한다 (P0-7, 2026-06 코드리뷰).
///
/// 기존에는 프론트가 `delete_schedule`(원래 요일 종료) → `set_schedule`(새 요일) 을 순차
/// 호출하여, 중간 실패 시 "원래 요일은 종료됐는데 새 요일은 미등록"인 반쪽 상태가 남을 수
/// 있었다. 본 함수는 원래 요일 마감 + 새 요일 upsert 를 하나의 트랜잭션으로 묶어 부분 실패를
/// 제거한다 — 실패 시 전체 롤백으로 기존 스케줄이 그대로 유지된다.
pub(crate) async fn change_schedule_day_impl(
    pool: &sqlx::SqlitePool,
    payload: ScheduleSet,
    old_day_of_week: i64,
) -> Result<StudentSchedule, AppError> {
    if old_day_of_week == payload.day_of_week {
        // 같은 요일이면 일반 upsert 와 동일.
        return set_schedule_impl(pool, payload).await;
    }
    let mut tx = pool.begin().await.map_err(AppError::Db)?;
    // 1. 원래 요일 현행 행 마감 (effective_from 부터 새 요일로 이동)
    close_current_row_tx(&mut tx, payload.student_id, old_day_of_week, &payload.effective_from)
        .await?;
    // 2. 새 요일 upsert — 혹시 현행 행이 있으면 마감 후 INSERT (set_schedule 과 동일 규약)
    close_current_row_tx(&mut tx, payload.student_id, payload.day_of_week, &payload.effective_from)
        .await?;
    let schedule = insert_schedule_tx(&mut tx, &payload).await?;
    tx.commit().await.map_err(AppError::Db)?;
    Ok(schedule)
}

/// 원생의 (요일별) 스케줄을 설정 또는 변경한다.
///
/// 흐름:
/// 1. 동일 (원생, 요일) 의 현행 스케줄(`effective_to IS NULL`) 이 있으면 `effective_to`
///    를 신규 effective_from 으로 설정하여 마감
/// 2. 신규 행을 `effective_to NULL` 로 INSERT
///
/// 단일 트랜잭션 안에서 수행하여 부분 인덱스 UNIQUE 충돌을 회피한다.
#[tauri::command]
pub async fn set_schedule(payload: ScheduleSet) -> Result<StudentSchedule, String> {
    let pool = db::pool().map_err(String::from)?;
    set_schedule_impl(pool, payload).await.map_err(String::from)
}

/// 요일 변경 (원래 요일 종료 + 새 요일 등록) 을 원자적으로 수행한다 — P0-7.
#[tauri::command]
pub async fn change_schedule_day(
    payload: ScheduleSet,
    old_day_of_week: i64,
) -> Result<StudentSchedule, String> {
    let pool = db::pool().map_err(String::from)?;
    change_schedule_day_impl(pool, payload, old_day_of_week)
        .await
        .map_err(String::from)
}

/// 원생의 특정 요일 스케줄을 마감 처리한다 (Sprint 4 T9 / 사용자 이슈 #10).
///
/// 현행 행(`effective_to IS NULL`) 의 effective_to 를 `today` 로 설정 — 다음날부터 해당
/// 요일에 수업 없음. 행 자체는 보존(이력 추적). 호출 시점에 현행 행이 없으면 no-op.
#[tauri::command]
pub async fn delete_schedule(
    student_id: i64,
    day_of_week: i64,
    today: String,
) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    sqlx::query(
        "UPDATE student_schedules SET \
            effective_to = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE student_id = ? AND day_of_week = ? AND effective_to IS NULL",
    )
    .bind(&today)
    .bind(student_id)
    .bind(day_of_week)
    .execute(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    Ok(())
}

/// 원생의 현행 스케줄 목록 — `effective_to IS NULL` 행만, 요일 오름차순.
#[tauri::command]
pub async fn get_schedules(student_id: i64) -> Result<Vec<StudentSchedule>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT id, student_id, day_of_week, start_time, duration_hours, \
                effective_from, effective_to, created_at, updated_at \
         FROM student_schedules \
         WHERE student_id = ? AND effective_to IS NULL \
         ORDER BY day_of_week ASC",
    )
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    rows.iter()
        .map(StudentSchedule::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

/// 원생의 전체 스케줄 변경 이력 — 최신 effective_from 부터 역순.
#[tauri::command]
pub async fn get_schedule_history(student_id: i64) -> Result<Vec<StudentSchedule>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT id, student_id, day_of_week, start_time, duration_hours, \
                effective_from, effective_to, created_at, updated_at \
         FROM student_schedules \
         WHERE student_id = ? \
         ORDER BY effective_from DESC, id DESC",
    )
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    rows.iter()
        .map(StudentSchedule::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

/// 원생의 주 총 수업시간 — 현행 스케줄의 `duration_hours` 합산 (PI-03 후보).
#[tauri::command]
pub async fn get_weekly_hours(student_id: i64) -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "SELECT COALESCE(SUM(duration_hours), 0) AS total \
         FROM student_schedules \
         WHERE student_id = ? AND effective_to IS NULL",
    )
    .bind(student_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    row.try_get("total").map_err(AppError::Db).map_err(String::from)
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    async fn create_test_student(pool: &sqlx::SqlitePool, serial: &str) -> i64 {
        let row = sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES (?, '학생', 'male', 'elementary', 1, '2026-03-01') \
             RETURNING id",
        )
        .bind(serial)
        .fetch_one(pool)
        .await
        .unwrap();
        row.try_get("id").unwrap()
    }

    async fn insert_schedule(
        pool: &sqlx::SqlitePool,
        student_id: i64,
        day: i64,
        start: &str,
        hours: i64,
        from: &str,
        to: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO student_schedules \
                (student_id, day_of_week, start_time, duration_hours, effective_from, effective_to) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(student_id)
        .bind(day)
        .bind(start)
        .bind(hours)
        .bind(from)
        .bind(to)
        .execute(pool)
        .await
        .unwrap();
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn weekly_hours_sums_only_current_schedules() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "weekly1").await;

        // 현행 3건: 월(2h), 수(1h), 금(2h) = 5h
        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;
        insert_schedule(&pool, sid, 3, "15:00", 1, "2026-03-01", None).await;
        insert_schedule(&pool, sid, 5, "15:00", 2, "2026-03-01", None).await;
        // 마감된 행 (effective_to 설정) — 합산 제외
        insert_schedule(&pool, sid, 2, "15:00", 99, "2026-02-01", Some("2026-03-01")).await;

        // 직접 SQL 검증 (IPC 는 전역 POOL 필요)
        let row = sqlx::query(
            "SELECT COALESCE(SUM(duration_hours), 0) AS total \
             FROM student_schedules WHERE student_id = ? AND effective_to IS NULL",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        let total: i64 = row.try_get("total").unwrap();
        assert_eq!(total, 5, "현행 스케줄만 합산되어야 함");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn schedule_change_creates_history_entry() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "hist1").await;

        // 1단계: 월요일 첫 스케줄
        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;

        // 2단계: 스케줄 변경 — 기존 현행 마감 + 신규 INSERT (set_schedule 패턴 직접 재현)
        let mut tx = pool.begin().await.unwrap();
        sqlx::query(
            "UPDATE student_schedules SET effective_to = ? \
             WHERE student_id = ? AND day_of_week = 1 AND effective_to IS NULL",
        )
        .bind("2026-04-01")
        .bind(sid)
        .execute(&mut *tx)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO student_schedules \
                (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 1, '16:00', 3, '2026-04-01')",
        )
        .bind(sid)
        .execute(&mut *tx)
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // 이력 2건 (마감 + 현행)
        let rows = sqlx::query(
            "SELECT id, student_id, day_of_week, start_time, duration_hours, \
                    effective_from, effective_to, created_at, updated_at \
             FROM student_schedules WHERE student_id = ? ORDER BY effective_from",
        )
        .bind(sid)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 2, "이력 2건 (이전 + 신규)");
        let schedules: Vec<StudentSchedule> = rows.iter().map(|r| StudentSchedule::from_row(r).unwrap()).collect();
        assert_eq!(schedules[0].effective_to.as_deref(), Some("2026-04-01"));
        assert!(schedules[1].effective_to.is_none(), "신규는 현행");
        assert_eq!(schedules[1].duration_hours, 3);
    }

    /// P0-7: 요일 변경이 단일 트랜잭션으로 — 원래 요일 마감 + 새 요일 현행 등록.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn change_schedule_day_closes_old_and_creates_new_atomically() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "chday1").await;
        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;

        let result = change_schedule_day_impl(
            &pool,
            ScheduleSet {
                student_id: sid,
                day_of_week: 3,
                start_time: "16:00:00".to_string(),
                duration_hours: 2,
                effective_from: "2026-06-15".to_string(),
            },
            1,
        )
        .await
        .expect("요일 변경 성공");
        assert_eq!(result.day_of_week, 3);
        assert!(result.effective_to.is_none(), "새 요일은 현행");

        // 원래 요일(월) 은 변경일로 마감, 현행 행은 수요일 1건만.
        let rows = sqlx::query(
            "SELECT day_of_week, effective_to FROM student_schedules WHERE student_id = ? ORDER BY id",
        )
        .bind(sid)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 2);
        let old_to: Option<String> = rows[0].try_get("effective_to").unwrap();
        assert_eq!(old_to.as_deref(), Some("2026-06-15"), "원래 요일 마감");
        let new_to: Option<String> = rows[1].try_get("effective_to").unwrap();
        assert!(new_to.is_none(), "새 요일 현행");
    }

    /// P0-7: 새 요일에 이미 현행 행이 있으면 마감 후 INSERT (UNIQUE 충돌 없이 upsert).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn change_schedule_day_upserts_when_target_day_occupied() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "chday2").await;
        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;
        insert_schedule(&pool, sid, 3, "17:00", 1, "2026-03-01", None).await;

        let result = change_schedule_day_impl(
            &pool,
            ScheduleSet {
                student_id: sid,
                day_of_week: 3,
                start_time: "16:00:00".to_string(),
                duration_hours: 2,
                effective_from: "2026-06-15".to_string(),
            },
            1,
        )
        .await
        .expect("점유된 요일로도 변경 성공 (upsert)");
        assert_eq!(result.day_of_week, 3);

        // 현행 행은 수요일 신규 1건만 — 월요일·기존 수요일 모두 마감.
        let current: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM student_schedules WHERE student_id = ? AND effective_to IS NULL",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(current, 1, "현행 행 1건 (새 수요일)");
    }

    /// P0-7: 같은 요일이면 set_schedule 과 동일한 upsert 동작.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn change_schedule_day_same_day_acts_as_upsert() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "chday3").await;
        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;

        let result = change_schedule_day_impl(
            &pool,
            ScheduleSet {
                student_id: sid,
                day_of_week: 1,
                start_time: "17:00:00".to_string(),
                duration_hours: 3,
                effective_from: "2026-06-15".to_string(),
            },
            1,
        )
        .await
        .expect("같은 요일 변경 성공");
        assert_eq!(result.duration_hours, 3);
        let current: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM student_schedules WHERE student_id = ? AND effective_to IS NULL",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(current, 1);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn unique_constraint_blocks_concurrent_current_schedules() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let sid = create_test_student(&pool, "uniq1").await;

        insert_schedule(&pool, sid, 1, "15:00", 2, "2026-03-01", None).await;
        let err = sqlx::query(
            "INSERT INTO student_schedules \
                (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 1, '17:00', 2, '2026-03-15')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect_err("부분 인덱스 UNIQUE 위반");
        let msg = err.to_string();
        assert!(
            msg.contains("UNIQUE"),
            "UNIQUE 제약 메시지: {}",
            msg
        );
    }
}
