//! 청구 도메인 IPC (Sprint 11 T2~T3, PRD §4.9).
//!
//! ## 인터페이스 (T2 — 본 모듈)
//!
//! - [`generate_bills`] — 재원 + 월중입퇴교 원생 일괄 청구 생성. UNIQUE `(student_id, bill_year_month)`
//!   덕분에 동일 월 중복 호출은 INSERT OR IGNORE 로 안전.
//! - [`list_bills`] — 월별 청구 목록. 정렬: 미확정 + 월중입퇴교 상단 (AC-4.9-4).
//! - [`get_bill`] — 단건 조회.
//! - [`update_bill`] — 금액 조정. 상태별 제약: `draft`/`confirmed` 자유(프론트 확인 다이얼로그 책임).
//!   단, 수납완료(`is_paid=1`)된 청구는 금액 수정 거부.
//! - [`get_default_billing_year_month`] — UI 디폴트(마지막 교습기간 월) 헬퍼.
//!
//! ## 상태 머신
//! - confirm_bill / confirm_all_bills — `draft` → `confirmed`. (마감 개념은 V111 에서 폐기)
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
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

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
    pub confirmed_at: Option<String>,
    /// payments.is_paid=1 행이 존재하면 true (수납완료 라벨용).
    pub is_paid: bool,
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
    // T1 (Sprint 20): 청구 대상 기간을 달력월이 아닌 교습기간(study_periods) 기준으로 산정.
    // 교습기간 미등록 월은 청구 생성을 차단한다 — 달력월 기준으로 대상을 잡으면 교습기간
    // 종료 이후 입교한 원생까지 청구되는 버그(예: 7월 교습기간 7/2~29, 입교 7/30)를 방지.
    let (period_start, period_end) = match load_billing_period_range(pool, year_month).await? {
        Some(range) => range,
        None => {
            return Err(format!(
                "{} 교습기간이 등록되지 않았습니다. 학사 캘린더에서 먼저 교습기간을 등록하세요.",
                year_month
            ))
        }
    };

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
                   AND sch.effective_from <= ? \
         WHERE s.enroll_date <= ? \
           AND (s.withdraw_date IS NULL OR s.withdraw_date >= ?) \
         GROUP BY s.id, s.enroll_date, s.withdraw_date \
         ORDER BY s.id",
    )
    .bind(&period_end)
    .bind(&period_end)
    .bind(&period_start)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| format!("청구 대상 원생 조회 실패: {}", e))?;

    // 표준 교습비를 1회 로드해 N+1 제거 (weekly_hours → amount, 활성 항목만).
    // standard_fees 는 소규모 코드 테이블이라 인덱스 없이도 단일 조회로 충분.
    let fee_rows = sqlx::query("SELECT weekly_hours, amount FROM standard_fees WHERE is_active = 1")
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| format!("표준 교습비 조회 실패: {}", e))?;
    let fee_map: HashMap<i64, i64> = fee_rows
        .iter()
        .map(|r| (r.get::<i64, _>("weekly_hours"), r.get::<i64, _>("amount")))
        .collect();

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

        // standard_fees 매핑 — 사전 로드한 맵에서 조회(없으면 skip).
        let bill_amount = match fee_map.get(&weekly_hours) {
            Some(&a) => a,
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
///   1. 미확정(`draft`) 먼저, 확정(`confirmed`) 다음
///   2. 월중입퇴교(`is_mid_month=1`) 가 같은 상태 내 우선
///   3. 학생 학년별(school_level→grade)+이름 ASC — Sprint 19 T3(사용자 요청 1번). 확정/미확정
///      워크플로우 그룹핑(1·2)은 업무상 우선순위라 유지하고, 그 안에서만 학년+이름 정렬 적용
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
                b.is_mid_month, b.mid_month_type, b.confirmed_at, \
                COALESCE(p.is_paid, 0) AS is_paid \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.bill_year_month = ? \
         ORDER BY \
            CASE b.status WHEN 'draft' THEN 0 ELSE 1 END ASC, \
            b.is_mid_month DESC, \
            s.school_level ASC, s.grade ASC, s.name ASC",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("청구 목록 조회 실패: {}", e))?;

    rows.into_iter().map(row_to_bill).collect()
}

/// 청구가 생성된 distinct 청구년월 목록 (내림차순). 월별 집계 탭 기간 선택용 —
/// 실제 청구가 존재하는 년월만 제시한다.
#[tauri::command]
pub async fn list_billed_months() -> Result<Vec<String>, String> {
    let pool = db::pool().map_err(String::from)?;
    list_billed_months_impl(pool).await
}

pub(crate) async fn list_billed_months_impl(pool: &SqlitePool) -> Result<Vec<String>, String> {
    sqlx::query_scalar("SELECT DISTINCT bill_year_month FROM bills ORDER BY bill_year_month DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| format!("청구년월 목록 조회 실패: {}", e))
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
                b.is_mid_month, b.mid_month_type, b.confirmed_at, \
                COALESCE(p.is_paid, 0) AS is_paid \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("청구 조회 실패: {}", e))?
    .ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;
    row_to_bill(row)
}

/// 청구 금액 조정 (PRD §4.9.3).
///
/// `draft` / `confirmed` 모두 자유롭게 수정 (프론트 확인 다이얼로그는 별도 책임).
/// 단, 수납완료(`payments.is_paid=1`)된 청구는 이미 수금이 끝났으므로 금액 수정을 거부한다.
#[tauri::command]
pub async fn update_bill(id: i64, adjusted_amount: i64) -> Result<Bill, String> {
    let pool = db::pool().map_err(String::from)?;
    update_bill_impl(pool, id, adjusted_amount).await
}

