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
use crate::commands::pagination::clamp_list_limit;
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

    /// 페이지네이션 적용 list SQL — `LIMIT ? OFFSET ?` 가 ORDER BY 뒤에 자동 부착.
    fn list_sql(self) -> &'static str {
        match self {
            Self::Schools => {
                // Sprint 19 T9: school_type 을 extra 로 노출 — 원생 폼의 학교급 필터링에 사용.
                "SELECT id, name AS code, name AS label, school_type AS extra, sort_order, is_active \
                 FROM schools ORDER BY sort_order ASC, name ASC LIMIT ? OFFSET ?"
            }
            Self::PaymentMethods => {
                "SELECT id, code, label, display_order AS sort_order, is_active \
                 FROM payment_methods ORDER BY display_order ASC, label ASC LIMIT ? OFFSET ?"
            }
            Self::CardCompanies => {
                "SELECT id, code, label, display_order AS sort_order, is_active \
                 FROM card_companies ORDER BY display_order ASC, label ASC LIMIT ? OFFSET ?"
            }
        }
    }

    /// 동일 필터(테이블)에 대한 COUNT 쿼리.
    fn count_sql(self) -> &'static str {
        match self {
            Self::Schools => "SELECT COUNT(*) AS cnt FROM schools",
            Self::PaymentMethods => "SELECT COUNT(*) AS cnt FROM payment_methods",
            Self::CardCompanies => "SELECT COUNT(*) AS cnt FROM card_companies",
        }
    }

    fn insert_sql(self) -> &'static str {
        match self {
            Self::Schools => {
                "INSERT INTO schools (name, school_type, sort_order, is_active) \
                 VALUES (?, COALESCE(?, 'etc'), ?, 1) \
                 RETURNING id, name AS code, name AS label, school_type AS extra, sort_order, is_active"
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
                // Sprint 19 T9: school_type 도 수정 가능 — NULL(미지정) 이면 기존 값 유지.
                "UPDATE schools SET name = ?, school_type = COALESCE(?, school_type), \
                    sort_order = ?, is_active = ?, \
                    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                 WHERE id = ? \
                 RETURNING id, name AS code, name AS label, school_type AS extra, sort_order, is_active"
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
    /// Sprint 19 T9: schools 는 school_type. payment_methods/card_companies 는
    /// 해당 컬럼이 SELECT 절에 없어 항상 None.
    pub extra: Option<String>,
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
            extra: row.try_get("extra").ok(),
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
/// - schools: `label` 이 학교명으로 사용됨 (code 변경 없음 — schools.name 이 UNIQUE PK 역할).
///   `extra` = school_type 변경값(Sprint 19 T9), None 이면 기존 값 유지
/// - payment_methods / card_companies: code 는 변경 불가 (UNIQUE), label/sort_order/is_active 만 수정 (`extra` 무시)
#[derive(Debug, Deserialize)]
pub struct CodeUpdate {
    pub label: String,
    pub extra: Option<String>,
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

/// R14 페이지네이션 — 정책은 [`clamp_list_limit`] 참조.
#[tauri::command]
pub async fn list_codes(
    table: CodeTable,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<CodeEntry>, String> {
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
    let limit = clamp_list_limit(limit);
    let offset = offset.unwrap_or(0);
    let rows = sqlx::query(table.list_sql())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    rows.iter()
        .map(CodeEntry::from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(String::from)
}

/// 코드 테이블의 총 항목 수 (R14 페이지네이션 UI 보조).
#[tauri::command]
pub async fn count_codes(table: CodeTable) -> Result<i64, String> {
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
    let row = sqlx::query(table.count_sql())
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;
    row.try_get::<i64, _>("cnt")
        .map_err(AppError::Db)
        .map_err(String::from)
}

#[tauri::command]
pub async fn create_code(table: CodeTable, payload: NewCode) -> Result<CodeEntry, String> {
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
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
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
    // bind 순서는 각 테이블 update_sql() 의 `?` 순서와 대응: schools 만 `extra`(school_type)
    // 파라미터가 label 다음에 하나 더 끼어든다 — label, [extra], sort_order, is_active, id.
    let mut q = sqlx::query(table.update_sql()).bind(&payload.label);
    if table == CodeTable::Schools {
        q = q.bind(payload.extra.as_deref());
    }
    q = q
        .bind(payload.sort_order)
        .bind(if payload.is_active { 1 } else { 0 })
        .bind(id);
    let row = q
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
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
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

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::pagination::{DEFAULT_LIST_LIMIT, MAX_LIST_LIMIT};

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
            .bind(DEFAULT_LIST_LIMIT)
            .bind(0u32)
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
            .bind(DEFAULT_LIST_LIMIT)
            .bind(0u32)
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
            .bind(DEFAULT_LIST_LIMIT)
            .bind(0u32)
            .fetch_all(&pool)
            .await
            .unwrap();
        let entry: CodeEntry = CodeEntry::from_row(rows.last().unwrap()).unwrap();
        assert_eq!(entry.code, "테스트초");
        assert_eq!(entry.sort_order, 5);
        assert!(entry.is_active);
    }

    // ------------------------------------------------------------------------
    // Sprint 19 T9 — school_type(extra) 조회/생성/수정
    // ------------------------------------------------------------------------

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn list_schools_exposes_school_type_as_extra() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        sqlx::query(
            "INSERT INTO schools (name, school_type, sort_order, is_active) \
             VALUES ('테스트중', 'middle', 1, 1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        let rows = sqlx::query(CodeTable::Schools.list_sql())
            .bind(DEFAULT_LIST_LIMIT)
            .bind(0u32)
            .fetch_all(&pool)
            .await
            .unwrap();
        let entry = CodeEntry::from_row(rows.last().unwrap()).unwrap();
        assert_eq!(entry.extra.as_deref(), Some("middle"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn create_school_stores_school_type_from_extra() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let row = sqlx::query(CodeTable::Schools.insert_sql())
            .bind("신규초등학교")
            .bind(Some("elementary"))
            .bind(1i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        let entry = CodeEntry::from_row(&row).unwrap();
        assert_eq!(entry.extra.as_deref(), Some("elementary"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn update_school_changes_school_type() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let created = sqlx::query(CodeTable::Schools.insert_sql())
            .bind("이전학교")
            .bind(Some("etc"))
            .bind(1i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        let id: i64 = created.try_get("id").unwrap();

        let updated = sqlx::query(CodeTable::Schools.update_sql())
            .bind("이전학교")
            .bind(Some("middle"))
            .bind(2i64)
            .bind(1i64)
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let entry = CodeEntry::from_row(&updated).unwrap();
        assert_eq!(entry.extra.as_deref(), Some("middle"));
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn update_school_keeps_school_type_when_extra_is_none() {
        // COALESCE(?, school_type) — extra 미지정(None) 시 기존 값 유지.
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let created = sqlx::query(CodeTable::Schools.insert_sql())
            .bind("유지학교")
            .bind(Some("elementary"))
            .bind(1i64)
            .fetch_one(&pool)
            .await
            .unwrap();
        let id: i64 = created.try_get("id").unwrap();

        let updated = sqlx::query(CodeTable::Schools.update_sql())
            .bind("유지학교")
            .bind(None::<String>)
            .bind(3i64)
            .bind(1i64)
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let entry = CodeEntry::from_row(&updated).unwrap();
        assert_eq!(entry.extra.as_deref(), Some("elementary"), "extra=None 이면 기존 유지");
    }

    // ------------------------------------------------------------------------
    // R14 페이지네이션
    // ------------------------------------------------------------------------

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn list_codes_limit_offset_respected() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        // V001 시드: card_companies 10건 이상. 첫 페이지 3건 vs 두 번째 페이지 3건이 disjoint.
        let page1 = sqlx::query(CodeTable::CardCompanies.list_sql())
            .bind(3u32)
            .bind(0u32)
            .fetch_all(&pool)
            .await
            .unwrap();
        let page2 = sqlx::query(CodeTable::CardCompanies.list_sql())
            .bind(3u32)
            .bind(3u32)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page2.len(), 3);
        let p1_codes: Vec<String> = page1
            .iter()
            .map(|r| CodeEntry::from_row(r).unwrap().code)
            .collect();
        let p2_codes: Vec<String> = page2
            .iter()
            .map(|r| CodeEntry::from_row(r).unwrap().code)
            .collect();
        for c in &p1_codes {
            assert!(!p2_codes.contains(c), "페이지 1·2 disjoint 보장: {}", c);
        }
    }

    #[cfg(not(feature = "cipher"))]
    #[tokio::test]
    async fn count_codes_matches_seed_total() {
        let pool = db::test_pool_in_memory().await.expect("인메모리 pool");
        let total: (i64,) = sqlx::query_as(CodeTable::CardCompanies.count_sql())
            .fetch_one(&pool)
            .await
            .unwrap();
        let rows = sqlx::query(CodeTable::CardCompanies.list_sql())
            .bind(MAX_LIST_LIMIT)
            .bind(0u32)
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(
            total.0 as usize,
            rows.len(),
            "COUNT(*) 가 실제 행 수와 일치"
        );
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
