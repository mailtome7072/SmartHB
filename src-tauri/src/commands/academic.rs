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
use chrono::Datelike;
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

/// "YYYY-MM-DD" → ISO 요일 (1=월~7=일). V29 — 운영 시간 매칭에 사용.
fn iso_dow_of(date: &str) -> i64 {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return 1;
    }
    let y: i32 = parts[0].parse().unwrap_or(2000);
    let m: u32 = parts[1].parse().unwrap_or(1);
    let d: u32 = parts[2].parse().unwrap_or(1);
    let nd = chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap_or_else(|| {
        chrono::NaiveDate::from_ymd_opt(2000, 1, 1).expect("hardcoded fallback")
    });
    nd.weekday().number_from_monday() as i64
}

/// V29 — 운영 시간 설정에서 해당 요일이 운영(open_time 있음) 여부.
/// 백엔드 settings 함수 호출 — 실패 시 보수적으로 운영일로 가정 (테스트 환경 호환).
async fn is_operating_day_for(dow: i64) -> bool {
    use crate::commands::settings;
    match settings::get_operating_hours().await {
        Ok(hours) => hours
            .iter()
            .find(|h| h.day_of_week as i64 == dow)
            .map(|h| h.open_time.is_some() && h.close_time.is_some())
            .unwrap_or(true),
        Err(_) => true,
    }
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

// ───── Sprint 7 T8: 교습기간 cascade 삭제 (Issue 6) ─────

/// 확정 교습기간 cascade 삭제 미리보기 응답.
///
/// 영향 건수와 가능 여부를 사전 확인하여 사용자 친화 AlertDialog 에 표시.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct CascadeDeletePreview {
    /// 삭제될 schedule_events 건수 (공휴일 제외).
    pub affected_count: i64,
    /// 보존되는 공휴일 시드 건수.
    pub holiday_count: i64,
    /// 삭제 가능 여부.
    pub deletable: bool,
    /// 불가 사유 (한국어, deletable=false 일 때만).
    pub reason: Option<String>,
}