pub(crate) async fn update_bill_impl(
    pool: &SqlitePool,
    id: i64,
    adjusted_amount: i64,
) -> Result<Bill, String> {
    if adjusted_amount < 0 {
        return Err("조정 금액은 0 이상이어야 합니다.".to_string());
    }

    // 청구 존재 확인 + 수납완료 여부를 1쿼리로 조회 (F1: 2회 왕복 통합).
    // 외부 Option = bills 행 존재, 내부 Option = is_paid (payments 없으면 NULL).
    let row: Option<Option<i64>> = sqlx::query_scalar(
        "SELECT p.is_paid FROM bills b LEFT JOIN payments p ON p.bill_id = b.id WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("청구 조회 실패: {}", e))?;
    let is_paid = row.ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;

    // 수납완료된 청구는 금액 수정 불가 (status 무관 — 이미 수금 완료).
    if is_paid == Some(1) {
        return Err("수납완료된 청구는 수정할 수 없습니다.".to_string());
    }

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

    get_bill_impl(pool, id).await
}

/// 청구 삭제 (Sprint 20 T3, ADR-010 B안).
///
/// 가드: **미수납(`payments.is_paid=0` 또는 payments 행 없음)** 이면 상태(draft/confirmed)
/// 무관 삭제 허용. **수납완료(`is_paid=1`)는 거부** — 먼저 수납을 해제해야 한다.
/// `payments.bill_id ON DELETE CASCADE`(V109) 로 수납 행도 함께 삭제된다 —
/// FK 강제(`PRAGMA foreign_keys=ON`, `db.rs`)가 전제.
#[tauri::command]
pub async fn delete_bill(id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    delete_bill_impl(pool, id).await
}

pub(crate) async fn delete_bill_impl(pool: &SqlitePool, id: i64) -> Result<(), String> {
    // 존재 확인 + 삭제 가드용 정보(수납여부·감사 상세)를 1쿼리로 조회.
    let row = sqlx::query(
        "SELECT b.student_id, b.bill_year_month, b.adjusted_amount, b.status, p.is_paid \
         FROM bills b LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("청구 조회 실패: {}", e))?
    .ok_or_else(|| format!("청구를 찾을 수 없습니다 (id={}).", id))?;

    let is_paid: Option<i64> = row.try_get("is_paid").map_err(|e| e.to_string())?;
    // ADR-010 B안: 수납완료된 청구는 삭제 거부.
    if is_paid == Some(1) {
        return Err("수납완료된 청구는 삭제할 수 없습니다. 먼저 수납을 해제하세요.".to_string());
    }

    let student_id: i64 = row.try_get("student_id").map_err(|e| e.to_string())?;
    let year_month: String = row.try_get("bill_year_month").map_err(|e| e.to_string())?;
    let amount: i64 = row.try_get("adjusted_amount").map_err(|e| e.to_string())?;
    let status: String = row.try_get("status").map_err(|e| e.to_string())?;
    let had_payment = is_paid.is_some();

    // payments 는 ON DELETE CASCADE 로 함께 삭제된다.
    sqlx::query("DELETE FROM bills WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("청구 삭제 실패: {}", e))?;

    // 감사 로그 — subject=청구 id, details 는 PII 없는 메타(원생명 미포함, student_id 만).
    let details = format!(
        r#"{{"billId":{},"studentId":{},"yearMonth":"{}","amount":{},"status":"{}","hadPayment":{}}}"#,
        id, student_id, year_month, amount, status, had_payment
    );
    audit::try_record(AuditEventType::BillDeleted, Some(&id.to_string()), Some(&details)).await;

    Ok(())
}

// ─────────────────────── T3: 상태 머신 ───────────────────────

/// 청구 단건 확정 — `draft` → `confirmed` (PRD §4.9.3).
///
/// `confirmed` 상태에서 호출 시 거부 (재확정 불가).
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

/// 수납 관리 뷰 한 행 — 청구 + payments 정보 통합 (post-Sprint 11 hotfix).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentViewRow {
    pub bill_id: i64,
    pub payment_id: Option<i64>,
    pub student_id: i64,
    pub student_name: String,
    pub student_serial_no: String,
    /// Sprint 19 사용자 요청 — 기본 정렬(학년별+이름) 및 화면 표시용.
    pub student_grade: i64,
    pub student_school_level: String,
    pub adjusted_amount: i64,
    pub is_mid_month: bool,
    pub mid_month_type: Option<String>,
    pub is_paid: bool,
    pub paid_date: Option<String>,
    pub payer_name: Option<String>,
    pub payment_method_id: Option<i64>,
    pub payment_method_label: Option<String>,
    pub card_company_id: Option<i64>,
    pub card_company_label: Option<String>,
}

/// 해당 월의 모든 청구 + 수납 정보 통합 조회 — PaymentsView 가 사용.
#[tauri::command]
pub async fn list_payment_view(year_month: String) -> Result<Vec<PaymentViewRow>, String> {
    let pool = db::pool().map_err(String::from)?;
    list_payment_view_impl(pool, &year_month).await
}

pub(crate) async fn list_payment_view_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<PaymentViewRow>, String> {
    validate_year_month(year_month)?;
    // Sprint 19 사용자 요청 — 미수납/월중입퇴교 업무 그룹핑(기존 우선순위)은 유지하되,
    // 그 안에서의 정렬 기준을 이름 단독에서 학년별(school_level→grade)+이름으로 강화
    // (billing.rs list_bills_impl과 동일 정책).
    let rows = sqlx::query(
        "SELECT b.id AS bill_id, p.id AS payment_id, b.student_id, \
                s.name AS student_name, s.serial_no, s.grade, s.school_level, \
                b.adjusted_amount, \
                b.is_mid_month, b.mid_month_type, \
                COALESCE(p.is_paid, 0) AS is_paid, \
                p.paid_date, p.payer_name, \
                p.payment_method_id, pm.label AS payment_method_label, \
                p.card_company_id, cc.label AS card_company_label \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payments p ON p.bill_id = b.id \
         LEFT JOIN payment_methods pm ON pm.id = p.payment_method_id \
         LEFT JOIN card_companies cc ON cc.id = p.card_company_id \
         WHERE b.bill_year_month = ? \
         ORDER BY \
            COALESCE(p.is_paid, 0) ASC, \
            b.is_mid_month DESC, \
            s.school_level ASC, s.grade ASC, s.name ASC",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("수납 뷰 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(PaymentViewRow {
                bill_id: r.try_get("bill_id").map_err(|e| e.to_string())?,
                payment_id: r.try_get("payment_id").map_err(|e| e.to_string())?,
                student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
                student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
                student_serial_no: r.try_get("serial_no").map_err(|e| e.to_string())?,
                student_grade: r.try_get("grade").map_err(|e| e.to_string())?,
                student_school_level: r.try_get("school_level").map_err(|e| e.to_string())?,
                adjusted_amount: r.try_get("adjusted_amount").map_err(|e| e.to_string())?,
                is_mid_month: {
                    let v: i64 = r.try_get("is_mid_month").map_err(|e| e.to_string())?;
                    v != 0
                },
                mid_month_type: r.try_get("mid_month_type").map_err(|e| e.to_string())?,
                is_paid: {
                    let v: i64 = r.try_get("is_paid").map_err(|e| e.to_string())?;
                    v != 0
                },
                paid_date: r.try_get("paid_date").map_err(|e| e.to_string())?,
                payer_name: r.try_get("payer_name").map_err(|e| e.to_string())?,
                payment_method_id: r.try_get("payment_method_id").map_err(|e| e.to_string())?,
                payment_method_label: r
                    .try_get("payment_method_label")
                    .map_err(|e| e.to_string())?,
                card_company_id: r.try_get("card_company_id").map_err(|e| e.to_string())?,
                card_company_label: r
                    .try_get("card_company_label")
                    .map_err(|e| e.to_string())?,
            })
        })
        .collect()
}

// ─────────────────────── 검색 + 자동 채움 (post-Sprint 11) ───────────────────────

/// 검색 결과 한 행 — 매칭된 학생 + 그 학생의 가장 최근 수납 정보(자동 채움용).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BillingSearchResult {
    pub student_id: i64,
    pub student_name: String,
    pub latest_payer_name: Option<String>,
    pub latest_payment_method_id: Option<i64>,
    pub latest_card_company_id: Option<i64>,
}

/// 청구·수납 통합 검색 — 원생 이름 / 연락처(`-` 제거 후 완전 일치) / 입금자 이름(완전 일치).
///
/// 입금자로 매칭된 경우 그 입금자가 과거 수납한 모든 원생을 결과에 포함.
/// 각 행에 그 학생의 가장 최근 `is_paid=1` payments 의 (입금자/결제수단/카드사) 정보를 함께 반환 —
/// 미수납 청구 자동 채움에 사용.
#[tauri::command]
pub async fn search_students_for_billing(
    query: String,
) -> Result<Vec<BillingSearchResult>, String> {
    let pool = db::pool().map_err(String::from)?;
    search_students_for_billing_impl(pool, &query).await
}

pub(crate) async fn search_students_for_billing_impl(
    pool: &SqlitePool,
    query: &str,
) -> Result<Vec<BillingSearchResult>, String> {
    let q = query.trim();
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        "WITH matching_ids AS ( \
            SELECT id FROM students WHERE name = ? \
            UNION \
            SELECT id FROM students \
             WHERE REPLACE(COALESCE(phone_student, ''), '-', '') = ? \
                OR REPLACE(COALESCE(phone_mother, ''), '-', '') = ? \
                OR REPLACE(COALESCE(phone_father, ''), '-', '') = ? \
            UNION \
            SELECT b.student_id FROM bills b \
              JOIN payments p ON p.bill_id = b.id \
             WHERE p.payer_name = ? \
         ), \
         latest_per_student AS ( \
            SELECT b.student_id, p.payer_name, p.payment_method_id, p.card_company_id, \
                   ROW_NUMBER() OVER ( \
                     PARTITION BY b.student_id \
                     ORDER BY p.paid_date DESC, p.created_at DESC \
                   ) AS rn \
            FROM payments p JOIN bills b ON b.id = p.bill_id \
            WHERE p.is_paid = 1 \
         ) \
         SELECT s.id AS student_id, s.name AS student_name, \
                lp.payer_name AS latest_payer_name, \
                lp.payment_method_id AS latest_payment_method_id, \
                lp.card_company_id AS latest_card_company_id \
         FROM students s \
         LEFT JOIN latest_per_student lp ON lp.student_id = s.id AND lp.rn = 1 \
         WHERE s.id IN (SELECT id FROM matching_ids) \
         ORDER BY s.name",
    )
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .bind(q)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("청구 검색 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(BillingSearchResult {
                student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
                student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
                latest_payer_name: r
                    .try_get("latest_payer_name")
                    .map_err(|e| e.to_string())?,
                latest_payment_method_id: r
                    .try_get("latest_payment_method_id")
                    .map_err(|e| e.to_string())?,
                latest_card_company_id: r
                    .try_get("latest_card_company_id")
                    .map_err(|e| e.to_string())?,
            })
        })
        .collect()
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

