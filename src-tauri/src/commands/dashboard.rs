//! 대시보드 집계 IPC (Sprint 14 T3, PRD §4.11).
//!
//! 6개 위젯 + 5종 알림의 백엔드 집계. 위젯별 전용 IPC 로 분리(R95) — 프론트(TanStack Query)가
//! 병렬 호출 + 캐싱한다.
//!
//! ## 설계
//! - 내부 함수는 `&SqlitePool` + (날짜 의존 시) `today: NaiveDate` 주입 → 인메모리 테스트 가능.
//! - IPC 커맨드는 전역 `db::pool()` + `chrono::Local::now()` 를 주입하는 얇은 래퍼.
//!
//! ## 정의(모호 항목 — 사용자 검증 후 조정 가능)
//! - **출결 진행률**: 당월 1일~오늘 중 현행 스케줄 요일에 해당하는 "수업일" 가운데 정규출결
//!   레코드가 없는 일자를 "미입력"으로 본다. (휴원일/방학 제외는 후속 — 현재 미반영)
//! - **분기**: 학사력(3·6·9·12월 시작) 기준 최근 4분기 입/퇴교 수.
//! - **보강 소멸 임박**: makeup_deadline(YYYY-MM)이 당월인 결석 건수. (월 단위 컬럼이라 'D-7'을
//!   월 단위로 근사 — 일 단위 정밀화는 후속)

use crate::commands::db::pool;
use crate::error::AppError;
use chrono::{Datelike, Months, NaiveDate};
use serde::Serialize;
use sqlx::{Row, SqlitePool};

const KEY_DASHBOARD_MEMO: &str = "dashboard_memo";

/// 라벨 + 개수 — 성별/학년/학교 분포 공통 표현.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct LabelCount {
    pub label: String,
    pub count: i64,
}

/// 분기별 입/퇴교 추이 1건.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct QuarterEnrollment {
    pub label: String,
    pub enrolled: i64,
    pub withdrawn: i64,
}

/// Feature 4.11.1 교습소 현황.
#[derive(Debug, Serialize)]
pub struct AcademyOverview {
    pub total_active: i64,
    pub by_gender: Vec<LabelCount>,
    pub by_grade: Vec<LabelCount>,
    pub by_school: Vec<LabelCount>,
    pub quarterly: Vec<QuarterEnrollment>,
}

/// 시간대별 수업 — 시작 시간 + 원생 명단.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct TodaySlot {
    pub start_time: String,
    pub students: Vec<String>,
}

/// Feature 4.11.2 당일 수업 정보.
#[derive(Debug, Serialize)]
pub struct TodaySchedule {
    /// ISO weekday 1=월~7=일.
    pub weekday: u8,
    pub slots: Vec<TodaySlot>,
}

/// Feature 4.11.3 월 핵심 요약.
#[derive(Debug, Serialize)]
pub struct MonthlySummary {
    pub year_month: String,
    pub bill_total: i64,
    pub paid_total: i64,
    pub unpaid_total: i64,
    pub bill_count: i64,
    pub paid_count: i64,
    pub enrolled_this_month: i64,
    pub withdrawn_this_month: i64,
    pub attendance_recorded_days: i64,
}

/// Feature 4.11.4 알림 1건.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct DashboardAlert {
    /// 알림 종류 식별자 (프론트 라우팅/아이콘 매핑).
    pub kind: String,
    /// 'red' | 'orange' | 'blue'.
    pub severity: String,
    pub message: String,
    pub count: i64,
}

/// Feature 4.11.5 출결 입력 진행률.
#[derive(Debug, Serialize)]
pub struct AttendanceProgress {
    pub year_month: String,
    pub expected_days: i64,
    pub recorded_days: i64,
    pub missing_dates: Vec<String>,
}

fn today_naive() -> NaiveDate {
    chrono::Local::now().date_naive()
}

fn year_month_of(date: NaiveDate) -> String {
    date.format("%Y-%m").to_string()
}

