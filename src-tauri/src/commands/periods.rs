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

/// 정규/보강 출결의 year_month 를 event_date 가 속한 확정 교습기간으로 재동기화한다 (Sprint 24 수동검증 이슈).
///
/// V313 마이그레이션과 **동일 로직의 런타임 판(idempotent)**. V313 은 1회성이라, 그 이후 사용자가
/// 교습기간 경계를 (재)설정/확정하면 기존 출결의 year_month 가 새 경계와 어긋나(예: 7/30 이 7월
/// 기간→8월 기간으로 이동) 그리드에서 누락된다. 교습기간 변경 직후 + 앱 시작 시 호출해 자가 치유한다.
/// 확정 교습기간에 속하지 않는 날짜(서브쿼리 NULL)는 건드리지 않는다. 반환: 교정된 총 행 수.
/// (V313 SQL 을 바꾸면 이 함수도 함께 동기화할 것.)
pub(crate) async fn reconcile_attendance_year_month(pool: &SqlitePool) -> Result<u64, String> {
    let reg = sqlx::query(
        "UPDATE regular_attendances SET year_month = ( \
             SELECT sp.year_month FROM study_periods sp \
             WHERE sp.start_date <= regular_attendances.event_date \
               AND sp.end_date >= regular_attendances.event_date AND sp.is_confirmed = 1 \
         ) \
         WHERE year_month != ( \
             SELECT sp.year_month FROM study_periods sp \
             WHERE sp.start_date <= regular_attendances.event_date \
               AND sp.end_date >= regular_attendances.event_date AND sp.is_confirmed = 1 \
         )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("정규 출결 year_month 재동기화 실패: {}", e))?;

    let mk = sqlx::query(
        "UPDATE makeup_attendances SET year_month = ( \
             SELECT sp.year_month FROM study_periods sp \
             WHERE sp.start_date <= makeup_attendances.event_date \
               AND sp.end_date >= makeup_attendances.event_date AND sp.is_confirmed = 1 \
         ) \
         WHERE year_month != ( \
             SELECT sp.year_month FROM study_periods sp \
             WHERE sp.start_date <= makeup_attendances.event_date \
               AND sp.end_date >= makeup_attendances.event_date AND sp.is_confirmed = 1 \
         )",
    )
    .execute(pool)
    .await
    .map_err(|e| format!("보강 출결 year_month 재동기화 실패: {}", e))?;

    Ok(reg.rows_affected() + mk.rows_affected())
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
    async fn reconcile_retags_attendance_after_period_boundary_change() {
        let pool = test_pool_in_memory().await.unwrap();
        // 원생 1명 + 8월 교습기간(7/30~9/2, 확정).
        sqlx::query("INSERT INTO students (id, serial_no, name, gender, school_level, grade, enroll_date) VALUES (1,'1','가','male','elementary',3,'2026-04-01')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) VALUES ('2026-08','2026-07-30','2026-09-02',1)")
            .execute(&pool).await.unwrap();
        // 경계일 7/30 출결이 과거 경계 기준 '2026-07'로 오태깅된 상태(교습기간 재설정 후 방치 상황 재현).
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (1,'2026-07-30','2026-07','present',60)")
            .execute(&pool).await.unwrap();
        // 보강도 동일.
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (1,'2026-09-01','2026-09','makeup_attended',60)")
            .execute(&pool).await.unwrap();

        let changed = reconcile_attendance_year_month(&pool).await.unwrap();
        assert_eq!(changed, 2, "정규+보강 각 1건 재동기화");
        let reg: String = sqlx::query_scalar("SELECT year_month FROM regular_attendances WHERE event_date='2026-07-30'").fetch_one(&pool).await.unwrap();
        assert_eq!(reg, "2026-08", "7/30 은 8월 교습기간으로 재태깅");
        let mk: String = sqlx::query_scalar("SELECT year_month FROM makeup_attendances WHERE event_date='2026-09-01'").fetch_one(&pool).await.unwrap();
        assert_eq!(mk, "2026-08", "9/1 보강도 8월로 재태깅");
        // 멱등: 재실행 시 0건.
        assert_eq!(reconcile_attendance_year_month(&pool).await.unwrap(), 0, "멱등");
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
