//! 표준 교습비 IPC (Sprint 2 T11, PRD §4.9, data-model §5.1).
//!
//! ## 인터페이스
//!
//! - [`list_fees`] — 전체 표준 교습비 목록 (`sort_order` 오름차순, 활성 + 비활성 모두)
//! - [`create_fee`] — 주 수업시간 구간 + 교습비 등록
//! - [`update_fee`] — 금액·정렬·활성 상태 수정
//! - [`match_fee_by_hours`] — 주 수업시간 → 가장 가까운 표준 교습비 매칭
//!
//! ## 매칭 정책
//!
//! `match_fee_by_hours(hours)`:
//! 1. `weekly_hours = hours` 정확 일치 + `is_active = 1` → 그 행 반환
//! 2. 일치 없으면 `is_active = 1` 행 중 `weekly_hours <= hours` 의 최댓값 → 그 행 반환
//!    (예: 4.5시간 → 4시간 행)
//! 3. 그것도 없으면 `None`

use crate::commands::db;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// 표준 교습비 — IPC 응답.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct StandardFee {
    pub id: i64,
    pub weekly_hours: i64,
    pub amount: i64,
    pub sort_order: i64,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl StandardFee {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        let is_active: i64 = row.try_get("is_active")?;
        Ok(Self {
            id: row.try_get("id")?,
            weekly_hours: row.try_get("weekly_hours")?,
            amount: row.try_get("amount")?,
            sort_order: row.try_get("sort_order")?,
            is_active: is_active != 0,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

/// 신규 등록 payload.
#[derive(Debug, Deserialize)]
pub struct NewFee {
    pub weekly_hours: i64,
    pub amount: i64,
    pub sort_order: Option<i64>,
}

/// 수정 payload — 전체 필드 PUT-like.
#[derive(Debug, Deserialize)]
pub struct FeeUpdate {
    pub weekly_hours: i64,
    pub amount: i64,
    pub sort_order: i64,
    pub is_active: bool,
}

fn map_weekly_hours_unique(hours: i64, err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        let msg = db_err.message();
        if msg.contains("UNIQUE") && msg.contains("weekly_hours") {
            return AppError::UserFacing(format!(
                "주 {}시간 교습비는 이미 등록되어 있습니다. 기존 항목을 수정해 주세요.",
                hours
            ));
        }
    }
    AppError::Db(err)
}

#[tauri::command]
pub async fn list_fees() -> Result<Vec<StandardFee>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(
        "SELECT id, weekly_hours, amount, sort_order, is_active, created_at, updated_at \
         FROM standard_fees \
         ORDER BY sort_order ASC, weekly_hours ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    rows.iter()
        .map(StandardFee::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

#[tauri::command]
pub async fn create_fee(payload: NewFee) -> Result<StandardFee, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "INSERT INTO standard_fees (weekly_hours, amount, sort_order) \
         VALUES (?, ?, COALESCE(?, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM standard_fees))) \
         RETURNING id, weekly_hours, amount, sort_order, is_active, created_at, updated_at",
    )
    .bind(payload.weekly_hours)
    .bind(payload.amount)
    .bind(payload.sort_order)
    .fetch_one(pool)
    .await
    .map_err(|e| map_weekly_hours_unique(payload.weekly_hours, e))
    .map_err(String::from)?;
    StandardFee::from_row(&row).map_err(String::from)
}

#[tauri::command]
pub async fn update_fee(id: i64, payload: FeeUpdate) -> Result<StandardFee, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "UPDATE standard_fees SET \
            weekly_hours = ?, amount = ?, sort_order = ?, is_active = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ? \
         RETURNING id, weekly_hours, amount, sort_order, is_active, created_at, updated_at",
    )
    .bind(payload.weekly_hours)
    .bind(payload.amount)
    .bind(payload.sort_order)
    .bind(if payload.is_active { 1 } else { 0 })
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| map_weekly_hours_unique(payload.weekly_hours, e))
    .map_err(String::from)?;
    let row = row.ok_or_else(|| {
        String::from(AppError::UserFacing(format!(
            "교습비 항목을 찾을 수 없습니다 (id={}).",
            id
        )))
    })?;
    StandardFee::from_row(&row).map_err(String::from)
}

/// 주 수업시간 → 매칭 교습비 (정확 일치 우선, 없으면 이하 최댓값).
#[tauri::command]
pub async fn match_fee_by_hours(weekly_hours: i64) -> Result<Option<StandardFee>, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(
        "SELECT id, weekly_hours, amount, sort_order, is_active, created_at, updated_at \
         FROM standard_fees \
         WHERE is_active = 1 AND weekly_hours <= ? \
         ORDER BY weekly_hours DESC LIMIT 1",
    )
    .bind(weekly_hours)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)
    .map_err(String::from)?;
    row.as_ref()
        .map(StandardFee::from_row)
        .transpose()
        .map_err(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn match_fee_returns_exact_match() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let row = sqlx::query(
            "SELECT id, weekly_hours, amount, sort_order, is_active, created_at, updated_at \
             FROM standard_fees WHERE is_active = 1 AND weekly_hours <= 4 \
             ORDER BY weekly_hours DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let fee = StandardFee::from_row(&row).unwrap();
        assert_eq!(fee.weekly_hours, 4);
        assert_eq!(fee.amount, 250000);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn match_fee_returns_floor_for_non_exact() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 7시간 입력 — 최대 6시간 행 반환 (이하 최댓값)
        let row = sqlx::query(
            "SELECT weekly_hours, amount FROM standard_fees \
             WHERE is_active = 1 AND weekly_hours <= 7 \
             ORDER BY weekly_hours DESC LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let hours: i64 = row.try_get("weekly_hours").unwrap();
        assert_eq!(hours, 6, "7시간 → 6시간 행 매칭 (이하 최댓값)");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn match_fee_returns_none_when_below_smallest() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 1시간 (시드 최소 2시간 미만) — 매칭 없음
        let row = sqlx::query(
            "SELECT 1 FROM standard_fees \
             WHERE is_active = 1 AND weekly_hours <= 1 LIMIT 1",
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
        assert!(row.is_none(), "1시간 입력 시 매칭 없음 (시드 최소 2시간)");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn weekly_hours_unique_violation_returns_korean() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // 시드에 2시간 이미 존재 — 동일 weekly_hours INSERT 시 UNIQUE 위반
        let err = sqlx::query(
            "INSERT INTO standard_fees (weekly_hours, amount) VALUES (2, 999999)",
        )
        .execute(&pool)
        .await
        .expect_err("UNIQUE 위반");
        let mapped = map_weekly_hours_unique(2, err);
        let msg: String = mapped.into();
        assert!(msg.contains("주 2시간 교습비"), "msg={}", msg);
        assert!(msg.contains("이미 등록"));
    }
}