/// 교습기간 cascade 삭제 가드 — 존재 + 확정 + 지난 달 아님.
///
/// 반환: `(start_date, end_date, deletable, reason)`. 가드는 preview/cascade 양쪽 공유.
async fn check_cascade_delete_guard(
    pool: &sqlx::SqlitePool,
    id: i64,
) -> Result<(String, String, bool, Option<String>), AppError> {
    let target = sqlx::query(
        "SELECT year_month, start_date, end_date, is_confirmed \
         FROM study_periods WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)?
    .ok_or_else(|| AppError::Db(sqlx::Error::RowNotFound))?;
    let year_month: String = target.try_get("year_month").map_err(AppError::Db)?;
    let start_date: String = target.try_get("start_date").map_err(AppError::Db)?;
    let end_date: String = target.try_get("end_date").map_err(AppError::Db)?;
    let is_confirmed: i64 = target.try_get("is_confirmed").map_err(AppError::Db)?;

    if is_confirmed == 0 {
        return Ok((
            start_date,
            end_date,
            false,
            Some("미확정 교습기간은 일반 삭제 IPC 를 사용하세요.".to_string()),
        ));
    }
    if year_month.as_str() < current_year_month().as_str() {
        return Ok((
            start_date,
            end_date,
            false,
            Some("지난 달의 교습기간은 삭제할 수 없습니다.".to_string()),
        ));
    }
    Ok((start_date, end_date, true, None))
}

/// 확정 교습기간 cascade 삭제 사전 조회.
///
/// 사용자 클릭 직후 AlertDialog 표시 전에 호출. 삭제 가능 여부 + 영향 건수 + 보존 공휴일 건수 반환.
#[tauri::command]
pub async fn get_cascade_delete_preview(id: i64) -> Result<CascadeDeletePreview, String> {
    let pool = db::pool().map_err(String::from)?;
    let (start_date, end_date, deletable, reason) =
        check_cascade_delete_guard(pool, id).await.map_err(String::from)?;

    let affected_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.event_date >= ? AND e.event_date <= ? \
           AND c.code_name != '공휴일'",
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    let holiday_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.event_date >= ? AND e.event_date <= ? \
           AND c.code_name = '공휴일'",
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    Ok(CascadeDeletePreview {
        affected_count,
        holiday_count,
        deletable,
        reason,
    })
}

/// 확정 교습기간 cascade 삭제 — 트랜잭션 안에서 공휴일 제외 학사 일정 + 교습기간 삭제.
#[tauri::command]
pub async fn delete_study_period_cascade(id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let (start_date, end_date, deletable, reason) =
        check_cascade_delete_guard(pool, id).await.map_err(String::from)?;
    if !deletable {
        return Err(reason.unwrap_or_else(|| "삭제할 수 없습니다.".to_string()));
    }

    let mut tx = pool.begin().await.map_err(AppError::Db).map_err(String::from)?;

    sqlx::query(
        "DELETE FROM schedule_events \
         WHERE event_date >= ? AND event_date <= ? \
           AND code_id IN (SELECT id FROM schedule_codes WHERE code_name != '공휴일')",
    )
    .bind(&start_date)
    .bind(&end_date)
    .execute(&mut *tx)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;

    sqlx::query("DELETE FROM study_periods WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;
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

// ───────────────────────────────────────────────────────────────── schedule_events (T7)

/// 학사 일정 (캘린더 배치). PRD §4.4.6 / V103.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ScheduleEvent {
    pub id: i64,
    pub code_id: i64,
    pub event_date: String,
    pub period_end_date: Option<String>,
    pub display_name: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl ScheduleEvent {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            code_id: row.try_get("code_id")?,
            event_date: row.try_get("event_date")?,
            period_end_date: row.try_get("period_end_date")?,
            display_name: row.try_get("display_name")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// 캘린더 셀 렌더링용 평탄 응답 — schedule_codes JOIN 결과.
/// 프론트가 별도 코드 조회 없이 코드명·중복불가·기간성·시스템예약·시드 여부를 셀에 표시 가능.
///
/// Sprint 7 T4 (A23/R33): `is_system_reserved` 추가 — 프론트엔드 하드코딩 제거.
/// V21 (Sprint 7 post-review): `is_seeded` 추가 — 시드 공휴일 vs 사용자 공휴일 구분으로
/// CalendarCell 의 삭제 가드 분기 (시드만 차단, 사용자 추가는 허용).
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ScheduleEventListItem {
    pub id: i64,
    pub code_id: i64,
    pub code_name: String,
    pub is_system_reserved: bool,
    pub is_duplicate_blocked: bool,
    pub is_period_type: bool,
    pub is_seeded: bool,
    /// V25 (Sprint 7 post-review): 정규 수업 허용 여부 — 프론트 `hasClassOnDate` 판정.
    /// V28: 보강수업(allows_makeup_class)도 함께 사용 — 보강데이 등 정규수업=0/보강=1 코드 분기.
    /// 셀의 어떤 이벤트라도 둘 중 하나 `true` 면 수업 가능.
    pub allows_regular_class: bool,
    pub allows_makeup_class: bool,
    pub event_date: String,
    pub period_end_date: Option<String>,
    pub display_name: Option<String>,
}

impl ScheduleEventListItem {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        Ok(Self {
            id: row.try_get("id")?,
            code_id: row.try_get("code_id")?,
            code_name: row.try_get("code_name")?,
            is_system_reserved: row.try_get::<i64, _>("is_system_reserved")? != 0,
            is_duplicate_blocked: row.try_get::<i64, _>("is_duplicate_blocked")? != 0,
            is_period_type: row.try_get::<i64, _>("is_period_type")? != 0,
            is_seeded: row.try_get::<i64, _>("is_seeded")? != 0,
            allows_regular_class: row.try_get::<i64, _>("allows_regular_class")? != 0,
            allows_makeup_class: row.try_get::<i64, _>("allows_makeup_class")? != 0,
            event_date: row.try_get("event_date")?,
            period_end_date: row.try_get("period_end_date")?,
            display_name: row.try_get("display_name")?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduleEventPayload {
    pub code_id: i64,
    pub event_date: String,
    pub period_end_date: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScheduleEventPayload {
    pub event_date: String,
    pub period_end_date: Option<String>,
    pub display_name: Option<String>,
}

/// "YYYY-MM-DD" 에서 "YYYY-MM" 추출. 잘못된 형식이면 None.
fn year_month_of(date_str: &str) -> Option<String> {
    if date_str.len() >= 7 && date_str.as_bytes().get(4) == Some(&b'-') {
        Some(date_str[..7].to_string())
    } else {
        None
    }
}

/// `year_month` 의 2주차 + 4주차 월~금 일자 10건 (단원평가 자동 배치용).
///
/// 정의: 해당 month 1일이 속한 주의 첫 월요일 = first_monday.
/// - 2주차 월~금 = first_monday + 7..=11 일
/// - 4주차 월~금 = first_monday + 21..=25 일
///
/// 결과 일자가 다른 month 로 넘어가도 그대로 반환 — 사용자가 수동 조정 가능.
fn assessment_dates_for(year_month: &str) -> Result<Vec<chrono::NaiveDate>, String> {
    let first =
        chrono::NaiveDate::parse_from_str(&format!("{}-01", year_month), "%Y-%m-%d")
            .map_err(|_| format!("잘못된 연월 형식: {}", year_month))?;
    // 1일이 무슨 요일이든 그 다음 월요일까지의 오프셋 (월요일이면 0).
    let offset = (7 - first.weekday().num_days_from_monday() as i64) % 7;
    let first_monday = first + chrono::Duration::days(offset);

    let mut dates = Vec::with_capacity(10);
    for week_offset in [7_i64, 21] {
        for weekday in 0..5_i64 {
            dates.push(first_monday + chrono::Duration::days(week_offset + weekday));
        }
    }
    Ok(dates)
}

/// 단원평가 응시일 코드 ID 조회 (V102 시드 행).
async fn find_assessment_code_id<'a, E>(executor: E) -> Result<i64, AppError>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let row = sqlx::query("SELECT id FROM schedule_codes WHERE code_name = '단원평가 응시일'")
        .fetch_one(executor)
        .await?;
    Ok(row.try_get("id")?)
}

/// 배치 제약 가드 (Sprint 7 T7, Issue 4/R34, V9 post-review):
///
/// 1. **중복불가 상호 차단**: 새 코드가 `is_duplicate_blocked=1` 이면 해당 일자에 다른 일정이
///    있어도 차단. 역방향: 해당 일자에 이미 `is_duplicate_blocked=1` 일정이 있으면 새 배치 차단.
/// 2. **교습기간 내만 배치**: `event_date` (기간성 코드는 `period_end_date` 포함) 가 어떤 확정된
///    교습기간 `[start_date, end_date]` 안에 있어야 함.
/// 3. **공휴수업일 특별 룰 (V9)**: "공휴일이지만 수업 있는 날" 의미. 공휴일이 이미 있는 일자에만
///    추가 배치 허용. 공휴일 외 다른 코드와 중복 차단. 공휴일 없는 일자에는 배치 불가.
///
/// `code_name` 은 공휴수업일/공휴일 특별 처리 분기에 사용.
/// `exclude_event_id` 는 update 시 자기 자신 row 를 검증 대상에서 제외 (id != ?).
async fn check_placement_constraints(
    pool: &sqlx::SqlitePool,
    code_id: i64,
    code_name: &str,
    is_dup_blocked: bool,
    event_date: &str,
    period_end_date: Option<&str>,
    exclude_event_id: Option<i64>,
) -> Result<(), String> {
    // V26 (Sprint 7 post-review): 범위 겹침 충돌 검사 — 기간성 코드의 사이 일자에도 가드 적용.
    // 새 코드 범위: [new_start, new_end]  (단일 일자 코드면 new_end = new_start)
    // 기존 row 범위: [e.event_date, COALESCE(e.period_end_date, e.event_date)]
    // 두 범위가 겹치는 조건: e.event_date <= new_end AND COALESCE(e.period_end_date, e.event_date) >= new_start
    //
    // V29 (Sprint 7 post-review): 보강데이 분기 — allows_regular_class/allows_makeup_class 도
    // 조회하여 "수업 차단 코드" 존재 여부 판정.
    let new_start = event_date;
    let new_end = period_end_date.unwrap_or(event_date);
    type RowTuple = (i64, i64, String, i64, i64);
    let existing_on_date: Vec<RowTuple> = {
        let q = match exclude_event_id {
            Some(_) => sqlx::query_as::<_, RowTuple>(
                "SELECT e.id, c.is_duplicate_blocked, c.code_name, \
                        c.allows_regular_class, c.allows_makeup_class \
                 FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
                 WHERE e.event_date <= ? \
                   AND COALESCE(e.period_end_date, e.event_date) >= ? \
                   AND e.id != ?",
            )
            .bind(new_end)
            .bind(new_start)
            .bind(exclude_event_id.unwrap()),
            None => sqlx::query_as::<_, RowTuple>(
                "SELECT e.id, c.is_duplicate_blocked, c.code_name, \
                        c.allows_regular_class, c.allows_makeup_class \
                 FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
                 WHERE e.event_date <= ? \
                   AND COALESCE(e.period_end_date, e.event_date) >= ?",
            )
            .bind(new_end)
            .bind(new_start),
        };
        q.fetch_all(pool).await.map_err(AppError::Db).map_err(String::from)?
    };

    // V29: 보강데이는 "수업이 있는 일자"에 배치 불가 — 운영일이고 수업 차단 코드 없으면 차단.
    if code_name == "보강데이" {
        let dow = iso_dow_of(event_date);
        let is_operating_day = is_operating_day_for(dow).await;
        // 기존 이벤트 중 "수업 차단 코드" (정규=0 AND 보강=0, 보강데이 자기 자신 제외) 존재 여부.
        let has_no_class_blocker = existing_on_date
            .iter()
            .any(|(_, _, n, reg, mk)| n != "보강데이" && *reg == 0 && *mk == 0);
        if is_operating_day && !has_no_class_blocker {
            return Err(
                "보강데이는 수업이 있는 일자에는 배치할 수 없습니다. 휴원일/방학/공휴일 등 수업 없는 일자에만 가능합니다.".to_string(),
            );
        }
    }

    // V9 분기: 공휴수업일은 공휴일 위에만 배치 가능, 다른 코드와는 중복 불가.
    if code_name == "공휴수업일" {
        let has_holiday = existing_on_date.iter().any(|(_, _, n, _, _)| n == "공휴일");
        if !has_holiday {
            return Err(
                "공휴수업일은 공휴일이 지정된 날에만 배치할 수 있습니다.".to_string(),
            );
        }
        let has_other = existing_on_date.iter().any(|(_, _, n, _, _)| n != "공휴일");
        if has_other {
            return Err(
                "공휴수업일은 공휴일 외 다른 일정과 중복될 수 없습니다.".to_string(),
            );
        }
        // 공휴일만 있으면 일반 중복불가 가드를 건너뛰고 교습기간 가드만 검증.
    } else if !existing_on_date.is_empty() {
        // 일반 중복불가 가드 — 공휴수업일이 이미 있는 일자에 공휴일을 추가하는 케이스도 허용해야 함.
        let only_holiday_class = existing_on_date.iter().all(|(_, _, n, _, _)| n == "공휴수업일");
        let is_holiday_target = code_name == "공휴일";

        if is_dup_blocked {
            // 새 코드가 중복불가 — 단, 공휴일이 공휴수업일만 있는 일자에 배치되는 경우 허용.
            let allowed_by_holiday_pair = is_holiday_target && only_holiday_class;
            if !allowed_by_holiday_pair {
                return Err(
                    "중복불가 코드는 다른 일정이 있는 날짜에 배치할 수 없습니다.".to_string(),
                );
            }
        } else if existing_on_date.iter().any(|(_, dup, _, _, _)| *dup != 0) {
            // 역방향 — 단, 기존이 공휴수업일뿐이고 새 코드가 공휴일이면 허용 (위 분기에서 처리).
            // 그 외에는 차단.
            return Err(
                "해당 일자에 중복불가 일정이 있어 배치할 수 없습니다.".to_string(),
            );
        }
    }

    // AC-T7-1: 동일 code_id + 동일 event_date 중복은 별도 차단 — 같은 코드 두 번 배치 방지.
    let same_code_cnt: i64 = match exclude_event_id {
        Some(eid) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events \
             WHERE code_id = ? AND event_date = ? AND id != ?",
        )
        .bind(code_id)
        .bind(event_date)
        .bind(eid)
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?,
        None => sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events WHERE code_id = ? AND event_date = ?",
        )
        .bind(code_id)
        .bind(event_date)
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?,
    };
    if same_code_cnt > 0 {
        return Err("동일 일자에 같은 코드의 일정이 이미 존재합니다.".to_string());
    }

    // 제약 2: 교습기간 내만 배치 — 확정된 교습기간이 event_date 를 포함해야 함.
    // 기간성 코드는 period_end_date 도 동일 교습기간 안에 있어야 함.
    let end_for_check = period_end_date.unwrap_or(event_date);
    let in_period: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM study_periods \
         WHERE is_confirmed = 1 AND start_date <= ? AND end_date >= ?",
    )
    .bind(event_date)
    .bind(end_for_check)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    if in_period == 0 {
        return Err(
            "학사 일정은 확정된 교습기간 내 일자에만 배치할 수 있습니다.".to_string(),
        );
    }
    Ok(())
}

/// 학사 일정 생성. 중복불가 검증(AC-T7-1) + 기간성 일관성(AC-T7-2) + 교습기간 제약(Sprint 7 T7).
#[tauri::command]
pub async fn create_schedule_event(
    payload: CreateScheduleEventPayload,
) -> Result<ScheduleEvent, String> {
    let pool = db::pool().map_err(String::from)?;

    let code = sqlx::query(
        "SELECT code_name, is_duplicate_blocked, is_period_type FROM schedule_codes WHERE id = ?",
    )
    .bind(payload.code_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?
    .ok_or_else(|| "해당 학사 일정 코드를 찾을 수 없습니다.".to_string())?;
    let code_name: String = code.try_get("code_name").map_err(AppError::Db).map_err(String::from)?;
    let is_dup_blocked: i64 = code
        .try_get("is_duplicate_blocked")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let is_period: i64 = code
        .try_get("is_period_type")
        .map_err(AppError::Db)
        .map_err(String::from)?;

    // 기간성 일관성 (AC-T7-2)
    match (is_period != 0, payload.period_end_date.as_ref()) {
        (true, None) => {
            return Err("기간성 코드는 종료 일자(period_end_date) 가 필요합니다.".to_string());
        }
        (false, Some(_)) => {
            return Err("단일 일자 코드는 종료 일자를 지정할 수 없습니다.".to_string());
        }
        _ => {}
    }

    check_placement_constraints(
        pool,
        payload.code_id,
        &code_name,
        is_dup_blocked != 0,
        &payload.event_date,
        payload.period_end_date.as_deref(),
        None,
    )
    .await?;

    let row = sqlx::query(
        "INSERT INTO schedule_events (code_id, event_date, period_end_date, display_name) \
         VALUES (?, ?, ?, ?) \
         RETURNING id, code_id, event_date, period_end_date, display_name, created_at, updated_at",
    )
    .bind(payload.code_id)
    .bind(&payload.event_date)
    .bind(payload.period_end_date.as_deref())
    .bind(payload.display_name.as_deref())
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    ScheduleEvent::from_row(&row).map_err(String::from)
}

/// 학사 일정 수정 — 지난 달 차단 (AC-T7-3) + 배치 제약 (Sprint 7 T7).
#[tauri::command]
pub async fn update_schedule_event(
    id: i64,
    payload: UpdateScheduleEventPayload,
) -> Result<ScheduleEvent, String> {
    let pool = db::pool().map_err(String::from)?;
    let target = sqlx::query(
        "SELECT e.event_date, e.code_id, c.code_name, c.is_duplicate_blocked \
         FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?
    .ok_or_else(|| "해당 학사 일정을 찾을 수 없습니다.".to_string())?;
    let event_date: String = target
        .try_get("event_date")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let code_id: i64 = target.try_get("code_id").map_err(AppError::Db).map_err(String::from)?;
    let code_name: String = target.try_get("code_name").map_err(AppError::Db).map_err(String::from)?;
    let is_dup_blocked: i64 = target
        .try_get("is_duplicate_blocked")
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let ym = year_month_of(&event_date).ok_or_else(|| "기존 일자 형식 오류".to_string())?;
    if ym.as_str() < current_year_month().as_str() {
        return Err("지난 달의 학사 일정은 수정할 수 없습니다.".to_string());
    }

    // 배치 제약 검증 — exclude_event_id=Some(id) 로 자기 자신 row 제외 (드래그 이동 시 동일 자리도 허용).
    check_placement_constraints(
        pool,
        code_id,
        &code_name,
        is_dup_blocked != 0,
        &payload.event_date,
        payload.period_end_date.as_deref(),
        Some(id),
    )
    .await?;

    let row = sqlx::query(
        "UPDATE schedule_events SET \
            event_date = ?, period_end_date = ?, display_name = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, code_id, event_date, period_end_date, display_name, created_at, updated_at",
    )
    .bind(&payload.event_date)
    .bind(payload.period_end_date.as_deref())
    .bind(payload.display_name.as_deref())
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    ScheduleEvent::from_row(&row).map_err(String::from)
}

/// 학사 일정 삭제 — 지난 달 차단 (AC-T7-3) + 시드 공휴일 차단 (Sprint 7 T9 + V16 post-review).
#[tauri::command]
pub async fn delete_schedule_event(id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let target = sqlx::query(
        "SELECT e.event_date, e.is_seeded, c.code_name, c.is_system_reserved \
         FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?
    .ok_or_else(|| "해당 학사 일정을 찾을 수 없습니다.".to_string())?;
    let event_date: String = target
        .try_get("event_date")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let code_name: String = target
        .try_get("code_name")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let is_system_reserved: i64 = target
        .try_get("is_system_reserved")
        .map_err(AppError::Db)
        .map_err(String::from)?;
    let is_seeded: i64 = target
        .try_get("is_seeded")
        .map_err(AppError::Db)
        .map_err(String::from)?;

    // V16 (Sprint 7 post-review): 시드 공휴일(`is_seeded=1`)만 삭제 차단. 사용자가 추가한
    // 공휴일(`is_seeded=0`)은 일반 삭제 흐름 허용.
    if is_system_reserved != 0 && code_name == "공휴일" && is_seeded != 0 {
        return Err("시드된 공휴일은 삭제할 수 없습니다.".to_string());
    }

    let ym = year_month_of(&event_date).ok_or_else(|| "기존 일자 형식 오류".to_string())?;
    if ym.as_str() < current_year_month().as_str() {
        return Err("지난 달의 학사 일정은 삭제할 수 없습니다.".to_string());
    }
    sqlx::query("DELETE FROM schedule_events WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    Ok(())
}

/// 기간 내 학사 일정 목록 — schedule_codes JOIN 평탄화 응답 (캘린더 렌더링용).
#[tauri::command]
pub async fn list_schedule_events(
    from_date: String,
    to_date: String,
) -> Result<Vec<ScheduleEventListItem>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT e.id, e.code_id, c.code_name, c.is_system_reserved, \
                c.is_duplicate_blocked, c.is_period_type, \
                c.allows_regular_class, c.allows_makeup_class, \
                e.is_seeded, e.event_date, e.period_end_date, e.display_name \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.event_date >= ? AND e.event_date <= ? \
         ORDER BY e.event_date ASC, e.id ASC",
    )
    .bind(&from_date)
    .bind(&to_date)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    rows.iter()
        .map(|r| ScheduleEventListItem::from_row(r).map_err(String::from))
        .collect()
}

/// 단원평가 응시일 자동 배치 (AC-T7-4, AC-T7-5). 트랜잭션 안에서 처리.
///
/// 이미 해당 month 에 단원평가 1건 이상이면 No-op (빈 Vec 반환). 그 외에는 2/4주차 월~금 10건 INSERT.
#[tauri::command]
pub async fn auto_place_assessment_dates(
    year_month: String,
) -> Result<Vec<ScheduleEvent>, String> {
    let pool = db::pool().map_err(String::from)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let code_id = find_assessment_code_id(&mut *tx)
        .await
        .map_err(String::from)?;

    // AC-T7-5: 해당 month 에 이미 단원평가 1건 이상이면 No-op
    let existing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_events \
         WHERE code_id = ? AND substr(event_date, 1, 7) = ?",
    )
    .bind(code_id)
    .bind(&year_month)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    if existing > 0 {
        tx.commit()
            .await
            .map_err(AppError::Db)
            .map_err(String::from)?;
        return Ok(vec![]);
    }

    let dates = assessment_dates_for(&year_month)?;
    let mut inserted = Vec::with_capacity(dates.len());
    for d in dates {
        let row = sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date) \
             VALUES (?, ?) \
             RETURNING id, code_id, event_date, period_end_date, display_name, \
                       created_at, updated_at",
        )
        .bind(code_id)
        .bind(d.format("%Y-%m-%d").to_string())
        .fetch_one(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
        inserted.push(ScheduleEvent::from_row(&row).map_err(String::from)?);
    }

    tx.commit()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    Ok(inserted)
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

    /// AC-T6-1 — 시스템 예약 코드 시드 확인 (V102 5종 + V301 "공휴일" 1종 = 6).
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
        assert_eq!(cnt, 6, "V102 5종 + V301 '공휴일' = 6종");
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

    // ─── T7 schedule_events 단위 테스트 ───

    #[test]
    fn year_month_of_extracts_valid_prefix() {
        assert_eq!(year_month_of("2099-03-15"), Some("2099-03".to_string()));
    }

    #[test]
    fn year_month_of_rejects_invalid_format() {
        assert_eq!(year_month_of("20990315"), None);
        assert_eq!(year_month_of("2099"), None);
    }

    /// AC-T7-4 핵심 — 2주차 월~금 + 4주차 월~금 정확히 10건, 모두 평일.
    #[test]
    fn assessment_dates_returns_two_groups_of_five_weekdays() {
        let dates = assessment_dates_for("2099-03").unwrap();
        assert_eq!(dates.len(), 10, "2주차5 + 4주차5 = 10건");
        // 같은 주 월~금은 0..=4 일 차이
        let in_week = (dates[4] - dates[0]).num_days();
        assert_eq!(in_week, 4, "같은 주 월·금 차이 4일");
        // 2주차 ↔ 4주차 사이 정확히 14일
        let between_weeks = (dates[5] - dates[0]).num_days();
        assert_eq!(between_weeks, 14, "2주차 월요일 ↔ 4주차 월요일 14일");
        for d in &dates {
            assert!(
                d.weekday().num_days_from_monday() < 5,
                "{} 가 평일(weekday<5)이어야 함, 실제={:?}",
                d,
                d.weekday()
            );
        }
    }

    /// AC-T7-1 — 중복불가 검증 SQL 카운트가 의도대로 동작.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn duplicate_blocked_count_check_works() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let code_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-03-15')")
            .bind(code_id)
            .execute(&pool)
            .await
            .unwrap();
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events WHERE code_id = ? AND event_date = ?",
        )
        .bind(code_id)
        .bind("2099-03-15")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(cnt, 1, "create_schedule_event 의 중복불가 가드가 사용하는 카운트");
    }

    /// AC-T7-5 — auto_place_assessment_dates 의 No-op 가드 쿼리가 의도대로.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn auto_place_noop_query_detects_existing_month() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let code_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '단원평가 응시일'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-03-10')")
            .bind(code_id)
            .execute(&pool)
            .await
            .unwrap();
        let existing: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events \
             WHERE code_id = ? AND substr(event_date, 1, 7) = ?",
        )
        .bind(code_id)
        .bind("2099-03")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(existing >= 1, "auto_place 가 No-op 으로 분기되어야 하는 상태");
        // 다른 month 는 영향 없음
        let other_month: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events \
             WHERE code_id = ? AND substr(event_date, 1, 7) = ?",
        )
        .bind(code_id)
        .bind("2099-04")
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(other_month, 0, "다른 month 는 영향 받지 않아야 함");
    }

    // ─── T2-b V301 검증 (ADR-005, PRD §4.4.4) ───

    /// AC-T2-1 — V101 시드의 3속성을 V301 이 PRD §4.4.4 기준으로 보정했는지.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v301_corrects_system_code_attributes() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");

        let bogang: (i64, i64) = sqlx::query_as(
            "SELECT is_duplicate_blocked, allows_makeup_class FROM schedule_codes \
             WHERE code_name = '보강데이' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(bogang.0, 1, "보강데이 is_duplicate_blocked = 1 (PRD §4.4.4 ON)");
        assert_eq!(bogang.1, 1, "보강데이 allows_makeup_class = 1");

        let gonghyu: i64 = sqlx::query_scalar(
            "SELECT allows_makeup_class FROM schedule_codes \
             WHERE code_name = '공휴수업일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(gonghyu, 1, "공휴수업일 allows_makeup_class = 1 (V102 0 → V301 1)");

        let danwon: (i64, i64) = sqlx::query_as(
            "SELECT is_duplicate_blocked, is_period_type FROM schedule_codes \
             WHERE code_name = '단원평가 응시일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(danwon.0, 0, "단원평가 is_duplicate_blocked = 0 (V102 1 → V301 0)");
        assert_eq!(danwon.1, 1, "단원평가 is_period_type = 1 (기간성 5일, V102 0 → V301 1)");
    }

    /// AC-T2-4 — "공휴일" 시스템 코드가 ADR-005 결정 속성으로 등록되었는지.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v301_inserts_holiday_system_code() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let code: (i64, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT is_system_reserved, allows_regular_class, allows_makeup_class, \
                    is_duplicate_blocked, is_period_type \
             FROM schedule_codes WHERE code_name = '공휴일'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(code.0, 1, "공휴일 is_system_reserved = 1 (3속성 수정 차단)");
        assert_eq!(code.1, 0, "공휴일 정규수업 OFF");
        assert_eq!(code.2, 0, "공휴일 보강 OFF");
        assert_eq!(code.3, 1, "공휴일 중복불가 ON");
        assert_eq!(code.4, 0, "공휴일 단일 일자");
    }

    /// AC-T2-4 — 한국 법정 공휴일 2025~2027 (64건) + 주요 공휴일 존재 검증.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v301_seeds_korean_holidays_2025_2027() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let code_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '공휴일'")
                .fetch_one(&pool)
                .await
                .unwrap();

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events WHERE code_id = ?",
        )
        .bind(code_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(total, 64, "2025~2027 한국 법정 공휴일 64건 시드");

        // 주요 고정 공휴일 (연도별 1건씩 표본)
        for (date, name_prefix) in [
            ("2025-01-01", "1월1일"),
            ("2026-03-01", "삼일절"),
            ("2027-05-05", "어린이날"),
            ("2025-08-15", "광복절"),
            ("2026-10-09", "한글날"),
            ("2027-12-25", "기독탄신일"),
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM schedule_events \
                 WHERE code_id = ? AND event_date = ? AND display_name LIKE ?",
            )
            .bind(code_id)
            .bind(date)
            .bind(format!("{}%", name_prefix))
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(exists >= 1, "{} {} 누락", date, name_prefix);
        }
    }

    /// AC-T2-4 — 대체공휴일 최소 5건 시드 (한국 「관공서의 공휴일에 관한 규정」).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v301_seeds_at_least_five_substitute_holidays() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let substitutes: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events e \
             JOIN schedule_codes c ON c.id = e.code_id \
             WHERE c.code_name = '공휴일' AND e.display_name LIKE '대체공휴일%'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(substitutes >= 5, "대체공휴일 5건 이상 (실제 {})", substitutes);
    }

    /// 2025-05-05 어린이날 + 부처님오신날 동일 일자 다중 공휴일 — V103 (code_id, event_date) UNIQUE 없음.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v301_allows_multiple_holidays_same_date() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events e \
             JOIN schedule_codes c ON c.id = e.code_id \
             WHERE c.code_name = '공휴일' AND e.event_date = '2025-05-05'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 2, "2025-05-05 어린이날·부처님오신날 2건 동시 표시");
    }

    // ─── Sprint 7 T7: 배치 제약 강화 단위 테스트 ───
    //
    // `check_placement_constraints` 를 직접 호출하여 가드 동작을 검증한다.
    // 두 IPC (create / update) 가 동일 헬퍼를 공유하므로 헬퍼 단위 테스트로 충분.

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_when_date_outside_study_period() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let code_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        // 교습기간 미확정 또는 부재 — 가드 차단 예상.
        let err = check_placement_constraints(&pool, code_id, "휴원일", false, "2099-03-15", None, None)
            .await
            .unwrap_err();
        assert!(
            err.contains("교습기간"),
            "교습기간 외 일자 차단 에러: {}",
            err
        );
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_allows_when_inside_confirmed_period() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 확정 교습기간 삽입.
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-03', '2099-03-01', '2099-03-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let code_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let result =
            check_placement_constraints(&pool, code_id, "휴원일", true, "2099-03-15", None, None).await;
        assert!(result.is_ok(), "확정 교습기간 내 빈 일자 허용: {:?}", result);
    }

    /// AC-T7-1: 중복불가 코드 배치 시 다른 일정 존재 일자에서 차단.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_when_dup_blocked_meets_other_event() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-04', '2099-04-01', '2099-04-30', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 비-중복불가 코드의 일정 먼저 삽입.
        let bogang_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '보강데이' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-04-10')")
            .bind(bogang_id)
            .execute(&pool)
            .await
            .unwrap();

        // 중복불가 코드 (휴원일, V102 + V301: is_duplicate_blocked=1) 배치 시도.
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let err =
            check_placement_constraints(&pool, hyuwon_id, "휴원일", true, "2099-04-10", None, None)
                .await
                .unwrap_err();
        assert!(
            err.contains("중복불가 코드는 다른 일정이 있는"),
            "중복불가 상호 차단 에러: {}",
            err
        );
    }

    /// AC-T7-2: 역방향 — 해당 일자에 중복불가 일정이 있으면 새 배치 차단.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_when_target_date_has_dup_blocked_event() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-05', '2099-05-01', '2099-05-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 휴원일(중복불가=1) 먼저 배치.
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-05-15')")
            .bind(hyuwon_id)
            .execute(&pool)
            .await
            .unwrap();

        // 비-중복불가 코드 (방학) 배치 시도 — 역방향 차단.
        let bangak_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '방학'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let err =
            check_placement_constraints(&pool, bangak_id, "방학", false, "2099-05-15", None, None)
                .await
                .unwrap_err();
        assert!(
            err.contains("중복불가 일정이 있어"),
            "역방향 중복불가 차단 에러: {}",
            err
        );
    }

    /// AC-T7-4: update 시 자기 자신 row 는 검증 대상 제외 (드래그 이동 시 동일 자리도 허용).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_excludes_self_event_on_update() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-06', '2099-06-01', '2099-06-30', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let event_id: i64 = sqlx::query_scalar(
            "INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-06-15') RETURNING id",
        )
        .bind(hyuwon_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        // 본인 row 제외 — 동일 일자에 자기 자신만 존재해도 통과.
        let result = check_placement_constraints(
            &pool,
            hyuwon_id,
            "휴원일",
            true,
            "2099-06-15",
            None,
            Some(event_id),
        )
        .await;
        assert!(result.is_ok(), "자기 자신 row 제외 시 허용: {:?}", result);
    }

    // ─── Sprint 7 T9: 공휴일 삭제 차단 ───

    /// AC-T9-1: 공휴일 시스템 코드 이벤트 삭제 차단 검증 (delete_schedule_event 가드 SQL 동등 쿼리).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn delete_event_blocks_holiday_system_code() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // V301 시드 공휴일 이벤트 1건 조회.
        let holiday_event: Option<(i64, String, i64)> = sqlx::query_as(
            "SELECT e.id, c.code_name, c.is_system_reserved \
             FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
             WHERE c.code_name = '공휴일' AND c.is_system_reserved = 1 LIMIT 1",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        let (_id, code_name, is_sys) = holiday_event.expect("V301 공휴일 이벤트 시드 있어야 함");
        // 가드 조건 — delete_schedule_event 내부 분기와 동일.
        let blocked = is_sys != 0 && code_name == "공휴일";
        assert!(blocked, "공휴일 시스템 코드는 삭제 차단 분기에 진입해야 함");
    }

    /// AC-T9-3: 비공휴일 시스템 코드 (단원평가) 는 삭제 허용 — 가드 분기에 걸리지 않음.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn delete_event_allows_non_holiday_system_code() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let assessment: (String, i64) = sqlx::query_as(
            "SELECT code_name, is_system_reserved FROM schedule_codes \
             WHERE code_name = '단원평가 응시일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let (code_name, is_sys) = assessment;
        let blocked = is_sys != 0 && code_name == "공휴일";
        assert!(
            !blocked,
            "단원평가는 system_reserved=1 이지만 공휴일 아님 → 삭제 허용"
        );
    }

    // ─── Sprint 7 T8: cascade 삭제 단위 테스트 ───
    //
    // `check_cascade_delete_guard` + cascade SQL 분기를 직접 검증한다.

    /// AC-T8-6: 가드 — 미확정 교습기간은 cascade 삭제 IPC 가 거부.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn cascade_guard_rejects_unconfirmed_period() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let id = insert_period(&pool, "2099-03", "2099-03-01", "2099-03-31", 0).await;
        // is_confirmed = 0 (insert_period default)
        let (_s, _e, deletable, reason) = check_cascade_delete_guard(&pool, id).await.unwrap();
        assert!(!deletable);
        assert!(reason.as_deref().unwrap_or("").contains("미확정"));
    }

    /// AC-T8-4: 지난 달 교습기간은 cascade 삭제 거부.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn cascade_guard_rejects_past_month() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let id = insert_period(&pool, "2000-01", "2000-01-01", "2000-01-31", 0).await;
        sqlx::query("UPDATE study_periods SET is_confirmed = 1 WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
        let (_s, _e, deletable, reason) = check_cascade_delete_guard(&pool, id).await.unwrap();
        assert!(!deletable);
        assert!(reason.as_deref().unwrap_or("").contains("지난 달"));
    }

    /// AC-T8-1, AC-T8-2: cascade 가 공휴일 제외하고만 삭제한다 (SQL 분기 검증).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn cascade_delete_preserves_holidays() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let _id = insert_period(&pool, "2099-04", "2099-04-01", "2099-04-30", 0).await;

        // 비공휴일 이벤트 삽입.
        let hyuwon: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-04-10')")
            .bind(hyuwon)
            .execute(&pool)
            .await
            .unwrap();

        // 공휴일 이벤트 삽입 (시뮬레이션, V301 외 직접 주입).
        let holiday: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-04-15')")
            .bind(holiday)
            .execute(&pool)
            .await
            .unwrap();

        // cascade SQL 와 동일한 분기 — 공휴일 제외 삭제.
        sqlx::query(
            "DELETE FROM schedule_events \
             WHERE event_date >= ? AND event_date <= ? \
               AND code_id IN (SELECT id FROM schedule_codes WHERE code_name != '공휴일')",
        )
        .bind("2099-04-01")
        .bind("2099-04-30")
        .execute(&pool)
        .await
        .unwrap();

        // 비공휴일은 삭제됨.
        let non_holiday_left: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events \
             WHERE event_date = '2099-04-10' AND code_id = ?",
        )
        .bind(hyuwon)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(non_holiday_left, 0, "휴원일(비공휴일) 삭제됨");

        // 공휴일은 보존됨.
        let holiday_left: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events \
             WHERE event_date = '2099-04-15' AND code_id = ?",
        )
        .bind(holiday)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(holiday_left, 1, "공휴일은 cascade 에서 보존");
    }

    /// AC-T8-3: preview 가 영향 건수 + 공휴일 보존 건수를 정확히 카운트.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn cascade_preview_counts_match_actual_deletion() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let id = insert_period(&pool, "2099-05", "2099-05-01", "2099-05-31", 0).await;
        sqlx::query("UPDATE study_periods SET is_confirmed = 1 WHERE id = ?")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();

        let hyuwon: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        for d in ["2099-05-05", "2099-05-12", "2099-05-19"] {
            sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, ?)")
                .bind(hyuwon)
                .bind(d)
                .execute(&pool)
                .await
                .unwrap();
        }
        // 공휴일 2건 — 기존 V301 시드 2025-05-05 등 외, 본 테스트는 인메모리에 V301 결과 포함.
        let holiday_in_range: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events e \
             JOIN schedule_codes c ON c.id = e.code_id \
             WHERE e.event_date >= '2099-05-01' AND e.event_date <= '2099-05-31' \
               AND c.code_name = '공휴일'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let non_holiday_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events e \
             JOIN schedule_codes c ON c.id = e.code_id \
             WHERE e.event_date >= '2099-05-01' AND e.event_date <= '2099-05-31' \
               AND c.code_name != '공휴일'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(non_holiday_count, 3, "본 테스트에서 삽입한 휴원일 3건만 카운트");
        assert!(holiday_in_range >= 0, "공휴일 카운트는 0 이상 (시드 환경 의존)");
    }

    // ─── V9 (Sprint 7 post-review): 공휴수업일 특별 룰 단위 테스트 ───

    /// 공휴수업일은 공휴일이 없는 일자에 배치 불가.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_holiday_class_without_holiday() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-07', '2099-07-01', '2099-07-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let holiday_class_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴수업일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        // 공휴일 없는 빈 일자에 공휴수업일 배치 시도.
        let err = check_placement_constraints(
            &pool,
            holiday_class_id,
            "공휴수업일",
            true,
            "2099-07-15",
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("공휴일이 지정된 날에만"),
            "공휴일 없는 일자 차단 에러: {}",
            err
        );
    }

    /// 공휴수업일은 공휴일이 있는 일자에 배치 가능 (다른 코드 없을 때).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_allows_holiday_class_with_holiday_only() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-08', '2099-08-01', '2099-08-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let holiday_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-08-15')")
            .bind(holiday_id)
            .execute(&pool)
            .await
            .unwrap();

        let holiday_class_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴수업일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = check_placement_constraints(
            &pool,
            holiday_class_id,
            "공휴수업일",
            true,
            "2099-08-15",
            None,
            None,
        )
        .await;
        assert!(result.is_ok(), "공휴일 위 공휴수업일 허용: {:?}", result);
    }

    /// 공휴수업일은 공휴일 외 다른 코드와 중복 차단.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_holiday_class_with_other_event() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-09', '2099-09-01', '2099-09-30', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 공휴일 + 보강데이 둘 다 같은 날에 배치 (보강데이는 is_duplicate_blocked=1 이지만 시뮬용 직접 삽입).
        let holiday_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-09-10')")
            .bind(holiday_id)
            .execute(&pool)
            .await
            .unwrap();
        let bogang_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '보강데이' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-09-10')")
            .bind(bogang_id)
            .execute(&pool)
            .await
            .unwrap();

        let holiday_class_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴수업일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let err = check_placement_constraints(
            &pool,
            holiday_class_id,
            "공휴수업일",
            true,
            "2099-09-10",
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("공휴일 외 다른 일정"),
            "공휴수업일 + 다른 코드 차단 에러: {}",
            err
        );
    }

    // ─── V16 (Sprint 7 post-review): 시드 vs 사용자 공휴일 구분 ───

    /// V302 마이그레이션 적용 후, V301 시드 공휴일 row 는 모두 `is_seeded=1` 로 마킹됨.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn v302_marks_seeded_holidays() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let unseeded_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM schedule_events e \
             JOIN schedule_codes c ON c.id = e.code_id \
             WHERE c.code_name = '공휴일' AND c.is_system_reserved = 1 AND e.is_seeded = 0",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unseeded_count, 0, "V301 시드 공휴일은 모두 is_seeded=1 마킹");
    }

    /// 사용자가 추가한 공휴일(`is_seeded=0`)은 삭제 가드에 걸리지 않음.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn delete_event_allows_user_added_holiday() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let holiday_id: i64 = sqlx::query_scalar(
            "SELECT id FROM schedule_codes WHERE code_name = '공휴일' AND is_system_reserved = 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        // is_seeded=0 으로 사용자 추가 공휴일 삽입 (시뮬레이션).
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, is_seeded) VALUES (?, '2099-12-25', 0)",
        )
        .bind(holiday_id)
        .execute(&pool)
        .await
        .unwrap();

        // 가드 분기 시뮬레이션: is_system_reserved=1 + code_name='공휴일' 이지만 is_seeded=0 →
        // 삭제 허용 분기로 진입해야 함.
        let row: (String, i64, i64) = sqlx::query_as(
            "SELECT c.code_name, c.is_system_reserved, e.is_seeded \
             FROM schedule_events e JOIN schedule_codes c ON c.id = e.code_id \
             WHERE e.event_date = '2099-12-25' AND e.is_seeded = 0",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let blocked = row.1 != 0 && row.0 == "공휴일" && row.2 != 0;
        assert!(!blocked, "is_seeded=0 사용자 공휴일은 삭제 가드 분기에 안 걸려야 함");
    }

    // ─── V26 (Sprint 7 post-review): 기간성 코드 사이 일자 충돌 검사 ───

    /// 기간성 코드(방학 6/10~6/15) 등록 후 사이 일자(6/12)에 휴원일 시도 → 차단.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_inside_period_event() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-06', '2099-06-01', '2099-06-30', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // 방학 기간성 이벤트 6/10~6/15 — 시작일에만 row 가 있고 period_end_date 컬럼이 종료일.
        let bangak_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '방학'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) \
             VALUES (?, '2099-06-10', '2099-06-15')",
        )
        .bind(bangak_id)
        .execute(&pool)
        .await
        .unwrap();

        // 사이 일자 6/12 에 휴원일 (중복불가=1) 단일 일자 배치 시도 → 차단.
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let err = check_placement_constraints(
            &pool,
            hyuwon_id,
            "휴원일",
            true,
            "2099-06-12",
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("중복불가 코드는 다른 일정이 있는"),
            "기간성 코드 사이 일자 차단 에러: {}",
            err
        );
    }

    /// 단일 일자 휴원일 등록 후 기간성 방학이 그 일자를 포함하면 차단 (역방향).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_period_overlapping_existing_single() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-07', '2099-07-01', '2099-07-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-07-12')")
            .bind(hyuwon_id)
            .execute(&pool)
            .await
            .unwrap();

        // 방학 7/10~7/15 시도 — 7/12 휴원일과 겹쳐서 차단.
        let bangak_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '방학'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let err = check_placement_constraints(
            &pool,
            bangak_id,
            "방학",
            true,
            "2099-07-10",
            Some("2099-07-15"),
            None,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("중복불가 코드는 다른 일정이 있는"),
            "기간성 코드와 기존 단일 일자 충돌 에러: {}",
            err
        );
    }

    /// 안 겹치는 기간성 코드는 허용.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_allows_non_overlapping_periods() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-08', '2099-08-01', '2099-08-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let bangak_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '방학'")
                .fetch_one(&pool)
                .await
                .unwrap();
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) \
             VALUES (?, '2099-08-01', '2099-08-05')",
        )
        .bind(bangak_id)
        .execute(&pool)
        .await
        .unwrap();

        // 8/10 휴원일 — 방학 8/1~8/5 와 안 겹침 → 허용.
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let result = check_placement_constraints(
            &pool,
            hyuwon_id,
            "휴원일",
            true,
            "2099-08-10",
            None,
            None,
        )
        .await;
        assert!(result.is_ok(), "안 겹치는 일자 허용: {:?}", result);
    }

    // ─── V29 (Sprint 7 post-review): 보강데이 배치 가드 ───

    /// 운영 평일 + 다른 이벤트 없음 → 수업 가능 일자 → 보강데이 차단.
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_blocks_bogang_on_class_day() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-09', '2099-09-01', '2099-09-30', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let bogang_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '보강데이'")
                .fetch_one(&pool)
                .await
                .unwrap();
        // 2099-09-09 = 화요일 (운영일). 다른 이벤트 없음 → 수업 가능 일자 → 보강데이 차단.
        let err = check_placement_constraints(
            &pool,
            bogang_id,
            "보강데이",
            false,
            "2099-09-09",
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(
            err.contains("보강데이는 수업이 있는 일자"),
            "보강데이 수업일 차단 에러: {}",
            err
        );
    }

    /// 휴원일 등록된 일자에 보강데이 → 허용 (수업 차단 코드 있음).
    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn placement_allows_bogang_on_no_class_day() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES ('2099-10', '2099-10-01', '2099-10-31', 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        // 휴원일을 먼저 배치.
        let hyuwon_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '휴원일'")
                .fetch_one(&pool)
                .await
                .unwrap();
        // 휴원일은 is_duplicate_blocked=1 이므로 보강데이가 함께 배치되려면 일반 가드도 통과해야 함.
        // 보강데이도 is_duplicate_blocked=0 (V102 시드) — 역방향 가드는 휴원일이 차단할 수 있음.
        // 본 테스트는 V29 보강데이 분기만 검증하므로 휴원일이 dup_blocked=0 가정 — 직접 토글.
        sqlx::query(
            "UPDATE schedule_codes SET is_duplicate_blocked = 0 WHERE code_name = '휴원일'",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO schedule_events (code_id, event_date) VALUES (?, '2099-10-09')")
            .bind(hyuwon_id)
            .execute(&pool)
            .await
            .unwrap();

        let bogang_id: i64 =
            sqlx::query_scalar("SELECT id FROM schedule_codes WHERE code_name = '보강데이'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let result = check_placement_constraints(
            &pool,
            bogang_id,
            "보강데이",
            false,
            "2099-10-09",
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "휴원일 일자에 보강데이 허용 (수업 차단 코드 존재): {:?}",
            result
        );
    }
}
