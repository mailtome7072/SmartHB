//! 학사 스케줄 도메인 IPC (Sprint 6 T5+T6, PRD §4.4·§6.2).
//!
//! ## 인터페이스
//!
//! ### 교습기간 — study_periods (T5)
//! - [`create_study_period`] — 일자 중첩 금지(PRD §6.2)
//! - [`update_study_period`] — 지난 달(AC-4.4-1) 또는 마감(is_closed) 차단
//! - [`list_study_periods`] / [`get_study_period`]
//! - [`confirm_study_period`] — is_confirmed = 1
//! - [`delete_study_period`] — 미확정(is_confirmed=0) 만 허용
//!
//! ### 학사 일정 코드 — schedule_codes (T6)
//! - [`list_schedule_codes`]
//! - [`create_schedule_code`] — 사용자 추가 코드
//! - [`update_schedule_code`] — `is_system_reserved=1` 행의 3속성 수정 차단 (AC-4.4-5)
//! - [`toggle_schedule_code_active`] — 시스템 코드도 활성/비활성 토글은 허용
//!
//! ## V102 스키마 활용
//!
//! `study_periods`, `schedule_codes` 테이블은 V102(Sprint 2)에서 생성 완료. 시스템 예약
//! 5종(보강데이/공휴수업일/방학/단원평가 응시일/휴원일)도 V102 시드로 존재.
//! Sprint 6 은 IPC 레벨 구현만 — DB 변경 없음.

use crate::commands::db;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

// ───────────────────────────────────────────────────────────────── study_periods (T5)

/// 교습기간 (월 단위). PRD §4.4.2.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct StudyPeriod {
    pub id: i64,
    pub year_month: String,
    pub start_date: String,
    pub end_date: String,
    pub is_confirmed: bool,
    pub is_closed: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl StudyPeriod {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            year_month: row.try_get("year_month")?,
            start_date: row.try_get("start_date")?,
            end_date: row.try_get("end_date")?,
            is_confirmed: row.try_get::<i64, _>("is_confirmed")? != 0,
            is_closed: row.try_get::<i64, _>("is_closed")? != 0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// 신규 교습기간 payload.
#[derive(Debug, Deserialize)]
pub struct CreateStudyPeriodPayload {
    pub year_month: String,
    pub start_date: String,
    pub end_date: String,
}

/// 교습기간 수정 payload — id 는 별도 인자.
#[derive(Debug, Deserialize)]
pub struct UpdateStudyPeriodPayload {
    pub start_date: String,
    pub end_date: String,
}

/// 현재 연월 — 지난 달 차단 비교에 사용. 테스트에서 override 가능하도록 단순 헬퍼.
fn current_year_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

/// 교습기간 생성 (PRD §4.4.2). 일자 중첩 시 한국어 에러 반환 (AC-T5-1).
///
/// 중첩 판정: 두 구간 `[a.start, a.end]` 와 `[b.start, b.end]` 가 겹친다 ⇔
/// `a.start <= b.end AND a.end >= b.start`.
#[tauri::command]
pub async fn create_study_period(
    payload: CreateStudyPeriodPayload,
) -> Result<StudyPeriod, String> {
    let pool = db::pool().map_err(String::from)?;

    // 일자 중첩 검증
    let overlap = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM study_periods \
         WHERE start_date <= ? AND end_date >= ?",
    )
    .bind(&payload.end_date)
    .bind(&payload.start_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    let cnt: i64 = overlap.try_get("cnt").map_err(AppError::Db).map_err(String::from)?;
    if cnt > 0 {
        return Err("다른 교습기간과 일자가 중첩됩니다.".to_string());
    }

    let row = sqlx::query(
        "INSERT INTO study_periods (year_month, start_date, end_date) \
         VALUES (?, ?, ?) \
         RETURNING id, year_month, start_date, end_date, is_confirmed, is_closed, \
                   created_at, updated_at",
    )
    .bind(&payload.year_month)
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    StudyPeriod::from_row(&row).map_err(String::from)
}

/// 교습기간 수정 — 지난 달 또는 마감된 기간은 차단 (AC-4.4-1, AC-T5-2).
///
/// 자기 자신은 중첩 검증에서 제외 (`WHERE id != ?`).
#[tauri::command]
pub async fn update_study_period(
    id: i64,
    payload: UpdateStudyPeriodPayload,
) -> Result<StudyPeriod, String> {
    let pool = db::pool().map_err(String::from)?;

    // 대상 기간 조회 + 차단 조건 검증
    let target = sqlx::query("SELECT year_month, is_closed FROM study_periods WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?
        .ok_or_else(|| "해당 교습기간을 찾을 수 없습니다.".to_string())?;

    let year_month: String = target.try_get("year_month").map_err(AppError::Db).map_err(String::from)?;
    let is_closed: i64 = target.try_get("is_closed").map_err(AppError::Db).map_err(String::from)?;

    if is_closed != 0 {
        return Err("마감된 교습기간은 수정할 수 없습니다.".to_string());
    }
    if year_month.as_str() < current_year_month().as_str() {
        return Err("지난 달의 교습기간은 수정할 수 없습니다.".to_string());
    }

    // 일자 중첩 검증 — 자기 자신 제외
    let overlap = sqlx::query(
        "SELECT COUNT(*) AS cnt FROM study_periods \
         WHERE id != ? AND start_date <= ? AND end_date >= ?",
    )
    .bind(id)
    .bind(&payload.end_date)
    .bind(&payload.start_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    let cnt: i64 = overlap.try_get("cnt").map_err(AppError::Db).map_err(String::from)?;
    if cnt > 0 {
        return Err("다른 교습기간과 일자가 중첩됩니다.".to_string());
    }

    let row = sqlx::query(
        "UPDATE study_periods SET \
            start_date = ?, end_date = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, year_month, start_date, end_date, is_confirmed, is_closed, \
                   created_at, updated_at",
    )
    .bind(&payload.start_date)
    .bind(&payload.end_date)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    StudyPeriod::from_row(&row).map_err(String::from)
}

/// 교습기간 목록 — `from_month` ~ `to_month` 범위 (포함). 둘 다 "YYYY-MM" 형식.
#[tauri::command]
pub async fn list_study_periods(
    from_month: String,
    to_month: String,
) -> Result<Vec<StudyPeriod>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT id, year_month, start_date, end_date, is_confirmed, is_closed, \
                created_at, updated_at \
         FROM study_periods \
         WHERE year_month >= ? AND year_month <= ? \
         ORDER BY year_month ASC",
    )
    .bind(&from_month)
    .bind(&to_month)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    rows.iter()
        .map(|r| StudyPeriod::from_row(r).map_err(String::from))
        .collect()
}

/// 특정 월의 교습기간 조회. 없으면 None 반환.
#[tauri::command]
pub async fn get_study_period(year_month: String) -> Result<Option<StudyPeriod>, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "SELECT id, year_month, start_date, end_date, is_confirmed, is_closed, \
                created_at, updated_at \
         FROM study_periods WHERE year_month = ?",
    )
    .bind(&year_month)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    row.map(|r| StudyPeriod::from_row(&r).map_err(String::from))
        .transpose()
}

