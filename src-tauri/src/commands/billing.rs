//! 청구 도메인 IPC (Sprint 11 T2~T3, PRD §4.9).
//!
//! ## 인터페이스 (T2 — 본 모듈)
//!
//! - [`generate_bills`] — 재원 + 월중입퇴교 원생 일괄 청구 생성. UNIQUE `(student_id, bill_year_month)`
//!   덕분에 동일 월 중복 호출은 INSERT OR IGNORE 로 안전.
//! - [`list_bills`] — 월별 청구 목록. 정렬: 미확정 + 월중입퇴교 상단 (AC-4.9-4).
//! - [`get_bill`] — 단건 조회.
//! - [`update_bill`] — 금액 조정. 상태별 제약: `draft` 자유 / `confirmed` 자유(프론트 확인 다이얼로그 책임) /
//!   `closed` 는 `close_reason` 필수 (AC-4.9-8).
//! - [`get_default_billing_year_month`] — UI 디폴트(마지막 교습기간 월) 헬퍼.
//!
//! ## T3 (상태 머신) 예정
//! - confirm_bill / confirm_all_bills / close_billing_month / update_closed_bill
//!
//! ## 트랜잭션
//! `generate_bills` 는 `pool.begin()` + `SELECT 1 LIMIT 0` 패턴으로 BEGIN IMMEDIATE 효과를 흉내한다.
//! 단일 사용자 모델이라 실질 race 없음. UNIQUE 제약이 중복 차단 안전망.
//!
//! ## 비즈니스 규칙 (PRD §4.9)
//! - 청구 대상: `enroll_date <= 월말 AND (withdraw_date IS NULL OR withdraw_date >= 월초)`
//!   → 재원중 + 월중입교 + 월중퇴교 모두 포함 (월 일부라도 재원했으면 청구).
//! - `weekly_hours = SUM(student_schedules.duration_hours)` (현재 유효 스케줄, `effective_to IS NULL`).
//! - `bill_amount = standard_fees.amount WHERE weekly_hours = ?` (UNIQUE).
//!   - 매핑 없거나 `weekly_hours = 0` 이면 청구 skip (skipped_count 카운트).
//! - `adjusted_amount` 초기값 = `bill_amount` (사용자가 후속 조정).
//! - 월중입퇴교 플래그: `enroll_date` 또는 `withdraw_date` 가 해당 월 범위 안이면 1.

use crate::commands::attendance::validate_year_month;
use crate::commands::audit::{self, AuditEventType};
use crate::commands::db;
use crate::error::AppError;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::{Row, SqlitePool};

