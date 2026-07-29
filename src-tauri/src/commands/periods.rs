//! 교습기간(study_periods) 공용 조회 헬퍼 (Sprint 24 리뷰 L1).
//!
//! "날짜 → 소속 확정 교습기간 year_month" 및 "year_month → 확정 교습기간 범위" 프리미티브를
//! 단일 소스로 보관한다. 이전에는 attendance.rs 에 위치해 dashboard/diagnosis/makeup 등이
//! attendance 모듈을 직접 참조하는 결합이 있었으나(리뷰 L1), 도메인 중립 모듈로 분리해 해소한다.
//! academic.rs 가 이미 attendance 를 참조하므로 academic 로 옮기면 양방향 결합이 생겨 별도 모듈로 둔다.
//! 교습기간 일자 중첩은 academic.rs IPC 레벨에서 금지되므로 날짜당 확정 교습기간은 최대 1개다.

use sqlx::{Row, SqlitePool};

/// 주어진 날짜가 속한 확정 교습기간의 year_month. 없으면 None.
///
/// 다월 교습기간에서 "달력월(date[..7])"이 아닌 실제 소속 교습기간 라벨을 얻기 위한 프리미티브.
pub(crate) async fn period_year_month_for_date(
    pool: &SqlitePool,
    date: &str,
) -> Result<Option<String>, String> {
    sqlx::query_scalar(
        "SELECT year_month FROM study_periods \
         WHERE start_date <= ? AND end_date >= ? AND is_confirmed = 1",
    )
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("교습기간 조회 실패: {}", e))
}

/// `year_month` 의 확정 교습기간 (start_date, end_date). 미설정/미확정이면 Err(사용자 친화 메시지).
pub(crate) async fn load_confirmed_period(
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

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    #[tokio::test]
    async fn period_year_month_for_date_maps_multimonth_boundary() {
        let pool = test_pool_in_memory().await.unwrap();
        sqlx::query("INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) VALUES ('2026-08','2026-07-30','2026-09-02',1)")
            .execute(&pool).await.unwrap();
        // 다월 경계일(전월/익월)도 소속 교습기간으로 매핑.
        assert_eq!(
            period_year_month_for_date(&pool, "2026-07-30").await.unwrap().as_deref(),
            Some("2026-08")
        );
        assert_eq!(
            period_year_month_for_date(&pool, "2026-09-02").await.unwrap().as_deref(),
            Some("2026-08")
        );
        // 교습기간 밖은 None.
        assert_eq!(period_year_month_for_date(&pool, "2026-09-03").await.unwrap(), None);
    }

    #[tokio::test]
    async fn load_confirmed_period_errors_when_missing_or_unconfirmed() {
        let pool = test_pool_in_memory().await.unwrap();
        sqlx::query("INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) VALUES ('2026-08','2026-07-30','2026-09-02',0)")
            .execute(&pool).await.unwrap();
        assert!(load_confirmed_period(&pool, "2026-08").await.is_err(), "미확정은 Err");
        assert!(load_confirmed_period(&pool, "2026-01").await.is_err(), "미설정은 Err");
    }
}
