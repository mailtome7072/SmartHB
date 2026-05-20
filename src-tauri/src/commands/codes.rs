//! 코드 테이블 CRUD IPC (Sprint 2 T12, PRD §4.1·§4.9, data-model §1.3·§5.4).
//!
//! ## 대상 테이블 (3종)
//!
//! - `schools` — 학교명 (원생 등록 시 자동완성용)
//! - `payment_methods` — 결제 수단 (현금/카드/계좌이체 등)
//! - `card_companies` — 카드사 (신한/국민/삼성 등)
//!
//! 세 테이블은 컬럼 구조가 미세히 다르므로 (`schools.name` vs `payment_methods.code+label`)
//! 통합 응답 타입 `CodeEntry` 로 정규화하여 단일 IPC 패턴 (`list_codes(table)`) 으로 노출한다.
//! 내부 SQL 은 테이블별로 컬럼 매핑.
//!
//! ## 매핑
//!
//! | 응답 필드 | schools | payment_methods | card_companies |
//! |-----------|---------|------------------|----------------|
//! | `code`    | name    | code             | code           |
//! | `label`   | name    | label            | label          |
//! | `sort_order` | sort_order | display_order | display_order |
//! | `is_active`  | is_active  | is_active     | is_active     |
//!
//! schools 는 단일 `name` 컬럼이라 `code` = `label` = `name`. UI 가 동일 텍스트를 보여줘도 무방.

use crate::commands::db;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// 코드 테이블 식별자.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CodeTable {
    Schools,
    PaymentMethods,
    CardCompanies,
}

impl CodeTable {
    fn table_name(self) -> &'static str {
        match self {
            Self::Schools => "schools",
            Self::PaymentMethods => "payment_methods",
            Self::CardCompanies => "card_companies",
        }
    }

    fn list_sql(self) -> &'static str {
        match self {
            Self::Schools => {
                "SELECT id, name AS code, name AS label, sort_order, is_active \
                 FROM schools ORDER BY sort_order ASC, name ASC"
            }
            Self::PaymentMethods => {
                "SELECT id, code, label, display_order AS sort_order, is_active \
                 FROM payment_methods ORDER BY display_order ASC, label ASC"
            }
            Self::CardCompanies => {
                "SELECT id, code, label, display_order AS sort_order, is_active \
                 FROM card_companies ORDER BY display_order ASC, label ASC"
            }
        }
    }

    fn insert_sql(self) -> &'static str {
        match self {
            Self::Schools => {
                "INSERT INTO schools (name, school_type, sort_order, is_active) \
                 VALUES (?, COALESCE(?, 'etc'), ?, 1) \
                 RETURNING id, name AS code, name AS label, sort_order, is_active"
            }
            Self::PaymentMethods => {
                "INSERT INTO payment_methods (code, label, display_order, is_active) \
                 VALUES (?, ?, ?, 1) \
                 RETURNING id, code, label, display_order AS sort_order, is_active"
            }
            Self::CardCompanies => {
                "INSERT INTO card_companies (code, label, display_order, is_active) \
                 VALUES (?, ?, ?, 1) \
                 RETURNING id, code, label, display_order AS sort_order, is_active"
            }
        }
    }

    fn update_sql(self) -> &'static str {
        match self {
            Self::Schools => {
                "UPDATE schools SET name = ?, sort_order = ?, is_active = ?, \
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                 WHERE id = ? \
                 RETURNING id, name AS code, name AS label, sort_order, is_active"
            }
            Self::PaymentMethods => {
                "UPDATE payment_methods SET label = ?, display_order = ?, is_active = ? \
                 WHERE id = ? \
                 RETURNING id, code, label, display_order AS sort_order, is_active"
            }
            Self::CardCompanies => {
                "UPDATE card_companies SET label = ?, display_order = ?, is_active = ? \
                 WHERE id = ? \
                 RETURNING id, code, label, display_order AS sort_order, is_active"
            }
        }
    }
}

/// 코드 항목 — 세 테이블 공통 응답.
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct CodeEntry {
    pub id: i64,
    pub code: String,
    pub label: String,
    pub sort_order: i64,
    pub is_active: bool,
}

impl CodeEntry {
    fn from_row(row: &SqliteRow) -> Result<Self, AppError> {
        let is_active: i64 = row.try_get("is_active")?;
        Ok(Self {
            id: row.try_get("id")?,
            code: row.try_get("code")?,
            label: row.try_get("label")?,
            sort_order: row.try_get("sort_order")?,
            is_active: is_active != 0,
        })
    }
}

/// 신규 등록 payload — 테이블에 따라 `code` 필드 의미가 다름.
///
/// - schools: `code` = 학교명 (label 미사용 — UI 가 동일 값 전달 권장), `extra` = school_type (예: 'elementary')
/// - payment_methods / card_companies: `code` 와 `label` 모두 사용 (예: code='cash', label='현금')
#[derive(Debug, Deserialize)]
pub struct NewCode {
    pub code: String,
    pub label: Option<String>,
    pub extra: Option<String>,
    pub sort_order: Option<i64>,
}

/// 수정 payload — PUT-like.
///
/// - schools: `label` 이 학교명으로 사용됨 (code 변경 없음 — schools.name 이 UNIQUE PK 역할)
/// - payment_methods / card_companies: code 는 변경 불가 (UNIQUE), label/sort_order/is_active 만 수정
#[derive(Debug, Deserialize)]
pub struct CodeUpdate {
    pub label: String,
    pub sort_order: i64,
    pub is_active: bool,
}