// ─────────────────────── 직렬화 타입 ───────────────────────

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Bill {
    pub id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub student_serial_no: String,
    pub student_grade: i64,
    pub student_school_level: String,
    pub bill_year_month: String,
    pub weekly_hours: i64,
    pub bill_amount: i64,
    pub adjusted_amount: i64,
    pub status: String,
    pub is_mid_month: bool,
    pub mid_month_type: Option<String>,
    pub close_reason: Option<String>,
    pub closed_at: Option<String>,
    pub confirmed_at: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerateBillsResult {
    pub year_month: String,
    pub generated_count: i64,
    pub skipped_count: i64,
}

// ─────────────────────── IPC ───────────────────────

/// 재원 + 월중입퇴교 원생 일괄 청구 생성 (PRD §4.9.1, AC-4.9-1).
#[tauri::command]
pub async fn generate_bills(year_month: String) -> Result<GenerateBillsResult, String> {
    let pool = db::pool().map_err(String::from)?;
    generate_bills_impl(pool, &year_month).await
}

pub(crate) async fn generate_bills_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<GenerateBillsResult, String> {
    validate_year_month(year_month)?;
    let (period_start, period_end) = year_month_range(year_month)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    // BEGIN IMMEDIATE 효과 (students.rs 패턴 동일) — 단일 사용자 모델이라 실질 race 없으나 안전망.
    sqlx::query("SELECT 1 FROM bills LIMIT 0")
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    // 청구 대상 원생 + 주 수업시간 집계.
    let rows = sqlx::query(
        "SELECT s.id, s.enroll_date, s.withdraw_date, \
                COALESCE(SUM(sch.duration_hours), 0) AS weekly_hours \
         FROM students s \
         LEFT JOIN student_schedules sch \
                ON sch.student_id = s.id AND sch.effective_to IS NULL \
         WHERE s.enroll_date <= ? \
           AND (s.withdraw_date IS NULL OR s.withdraw_date >= ?) \
         GROUP BY s.id, s.enroll_date, s.withdraw_date \
         ORDER BY s.id",
    )
    .bind(&period_end)
    .bind(&period_start)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("청구 대상 원생 조회 실패: {}", e))?;

    let mut generated_count: i64 = 0;
    let mut skipped_count: i64 = 0;

    for r in rows {
        let student_id: i64 = r.try_get("id").map_err(|e| e.to_string())?;
        let enroll_date: String = r.try_get("enroll_date").map_err(|e| e.to_string())?;
        let withdraw_date: Option<String> = r
            .try_get("withdraw_date")
            .map_err(|e| e.to_string())?;
        let weekly_hours: i64 = r.try_get("weekly_hours").map_err(|e| e.to_string())?;

        if weekly_hours <= 0 {
            skipped_count += 1;
            continue;
        }

        // standard_fees 매핑 — 없으면 skip.
        let fee: Option<i64> = sqlx::query_scalar(
            "SELECT amount FROM standard_fees \
             WHERE weekly_hours = ? AND is_active = 1",
        )
        .bind(weekly_hours)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("표준 교습비 조회 실패: {}", e))?;

        let bill_amount = match fee {
            Some(a) => a,
            None => {
                skipped_count += 1;
                continue;
            }
        };

        let (is_mid_month, mid_month_type) =
            compute_mid_month_flag(&enroll_date, withdraw_date.as_deref(), &period_start, &period_end);

        // INSERT OR IGNORE — UNIQUE (student_id, bill_year_month) 위반 시 무동작.
        let res = sqlx::query(
            "INSERT OR IGNORE INTO bills \
                (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount, \
                 is_mid_month, mid_month_type, status) \
             VALUES (?, ?, ?, ?, ?, ?, ?, 'draft')",
        )
        .bind(student_id)
        .bind(year_month)
        .bind(weekly_hours)
        .bind(bill_amount)
        .bind(bill_amount)
        .bind(is_mid_month as i64)
        .bind(&mid_month_type)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("청구 INSERT 실패: {}", e))?;

        if res.rows_affected() > 0 {
            generated_count += 1;
        } else {
            skipped_count += 1; // 이미 존재
        }
    }

    tx.commit()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    Ok(GenerateBillsResult {
        year_month: year_month.to_string(),
        generated_count,
        skipped_count,
    })
}

/// 월별 청구 목록 조회 (PRD §4.9.2~§4.9.4, AC-4.9-4 정렬).
///
/// 정렬 우선순위:
///   1. 미확정(`draft`) 먼저, 확정(`confirmed`) 다음, 마감(`closed`) 마지막
///   2. 월중입퇴교(`is_mid_month=1`) 가 같은 상태 내 우선
///   3. 학생 이름 ASC
#[tauri::command]
pub async fn list_bills(year_month: String) -> Result<Vec<Bill>, String> {
    let pool = db::pool().map_err(String::from)?;
    list_bills_impl(pool, &year_month).await
}

pub(crate) async fn list_bills_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<Bill>, String> {
    validate_year_month(year_month)?;
    let rows = sqlx::query(
        "SELECT b.id, b.student_id, s.name AS student_name, s.serial_no, s.grade, s.school_level, \
                b.bill_year_month, b.weekly_hours, b.bill_amount, b.adjusted_amount, b.status, \
                b.is_mid_month, b.mid_month_type, b.close_reason, b.closed_at, b.confirmed_at \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         WHERE b.bill_year_month = ? \
         ORDER BY \
            CASE b.status WHEN 'draft' THEN 0 WHEN 'confirmed' THEN 1 ELSE 2 END ASC, \
            b.is_mid_month DESC, \
            s.name ASC",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("청구 목록 조회 실패: {}", e))?;

    rows.into_iter().map(row_to_bill).collect()
}