// ----------------------------------------------------------------------------
// 4.11.1 교습소 현황
// ----------------------------------------------------------------------------

/// 학사력 분기 시작일(해당 분기를 시작하는 3/6/9/12월 1일)을 반환.
fn quarter_start(date: NaiveDate) -> NaiveDate {
    let (y, m) = (date.year(), date.month());
    let (sy, sm) = match m {
        3..=5 => (y, 3),
        6..=8 => (y, 6),
        9..=11 => (y, 9),
        12 => (y, 12),
        _ => (y - 1, 12), // 1,2월 → 직전 해 12월 시작 분기
    };
    NaiveDate::from_ymd_opt(sy, sm, 1).expect("유효한 분기 시작일")
}

/// 분기 라벨 (예: "2026 1분기"). 시작월 3/6/9/12 → 1/2/3/4분기.
fn quarter_label(start: NaiveDate) -> String {
    let q = match start.month() {
        3 => 1,
        6 => 2,
        9 => 3,
        _ => 4, // 12
    };
    format!("{} {}분기", start.year(), q)
}

/// 최근 4분기 범위 (오래된 순) — (라벨, 시작일, 종료일).
fn recent_quarters(today: NaiveDate) -> Vec<(String, NaiveDate, NaiveDate)> {
    let current = quarter_start(today);
    let mut starts = Vec::new();
    for back in (0..4).rev() {
        let s = current
            .checked_sub_months(Months::new(3 * back))
            .expect("분기 시작 계산");
        starts.push(s);
    }
    starts
        .into_iter()
        .map(|s| {
            let end = s
                .checked_add_months(Months::new(3))
                .and_then(|d| d.pred_opt())
                .expect("분기 종료 계산");
            (quarter_label(s), s, end)
        })
        .collect()
}

async fn overview(pool: &SqlitePool, today: NaiveDate) -> Result<AcademyOverview, AppError> {
    let total_active: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM students WHERE withdraw_date IS NULL")
            .fetch_one(pool)
            .await
            .map_err(AppError::Db)?;

    // 성별 분포
    let gender_rows = sqlx::query(
        "SELECT gender, COUNT(*) AS c FROM students WHERE withdraw_date IS NULL GROUP BY gender",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;
    let by_gender = gender_rows
        .into_iter()
        .map(|r| {
            let g: String = r.get("gender");
            LabelCount {
                label: if g == "male" { "남".into() } else { "여".into() },
                count: r.get("c"),
            }
        })
        .collect();

    // 학년 분포 (school_level + grade)
    let grade_rows = sqlx::query(
        "SELECT school_level, grade, COUNT(*) AS c FROM students \
         WHERE withdraw_date IS NULL GROUP BY school_level, grade \
         ORDER BY school_level, grade",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;
    let by_grade = grade_rows
        .into_iter()
        .map(|r| {
            let level: String = r.get("school_level");
            let grade: i64 = r.get("grade");
            let prefix = if level == "elementary" { "초" } else { "중" };
            LabelCount {
                label: format!("{}{}", prefix, grade),
                count: r.get("c"),
            }
        })
        .collect();

    // 학교 분포 (미지정 포함)
    let school_rows = sqlx::query(
        "SELECT COALESCE(sc.name, '미지정') AS label, COUNT(*) AS c \
         FROM students s LEFT JOIN schools sc ON sc.id = s.school_id \
         WHERE s.withdraw_date IS NULL GROUP BY label ORDER BY c DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;
    let by_school = school_rows
        .into_iter()
        .map(|r| LabelCount {
            label: r.get("label"),
            count: r.get("c"),
        })
        .collect();

    // 분기별 입/퇴교 추이
    let mut quarterly = Vec::new();
    for (label, start, end) in recent_quarters(today) {
        let (s, e) = (start.to_string(), end.to_string());
        let enrolled: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM students WHERE enroll_date BETWEEN ? AND ?",
        )
        .bind(&s)
        .bind(&e)
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)?;
        let withdrawn: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM students WHERE withdraw_date BETWEEN ? AND ?",
        )
        .bind(&s)
        .bind(&e)
        .fetch_one(pool)
        .await
        .map_err(AppError::Db)?;
        quarterly.push(QuarterEnrollment {
            label,
            enrolled,
            withdrawn,
        });
    }

    Ok(AcademyOverview {
        total_active,
        by_gender,
        by_grade,
        by_school,
        quarterly,
    })
}