/// 청구 대상 기간을 교습기간(`study_periods`)에서 조회한다 — T1 (Sprint 20).
///
/// 반환: `Some((start_date, end_date))` = 해당 월 교습기간 존재 / `None` = 미등록.
/// `generate_bills_impl` 과 `get_billing_summary_impl` 이 **동일한 대상 기간·규칙**을
/// 쓰도록 단일 헬퍼로 통일한다. 한쪽만 교습기간 기준으로 바꾸면 청구 대상 수와 생성 청구
/// 수가 어긋나 "추가 청구 데이터 생성 (N명)" 유령 버튼이 발생한다 (R135).
async fn load_billing_period_range(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Option<(String, String)>, String> {
    let row = sqlx::query("SELECT start_date, end_date FROM study_periods WHERE year_month = ?")
        .bind(year_month)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("교습기간 조회 실패: {}", e))?;
    match row {
        Some(r) => {
            let start: String = r.try_get("start_date").map_err(|e| e.to_string())?;
            let end: String = r.try_get("end_date").map_err(|e| e.to_string())?;
            Ok(Some((start, end)))
        }
        None => Ok(None),
    }
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
        confirmed_at: r.try_get("confirmed_at").map_err(|e| e.to_string())?,
        is_paid: {
            let v: i64 = r.try_get("is_paid").map_err(|e| e.to_string())?;
            v != 0
        },
    })
}

// ─────────────────────── T4: 수납 (payments) ───────────────────────

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: i64,
    pub bill_id: i64,
    pub is_paid: bool,
    pub paid_date: Option<String>,
    pub payer_name: Option<String>,
    pub payment_method_id: Option<i64>,
    pub payment_method_label: Option<String>,
    pub card_company_id: Option<i64>,
    pub card_company_label: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInput {
    pub bill_id: i64,
    pub is_paid: bool,
    pub paid_date: Option<String>,
    pub payer_name: Option<String>,
    pub payment_method_id: Option<i64>,
    pub card_company_id: Option<i64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnpaidBill {
    pub bill_id: i64,
    pub student_id: i64,
    pub student_name: String,
    pub student_serial_no: String,
    /// Sprint 19 사용자 요청 — 기본 정렬(학년별+이름) 및 화면 표시용.
    pub student_grade: i64,
    pub student_school_level: String,
    pub adjusted_amount: i64,
    pub is_mid_month: bool,
    pub mid_month_type: Option<String>,
}

/// 월별 청구·수납 요약 (PRD §4.11.3 대시보드 위젯 선행 준비).
///
/// `total_billable_students` (hotfix post-Sprint 11): 해당 월에 **수업을 진행한 원생 수**.
/// 청구년월 'YYYY-MM' 은 그 해·달 수업 원생의 교습비 청구서를 의미한다 (예: '2026-05' = 2026년 5월).
/// 정의: `enroll_date <= 월말 AND (withdraw_date IS NULL OR withdraw_date >= 월초)`
///       AND 현재 유효 스케줄(`effective_to IS NULL`) 의 `duration_hours` 합 > 0
/// "추가 청구 데이터 생성" UX 트리거: `total_billable_students > bill_count`.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BillingSummary {
    pub year_month: String,
    pub total_billable_students: i64,
    pub bill_count: i64,
    pub total_billed: i64,    // adjusted_amount 합계
    pub total_paid: i64,      // is_paid=1 한정
    pub total_unpaid: i64,    // total_billed - total_paid
    pub paid_count: i64,
    pub unpaid_count: i64,
}

/// 결제수단별 수납 집계 (월별 집계 탭) — is_paid=1 한정, 청구액(adjusted_amount) 기준 총액.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethodSummary {
    /// 결제수단 코드 ID. 미지정(legacy 데이터에서 NULL) 시 None.
    pub payment_method_id: Option<i64>,
    pub payment_method_label: String,
    pub paid_count: i64,
    pub total_paid: i64,
}

/// 기간(연도 'YYYY' 또는 월 'YYYY-MM') 청구·수납 집계 (월별 집계 탭).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BillingPeriodStats {
    /// 요청 기간 문자열 — 'YYYY'(연도) 또는 'YYYY-MM'(월).
    pub period: String,
    pub bill_count: i64,
    pub total_billed: i64,
    pub paid_count: i64,
    pub total_paid: i64,
    pub total_unpaid: i64,
    pub unpaid_count: i64,
    /// 결제수단별 수납 총액 (is_paid=1 한정).
    pub by_method: Vec<PaymentMethodSummary>,
}

/// 수납 생성 — bill_id UNIQUE. 동일 청구에 이미 payments 행 있으면 에러 (update_payment 사용).
#[tauri::command]
pub async fn create_payment(input: PaymentInput) -> Result<Payment, String> {
    let pool = db::pool().map_err(String::from)?;
    create_payment_impl(pool, &input).await
}

pub(crate) async fn create_payment_impl(
    pool: &SqlitePool,
    input: &PaymentInput,
) -> Result<Payment, String> {
    validate_payment_input(pool, input).await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO payments \
            (bill_id, is_paid, paid_date, payer_name, payment_method_id, card_company_id) \
         VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(input.bill_id)
    .bind(input.is_paid as i64)
    .bind(&input.paid_date)
    .bind(&input.payer_name)
    .bind(input.payment_method_id)
    .bind(input.card_company_id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("수납 생성 실패: {}", e))?;
    get_payment_impl(pool, id).await
}

/// 수납 갱신 — id 기반. bill_id 변경 불가 (입력 무시).
#[tauri::command]
pub async fn update_payment(id: i64, input: PaymentInput) -> Result<Payment, String> {
    let pool = db::pool().map_err(String::from)?;
    update_payment_impl(pool, id, &input).await
}

pub(crate) async fn update_payment_impl(
    pool: &SqlitePool,
    id: i64,
    input: &PaymentInput,
) -> Result<Payment, String> {
    validate_payment_input(pool, input).await?;
    let res = sqlx::query(
        "UPDATE payments SET \
            is_paid = ?, paid_date = ?, payer_name = ?, \
            payment_method_id = ?, card_company_id = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?",
    )
    .bind(input.is_paid as i64)
    .bind(&input.paid_date)
    .bind(&input.payer_name)
    .bind(input.payment_method_id)
    .bind(input.card_company_id)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("수납 갱신 실패: {}", e))?;
    if res.rows_affected() == 0 {
        return Err(format!("수납을 찾을 수 없습니다 (id={}).", id));
    }
    get_payment_impl(pool, id).await
}

/// 미납 청구 목록 — 입금 일괄 처리 화면용 (AC-4.9-6).
///
/// 미납 정의: payments 없음 OR `is_paid=0`. 정렬: 학생명 ASC.
#[tauri::command]
pub async fn list_unpaid_bills(year_month: String) -> Result<Vec<UnpaidBill>, String> {
    let pool = db::pool().map_err(String::from)?;
    list_unpaid_bills_impl(pool, &year_month).await
}

pub(crate) async fn list_unpaid_bills_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<UnpaidBill>, String> {
    validate_year_month(year_month)?;
    // Sprint 19 사용자 요청 — 기본 정렬을 이름 단독에서 학년별(school_level→grade)+이름으로 강화.
    let rows = sqlx::query(
        "SELECT b.id AS bill_id, b.student_id, s.name AS student_name, s.serial_no, \
                s.grade, s.school_level, \
                b.adjusted_amount, b.is_mid_month, b.mid_month_type \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.bill_year_month = ? \
           AND (p.id IS NULL OR p.is_paid = 0) \
         ORDER BY s.school_level ASC, s.grade ASC, s.name ASC",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("미납 청구 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(UnpaidBill {
                bill_id: r.try_get("bill_id").map_err(|e| e.to_string())?,
                student_id: r.try_get("student_id").map_err(|e| e.to_string())?,
                student_name: r.try_get("student_name").map_err(|e| e.to_string())?,
                student_serial_no: r.try_get("serial_no").map_err(|e| e.to_string())?,
                student_grade: r.try_get("grade").map_err(|e| e.to_string())?,
                student_school_level: r.try_get("school_level").map_err(|e| e.to_string())?,
                adjusted_amount: r.try_get("adjusted_amount").map_err(|e| e.to_string())?,
                is_mid_month: {
                    let v: i64 = r.try_get("is_mid_month").map_err(|e| e.to_string())?;
                    v != 0
                },
                mid_month_type: r.try_get("mid_month_type").map_err(|e| e.to_string())?,
            })
        })
        .collect()
}