/// 단건 조회.
#[tauri::command]
pub async fn get_bill(id: i64) -> Result<Bill, String> {
    let pool = db::pool().map_err(String::from)?;
    get_bill_impl(pool, id).await
}

pub(crate) async fn get_bill_impl(pool: &SqlitePool, id: i64) -> Result<Bill, String> {
    let row = sqlx::query(
        "SELECT b.id, b.student_id, s.name AS student_name, s.serial_no, s.grade, s.school_level, \
                b.bill_year_month, b.weekly_hours, b.bill_amount, b.adjusted_amount, b.status, \
                b.is_mid_month, b.mid_month_type, b.close_reason, b.closed_at, b.confirmed_at \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("청구 조회 실패: {}", e))?
    .ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;
    row_to_bill(row)
}

/// 청구 금액 조정 (PRD §4.9.3, AC-4.9-8).
///
/// 상태별 제약:
/// - `draft` / `confirmed` — 자유롭게 수정 (프론트 확인 다이얼로그는 별도 책임)
/// - `closed` — `close_reason` 필수. NULL/공백 시 거부.
///
/// `close_reason` 은 closed 상태에서만 의미. draft/confirmed 호출 시 close_reason 인자는 무시됨.
#[tauri::command]
pub async fn update_bill(
    id: i64,
    adjusted_amount: i64,
    close_reason: Option<String>,
) -> Result<Bill, String> {
    let pool = db::pool().map_err(String::from)?;
    update_bill_impl(pool, id, adjusted_amount, close_reason.as_deref()).await
}

pub(crate) async fn update_bill_impl(
    pool: &SqlitePool,
    id: i64,
    adjusted_amount: i64,
    close_reason: Option<&str>,
) -> Result<Bill, String> {
    if adjusted_amount < 0 {
        return Err("조정 금액은 0 이상이어야 합니다.".to_string());
    }

    let current_status: Option<String> = sqlx::query_scalar(
        "SELECT status FROM bills WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("청구 상태 조회 실패: {}", e))?;

    let status = current_status.ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;

    if status == "closed" {
        let reason = close_reason
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                "마감된 청구를 수정하려면 사유(close_reason)를 입력해야 합니다.".to_string()
            })?;
        sqlx::query(
            "UPDATE bills SET adjusted_amount = ?, close_reason = ?, \
                  updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?",
        )
        .bind(adjusted_amount)
        .bind(reason)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("청구 수정 실패: {}", e))?;
        // Sprint 11 T3: 마감 후 수정은 audit 로그로 추적 (AC-4.9-8 운영 추적).
        audit::try_record(
            AuditEventType::BillClosedModified,
            Some(&id.to_string()),
            Some(&format!(
                r#"{{"adjustedAmount":{},"closeReason":{}}}"#,
                adjusted_amount,
                serde_json::to_string(reason).unwrap_or_else(|_| "\"\"".to_string()),
            )),
        )
        .await;
    } else {
        sqlx::query(
            "UPDATE bills SET adjusted_amount = ?, \
                  updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ?",
        )
        .bind(adjusted_amount)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("청구 수정 실패: {}", e))?;
    }

    get_bill_impl(pool, id).await
}

// ─────────────────────── T3: 상태 머신 ───────────────────────

/// 청구 단건 확정 — `draft` → `confirmed` (PRD §4.9.3).
///
/// `confirmed` / `closed` 상태에서 호출 시 거부 (재확정·재진입 불가).
#[tauri::command]
pub async fn confirm_bill(id: i64) -> Result<Bill, String> {
    let pool = db::pool().map_err(String::from)?;
    confirm_bill_impl(pool, id).await
}