// ----------------------------------------------------------------------------
// 4.11.2 당일 수업
// ----------------------------------------------------------------------------

async fn today_schedule(pool: &SqlitePool, weekday: u8) -> Result<TodaySchedule, AppError> {
    let rows = sqlx::query(
        "SELECT ss.start_time AS start_time, s.name AS name \
         FROM student_schedules ss JOIN students s ON s.id = ss.student_id \
         WHERE ss.effective_to IS NULL AND ss.day_of_week = ? AND s.withdraw_date IS NULL \
         ORDER BY ss.start_time, s.name",
    )
    .bind(weekday as i64)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    // 시작 시간별 그룹화 (행이 start_time 정렬돼 있으므로 순차 그룹).
    let mut slots: Vec<TodaySlot> = Vec::new();
    for r in rows {
        let start_time: String = r.get("start_time");
        let name: String = r.get("name");
        match slots.last_mut() {
            Some(slot) if slot.start_time == start_time => slot.students.push(name),
            _ => slots.push(TodaySlot {
                start_time,
                students: vec![name],
            }),
        }
    }
    Ok(TodaySchedule { weekday, slots })
}

// ----------------------------------------------------------------------------
// 4.11.3 월 핵심 요약
// ----------------------------------------------------------------------------

async fn monthly_summary(pool: &SqlitePool, ym: &str) -> Result<MonthlySummary, AppError> {
    // 청구/입금 — 금액 기준은 adjusted_amount, 입금은 payments.is_paid=1 (billing.rs 와 정합).
    let row = sqlx::query(
        "SELECT \
            COALESCE(SUM(b.adjusted_amount), 0) AS bill_total, \
            COUNT(*) AS bill_count, \
            COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN b.adjusted_amount ELSE 0 END), 0) AS paid_total, \
            COALESCE(SUM(CASE WHEN p.is_paid = 1 THEN 1 ELSE 0 END), 0) AS paid_count \
         FROM bills b LEFT JOIN payments p ON p.bill_id = b.id \
         WHERE b.bill_year_month = ?",
    )
    .bind(ym)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    let bill_total: i64 = row.get("bill_total");
    let paid_total: i64 = row.get("paid_total");
    let bill_count: i64 = row.get("bill_count");
    let paid_count: i64 = row.get("paid_count");

    let enrolled_this_month: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM students WHERE substr(enroll_date, 1, 7) = ?")
            .bind(ym)
            .fetch_one(pool)
            .await
            .map_err(AppError::Db)?;
    let withdrawn_this_month: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM students WHERE withdraw_date IS NOT NULL AND substr(withdraw_date, 1, 7) = ?",
    )
    .bind(ym)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    let attendance_recorded_days: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT event_date) FROM regular_attendances WHERE year_month = ?",
    )
    .bind(ym)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(MonthlySummary {
        year_month: ym.to_string(),
        bill_total,
        paid_total,
        unpaid_total: bill_total - paid_total,
        bill_count,
        paid_count,
        enrolled_this_month,
        withdrawn_this_month,
        attendance_recorded_days,
    })
}

// ----------------------------------------------------------------------------
// 4.11.5 출결 입력 진행률
// ----------------------------------------------------------------------------

/// 월의 마지막 날짜.
fn last_day_of_month(ym: &str) -> Result<NaiveDate, AppError> {
    let first = NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d")
        .map_err(|e| AppError::Config(format!("year_month 파싱 실패: {}", e)))?;
    let next = first
        .checked_add_months(Months::new(1))
        .ok_or_else(|| AppError::Config("월 계산 실패".into()))?;
    next.pred_opt()
        .ok_or_else(|| AppError::Config("말일 계산 실패".into()))
}

