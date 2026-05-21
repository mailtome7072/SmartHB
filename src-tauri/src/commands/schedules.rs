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

/// 원생의 (요일별) 스케줄을 설정 또는 변경한다.
///
/// 흐름:
/// 1. 동일 (원생, 요일) 의 현행 스케줄(`effective_to IS NULL`) 이 있으면 `effective_to`
///    를 `effective_from - 1일`(또는 동일 일자) 로 설정하여 마감
/// 2. 신규 행을 `effective_to NULL` 로 INSERT
///
/// 단일 트랜잭션 안에서 수행하여 부분 인덱스 UNIQUE 충돌을 회피한다.
#[tauri::command]
pub async fn set_schedule(payload: ScheduleSet) -> Result<StudentSchedule, String> {
    let pool = db::pool().map_err(String::from)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    // 기존 현행 스케줄 마감. effective_to 는 신규 effective_from 와 동일 일자로 설정
    // (이전 스케줄이 effective_from 직전까지 유효). UNIQUE 부분 인덱스는 effective_to IS NULL
    // 행만 대상이므로 마감 후 신규 INSERT 가능.
    sqlx::query(
        "UPDATE student_schedules SET \
            effective_to = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE student_id = ? AND day_of_week = ? AND effective_to IS NULL",
    )
    .bind(&payload.effective_from)
    .bind(payload.student_id)
    .bind(payload.day_of_week)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

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
    .map_err(AppError::Db)
    .map_err(String::from)?;

    let schedule = StudentSchedule::from_row(&row).map_err(String::from)?;
    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;
    Ok(schedule)
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

#[cfg(test)]
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