pub(crate) async fn confirm_bill_impl(pool: &SqlitePool, id: i64) -> Result<Bill, String> {
    let current: Option<String> = sqlx::query_scalar("SELECT status FROM bills WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("청구 상태 조회 실패: {}", e))?;
    let status = current.ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;
    if status != "draft" {
        return Err(format!(
            "확정은 미확정(draft) 상태에서만 가능합니다 (현재: {}).",
            status
        ));
    }
    sqlx::query(
        "UPDATE bills SET status='confirmed', \
             confirmed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE id = ?",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("청구 확정 실패: {}", e))?;
    audit::try_record(
        AuditEventType::BillConfirmed,
        Some(&id.to_string()),
        None,
    )
    .await;
    get_bill_impl(pool, id).await
}

/// 월 전체 미확정 청구 일괄 확정. 반환값은 전이된 건수.
#[tauri::command]
pub async fn confirm_all_bills(year_month: String) -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    confirm_all_bills_impl(pool, &year_month).await
}

pub(crate) async fn confirm_all_bills_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<i64, String> {
    validate_year_month(year_month)?;
    let res = sqlx::query(
        "UPDATE bills SET status='confirmed', \
             confirmed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE bill_year_month = ? AND status = 'draft'",
    )
    .bind(year_month)
    .execute(pool)
    .await
    .map_err(|e| format!("청구 일괄 확정 실패: {}", e))?;
    let affected = res.rows_affected() as i64;
    audit::try_record(
        AuditEventType::BillConfirmed,
        Some(year_month),
        Some(&format!(r#"{{"batch":true,"count":{}}}"#, affected)),
    )
    .await;
    Ok(affected)
}

/// 월 전체 마감 (PRD §4.9.7, AC-4.9-7).
///
/// 전제: 해당 월 모든 청구가 `confirmed`. `draft` 가 1건이라도 있으면 에러 + 미확정 건수 메시지.
/// PI-11 확정: 마감 해제(reopen) 불가 — 본 IPC 의 역연산은 제공하지 않음.
#[tauri::command]
pub async fn close_billing_month(year_month: String) -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    close_billing_month_impl(pool, &year_month).await
}

pub(crate) async fn close_billing_month_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<i64, String> {
    validate_year_month(year_month)?;
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    sqlx::query("SELECT 1 FROM bills LIMIT 0")
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bills WHERE bill_year_month = ? AND status = 'draft'",
    )
    .bind(year_month)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("미확정 청구 카운트 실패: {}", e))?;

    if pending > 0 {
        return Err(format!(
            "미확정 청구가 {}건 남아 있어 마감할 수 없습니다. 먼저 확정해 주세요.",
            pending
        ));
    }

    let res = sqlx::query(
        "UPDATE bills SET status='closed', \
             closed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE bill_year_month = ? AND status = 'confirmed'",
    )
    .bind(year_month)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("청구 마감 실패: {}", e))?;

    tx.commit()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let closed = res.rows_affected() as i64;
    audit::try_record(
        AuditEventType::BillMonthClosed,
        Some(year_month),
        Some(&format!(r#"{{"closedCount":{}}}"#, closed)),
    )
    .await;
    Ok(closed)
}

/// UI 디폴트 청구년월 — 가장 최근(MAX) 교습기간 월 반환. 없으면 None.
#[tauri::command]
pub async fn get_default_billing_year_month() -> Result<Option<String>, String> {
    let pool = db::pool().map_err(String::from)?;
    get_default_billing_year_month_impl(pool).await
}

pub(crate) async fn get_default_billing_year_month_impl(
    pool: &SqlitePool,
) -> Result<Option<String>, String> {
    let ym: Option<String> = sqlx::query_scalar(
        "SELECT year_month FROM study_periods ORDER BY year_month DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("교습기간 조회 실패: {}", e))?
    .flatten();
    Ok(ym)
}

// ─────────────────────── 헬퍼 ───────────────────────

/// YYYY-MM 으로부터 (월초 YYYY-MM-01, 월말 YYYY-MM-DD) 일자 문자열 쌍을 반환.
fn year_month_range(ym: &str) -> Result<(String, String), String> {
    let year: i32 = ym[..4].parse().map_err(|e| format!("연도 파싱 실패: {}", e))?;
    let month: u32 = ym[5..].parse().map_err(|e| format!("월 파싱 실패: {}", e))?;
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("월초 일자 생성 실패: {}-{:02}-01", year, month))?;
    // 다음 달 첫째 날 - 1일 = 이번 달 말일.
    let next_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "다음 달 일자 생성 실패".to_string())?;
    let last = next_first
        .pred_opt()
        .ok_or_else(|| "월말 일자 계산 실패".to_string())?;
    Ok((first.to_string(), last.to_string()))
}