fn map_code_unique(code: &str, table: CodeTable, err: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &err {
        let msg = db_err.message();
        if msg.contains("UNIQUE") {
            let table_label = match table {
                CodeTable::Schools => "학교명",
                CodeTable::PaymentMethods => "결제 수단 코드",
                CodeTable::CardCompanies => "카드사 코드",
            };
            return AppError::UserFacing(format!(
                "{} '{}' 은(는) 이미 등록되어 있습니다.",
                table_label, code
            ));
        }
    }
    AppError::Db(err)
}

#[tauri::command]
pub async fn list_codes(table: CodeTable) -> Result<Vec<CodeEntry>, String> {
    let pool = db::pool().map_err(String::from)?;
    let rows = sqlx::query(table.list_sql())
        .fetch_all(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    rows.iter()
        .map(CodeEntry::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

#[tauri::command]
pub async fn create_code(table: CodeTable, payload: NewCode) -> Result<CodeEntry, String> {
    let pool = db::pool().map_err(String::from)?;
    let next_order = payload.sort_order.unwrap_or(0);

    let mut q = sqlx::query(table.insert_sql());
    q = match table {
        CodeTable::Schools => q
            .bind(&payload.code)
            .bind(payload.extra.as_deref())
            .bind(next_order),
        CodeTable::PaymentMethods | CodeTable::CardCompanies => {
            // label 미지정 시 code 와 동일 값으로 채움 (UI 가 보통 같이 보냄)
            let label = payload.label.as_deref().unwrap_or(payload.code.as_str());
            q.bind(&payload.code).bind(label).bind(next_order)
        }
    };

    let row = q
        .fetch_one(pool)
        .await
        .map_err(|e| map_code_unique(&payload.code, table, e))
        .map_err(String::from)?;
    CodeEntry::from_row(&row).map_err(String::from)
}

#[tauri::command]
pub async fn update_code(
    table: CodeTable,
    id: i64,
    payload: CodeUpdate,
) -> Result<CodeEntry, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query(table.update_sql())
        .bind(&payload.label)
        .bind(payload.sort_order)
        .bind(if payload.is_active { 1 } else { 0 })
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| map_code_unique(&payload.label, table, e))
        .map_err(String::from)?;
    let row = row.ok_or_else(|| {
        String::from(AppError::UserFacing(format!(
            "{} 항목을 찾을 수 없습니다 (id={}).",
            table.table_name(),
            id
        )))
    })?;
    CodeEntry::from_row(&row).map_err(String::from)
}

/// 코드 항목 정렬 순서 일괄 변경 — `(id, sort_order)` 쌍 배열로 받아 한 트랜잭션에 UPDATE.
#[tauri::command]
pub async fn reorder_codes(table: CodeTable, orders: Vec<(i64, i64)>) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let mut tx = pool.begin().await.map_err(AppError::Db).map_err(String::from)?;
    let column = match table {
        CodeTable::Schools => "sort_order",
        CodeTable::PaymentMethods | CodeTable::CardCompanies => "display_order",
    };
    let sql = format!(
        "UPDATE {} SET {} = ? WHERE id = ?",
        table.table_name(),
        column
    );
    for (id, order) in orders {
        sqlx::query(&sql)
            .bind(order)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Db)
            .map_err(String::from)?;
    }
    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_table_enum_round_trip() {
        for t in [
            CodeTable::Schools,
            CodeTable::PaymentMethods,
            CodeTable::CardCompanies,
        ] {
            let json = serde_json::to_string(&t).unwrap();
            let back: CodeTable = serde_json::from_str(&json).unwrap();
            assert_eq!(t, back);
        }
        assert_eq!(
            serde_json::to_string(&CodeTable::PaymentMethods).unwrap(),
            r#""payment-methods""#
        );
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn list_payment_methods_returns_v001_seed() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let rows = sqlx::query(CodeTable::PaymentMethods.list_sql())
            .fetch_all(&pool)
            .await
            .unwrap();
        let codes: Vec<CodeEntry> =
            rows.iter().map(|r| CodeEntry::from_row(r).unwrap()).collect();
        assert!(codes.len() >= 4, "V001 시드 4건 이상");
        assert!(codes.iter().any(|c| c.code == "cash" && c.label == "현금"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn list_card_companies_returns_v001_seed() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let rows = sqlx::query(CodeTable::CardCompanies.list_sql())
            .fetch_all(&pool)
            .await
            .unwrap();
        assert!(rows.len() >= 10, "V001 시드 10건 이상");
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn schools_supports_sort_order_and_is_active_after_v105() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO schools (name, school_type, sort_order, is_active) \
             VALUES ('테스트초', 'elementary', 5, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let rows = sqlx::query(CodeTable::Schools.list_sql())
            .fetch_all(&pool)
            .await
            .unwrap();
        let entry: CodeEntry = CodeEntry::from_row(rows.last().unwrap()).unwrap();
        assert_eq!(entry.code, "테스트초");
        assert_eq!(entry.sort_order, 5);
        assert!(entry.is_active);
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn duplicate_code_returns_korean_message() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let err = sqlx::query(
            "INSERT INTO payment_methods (code, label, display_order) VALUES ('cash', '현금2', 99)",
        )
        .execute(&pool)
        .await
        .expect_err("V001 시드 'cash' 와 UNIQUE 충돌");
        let mapped = map_code_unique("cash", CodeTable::PaymentMethods, err);
        let msg: String = mapped.into();
        assert!(msg.contains("결제 수단 코드"), "msg={}", msg);
        assert!(msg.contains("이미 등록"));
    }
}