async fn attendance_progress(
    pool: &SqlitePool,
    ym: &str,
    today: NaiveDate,
) -> Result<AttendanceProgress, AppError> {
    // 현행 스케줄(재원생)의 수업 요일 집합.
    let weekday_rows = sqlx::query(
        "SELECT DISTINCT ss.day_of_week AS dow FROM student_schedules ss \
         JOIN students s ON s.id = ss.student_id \
         WHERE ss.effective_to IS NULL AND s.withdraw_date IS NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;
    let class_weekdays: std::collections::HashSet<u32> = weekday_rows
        .into_iter()
        .map(|r| r.get::<i64, _>("dow") as u32)
        .collect();

    // 당월 정규출결이 기록된 일자.
    let recorded_rows =
        sqlx::query("SELECT DISTINCT event_date AS d FROM regular_attendances WHERE year_month = ?")
            .bind(ym)
            .fetch_all(pool)
            .await
            .map_err(AppError::Db)?;
    let recorded: std::collections::HashSet<String> =
        recorded_rows.into_iter().map(|r| r.get::<String, _>("d")).collect();

    // 1일 ~ min(오늘, 말일) 중 수업 요일에 해당하는 후보일.
    let first = NaiveDate::parse_from_str(&format!("{}-01", ym), "%Y-%m-%d")
        .map_err(|e| AppError::Config(format!("year_month 파싱 실패: {}", e)))?;
    let month_end = last_day_of_month(ym)?;
    let upper = today.min(month_end);

    let mut expected = 0i64;
    let mut missing_dates = Vec::new();
    let mut cursor = first;
    while cursor <= upper {
        let iso = cursor.weekday().number_from_monday(); // 1=월~7=일
        if class_weekdays.contains(&iso) {
            expected += 1;
            let ds = cursor.to_string();
            if !recorded.contains(&ds) {
                missing_dates.push(ds);
            }
        }
        cursor = match cursor.succ_opt() {
            Some(d) => d,
            None => break,
        };
    }
    let recorded_days = expected - missing_dates.len() as i64;

    Ok(AttendanceProgress {
        year_month: ym.to_string(),
        expected_days: expected,
        recorded_days,
        missing_dates,
    })
}

// ----------------------------------------------------------------------------
// 4.11.4 알림 5종
// ----------------------------------------------------------------------------

async fn dashboard_alerts(
    pool: &SqlitePool,
    today: NaiveDate,
) -> Result<Vec<DashboardAlert>, AppError> {
    let ym = year_month_of(today);
    let mut alerts = Vec::new();

    // 1) 출결 미입력 (당월, 오늘까지 수업일 중 미기록) — 빨강.
    let progress = attendance_progress(pool, &ym, today).await?;
    let missing = progress.missing_dates.len() as i64;
    if missing > 0 {
        alerts.push(DashboardAlert {
            kind: "attendance_missing".into(),
            severity: "red".into(),
            message: format!("출결 미입력 일자가 {}일 있습니다.", missing),
            count: missing,
        });
    }

    // 2) 보강 소멸 임박 (makeup_deadline 이 당월인 결석) — 주황.
    let expiring: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM regular_attendances \
         WHERE status = 'absent' AND makeup_deadline = ?",
    )
    .bind(&ym)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;
    if expiring > 0 {
        alerts.push(DashboardAlert {
            kind: "makeup_expiring".into(),
            severity: "orange".into(),
            message: format!("이번 달 보강 소멸 예정 결석이 {}건 있습니다.", expiring),
            count: expiring,
        });
    }

    // 3) 미확정 청구 (당월 draft) — 주황.
    let draft: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bills WHERE status = 'draft' AND bill_year_month = ?",
    )
    .bind(&ym)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;
    if draft > 0 {
        alerts.push(DashboardAlert {
            kind: "draft_bills".into(),
            severity: "orange".into(),
            message: format!("미확정 청구가 {}건 있습니다.", draft),
            count: draft,
        });
    }

    // 4) 학사 미수립 (오늘 25일 이후 + 다음 달 교습기간 미등록) — 빨강 (AC-4.11-5).
    if today.day() >= 25 {
        let next_month = year_month_of(
            today
                .checked_add_months(Months::new(1))
                .unwrap_or(today),
        );
        let exists: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM study_periods WHERE year_month = ?")
                .bind(&next_month)
                .fetch_one(pool)
                .await
                .map_err(AppError::Db)?;
        if exists == 0 {
            alerts.push(DashboardAlert {
                kind: "academic_not_set".into(),
                severity: "red".into(),
                message: format!("다음 달({}) 교습기간이 아직 수립되지 않았습니다.", next_month),
                count: 1,
            });
        }
    }

    // 5) 자가 진단 이상 (최신 결과 issues_found > 0) — 주황.
    let latest_issues: Option<i64> = sqlx::query_scalar(
        "SELECT issues_found FROM diagnosis_history ORDER BY run_date DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)?;
    if let Some(n) = latest_issues {
        if n > 0 {
            alerts.push(DashboardAlert {
                kind: "diagnosis_issues".into(),
                severity: "orange".into(),
                message: format!("최근 자가 진단에서 이상 {}건이 발견되었습니다.", n),
                count: n,
            });
        }
    }

    Ok(alerts)
}