/// 월중입퇴교 플래그 산출.
///
/// 우선순위: 월중입교 > 월중퇴교 (둘 다 해당하는 경우 — 같은 월에 입학·퇴교한 극단 케이스 — enrolled 로 표기).
fn compute_mid_month_flag(
    enroll_date: &str,
    withdraw_date: Option<&str>,
    period_start: &str,
    period_end: &str,
) -> (bool, Option<String>) {
    if enroll_date >= period_start && enroll_date <= period_end {
        return (true, Some("enrolled".to_string()));
    }
    if let Some(wd) = withdraw_date {
        if wd >= period_start && wd <= period_end {
            return (true, Some("withdrawn".to_string()));
        }
    }
    (false, None)
}

fn row_to_bill(r: sqlx::sqlite::SqliteRow) -> Result<Bill, String> {
    Ok(Bill {
        id: r.try_get("id").map_err(|e| e.to_string())?,
        student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
        student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
        student_serial_no: r.try_get("serial_no").map_err(|e| e.to_string())?,
        student_grade: r.try_get("grade").map_err(|e| e.to_string())?,
        student_school_level: r.try_get("school_level").map_err(|e| e.to_string())?,
        bill_year_month: r.try_get("bill_year_month").map_err(|e| e.to_string())?,
        weekly_hours: r.try_get("weekly_hours").map_err(|e| e.to_string())?,
        bill_amount: r.try_get("bill_amount").map_err(|e| e.to_string())?,
        adjusted_amount: r.try_get("adjusted_amount").map_err(|e| e.to_string())?,
        status: r.try_get("status").map_err(|e| e.to_string())?,
        is_mid_month: {
            let v: i64 = r.try_get("is_mid_month").map_err(|e| e.to_string())?;
            v != 0
        },
        mid_month_type: r.try_get("mid_month_type").map_err(|e| e.to_string())?,
        close_reason: r.try_get("close_reason").map_err(|e| e.to_string())?,
        closed_at: r.try_get("closed_at").map_err(|e| e.to_string())?,
        confirmed_at: r.try_get("confirmed_at").map_err(|e| e.to_string())?,
    })
}