/// 교습기간 확정 (`is_confirmed = 1`). AC-T5-3.
#[tauri::command]
pub async fn confirm_study_period(id: i64) -> Result<StudyPeriod, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "UPDATE study_periods SET \
            is_confirmed = 1, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, year_month, start_date, end_date, is_confirmed, is_closed, \
                   created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?
    .ok_or_else(|| "해당 교습기간을 찾을 수 없습니다.".to_string())?;
    StudyPeriod::from_row(&row).map_err(String::from)
}

/// 미확정 교습기간 삭제. is_confirmed=1 또는 is_closed=1 이면 차단.
#[tauri::command]
pub async fn delete_study_period(id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let target = sqlx::query("SELECT is_confirmed, is_closed FROM study_periods WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?
        .ok_or_else(|| "해당 교습기간을 찾을 수 없습니다.".to_string())?;
    let is_confirmed: i64 = target.try_get("is_confirmed").map_err(AppError::Db).map_err(String::from)?;
    let is_closed: i64 = target.try_get("is_closed").map_err(AppError::Db).map_err(String::from)?;
    if is_confirmed != 0 || is_closed != 0 {
        return Err("확정 또는 마감된 교습기간은 삭제할 수 없습니다.".to_string());
    }
    sqlx::query("DELETE FROM study_periods WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    Ok(())
}

// ───────────────────────────────────────────────────────────────── schedule_codes (T6)

/// 학사 일정 코드 (3속성 모델). PRD §4.4.3~4.4.5.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ScheduleCode {
    pub id: i64,
    pub code_name: String,
    pub is_system_reserved: bool,
    pub allows_regular_class: bool,
    pub allows_makeup_class: bool,
    pub is_duplicate_blocked: bool,
    pub is_period_type: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl ScheduleCode {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            code_name: row.try_get("code_name")?,
            is_system_reserved: row.try_get::<i64, _>("is_system_reserved")? != 0,
            allows_regular_class: row.try_get::<i64, _>("allows_regular_class")? != 0,
            allows_makeup_class: row.try_get::<i64, _>("allows_makeup_class")? != 0,
            is_duplicate_blocked: row.try_get::<i64, _>("is_duplicate_blocked")? != 0,
            is_period_type: row.try_get::<i64, _>("is_period_type")? != 0,
            is_active: row.try_get::<i64, _>("is_active")? != 0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleCodePayload {
    pub code_name: String,
    pub allows_regular_class: bool,
    pub allows_makeup_class: bool,
    pub is_duplicate_blocked: bool,
    pub is_period_type: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleCodePayload {
    pub allows_regular_class: bool,
    pub allows_makeup_class: bool,
    pub is_duplicate_blocked: bool,
    pub is_period_type: bool,
}

/// 학사 일정 코드 전체 목록 — 시스템 예약 5종 + 사용자 추가. is_active 포함.
#[tauri::command]
pub async fn list_schedule_codes() -> Result<Vec<ScheduleCode>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT id, code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
                is_duplicate_blocked, is_period_type, is_active, created_at, updated_at \
         FROM schedule_codes \
         ORDER BY is_system_reserved DESC, id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    rows.iter()
        .map(|r| ScheduleCode::from_row(r).map_err(String::from))
        .collect()
}

/// 사용자 추가 학사 일정 코드 생성. 보수적 디폴트(OFF/OFF/ON)는 호출 측(프론트엔드) 책임.
/// `code_name` UNIQUE 위반 시 한국어 에러 (AC-T6-4).
#[tauri::command]
pub async fn create_schedule_code(
    payload: CreateScheduleCodePayload,
) -> Result<ScheduleCode, String> {
    let pool = db::pool().map_err(String::from)?;
    let result = sqlx::query(
        "INSERT INTO schedule_codes \
            (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
             is_duplicate_blocked, is_period_type) \
         VALUES (?, 0, ?, ?, ?, ?) \
         RETURNING id, code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
                   is_duplicate_blocked, is_period_type, is_active, created_at, updated_at",
    )
    .bind(&payload.code_name)
    .bind(payload.allows_regular_class)
    .bind(payload.allows_makeup_class)
    .bind(payload.is_duplicate_blocked)
    .bind(payload.is_period_type)
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => ScheduleCode::from_row(&row).map_err(String::from),
        Err(sqlx::Error::Database(e)) if e.message().contains("UNIQUE") => {
            Err(format!("이미 존재하는 코드명입니다: {}", payload.code_name))
        }
        Err(e) => Err(AppError::Db(e).into()),
    }
}

/// 사용자 추가 코드의 3속성 수정. 시스템 예약 코드(`is_system_reserved=1`) 는 차단 (AC-4.4-5, AC-T6-1).
#[tauri::command]
pub async fn update_schedule_code(
    id: i64,
    payload: UpdateScheduleCodePayload,
) -> Result<ScheduleCode, String> {
    let pool = db::pool().map_err(String::from)?;
    let target = sqlx::query("SELECT is_system_reserved FROM schedule_codes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?
        .ok_or_else(|| "해당 학사 일정 코드를 찾을 수 없습니다.".to_string())?;
    let is_system_reserved: i64 = target
        .try_get("is_system_reserved")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    if is_system_reserved != 0 {
        return Err("시스템 예약 코드의 속성은 수정할 수 없습니다.".to_string());
    }

    let row = sqlx::query(
        "UPDATE schedule_codes SET \
            allows_regular_class = ?, allows_makeup_class = ?, \
            is_duplicate_blocked = ?, is_period_type = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
                   is_duplicate_blocked, is_period_type, is_active, created_at, updated_at",
    )
    .bind(payload.allows_regular_class)
    .bind(payload.allows_makeup_class)
    .bind(payload.is_duplicate_blocked)
    .bind(payload.is_period_type)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    ScheduleCode::from_row(&row).map_err(String::from)
}

/// 코드 활성/비활성 토글. 시스템 예약 코드도 토글 허용 (AC-T6-2).
#[tauri::command]
pub async fn toggle_schedule_code_active(id: i64) -> Result<ScheduleCode, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "UPDATE schedule_codes SET \
            is_active = 1 - is_active, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
                   is_duplicate_blocked, is_period_type, is_active, created_at, updated_at",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?
    .ok_or_else(|| "해당 학사 일정 코드를 찾을 수 없습니다.".to_string())?;
    ScheduleCode::from_row(&row).map_err(String::from)
}

// ───────────────────────────────────────────────────────────────── tests

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    // 비즈니스 로직 단위 테스트 — IPC 함수는 전역 POOL 의존이라 직접 호출 불가.
    // schedules.rs 패턴과 동일하게 인메모리 pool 에 raw SQL 을 실행하여 핵심 규칙을 검증.

    async fn insert_period(
        pool: &SqlitePool,
        ym: &str,
        start: &str,
        end: &str,
        is_closed: i64,
    ) -> i64 {
        let row = sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_closed) \
             VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(ym)
        .bind(start)
        .bind(end)
        .bind(is_closed)
        .fetch_one(pool)
        .await
        .unwrap();
        row.try_get("id").unwrap()
    }

    /// AC-T5-1 — 일자 중첩 검증 핵심 SQL 이 정확히 동작하는지.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn overlap_detection_blocks_intersecting_range() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        insert_period(&pool, "2099-03", "2099-03-01", "2099-03-31", 0).await;

        // 중첩 케이스 — 2099-03-15 ~ 2099-04-15 가 기존 [03-01, 03-31] 과 겹침
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM study_periods WHERE start_date <= ? AND end_date >= ?",
        )
        .bind("2099-04-15")
        .bind("2099-03-15")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cnt, 1, "중첩된 기존 행 1개 감지");

        // 비중첩 케이스 — 2099-04-01 ~ 2099-04-30 은 [03-01, 03-31] 과 겹치지 않음
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM study_periods WHERE start_date <= ? AND end_date >= ?",
        )
        .bind("2099-04-30")
        .bind("2099-04-01")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cnt, 0, "비중첩 — 충돌 없음");
    }

    /// AC-T5-2 — 마감(`is_closed = 1`) 행은 수정 차단 조건에 걸려야 함.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn closed_period_is_blocked_by_flag() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let id = insert_period(&pool, "2099-03", "2099-03-01", "2099-03-31", 1).await;
        let closed: i64 = sqlx::query_scalar("SELECT is_closed FROM study_periods WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(closed, 1, "마감 플래그가 조회 시 그대로 반영");
    }

    /// AC-T5-3 — 확정 후 is_confirmed 가 1 로 변경되는지 직접 검증.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn confirm_sets_flag_to_one() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let id = insert_period(&pool, "2099-04", "2099-04-01", "2099-04-30", 0).await;
        sqlx::query("UPDATE study_periods SET is_confirmed = 1 WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let val: i64 = sqlx::query_scalar("SELECT is_confirmed FROM study_periods WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(val, 1);
    }

    /// AC-T6-1 — V102 시드의 시스템 예약 5종이 `is_system_reserved = 1` 로 존재.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn system_reserved_codes_seeded() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_codes WHERE is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cnt, 5, "V102 시스템 예약 5종 시드 확인");
    }

    /// AC-T6-4 — 사용자 추가 코드 INSERT 후 동일 code_name 으로 재시도 시 UNIQUE 위반.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn code_name_unique_violation_detected() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO schedule_codes \
                (code_name, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type) \
             VALUES ('체험학습', 0, 0, 1, 0)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let result = sqlx::query(
            "INSERT INTO schedule_codes \
                (code_name, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type) \
             VALUES ('체험학습', 0, 0, 1, 0)",
        )
        .execute(&pool)
        .await;
        assert!(result.is_err(), "동일 code_name 재삽입 시 UNIQUE 위반");
        let err = format!("{:?}", result.unwrap_err());
        assert!(err.contains("UNIQUE"), "UNIQUE 키워드 포함: {}", err);
    }

    /// AC-T6-5 — 사용자 추가 코드 CRUD smoke + 시스템 코드 토글은 허용 (AC-T6-2).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn user_code_crud_and_system_toggle() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 사용자 추가
        let new_id: i64 = sqlx::query_scalar(
            "INSERT INTO schedule_codes \
                (code_name, allows_regular_class, allows_makeup_class, is_duplicate_blocked, is_period_type) \
             VALUES ('자체평가', 1, 0, 0, 0) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        // 사용자 코드 속성 업데이트 (is_system_reserved=0 이라 허용)
        sqlx::query("UPDATE schedule_codes SET allows_makeup_class = 1 WHERE id = ?")
            .bind(new_id)
            .execute(&pool)
            .await
            .unwrap();
        let makeup: i64 =
            sqlx::query_scalar("SELECT allows_makeup_class FROM schedule_codes WHERE id = ?")
                .bind(new_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(makeup, 1);

        // 시스템 코드 활성/비활성 토글은 SQL 레벨에서 허용. 어플리케이션 가드는 update_schedule_code 만 막음.
        let system_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '보강데이'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("UPDATE schedule_codes SET is_active = 1 - is_active WHERE id = ?")
            .bind(system_id)
            .execute(&pool)
            .await
            .unwrap();
        let active: i64 =
            sqlx::query_scalar("SELECT is_active FROM schedule_codes WHERE id = ?")
                .bind(system_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(active, 0, "토글 후 비활성");
    }
}