// ----------------------------------------------------------------------------
// 4.11.6 메모
// ----------------------------------------------------------------------------

async fn get_memo(pool: &SqlitePool) -> Result<Option<String>, AppError> {
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(KEY_DASHBOARD_MEMO)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)?;
    Ok(row.map(|r| r.get::<String, _>("value")))
}

async fn save_memo(pool: &SqlitePool, content: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(KEY_DASHBOARD_MEMO)
    .bind(content)
    .execute(pool)
    .await
    .map_err(AppError::Db)?;
    Ok(())
}

// ----------------------------------------------------------------------------
// Tauri IPC commands (얇은 래퍼)
// ----------------------------------------------------------------------------

#[tauri::command]
pub async fn get_academy_overview() -> Result<AcademyOverview, String> {
    let pool = pool().map_err(String::from)?;
    overview(pool, today_naive()).await.map_err(String::from)
}

#[tauri::command]
pub async fn get_today_schedule() -> Result<TodaySchedule, String> {
    let weekday = today_naive().weekday().number_from_monday() as u8;
    let pool = pool().map_err(String::from)?;
    today_schedule(pool, weekday).await.map_err(String::from)
}

#[tauri::command]
pub async fn get_monthly_summary(year_month: String) -> Result<MonthlySummary, String> {
    let pool = pool().map_err(String::from)?;
    monthly_summary(pool, &year_month).await.map_err(String::from)
}

#[tauri::command]
pub async fn get_attendance_progress(year_month: String) -> Result<AttendanceProgress, String> {
    let pool = pool().map_err(String::from)?;
    attendance_progress(pool, &year_month, today_naive())
        .await
        .map_err(String::from)
}

#[tauri::command]
pub async fn get_dashboard_alerts() -> Result<Vec<DashboardAlert>, String> {
    let pool = pool().map_err(String::from)?;
    dashboard_alerts(pool, today_naive()).await.map_err(String::from)
}

#[tauri::command]
pub async fn get_dashboard_memo() -> Result<Option<String>, String> {
    let pool = pool().map_err(String::from)?;
    get_memo(pool).await.map_err(String::from)
}