/// 다수 입금 일괄 처리 — 단일 트랜잭션. 하나라도 실패 시 전체 롤백.
///
/// 각 entry 는 bill_id 기준 UPSERT — payments 행 없으면 INSERT, 있으면 UPDATE.
/// 반환값: 처리된 entry 수 (성공 시 입력 길이와 동일).
#[tauri::command]
pub async fn batch_update_payments(items: Vec<PaymentInput>) -> Result<i64, String> {
    let pool = db::pool().map_err(String::from)?;
    batch_update_payments_impl(pool, &items).await
}

pub(crate) async fn batch_update_payments_impl(
    pool: &SqlitePool,
    items: &[PaymentInput],
) -> Result<i64, String> {
    if items.is_empty() {
        return Ok(0);
    }
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    // BEGIN IMMEDIATE 효과
    sqlx::query("SELECT 1 FROM payments LIMIT 0")
        .execute(&mut *tx)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    for item in items {
        validate_payment_input_in_tx(&mut tx, item).await?;
        sqlx::query(
            "INSERT INTO payments \
                (bill_id, is_paid, paid_date, payer_name, payment_method_id, card_company_id) \
             VALUES (?, ?, ?, ?, ?, ?) \
             ON CONFLICT(bill_id) DO UPDATE SET \
                is_paid = excluded.is_paid, \
                paid_date = excluded.paid_date, \
                payer_name = excluded.payer_name, \
                payment_method_id = excluded.payment_method_id, \
                card_company_id = excluded.card_company_id, \
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
        )
        .bind(item.bill_id)
        .bind(item.is_paid as i64)
        .bind(&item.paid_date)
        .bind(&item.payer_name)
        .bind(item.payment_method_id)
        .bind(item.card_company_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("수납 UPSERT 실패 (bill_id={}): {}", item.bill_id, e))?;
    }

    tx.commit()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    Ok(items.len() as i64)
}

/// 월별 청구·수납 요약 (PRD §4.11.3 대시보드 위젯 선행 준비).
#[tauri::command]
pub async fn get_billing_summary(year_month: String) -> Result<BillingSummary, String> {
    let pool = db::pool().map_err(String::from)?;
    get_billing_summary_impl(pool, &year_month).await
}

pub(crate) async fn get_billing_summary_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<BillingSummary, String> {
    validate_year_month(year_month)?;
    // T1 (Sprint 20): total_billable_students 산정을 generate_bills 와 동일한 교습기간 기준으로
    // 통일한다. 한쪽만 바꾸면 "추가 청구 데이터 생성 (N명)" 유령 버튼 발생 (R135).
    let period = load_billing_period_range(pool, year_month).await?;

    let row = sqlx::query(
        "SELECT \
            COUNT(*) AS bill_count, \
            COALESCE(SUM(b.adjusted_amount), 0) AS total_billed, \
            COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN b.adjusted_amount ELSE 0 END), 0) AS total_paid, \
            COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN 1 ELSE 0 END), 0) AS paid_count \
         FROM bills b \
         LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.bill_year_month = ?",
    )
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("청구 요약 조회 실패: {}", e))?;

    // 해당 교습기간에 수업을 진행하는 청구 대상 원생 수 (generate_bills 와 동일 규칙:
    // enroll_date ≤ 교습기간종료, 미퇴교, effective_from ≤ 교습기간종료, 주 수업시간 > 0).
    // 교습기간 미등록 월은 청구 생성이 차단되므로 대상도 0 으로 본다.
    let total_billable_students: i64 = match &period {
        Some((period_start, period_end)) => sqlx::query_scalar(
            "SELECT COUNT(*) FROM ( \
                SELECT s.id \
                FROM students s \
                INNER JOIN student_schedules sch \
                    ON sch.student_id = s.id AND sch.effective_to IS NULL \
                       AND sch.effective_from <= ? \
                WHERE s.enroll_date <= ? \
                  AND (s.withdraw_date IS NULL OR s.withdraw_date >= ?) \
                GROUP BY s.id \
                HAVING SUM(sch.duration_hours) > 0 \
             )",
        )
        .bind(period_end)
        .bind(period_end)
        .bind(period_start)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("청구 대상 원생 수 조회 실패: {}", e))?,
        None => 0,
    };

    let bill_count: i64 = row.try_get("bill_count").map_err(|e| e.to_string())?;
    let total_billed: i64 = row.try_get("total_billed").map_err(|e| e.to_string())?;
    let total_paid: i64 = row.try_get("total_paid").map_err(|e| e.to_string())?;
    let paid_count: i64 = row.try_get("paid_count").map_err(|e| e.to_string())?;

    Ok(BillingSummary {
        year_month: year_month.to_string(),
        total_billable_students,
        bill_count,
        total_billed,
        total_paid,
        total_unpaid: total_billed - total_paid,
        paid_count,
        unpaid_count: bill_count - paid_count,
    })
}

/// 기간 문자열을 `bill_year_month LIKE` 패턴으로 변환 + 검증.
/// - 'YYYY' (4자리 숫자) → 연도 집계 → "YYYY-%"
/// - 'YYYY-MM' → 월 집계 → "YYYY-MM" (와일드카드 없음 = 정확 일치)
fn period_like_pattern(period: &str) -> Result<String, String> {
    if period.len() == 4 && period.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(format!("{}-%", period));
    }
    validate_year_month(period)?;
    Ok(period.to_string())
}

/// 기간(연도 'YYYY' 또는 월 'YYYY-MM') 청구·수납 집계 + 결제수단별 수납 총액 (월별 집계 탭).
#[tauri::command]
pub async fn get_billing_period_stats(period: String) -> Result<BillingPeriodStats, String> {
    let pool = db::pool().map_err(String::from)?;
    get_billing_period_stats_impl(pool, &period).await
}