// ─────────────────────── 테스트 ───────────────────────

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    async fn seed_student(
        pool: &SqlitePool,
        serial: &str,
        name: &str,
        enroll: &str,
        withdraw: Option<&str>,
    ) -> i64 {
        let withdraw_clause = withdraw.map(|w| format!("'{}'", w)).unwrap_or_else(|| "NULL".to_string());
        let sql = format!(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date, withdraw_date) \
             VALUES (?, ?, 'male', 'elementary', 3, ?, {}) RETURNING id",
            withdraw_clause
        );
        sqlx::query_scalar(&sql)
            .bind(serial)
            .bind(name)
            .bind(enroll)
            .fetch_one(pool)
            .await
            .expect("seed student")
    }

    async fn seed_schedule(pool: &SqlitePool, student_id: i64, dow: i64, hours: i64) {
        sqlx::query(
            "INSERT INTO student_schedules \
                (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, ?, '16:00', ?, '2026-01-01')",
        )
        .bind(student_id)
        .bind(dow)
        .bind(hours)
        .execute(pool)
        .await
        .expect("seed schedule");
    }

    async fn seed_standard_fee(pool: &SqlitePool, weekly_hours: i64, amount: i64) {
        sqlx::query(
            "INSERT INTO standard_fees (weekly_hours, amount) VALUES (?, ?)",
        )
        .bind(weekly_hours)
        .bind(amount)
        .execute(pool)
        .await
        .expect("seed fee");
    }

    #[tokio::test]
    async fn year_month_range_computes_first_and_last_day() {
        assert_eq!(
            year_month_range("2026-02").unwrap(),
            ("2026-02-01".to_string(), "2026-02-28".to_string())
        );
        assert_eq!(
            year_month_range("2024-02").unwrap(),
            ("2024-02-01".to_string(), "2024-02-29".to_string()),
            "윤년"
        );
        assert_eq!(
            year_month_range("2026-12").unwrap(),
            ("2026-12-01".to_string(), "2026-12-31".to_string())
        );
    }

    #[test]
    fn mid_month_flag_detects_enrolled() {
        let (flag, t) = compute_mid_month_flag("2026-05-15", None, "2026-05-01", "2026-05-31");
        assert!(flag);
        assert_eq!(t.as_deref(), Some("enrolled"));
    }

    #[test]
    fn mid_month_flag_detects_withdrawn() {
        let (flag, t) =
            compute_mid_month_flag("2026-04-01", Some("2026-05-20"), "2026-05-01", "2026-05-31");
        assert!(flag);
        assert_eq!(t.as_deref(), Some("withdrawn"));
    }

    #[test]
    fn mid_month_flag_false_for_full_month_active() {
        let (flag, t) =
            compute_mid_month_flag("2026-01-01", None, "2026-05-01", "2026-05-31");
        assert!(!flag);
        assert!(t.is_none());
    }

    #[test]
    fn mid_month_flag_enroll_priority_over_withdraw() {
        // 같은 달에 입학·퇴교한 극단 케이스 — enrolled 우선.
        let (flag, t) = compute_mid_month_flag(
            "2026-05-05",
            Some("2026-05-25"),
            "2026-05-01",
            "2026-05-31",
        );
        assert!(flag);
        assert_eq!(t.as_deref(), Some("enrolled"));
    }

    #[tokio::test]
    async fn generate_bills_creates_for_active_student_with_fee() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 2).await; // 2시간
        seed_standard_fee(&pool, 2, 200_000).await;

        let result = generate_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(result.generated_count, 1);
        assert_eq!(result.skipped_count, 0);

        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(bills.len(), 1);
        assert_eq!(bills[0].bill_amount, 200_000);
        assert_eq!(bills[0].adjusted_amount, 200_000);
        assert_eq!(bills[0].status, "draft");
        assert!(!bills[0].is_mid_month);
    }

    #[tokio::test]
    async fn generate_bills_unique_blocks_duplicate_run() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 2).await;
        seed_standard_fee(&pool, 2, 200_000).await;

        let first = generate_bills_impl(&pool, "2026-05").await.expect("first");
        assert_eq!(first.generated_count, 1);

        // AC-4.9-1: 동일 월 재호출은 UNIQUE 로 차단 + skipped 카운트
        let second = generate_bills_impl(&pool, "2026-05").await.expect("second");
        assert_eq!(second.generated_count, 0);
        assert_eq!(second.skipped_count, 1);
    }

    #[tokio::test]
    async fn generate_bills_skips_student_without_schedule_or_fee() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 스케줄 없음 → skip (weekly_hours=0)
        seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        // 스케줄 있으나 표준 교습비 매핑 없음 → skip
        // 시드된 standard_fees 는 3~6시간만. 99시간은 매핑 없음 → skip 대상.
        let s2 = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        seed_schedule(&pool, s2, 1, 99).await;

        let result = generate_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(result.generated_count, 0);
        assert_eq!(result.skipped_count, 2);
    }

    #[tokio::test]
    async fn generate_bills_includes_mid_month_enrolled() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-05-15", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(result.generated_count, 1);
        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        assert!(bills[0].is_mid_month);
        assert_eq!(bills[0].mid_month_type.as_deref(), Some("enrolled"));
    }

    #[tokio::test]
    async fn generate_bills_includes_mid_month_withdrawn() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", Some("2026-05-20")).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(result.generated_count, 1);
        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        assert!(bills[0].is_mid_month);
        assert_eq!(bills[0].mid_month_type.as_deref(), Some("withdrawn"));
    }

    #[tokio::test]
    async fn generate_bills_excludes_already_withdrawn_before_month() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", Some("2026-04-30")).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(result.generated_count, 0, "4월말 퇴교한 학생은 5월 청구 대상 아님");
    }

    #[tokio::test]
    async fn list_bills_orders_draft_then_midmonth_then_name() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 3명: A(draft, normal), B(draft, mid_month enrolled), C(confirmed, normal)
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-05-10", None).await;
        let c = seed_student(&pool, "3", "원생C", "2026-01-01", None).await;
        for s in [a, b, c] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;

        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        // C 를 confirmed 로 전이
        sqlx::query("UPDATE bills SET status = 'confirmed' WHERE student_id = ?")
            .bind(c)
            .execute(&pool)
            .await
            .expect("confirm");

        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(bills.len(), 3);
        // draft + mid_month → 1순위 (B)
        assert_eq!(bills[0].student_id, b);
        // draft + 일반 → 2순위 (A, 이름 ASC)
        assert_eq!(bills[1].student_id, a);
        // confirmed → 마지막 (C)
        assert_eq!(bills[2].student_id, c);
    }

    #[tokio::test]
    async fn update_bill_changes_adjusted_amount_in_draft() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        let bill_id = bills[0].id;

        let updated = update_bill_impl(&pool, bill_id, 90_000, None).await.expect("ok");
        assert_eq!(updated.adjusted_amount, 90_000);
        assert_eq!(updated.bill_amount, 100_000, "표준 금액은 불변");
    }

    #[tokio::test]
    async fn update_bill_closed_requires_close_reason() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_id: i64 = sqlx::query_scalar("SELECT id FROM bills LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        // closed 로 강제 전이 (T3 IPC 전 시점이라 직접 UPDATE)
        sqlx::query(
            "UPDATE bills SET status='closed', closed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?",
        )
        .bind(bill_id)
        .execute(&pool)
        .await
        .unwrap();

        // AC-4.9-8: close_reason 없이 수정 시도 → 거부
        let err = update_bill_impl(&pool, bill_id, 80_000, None).await;
        assert!(err.is_err());
        let err_blank = update_bill_impl(&pool, bill_id, 80_000, Some("   ")).await;
        assert!(err_blank.is_err(), "공백 사유도 거부");

        // 사유 있으면 통과
        let ok = update_bill_impl(&pool, bill_id, 80_000, Some("월말 환불 처리"))
            .await
            .expect("close_reason 있으면 통과");
        assert_eq!(ok.adjusted_amount, 80_000);
        assert_eq!(ok.close_reason.as_deref(), Some("월말 환불 처리"));
    }

    #[tokio::test]
    async fn update_bill_negative_amount_rejected() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_id: i64 = sqlx::query_scalar("SELECT id FROM bills LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        let err = update_bill_impl(&pool, bill_id, -1, None).await;
        assert!(err.is_err(), "음수 금액 거부");
    }

    #[tokio::test]
    async fn get_default_billing_year_month_returns_latest_period() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date) VALUES \
                ('2026-03', '2026-03-01', '2026-03-31'), \
                ('2026-05', '2026-05-01', '2026-05-31'), \
                ('2026-04', '2026-04-01', '2026-04-30')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let ym = get_default_billing_year_month_impl(&pool).await.expect("ok");
        assert_eq!(ym.as_deref(), Some("2026-05"));
    }

    #[tokio::test]
    async fn get_default_billing_year_month_none_when_no_periods() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let ym = get_default_billing_year_month_impl(&pool).await.expect("ok");
        assert!(ym.is_none());
    }

    // ─────────────────────── T3 상태 머신 ───────────────────────

    /// 청구 1건 생성 + bill_id 반환 헬퍼.
    async fn seed_bill(pool: &SqlitePool, year_month: &str) -> i64 {
        let sid = seed_student(pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(pool, sid, 1, 1).await;
        seed_standard_fee(pool, 1, 100_000).await;
        generate_bills_impl(pool, year_month).await.expect("gen");
        sqlx::query_scalar("SELECT id FROM bills LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("bill id")
    }

    #[tokio::test]
    async fn confirm_bill_transitions_draft_to_confirmed() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let bill = confirm_bill_impl(&pool, bid).await.expect("ok");
        assert_eq!(bill.status, "confirmed");
        assert!(bill.confirmed_at.is_some());
    }

    #[tokio::test]
    async fn confirm_bill_rejects_already_confirmed() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        confirm_bill_impl(&pool, bid).await.expect("first");
        let err = confirm_bill_impl(&pool, bid).await;
        assert!(err.is_err(), "재확정 불가");
    }

    #[tokio::test]
    async fn confirm_bill_rejects_closed() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        sqlx::query("UPDATE bills SET status='closed', closed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=?")
            .bind(bid).execute(&pool).await.unwrap();
        let err = confirm_bill_impl(&pool, bid).await;
        assert!(err.is_err(), "마감된 청구는 확정 불가");
    }

    #[tokio::test]
    async fn confirm_bill_rejects_nonexistent() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let err = confirm_bill_impl(&pool, 999).await;
        assert!(err.is_err(), "존재하지 않는 청구 거부");
    }

    #[tokio::test]
    async fn confirm_all_bills_transitions_only_drafts() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 3건 — 2건 draft, 1건은 이미 confirmed 로 강제 전이
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        let c = seed_student(&pool, "3", "원생C", "2026-01-01", None).await;
        for s in [a, b, c] { seed_schedule(&pool, s, 1, 1).await; }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        sqlx::query("UPDATE bills SET status='confirmed', confirmed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE student_id=?")
            .bind(c).execute(&pool).await.unwrap();

        let affected = confirm_all_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(affected, 2, "draft 2건만 영향");

        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM bills WHERE bill_year_month='2026-05' AND status='confirmed'",
        )
        .fetch_one(&pool).await.unwrap();
        assert_eq!(cnt, 3, "전체 3건 confirmed");
    }

    #[tokio::test]
    async fn confirm_all_bills_zero_when_no_drafts() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let affected = confirm_all_bills_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(affected, 0);
    }

    #[tokio::test]
    async fn close_billing_month_rejects_when_pending_drafts() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        for s in [a, b] { seed_schedule(&pool, s, 1, 1).await; }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        // 1건만 confirmed, 1건은 draft 유지
        sqlx::query("UPDATE bills SET status='confirmed', confirmed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE student_id=?")
            .bind(a).execute(&pool).await.unwrap();

        let err = close_billing_month_impl(&pool, "2026-05").await;
        assert!(err.is_err(), "draft 1건 남으면 마감 불가");
        let msg = err.unwrap_err();
        assert!(msg.contains("1건"), "메시지에 미확정 건수 포함: {}", msg);

        // draft 가 그대로 남아있는지 확인 (롤백)
        let draft_cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM bills WHERE bill_year_month='2026-05' AND status='draft'",
        )
        .fetch_one(&pool).await.unwrap();
        assert_eq!(draft_cnt, 1, "롤백되어 draft 보존");
    }

    #[tokio::test]
    async fn close_billing_month_transitions_confirmed_to_closed() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        confirm_bill_impl(&pool, bid).await.expect("confirm");

        let closed = close_billing_month_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(closed, 1);

        let bill = get_bill_impl(&pool, bid).await.expect("get");
        assert_eq!(bill.status, "closed");
        assert!(bill.closed_at.is_some());
    }

    #[tokio::test]
    async fn close_billing_month_with_no_bills_succeeds() {
        // AC-4.9-7 의 전제 — 미확정 0건이면 마감 OK. 청구가 아예 없는 월도 동일하게 0건 마감.
        let pool = db::test_pool_in_memory().await.expect("pool");
        let closed = close_billing_month_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(closed, 0);
    }
}