#[tauri::command]
pub async fn save_dashboard_memo(content: String) -> Result<(), String> {
    let pool = pool().map_err(String::from)?;
    save_memo(pool, &content).await.map_err(String::from)
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    async fn insert_student(
        pool: &SqlitePool,
        serial: &str,
        name: &str,
        gender: &str,
        level: &str,
        grade: i64,
        enroll: &str,
        withdraw: Option<&str>,
    ) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date, withdraw_date) \
             VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(serial).bind(name).bind(gender).bind(level).bind(grade).bind(enroll).bind(withdraw)
        .fetch_one(pool).await.expect("student insert");
        row.0
    }

    // ── 교습소 현황 ──
    #[tokio::test]
    async fn overview_counts_active_and_distributions() {
        let pool = test_pool_in_memory().await.unwrap();
        insert_student(&pool, "S1", "남초3", "male", "elementary", 3, "2026-01-01", None).await;
        insert_student(&pool, "S2", "여중1", "female", "middle", 1, "2026-01-01", None).await;
        insert_student(&pool, "S3", "퇴교생", "male", "elementary", 3, "2025-01-01", Some("2026-02-01")).await;
        let today = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let ov = overview(&pool, today).await.unwrap();
        assert_eq!(ov.total_active, 2, "재원 2명 (퇴교 1명 제외)");
        let male = ov.by_gender.iter().find(|l| l.label == "남").map(|l| l.count);
        assert_eq!(male, Some(1));
        assert!(ov.by_grade.iter().any(|l| l.label == "초3" && l.count == 1));
        assert!(ov.by_grade.iter().any(|l| l.label == "중1" && l.count == 1));
        assert_eq!(ov.quarterly.len(), 4);
    }

    #[test]
    fn quarter_helpers_map_academic_quarters() {
        // 2026-06-04 → 2분기 시작 2026-06-01
        let d = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        assert_eq!(quarter_start(d), NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(quarter_label(quarter_start(d)), "2026 2분기");
        // 1월 → 직전 해 12월 시작 4분기
        let jan = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert_eq!(quarter_start(jan), NaiveDate::from_ymd_opt(2025, 12, 1).unwrap());
        assert_eq!(quarter_label(quarter_start(jan)), "2025 4분기");
        // 최근 4분기, 오래된 순
        let qs = recent_quarters(d);
        assert_eq!(qs.len(), 4);
        assert_eq!(qs[3].0, "2026 2분기");
    }

    #[tokio::test]
    async fn overview_quarterly_counts_enroll_in_range() {
        let pool = test_pool_in_memory().await.unwrap();
        // 2026-06-04 기준 최근 4분기: 2025 3분기(9~11) / 4분기(12~2) / 2026 1분기(3~5) / 2분기(6~8)
        insert_student(&pool, "S1", "a", "male", "elementary", 3, "2026-06-02", None).await; // 2분기 입교
        insert_student(&pool, "S2", "b", "male", "elementary", 3, "2026-04-10", None).await; // 1분기 입교
        let today = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let ov = overview(&pool, today).await.unwrap();
        let q2 = ov.quarterly.iter().find(|q| q.label == "2026 2분기").unwrap();
        assert_eq!(q2.enrolled, 1);
        let q1 = ov.quarterly.iter().find(|q| q.label == "2026 1분기").unwrap();
        assert_eq!(q1.enrolled, 1);
    }

    // ── 당일 수업 ──
    #[tokio::test]
    async fn today_schedule_groups_by_start_time() {
        let pool = test_pool_in_memory().await.unwrap();
        let a = insert_student(&pool, "S1", "가", "male", "elementary", 3, "2026-01-01", None).await;
        let b = insert_student(&pool, "S2", "나", "female", "elementary", 4, "2026-01-01", None).await;
        // 둘 다 월요일(1) 15:00
        for sid in [a, b] {
            sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
                .bind(sid).execute(&pool).await.unwrap();
        }
        let sch = today_schedule(&pool, 1).await.unwrap();
        assert_eq!(sch.slots.len(), 1);
        assert_eq!(sch.slots[0].start_time, "15:00");
        assert_eq!(sch.slots[0].students.len(), 2);
    }

    // ── 월 핵심 요약 ──
    #[tokio::test]
    async fn monthly_summary_totals_billing_and_paid() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "가", "male", "elementary", 3, "2026-06-05", None).await;
        // 청구 2건, 1건 수납
        let b1: (i64,) = sqlx::query_as("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 200000, 200000) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        let sid2 = insert_student(&pool, "S2", "나", "female", "middle", 1, "2026-01-01", None).await;
        sqlx::query("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 150000, 150000)")
            .bind(sid2).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO payments (bill_id, is_paid, paid_date) VALUES (?, 1, '2026-06-10')")
            .bind(b1.0).execute(&pool).await.unwrap();

        let sum = monthly_summary(&pool, "2026-06").await.unwrap();
        assert_eq!(sum.bill_total, 350000);
        assert_eq!(sum.paid_total, 200000);
        assert_eq!(sum.unpaid_total, 150000);
        assert_eq!(sum.bill_count, 2);
        assert_eq!(sum.paid_count, 1);
        assert_eq!(sum.enrolled_this_month, 1, "2026-06 입교 1명");
    }

    // ── 출결 진행률 ──
    #[tokio::test]
    async fn attendance_progress_lists_missing_class_days() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "가", "male", "elementary", 3, "2026-01-01", None).await;
        // 월요일 수업. 2026-06 월요일: 1,8,15,22,29
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        // 6/1 만 출결 기록
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) VALUES (?, '2026-06-01', '2026-06', 120)")
            .bind(sid).execute(&pool).await.unwrap();
        // 오늘 = 6/15 → 후보 월요일 1,8,15 / 기록 1 → 미입력 8,15
        let today = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let prog = attendance_progress(&pool, "2026-06", today).await.unwrap();
        assert_eq!(prog.expected_days, 3);
        assert_eq!(prog.recorded_days, 1);
        assert_eq!(prog.missing_dates, vec!["2026-06-08", "2026-06-15"]);
    }

    // ── 알림 ──
    #[tokio::test]
    async fn alerts_flag_draft_bills_and_diagnosis() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "가", "male", "elementary", 3, "2026-01-01", None).await;
        sqlx::query("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount, status) VALUES (?, '2026-06', 4, 200000, 200000, 'draft')")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO diagnosis_history (run_date, run_type, total_checks, issues_found, details) VALUES ('2026-06-01', 'manual', 7, 2, '[]')")
            .execute(&pool).await.unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 6, 4).unwrap();
        let alerts = dashboard_alerts(&pool, today).await.unwrap();
        assert!(alerts.iter().any(|a| a.kind == "draft_bills" && a.count == 1));
        assert!(alerts.iter().any(|a| a.kind == "diagnosis_issues" && a.count == 2));
    }

    #[tokio::test]
    async fn alerts_academic_not_set_after_25th() {
        let pool = test_pool_in_memory().await.unwrap();
        // 6/26, 다음 달(2026-07) 교습기간 없음 → 알림.
        let today = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();
        let alerts = dashboard_alerts(&pool, today).await.unwrap();
        assert!(alerts.iter().any(|a| a.kind == "academic_not_set"));
        // 교습기간 등록 시 알림 사라짐.
        sqlx::query("INSERT INTO study_periods (year_month, start_date, end_date) VALUES ('2026-07', '2026-07-01', '2026-07-31')")
            .execute(&pool).await.unwrap();
        let alerts2 = dashboard_alerts(&pool, today).await.unwrap();
        assert!(!alerts2.iter().any(|a| a.kind == "academic_not_set"));
    }

    // ── 메모 ──
    #[tokio::test]
    async fn memo_roundtrip() {
        let pool = test_pool_in_memory().await.unwrap();
        assert_eq!(get_memo(&pool).await.unwrap(), None);
        save_memo(&pool, "오늘 보강 챙기기").await.unwrap();
        assert_eq!(get_memo(&pool).await.unwrap(), Some("오늘 보강 챙기기".to_string()));
        // 덮어쓰기
        save_memo(&pool, "수정됨").await.unwrap();
        assert_eq!(get_memo(&pool).await.unwrap(), Some("수정됨".to_string()));
    }
}