pub(crate) async fn get_billing_period_stats_impl(
    pool: &SqlitePool,
    period: &str,
) -> Result<BillingPeriodStats, String> {
    let pattern = period_like_pattern(period)?;

    let row = sqlx::query(
        "SELECT COUNT(*) AS bill_count, \
                COALESCE(SUM(b.adjusted_amount), 0) AS total_billed, \
                COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN b.adjusted_amount ELSE 0 END), 0) AS total_paid, \
                COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN 1 ELSE 0 END), 0) AS paid_count \
         FROM bills b \
         LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.bill_year_month LIKE ?",
    )
    .bind(&pattern)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("기간 집계 조회 실패: {}", e))?;

    let bill_count: i64 = row.try_get("bill_count").map_err(|e| e.to_string())?;
    let total_billed: i64 = row.try_get("total_billed").map_err(|e| e.to_string())?;
    let total_paid: i64 = row.try_get("total_paid").map_err(|e| e.to_string())?;
    let paid_count: i64 = row.try_get("paid_count").map_err(|e| e.to_string())?;

    let method_rows = sqlx::query(
        "SELECT p.payment_method_id AS pm_id, \
                COALESCE(pm.label, '미지정') AS pm_label, \
                COUNT(p.id) AS paid_count, \
                COALESCE(SUM(b.adjusted_amount), 0) AS total_paid \
         FROM payments p \
         JOIN bills b ON b.id = p.bill_id \
         LEFT JOIN payment_methods pm ON pm.id = p.payment_method_id \
         WHERE b.bill_year_month LIKE ? AND p.is_paid = 1 \
         GROUP BY p.payment_method_id \
         ORDER BY COALESCE(pm.display_order, 9999), pm.label",
    )
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("결제수단별 수납 집계 조회 실패: {}", e))?;

    let by_method = method_rows
        .into_iter()
        .map(|r| {
            Ok(PaymentMethodSummary {
                payment_method_id: r.try_get("pm_id").map_err(|e| e.to_string())?,
                payment_method_label: r.try_get("pm_label").map_err(|e| e.to_string())?,
                paid_count: r.try_get("paid_count").map_err(|e| e.to_string())?,
                total_paid: r.try_get("total_paid").map_err(|e| e.to_string())?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(BillingPeriodStats {
        period: period.to_string(),
        bill_count,
        total_billed,
        paid_count,
        total_paid,
        total_unpaid: total_billed - total_paid,
        unpaid_count: bill_count - paid_count,
        by_method,
    })
}

// ─── T4 헬퍼 ───

pub(crate) async fn get_payment_impl(pool: &SqlitePool, id: i64) -> Result<Payment, String> {
    let row = sqlx::query(
        "SELECT p.id, p.bill_id, p.is_paid, p.paid_date, p.payer_name, \
                p.payment_method_id, pm.label AS payment_method_label, \
                p.card_company_id, cc.label AS card_company_label \
         FROM payments p \
         LEFT JOIN payment_methods pm ON pm.id = p.payment_method_id \
         LEFT JOIN card_companies cc ON cc.id = p.card_company_id \
         WHERE p.id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("수납 조회 실패: {}", e))?
    .ok_or_else(|| format!("수납을 찾을 수 없습니다 (id={}).", id))?;

    Ok(Payment {
        id: row.try_get("id").map_err(|e| e.to_string())?,
        bill_id: row.try_get("bill_id").map_err(|e| e.to_string())?,
        is_paid: {
            let v: i64 = row.try_get("is_paid").map_err(|e| e.to_string())?;
            v != 0
        },
        paid_date: row.try_get("paid_date").map_err(|e| e.to_string())?,
        payer_name: row.try_get("payer_name").map_err(|e| e.to_string())?,
        payment_method_id: row.try_get("payment_method_id").map_err(|e| e.to_string())?,
        payment_method_label: row.try_get("payment_method_label").map_err(|e| e.to_string())?,
        card_company_id: row.try_get("card_company_id").map_err(|e| e.to_string())?,
        card_company_label: row.try_get("card_company_label").map_err(|e| e.to_string())?,
    })
}

/// 카드 계열 결제수단 시 card_company_id 필수 검증 (AC-4.9-4).
/// is_paid=1 일 때 paid_date 필수 (CHECK 제약과 중복이지만 친화적 메시지).
async fn validate_payment_input(pool: &SqlitePool, input: &PaymentInput) -> Result<(), String> {
    if input.is_paid && input.paid_date.is_none() {
        return Err("입금 완료 상태에는 입금일이 필요합니다.".to_string());
    }
    if input.is_paid && input.payment_method_id.is_none() {
        return Err("입금 완료 상태에는 결제수단이 필요합니다.".to_string());
    }
    if let Some(pmid) = input.payment_method_id {
        let is_card: Option<i64> = sqlx::query_scalar(
            "SELECT is_card_type FROM payment_methods WHERE id = ?",
        )
        .bind(pmid)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("결제수단 조회 실패: {}", e))?;
        match is_card {
            Some(1) if input.card_company_id.is_none() => {
                return Err("카드 계열 결제수단은 카드사를 함께 선택해야 합니다.".to_string());
            }
            None => {
                return Err(format!("결제수단을 찾을 수 없습니다 (id={}).", pmid));
            }
            _ => {}
        }
    }
    Ok(())
}

/// 트랜잭션 안에서 호출하는 변종 — same logic, &mut tx 인자.
async fn validate_payment_input_in_tx(
    tx: &mut sqlx::SqliteConnection,
    input: &PaymentInput,
) -> Result<(), String> {
    if input.is_paid && input.paid_date.is_none() {
        return Err("입금 완료 상태에는 입금일이 필요합니다.".to_string());
    }
    if input.is_paid && input.payment_method_id.is_none() {
        return Err("입금 완료 상태에는 결제수단이 필요합니다.".to_string());
    }
    if let Some(pmid) = input.payment_method_id {
        let is_card: Option<i64> = sqlx::query_scalar(
            "SELECT is_card_type FROM payment_methods WHERE id = ?",
        )
        .bind(pmid)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| format!("결제수단 조회 실패: {}", e))?;
        match is_card {
            Some(1) if input.card_company_id.is_none() => {
                return Err("카드 계열 결제수단은 카드사를 함께 선택해야 합니다.".to_string());
            }
            None => {
                return Err(format!("결제수단을 찾을 수 없습니다 (id={}).", pmid));
            }
            _ => {}
        }
    }
    Ok(())
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
        // A73: withdraw 를 문자열 보간 대신 bind 파라미터로 (SQL 인젝션 방지 패턴 일관성).
        sqlx::query_scalar(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date, withdraw_date) \
             VALUES (?, ?, 'male', 'elementary', 3, ?, ?) RETURNING id",
        )
        .bind(serial)
        .bind(name)
        .bind(enroll)
        .bind(withdraw)
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

    /// 교습기간(study_periods) 시드 헬퍼 — T1(Sprint 20) 이후 청구 생성이 교습기간을 요구.
    async fn seed_period(pool: &SqlitePool, ym: &str, start: &str, end: &str) {
        sqlx::query(
            "INSERT OR IGNORE INTO study_periods (year_month, start_date, end_date) \
             VALUES (?, ?, ?)",
        )
        .bind(ym)
        .bind(start)
        .bind(end)
        .execute(pool)
        .await
        .expect("seed period");
    }

    async fn seed_standard_fee(pool: &SqlitePool, weekly_hours: i64, amount: i64) {
        // T1(Sprint 20): 청구 생성이 교습기간 존재를 요구하므로, 청구 테스트가 공통으로 쓰는
        // 2026-05 교습기간(5/1~5/31)을 이 헬퍼에서 함께 시드한다 (INSERT OR IGNORE, 중복 무해).
        seed_period(pool, "2026-05", "2026-05-01", "2026-05-31").await;
        sqlx::query(
            "INSERT INTO standard_fees (weekly_hours, amount) VALUES (?, ?)",
        )
        .bind(weekly_hours)
        .bind(amount)
        .execute(pool)
        .await
        .expect("seed fee");
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
        // 이 테스트는 seed_standard_fee 를 호출하지 않으므로 교습기간을 직접 시드.
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31").await;

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

    // ─────────────────────── T1 (Sprint 20): 교습기간 기준 전환 ───────────────────────

    #[tokio::test]
    async fn generate_bills_excludes_enroll_after_teaching_period_end() {
        // 7월 교습기간 7/2~7/29, 입교 7/30 → 교습기간 종료 이후 입교 → 7월 청구 제외.
        // (달력월 기준이었다면 7/30 ≤ 7/31 로 잘못 포함되던 버그)
        let pool = db::test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-07", "2026-07-02", "2026-07-29").await;
        let sid = seed_student(&pool, "1", "원생A", "2026-07-30", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-07").await.expect("ok");
        assert_eq!(result.generated_count, 0, "교습기간 종료 이후 입교생은 청구 제외");
    }

    #[tokio::test]
    async fn generate_bills_includes_enroll_within_teaching_period() {
        // 7월 교습기간 7/2~7/29, 입교 7/10 → 월중입교로 포함(mid_month enrolled).
        let pool = db::test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-07", "2026-07-02", "2026-07-29").await;
        let sid = seed_student(&pool, "1", "원생A", "2026-07-10", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-07").await.expect("ok");
        assert_eq!(result.generated_count, 1);
        let bills = list_bills_impl(&pool, "2026-07").await.expect("list");
        assert!(bills[0].is_mid_month);
        assert_eq!(bills[0].mid_month_type.as_deref(), Some("enrolled"));
    }

    #[tokio::test]
    async fn generate_bills_blocks_when_no_study_period() {
        // 교습기간 미등록 월 → 청구 생성 차단(에러 + 안내 메시지).
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await; // 2026-05 만 시드됨
        let err = generate_bills_impl(&pool, "2026-06").await;
        assert!(err.is_err(), "교습기간 미등록 월은 차단");
        assert!(
            err.unwrap_err().contains("교습기간"),
            "안내 메시지에 교습기간 언급"
        );
    }

    #[tokio::test]
    async fn generate_bills_excludes_schedule_effective_after_period_end() {
        // 교습기간 내 재원이지만 스케줄이 교습기간 종료 이후 시작 → weekly_hours 0 → skip.
        let pool = db::test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-07", "2026-07-02", "2026-07-29").await;
        let sid = seed_student(&pool, "1", "원생A", "2026-07-05", None).await;
        sqlx::query(
            "INSERT INTO student_schedules \
                (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 1, '16:00', 1, '2026-07-30')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("seed late schedule");
        seed_standard_fee(&pool, 1, 100_000).await;

        let result = generate_bills_impl(&pool, "2026-07").await.expect("ok");
        assert_eq!(
            result.generated_count, 0,
            "교습기간 종료 이후 시작 스케줄은 집계 제외"
        );
    }

    #[tokio::test]
    async fn summary_total_billable_matches_generate_target_teaching_period() {
        // (f) get_billing_summary 대상 수가 generate_bills 와 동일 규칙 → 유령 버튼 회귀 방지.
        // 7월 교습기간 7/2~7/29. A(1/1 입교, 대상) + B(7/30 입교, 종료 이후 → 비대상).
        let pool = db::test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-07", "2026-07-02", "2026-07-29").await;
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-07-30", None).await;
        seed_schedule(&pool, a, 1, 1).await;
        seed_schedule(&pool, b, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;

        generate_bills_impl(&pool, "2026-07").await.expect("gen");
        let s = get_billing_summary_impl(&pool, "2026-07").await.expect("ok");
        assert_eq!(s.bill_count, 1, "A 만 청구 생성");
        assert_eq!(
            s.total_billable_students, 1,
            "B(7/30 입교)는 대상 아님 → 유령 버튼 없음"
        );
    }

    // ─────────────────────── T3 (Sprint 20): 청구 삭제 (ADR-010 B안) ───────────────────────

    /// FK 강제(PRAGMA foreign_keys=ON) — 프로덕션(db.rs startup)과 동일 환경에서 payments
    /// CASCADE 삭제를 검증하기 위함. 인메모리 테스트 풀은 SQLite 기본값 OFF.
    async fn enable_fk(pool: &SqlitePool) {
        sqlx::query("PRAGMA foreign_keys=ON")
            .execute(pool)
            .await
            .expect("fk on");
    }

    #[tokio::test]
    async fn delete_bill_removes_draft_and_cascades_payment() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        enable_fk(&pool).await;
        let bid = seed_bill(&pool, "2026-05").await;
        // 미수납(is_paid=0) payment 행 생성 → CASCADE 삭제 대상.
        create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: false,
                paid_date: None,
                payer_name: None,
                payment_method_id: None,
                card_company_id: None,
            },
        )
        .await
        .expect("unpaid payment");

        delete_bill_impl(&pool, bid).await.expect("delete ok");

        let bills: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bills WHERE id=?")
            .bind(bid)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(bills, 0, "청구 삭제됨");
        let pays: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payments WHERE bill_id=?")
            .bind(bid)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(pays, 0, "payments CASCADE 삭제됨");
    }

    #[tokio::test]
    async fn delete_bill_allows_confirmed_when_unpaid() {
        // B안 핵심: 확정(confirmed) + 미수납 → 삭제 허용.
        let pool = db::test_pool_in_memory().await.expect("pool");
        enable_fk(&pool).await;
        let bid = seed_bill(&pool, "2026-05").await;
        confirm_bill_impl(&pool, bid).await.expect("confirm");
        delete_bill_impl(&pool, bid).await.expect("confirmed unpaid delete");
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bills WHERE id=?")
            .bind(bid)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cnt, 0);
    }

    #[tokio::test]
    async fn delete_bill_rejects_when_paid() {
        // B안: 수납완료(is_paid=1) → 삭제 거부.
        let pool = db::test_pool_in_memory().await.expect("pool");
        enable_fk(&pool).await;
        let bid = seed_bill(&pool, "2026-05").await;
        confirm_bill_impl(&pool, bid).await.expect("confirm");
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None,
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            }],
        )
        .await
        .expect("pay");

        let err = delete_bill_impl(&pool, bid).await;
        assert!(err.is_err(), "수납완료 청구는 삭제 거부");
        let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bills WHERE id=?")
            .bind(bid)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cnt, 1, "거부 시 청구 잔존");
    }

    #[tokio::test]
    async fn delete_bill_rejects_nonexistent() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let err = delete_bill_impl(&pool, 999).await;
        assert!(err.is_err(), "존재하지 않는 청구 삭제 거부");
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
    async fn list_bills_orders_by_grade_then_name_within_same_group() {
        // Sprint 19 T3(사용자 요청 1번): 동일 status+mid_month 그룹 내에서는
        // school_level→grade→name 순서 — 이름만으로는 구분 안 되는 경우도 검증.
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "학생가", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "학생나", "2026-01-01", None).await;
        for s in [a, b] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;
        // a 를 중학생으로 승격 — 이름순(가→나)과 반대로 나와야 school_level 이 실제
        // 정렬 기준임을 증명 (elementary 인 b 가 middle 인 a 보다 먼저).
        sqlx::query("UPDATE students SET school_level='middle', grade=2 WHERE id=?")
            .bind(a)
            .execute(&pool)
            .await
            .expect("학생 학교급 갱신");

        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bills = list_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(bills.len(), 2);
        assert_eq!(bills[0].student_id, b, "elementary 가 middle 보다 먼저");
        assert_eq!(bills[1].student_id, a);
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

        let updated = update_bill_impl(&pool, bill_id, 90_000).await.expect("ok");
        assert_eq!(updated.adjusted_amount, 90_000);
        assert_eq!(updated.bill_amount, 100_000, "표준 금액은 불변");
    }

    #[tokio::test]
    async fn update_bill_paid_rejected() {
        // 수납완료된 청구는 금액 수정 불가 (status 무관 — V111 마감 폐기 후 is_paid 기준).
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, sid, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_id: i64 = sqlx::query_scalar("SELECT id FROM bills LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        // 확정 + 수납완료
        confirm_bill_impl(&pool, bill_id).await.expect("confirm");
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None,
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            }],
        )
        .await
        .expect("pay");

        let err = update_bill_impl(&pool, bill_id, 80_000).await;
        assert!(err.is_err(), "수납완료된 청구는 수정 거부");
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

        let err = update_bill_impl(&pool, bill_id, -1).await;
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

    // ─────────────────────── T4 수납 ───────────────────────

    // V001 시드: payment_methods(id=2 card, is_card_type=1 by V109),
    //            card_companies(id=1 shinhan).
    const CARD_PAYMENT_METHOD_ID: i64 = 2;
    const CASH_PAYMENT_METHOD_ID: i64 = 1;
    const SHINHAN_CARD_ID: i64 = 1;

    #[tokio::test]
    async fn create_payment_inserts_and_returns_with_labels() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let p = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: Some("홍부모".to_string()),
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            },
        )
        .await
        .expect("ok");
        assert!(p.is_paid);
        assert_eq!(p.bill_id, bid);
        assert_eq!(p.payment_method_label.as_deref(), Some("현금"));
        assert!(p.card_company_label.is_none());
    }

    #[tokio::test]
    async fn create_payment_card_requires_card_company() {
        // AC-4.9-4
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let err = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None,
                payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                card_company_id: None,
            },
        )
        .await;
        assert!(err.is_err());
        let msg = err.unwrap_err();
        assert!(msg.contains("카드"), "에러 메시지에 카드 언급: {}", msg);
    }

    #[tokio::test]
    async fn create_payment_card_with_company_succeeds() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let p = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None,
                payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                card_company_id: Some(SHINHAN_CARD_ID),
            },
        )
        .await
        .expect("ok");
        assert_eq!(p.payment_method_label.as_deref(), Some("카드"));
        assert_eq!(p.card_company_label.as_deref(), Some("신한카드"));
    }

    #[tokio::test]
    async fn create_payment_rejects_paid_without_date() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let err = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: None,
                payer_name: None,
                payment_method_id: None,
                card_company_id: None,
            },
        )
        .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn create_payment_rejects_paid_without_method() {
        // #6: 입금 완료인데 결제수단 미선택이면 거부.
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let err = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-20".to_string()),
                payer_name: None,
                payment_method_id: None,
                card_company_id: None,
            },
        )
        .await;
        assert!(err.is_err(), "결제수단 누락 시 거부되어야 함");
    }

    #[tokio::test]
    async fn batch_cancel_payment_resets_is_paid() {
        // #5: is_paid=false + 입금정보 null 로 재UPSERT 하면 수납취소.
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, s, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bid: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(s)
            .fetch_one(&pool)
            .await
            .unwrap();
        // 먼저 수납완료
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: Some("홍부모".to_string()),
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            }],
        )
        .await
        .expect("pay");
        // 수납취소
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id: bid,
                is_paid: false,
                paid_date: None,
                payer_name: None,
                payment_method_id: None,
                card_company_id: None,
            }],
        )
        .await
        .expect("cancel");
        let unpaid = list_unpaid_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(unpaid.len(), 1, "수납취소 후 미납으로 복귀");
    }

    #[tokio::test]
    async fn update_payment_changes_fields() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let bid = seed_bill(&pool, "2026-05").await;
        let p = create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bid,
                is_paid: false,
                paid_date: None,
                payer_name: None,
                payment_method_id: None,
                card_company_id: None,
            },
        )
        .await
        .expect("create");

        let updated = update_payment_impl(
            &pool,
            p.id,
            &PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-20".to_string()),
                payer_name: Some("홍부모".to_string()),
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            },
        )
        .await
        .expect("update");
        assert!(updated.is_paid);
        assert_eq!(updated.paid_date.as_deref(), Some("2026-05-20"));
        assert_eq!(updated.payer_name.as_deref(), Some("홍부모"));
    }

    #[tokio::test]
    async fn list_unpaid_bills_excludes_paid() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 2명: A 미납, B 입금
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        seed_schedule(&pool, a, 1, 1).await;
        seed_schedule(&pool, b, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        // B 입금
        let b_bill: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id = ?")
            .bind(b)
            .fetch_one(&pool)
            .await
            .unwrap();
        create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: b_bill,
                is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None,
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            },
        )
        .await
        .expect("pay");

        let unpaid = list_unpaid_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(unpaid.len(), 1);
        assert_eq!(unpaid[0].student_id, a, "미납인 A 만 반환");
    }

    #[tokio::test]
    async fn batch_update_payments_upserts_all_in_transaction() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        for s in [a, b] { seed_schedule(&pool, s, 1, 1).await; }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_a: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(a).fetch_one(&pool).await.unwrap();
        let bill_b: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(b).fetch_one(&pool).await.unwrap();

        let processed = batch_update_payments_impl(
            &pool,
            &[
                PaymentInput {
                    bill_id: bill_a,
                    is_paid: true,
                    paid_date: Some("2026-05-15".to_string()),
                    payer_name: Some("부모A".to_string()),
                    payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
                PaymentInput {
                    bill_id: bill_b,
                    is_paid: true,
                    paid_date: Some("2026-05-16".to_string()),
                    payer_name: Some("부모B".to_string()),
                    payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                    card_company_id: Some(SHINHAN_CARD_ID),
                },
            ],
        )
        .await
        .expect("batch");
        assert_eq!(processed, 2);

        let paid_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payments WHERE is_paid = 1",
        )
        .fetch_one(&pool).await.unwrap();
        assert_eq!(paid_count, 2);
    }

    #[tokio::test]
    async fn batch_update_payments_rolls_back_on_card_missing() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        for s in [a, b] { seed_schedule(&pool, s, 1, 1).await; }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_a: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(a).fetch_one(&pool).await.unwrap();
        let bill_b: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(b).fetch_one(&pool).await.unwrap();

        // 두 번째 entry 가 카드사 누락 → 전체 롤백 기대.
        let err = batch_update_payments_impl(
            &pool,
            &[
                PaymentInput {
                    bill_id: bill_a,
                    is_paid: true,
                    paid_date: Some("2026-05-15".to_string()),
                    payer_name: None,
                    payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
                PaymentInput {
                    bill_id: bill_b,
                    is_paid: true,
                    paid_date: Some("2026-05-16".to_string()),
                    payer_name: None,
                    payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
            ],
        )
        .await;
        assert!(err.is_err());

        let paid_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM payments")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(paid_count, 0, "전체 롤백 — A 입금도 반영 안 됨");
    }

    #[tokio::test]
    async fn get_billing_summary_computes_totals() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        let c = seed_student(&pool, "3", "원생C", "2026-01-01", None).await;
        for s in [a, b, c] { seed_schedule(&pool, s, 1, 1).await; }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        // A 만 입금
        let bill_a: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(a).fetch_one(&pool).await.unwrap();
        create_payment_impl(
            &pool,
            &PaymentInput {
                bill_id: bill_a, is_paid: true,
                paid_date: Some("2026-05-15".to_string()),
                payer_name: None, payment_method_id: Some(CASH_PAYMENT_METHOD_ID), card_company_id: None,
            },
        )
        .await
        .expect("pay");

        let s = get_billing_summary_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(s.total_billable_students, 3, "수업 받은 원생 3명");
        assert_eq!(s.bill_count, 3);
        assert_eq!(s.total_billed, 300_000);
        assert_eq!(s.total_paid, 100_000);
        assert_eq!(s.total_unpaid, 200_000);
        assert_eq!(s.paid_count, 1);
        assert_eq!(s.unpaid_count, 2);
    }

    #[tokio::test]
    async fn list_billed_months_returns_distinct_desc() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, s, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        seed_period(&pool, "2026-03", "2026-03-01", "2026-03-31").await;
        generate_bills_impl(&pool, "2026-03").await.expect("gen3");
        generate_bills_impl(&pool, "2026-05").await.expect("gen5");

        let months = list_billed_months_impl(&pool).await.expect("list");
        assert_eq!(months, vec!["2026-05".to_string(), "2026-03".to_string()], "내림차순 distinct");
    }

    #[tokio::test]
    async fn billing_period_stats_groups_by_method() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-01-01", None).await;
        let c = seed_student(&pool, "3", "원생C", "2026-01-01", None).await;
        for s in [a, b, c] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bill_of = |sid: i64| {
            let pool = pool.clone();
            async move {
                sqlx::query_scalar::<_, i64>("SELECT id FROM bills WHERE student_id=?")
                    .bind(sid)
                    .fetch_one(&pool)
                    .await
                    .unwrap()
            }
        };
        // A, B → 현금 / C → 카드 / 미수납 없음 가정
        batch_update_payments_impl(
            &pool,
            &[
                PaymentInput {
                    bill_id: bill_of(a).await,
                    is_paid: true,
                    paid_date: Some("2026-05-10".to_string()),
                    payer_name: None,
                    payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
                PaymentInput {
                    bill_id: bill_of(b).await,
                    is_paid: true,
                    paid_date: Some("2026-05-11".to_string()),
                    payer_name: None,
                    payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
                PaymentInput {
                    bill_id: bill_of(c).await,
                    is_paid: true,
                    paid_date: Some("2026-05-12".to_string()),
                    payer_name: None,
                    payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                    card_company_id: Some(1),
                },
            ],
        )
        .await
        .expect("pay");

        // 월 집계 — 현금 1그룹(2명) + 카드 1그룹(1명)
        let month = get_billing_period_stats_impl(&pool, "2026-05")
            .await
            .expect("month stats");
        assert_eq!(month.bill_count, 3);
        assert_eq!(month.paid_count, 3);
        assert_eq!(month.total_paid, 300_000);
        let cash = month
            .by_method
            .iter()
            .find(|r| r.payment_method_id == Some(CASH_PAYMENT_METHOD_ID))
            .expect("현금 그룹");
        assert_eq!(cash.paid_count, 2);
        assert_eq!(cash.total_paid, 200_000);
        let card = month
            .by_method
            .iter()
            .find(|r| r.payment_method_id == Some(CARD_PAYMENT_METHOD_ID))
            .expect("카드 그룹");
        assert_eq!(card.paid_count, 1);
        assert_eq!(card.total_paid, 100_000);

        // 연도 집계 — 2026 전체. 본 테스트 데이터는 모두 2026-05 이므로 월 집계와 동일.
        let year = get_billing_period_stats_impl(&pool, "2026")
            .await
            .expect("year stats");
        assert_eq!(year.bill_count, 3, "연도 집계는 'YYYY-%' LIKE 매칭");
        assert_eq!(year.total_paid, 300_000);
    }

    /// hotfix post-Sprint 11: 수업 받은 원생 수 > 청구 건수 시 "추가 청구 데이터 생성" UX 트리거.
    #[tokio::test]
    async fn summary_total_billable_excludes_no_schedule_and_out_of_month() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 청구 대상 2명 (스케줄 있음)
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "원생B", "2026-05-15", None).await; // 월중입교
        for s in [a, b] { seed_schedule(&pool, s, 1, 1).await; }
        // 스케줄 없음 → 제외
        seed_student(&pool, "3", "원생C", "2026-01-01", None).await;
        // 4월말 퇴교 → 5월 청구 대상 아님
        seed_student(&pool, "4", "원생D", "2026-01-01", Some("2026-04-30")).await;

        seed_standard_fee(&pool, 1, 100_000).await;
        // 1명만 청구 생성 (A) — B 는 청구 미생성
        sqlx::query(
            "INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount, status) \
             VALUES (?, '2026-05', 1, 100000, 100000, 'draft')",
        )
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();

        let s = get_billing_summary_impl(&pool, "2026-05").await.expect("ok");
        assert_eq!(s.total_billable_students, 2, "A + B (스케줄 있고 5월 재원)");
        assert_eq!(s.bill_count, 1, "A 만 청구 생성됨");
        // → UI 는 "추가 청구 데이터 생성" 버튼 표시 (2 > 1)
    }

    // ─── P2-10 (2026-06 코드리뷰): 집계/검색 IPC 테스트 보강 ───

    /// list_payment_view_impl — 미수납 행 우선 정렬 + 결제수단 라벨 JOIN 검증.
    #[tokio::test]
    async fn payment_view_orders_unpaid_first_and_joins_labels() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "가나다", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "라마바", "2026-01-01", None).await;
        for s in [a, b] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");

        // A 만 현금 수납 완료 → B(미수납)가 먼저 나와야 한다.
        let bill_a: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id: bill_a,
                is_paid: true,
                paid_date: Some("2026-05-10".to_string()),
                payer_name: Some("가부모".to_string()),
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            }],
        )
        .await
        .expect("pay");

        let rows = list_payment_view_impl(&pool, "2026-05").await.expect("view");
        assert_eq!(rows.len(), 2);
        assert!(!rows[0].is_paid, "미수납 행이 먼저");
        assert_eq!(rows[0].student_id, b);
        let paid = rows.iter().find(|r| r.is_paid).expect("수납 행");
        assert_eq!(paid.student_id, a);
        assert_eq!(paid.payer_name.as_deref(), Some("가부모"));
        assert!(
            paid.payment_method_label.is_some(),
            "결제수단 라벨 JOIN 채워짐"
        );
    }

    #[tokio::test]
    async fn payment_view_orders_by_grade_then_name_within_same_group() {
        // 사용자 요청 — 수납관리(list_payment_view) 기본 정렬에도 학년별+이름 적용.
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "학생가", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "학생나", "2026-01-01", None).await;
        for s in [a, b] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;
        // a 를 중학생으로 승격 — elementary 인 b 가 먼저 나와야 한다(이름순 '가'<'나' 와 반대).
        sqlx::query("UPDATE students SET school_level='middle', grade=2 WHERE id=?")
            .bind(a)
            .execute(&pool)
            .await
            .expect("학생 학교급 갱신");

        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let rows = list_payment_view_impl(&pool, "2026-05").await.expect("view");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].student_id, b, "elementary 가 middle 보다 먼저");
        assert_eq!(rows[1].student_id, a);
    }

    #[tokio::test]
    async fn list_unpaid_bills_orders_by_grade_then_name() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "학생다", "2026-01-01", None).await;
        let b = seed_student(&pool, "2", "학생라", "2026-01-01", None).await;
        for s in [a, b] {
            seed_schedule(&pool, s, 1, 1).await;
        }
        seed_standard_fee(&pool, 1, 100_000).await;
        sqlx::query("UPDATE students SET school_level='middle', grade=1 WHERE id=?")
            .bind(a)
            .execute(&pool)
            .await
            .expect("학생 학교급 갱신");

        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let unpaid = list_unpaid_bills_impl(&pool, "2026-05").await.expect("list");
        assert_eq!(unpaid.len(), 2);
        assert_eq!(unpaid[0].student_id, b, "elementary 가 middle 보다 먼저");
        assert_eq!(unpaid[1].student_id, a);
    }

    /// search_students_for_billing_impl — 이름 매칭 + 최근 수납 정보(ROW_NUMBER) 자동 채움.
    #[tokio::test]
    async fn search_by_name_fills_latest_payment_info() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "홍길동", "2026-01-01", None).await;
        seed_schedule(&pool, a, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        seed_period(&pool, "2026-04", "2026-04-01", "2026-04-30").await;
        // 두 달치 청구 생성 후 각각 다른 시점에 수납 — 최근(5월) 정보가 채워져야 한다.
        generate_bills_impl(&pool, "2026-04").await.expect("gen4");
        generate_bills_impl(&pool, "2026-05").await.expect("gen5");
        let bill_apr: i64 =
            sqlx::query_scalar("SELECT id FROM bills WHERE student_id=? AND bill_year_month='2026-04'")
                .bind(a)
                .fetch_one(&pool)
                .await
                .unwrap();
        let bill_may: i64 =
            sqlx::query_scalar("SELECT id FROM bills WHERE student_id=? AND bill_year_month='2026-05'")
                .bind(a)
                .fetch_one(&pool)
                .await
                .unwrap();
        batch_update_payments_impl(
            &pool,
            &[
                PaymentInput {
                    bill_id: bill_apr,
                    is_paid: true,
                    paid_date: Some("2026-04-10".to_string()),
                    payer_name: Some("구입금자".to_string()),
                    payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                    card_company_id: None,
                },
                PaymentInput {
                    bill_id: bill_may,
                    is_paid: true,
                    paid_date: Some("2026-05-10".to_string()),
                    payer_name: Some("신입금자".to_string()),
                    payment_method_id: Some(CARD_PAYMENT_METHOD_ID),
                    card_company_id: Some(1),
                },
            ],
        )
        .await
        .expect("pay");

        let results = search_students_for_billing_impl(&pool, "홍길동")
            .await
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].student_id, a);
        assert_eq!(
            results[0].latest_payer_name.as_deref(),
            Some("신입금자"),
            "가장 최근(5월) 수납 정보가 채워져야 함 (ROW_NUMBER rn=1)"
        );
        assert_eq!(results[0].latest_payment_method_id, Some(CARD_PAYMENT_METHOD_ID));
    }

    /// search_students_for_billing_impl — 입금자 이름으로 검색 시 그 입금자가 낸 원생 반환.
    #[tokio::test]
    async fn search_by_payer_name_returns_payer_students() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let a = seed_student(&pool, "1", "원생A", "2026-01-01", None).await;
        seed_schedule(&pool, a, 1, 1).await;
        seed_standard_fee(&pool, 1, 100_000).await;
        generate_bills_impl(&pool, "2026-05").await.expect("gen");
        let bid: i64 = sqlx::query_scalar("SELECT id FROM bills WHERE student_id=?")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
        batch_update_payments_impl(
            &pool,
            &[PaymentInput {
                bill_id: bid,
                is_paid: true,
                paid_date: Some("2026-05-10".to_string()),
                payer_name: Some("김보호자".to_string()),
                payment_method_id: Some(CASH_PAYMENT_METHOD_ID),
                card_company_id: None,
            }],
        )
        .await
        .expect("pay");

        let results = search_students_for_billing_impl(&pool, "김보호자")
            .await
            .expect("search");
        assert_eq!(results.len(), 1, "입금자명으로 원생 매칭");
        assert_eq!(results[0].student_id, a);

        // 빈 쿼리는 빈 결과.
        let empty = search_students_for_billing_impl(&pool, "  ").await.expect("empty");
        assert!(empty.is_empty());
    }
}
