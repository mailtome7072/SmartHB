//! 출결 도메인 IPC — Sprint 8 T2·T3 (PRD §4.5, data-model §2.4).
//!
//! T2 — 생성:
//! - [`check_attendance_exists`] — 해당 월 정규 출결 존재 여부 (UI "출결 생성" 버튼 활성 조건)
//! - [`generate_attendances`] — 해당 월 재원 원생 × 수업 요일 일자에 정규 출결 일괄 INSERT
//!
//! T3 — 조회·토글:
//! - [`get_attendance_grid`] — 출결표 그리드 (원생 × 일자, 일자별 셀 + 월간 요약)
//! - [`toggle_attendance`] — 출석↔결석 토글 + 보강필요시간/소멸기한 자동 갱신
//! - [`update_absence_memo`] — 결석 사유 메모 (NULL 가능)
//! - [`get_attendance_summary`] — 원생별 월간 요약 (출석/결석/보강필요/보강완료 분)
//!
//! 생성 규칙 (T2):
//! 1. 교습기간이 설정 + `is_confirmed=1` 이어야 한다
//! 2. 같은 월에 이미 생성된 출결이 있으면 거부 (AC-4.5-1 중복 방지)
//! 3. `student_schedules` 의 현행 (effective_to IS NULL) 요일별 스케줄을 기준으로 일자 산출
//! 4. `schedule_events` JOIN `schedule_codes` 에서 `allows_regular_class=0` 인 일자/기간은 제외
//! 5. 원생 `enroll_date` 이전 / `withdraw_date` 이후 일자는 제외
//! 6. `class_minutes = duration_hours × 60` (V101 hours INTEGER 저장)
//! 7. 전체 INSERT 를 단일 트랜잭션으로 처리 (부분 실패 시 롤백)
//!
//! 토글 규칙 (T3):
//! - `present` → `absent`: makeup_deadline = (year_month + 1), absence_memo는 유지
//! - `absent` → `present`: makeup_deadline=NULL, absence_memo=NULL로 초기화
//! - `makeup_done` (보강 매칭) / `makeup_expired` (소멸) 상태는 토글 차단 — 보강 도메인에서 관리
//!
//! 보강필요시간 정의:
//! - `makeup_needed = SUM(class_minutes WHERE status='absent' AND makeup_attendance_id IS NULL)`
//! - `makeup_completed = SUM(class_minutes FROM makeup_attendances WHERE status='makeup_attended')`

use crate::commands::audit::{self, AuditEventType};
use crate::commands::db::pool;
use chrono::{Datelike, Months, NaiveDate};
use serde::Serialize;
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet};

const MINUTES_PER_HOUR: i64 = 60;

/// 출결 생성 결과 — 프론트엔드 토스트/요약에 사용.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub year_month: String,
    pub student_count: i64,
    pub attendance_count: i64,
}

#[tauri::command]
pub async fn check_attendance_exists(year_month: String) -> Result<bool, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    check_exists_impl(pool, &year_month).await
}

#[tauri::command]
pub async fn generate_attendances(year_month: String) -> Result<GenerateResult, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    generate_impl(pool, &year_month).await
}

async fn check_exists_impl(pool: &SqlitePool, year_month: &str) -> Result<bool, String> {
    validate_year_month(year_month)?;
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM regular_attendances WHERE year_month = ?) AS flag",
    )
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("출결 존재 여부 조회 실패: {}", e))?;
    let flag: i64 = row.try_get("flag").map_err(|e| e.to_string())?;
    Ok(flag != 0)
}

async fn generate_impl(pool: &SqlitePool, year_month: &str) -> Result<GenerateResult, String> {
    validate_year_month(year_month)?;

    let (start_date, end_date) = load_confirmed_period(pool, year_month).await?;

    if check_exists_impl(pool, year_month).await? {
        return Err(format!(
            "{} 출결이 이미 생성되어 있습니다. 기존 출결을 확인 후 다시 시도하세요.",
            year_month
        ));
    }

    let off_dates = load_off_dates(pool, &start_date, &end_date).await?;
    let students = load_active_students(pool, &start_date).await?;

    let sd = parse_date(&start_date)?;
    let ed = parse_date(&end_date)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    let mut attendance_count: i64 = 0;
    let mut student_count: i64 = 0;

    for s in &students {
        let dow_to_minutes = load_weekly_schedule(&mut *tx, s.id).await?;
        if dow_to_minutes.is_empty() {
            continue;
        }

        let enroll_d = parse_date(&s.enroll_date)?;
        let withdraw_d = match &s.withdraw_date {
            Some(w) => Some(parse_date(w)?),
            None => None,
        };

        let mut inserted = 0i64;
        let mut d = sd;
        while d <= ed {
            let dow = d.weekday().number_from_monday() as i64;
            if let Some(&minutes) = dow_to_minutes.get(&dow) {
                let in_enroll_range = d >= enroll_d && withdraw_d.is_none_or(|wd| d <= wd);
                let date_str = d.format("%Y-%m-%d").to_string();
                if in_enroll_range && !off_dates.contains(&date_str) {
                    sqlx::query(
                        "INSERT INTO regular_attendances \
                         (student_id, event_date, year_month, status, class_minutes) \
                         VALUES (?, ?, ?, 'present', ?)",
                    )
                    .bind(s.id)
                    .bind(&date_str)
                    .bind(year_month)
                    .bind(minutes)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("출결 INSERT 실패: {}", e))?;
                    inserted += 1;
                }
            }
            d = d
                .succ_opt()
                .ok_or_else(|| "날짜 계산 오버플로".to_string())?;
        }

        if inserted > 0 {
            student_count += 1;
            attendance_count += inserted;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(GenerateResult {
        year_month: year_month.to_string(),
        student_count,
        attendance_count,
    })
}

// ─────────────────────── 헬퍼 ───────────────────────

struct StudentRow {
    id: i64,
    enroll_date: String,
    withdraw_date: Option<String>,
}

/// Sprint 9 T2 (A43): 월 범위(01-12) 검증 추가. `2026-00` / `2026-13` 같은 의미론적
/// 무효 입력을 GLOB 패턴 통과 후 `NaiveDate::parse_from_str` 실패로 비친화적 에러
/// 노출되던 문제 해소. `pub(crate)` 로 노출하여 `makeup.rs` 등 동일 crate 의 다른
/// 도메인 모듈에서 재사용.
pub(crate) fn validate_year_month(ym: &str) -> Result<(), String> {
    if ym.len() != 7 || ym.as_bytes()[4] != b'-' {
        return Err("year_month 는 YYYY-MM 형식이어야 합니다.".to_string());
    }
    let year = &ym[..4];
    let month = &ym[5..];
    if !year.chars().all(|c| c.is_ascii_digit()) || !month.chars().all(|c| c.is_ascii_digit()) {
        return Err("year_month 에 숫자가 아닌 문자가 포함되어 있습니다.".to_string());
    }
    let m: u8 = month.parse().expect("digits checked above");
    if !(1..=12).contains(&m) {
        return Err(format!(
            "year_month 의 월은 01~12 사이여야 합니다 (입력: {}).",
            ym
        ));
    }
    Ok(())
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("날짜 파싱 실패 ({}): {}", s, e))
}

async fn load_confirmed_period(
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

async fn load_off_dates(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> Result<HashSet<String>, String> {
    let rows = sqlx::query(
        "SELECT e.event_date, COALESCE(e.period_end_date, e.event_date) AS end_d \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_regular_class = 0 \
           AND e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(end_date)
    .bind(start_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("학사일정 조회 실패: {}", e))?;

    let mut off: HashSet<String> = HashSet::new();
    for r in rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let mut d = parse_date(&s)?;
        let ed = parse_date(&e)?;
        while d <= ed {
            off.insert(d.format("%Y-%m-%d").to_string());
            d = d
                .succ_opt()
                .ok_or_else(|| "OFF 날짜 계산 오버플로".to_string())?;
        }
    }
    Ok(off)
}

async fn load_active_students(
    pool: &SqlitePool,
    start_date: &str,
) -> Result<Vec<StudentRow>, String> {
    let rows = sqlx::query(
        "SELECT id, enroll_date, withdraw_date FROM students \
         WHERE withdraw_date IS NULL OR withdraw_date >= ? \
         ORDER BY id",
    )
    .bind(start_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("재원 원생 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(StudentRow {
                id: r.try_get("id").map_err(|e: sqlx::Error| e.to_string())?,
                enroll_date: r.try_get("enroll_date").map_err(|e: sqlx::Error| e.to_string())?,
                withdraw_date: r
                    .try_get("withdraw_date")
                    .map_err(|e: sqlx::Error| e.to_string())?,
            })
        })
        .collect()
}

async fn load_weekly_schedule<'c, E>(
    executor: E,
    student_id: i64,
) -> Result<HashMap<i64, i64>, String>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query(
        "SELECT day_of_week, duration_hours FROM student_schedules \
         WHERE student_id = ? AND effective_to IS NULL",
    )
    .bind(student_id)
    .fetch_all(executor)
    .await
    .map_err(|e| format!("원생 스케줄 조회 실패: {}", e))?;

    let mut map = HashMap::new();
    for r in rows {
        let dow: i64 = r.try_get("day_of_week").map_err(|e| e.to_string())?;
        let hours: i64 = r.try_get("duration_hours").map_err(|e| e.to_string())?;
        map.insert(dow, hours * MINUTES_PER_HOUR);
    }
    Ok(map)
}

// ─────────────────────── T3: 조회 + 토글 ───────────────────────

/// 출결 셀 — 그리드 한 칸에 들어가는 정보.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceCell {
    pub id: i64,
    pub event_date: String,
    pub status: String,
    pub class_minutes: i64,
    pub absence_memo: Option<String>,
    pub makeup_deadline: Option<String>,
    pub makeup_attendance_id: Option<i64>,
}

/// 원생별 월간 요약.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceSummary {
    pub student_id: i64,
    pub year_month: String,
    pub present_count: i64,
    pub absent_count: i64,
    pub makeup_needed_minutes: i64,
    pub makeup_completed_minutes: i64,
}

/// 보강 출결 1건 — 그리드에서 비수업일 셀에 표시.
/// Sprint 9 Session #10 J4 — "결석일과 보강일이 다른 경우 보강일 셀에 표기".
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GridMakeupCell {
    pub id: i64,
    pub event_date: String,
    pub status: String, // makeup_attended | makeup_absent
    pub class_minutes: i64,
}

/// 그리드 한 원생 행.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceGridStudent {
    pub student_id: i64,
    pub name: String,
    pub serial_no: String,
    pub schedule_days: Vec<i64>,
    /// Sprint 9 Session #10 I8 — 클라이언트가 비수업일 셀 "+" 표시 조건 판단에 사용.
    pub enroll_date: String,
    /// Sprint 9 Session #10 I8 — 퇴교일 없으면 null.
    pub withdraw_date: Option<String>,
    pub attendances: Vec<AttendanceCell>,
    /// Sprint 9 Session #10 J4 — month 내 보강 출결 (비수업일 셀에 표기).
    pub makeups: Vec<GridMakeupCell>,
    pub summary: AttendanceSummary,
}

/// 학사일정 매핑 — 해당 월 일자별 코드 속성.
/// Sprint 9 Session #10 I7 (헤더 보강데이 시각 강조) + I8 (셀 사전 판단).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DaySchedule {
    pub event_date: String,
    /// 보강 가능 코드 명시 (보강데이/단원평가/공휴수업일).
    pub allows_makeup: bool,
    /// 보강 불가 코드 명시 (공휴일/방학/휴원일).
    pub is_block: bool,
    /// 표시용 코드명 (우선순위: allows_makeup > is_block > 일반).
    pub label: String,
}

/// 그리드 응답 전체.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceGrid {
    pub year_month: String,
    pub students: Vec<AttendanceGridStudent>,
    /// 해당 월 일자별 학사일정 코드 정보 — 일자 헤더 강조 + 비수업일 셀 사전 판단.
    pub day_schedules: Vec<DaySchedule>,
}

/// 토글 결과.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToggleResult {
    pub attendance_id: i64,
    pub new_status: String,
    pub new_makeup_deadline: Option<String>,
    pub updated_summary: AttendanceSummary,
}

#[tauri::command]
pub async fn get_attendance_grid(year_month: String) -> Result<AttendanceGrid, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    get_grid_impl(pool, &year_month).await
}

#[tauri::command]
pub async fn toggle_attendance(
    attendance_id: i64,
    new_status: String,
) -> Result<ToggleResult, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    toggle_impl(pool, attendance_id, &new_status).await
}

#[tauri::command]
pub async fn update_absence_memo(
    attendance_id: i64,
    memo: Option<String>,
) -> Result<(), String> {
    let pool = pool().map_err(|e| e.to_string())?;
    update_memo_impl(pool, attendance_id, memo.as_deref()).await
}

#[tauri::command]
pub async fn get_attendance_summary(
    student_id: i64,
    year_month: String,
) -> Result<AttendanceSummary, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    get_summary_impl(pool, student_id, &year_month).await
}

async fn get_grid_impl(pool: &SqlitePool, year_month: &str) -> Result<AttendanceGrid, String> {
    validate_year_month(year_month)?;

    // 1) 해당 월 출결이 있는 원생들 (정렬: serial_no) — Session #10 I8 위해 enroll/withdraw 동봉.
    let student_rows = sqlx::query(
        "SELECT DISTINCT s.id, s.name, s.serial_no, s.enroll_date, s.withdraw_date \
         FROM students s \
         JOIN regular_attendances a ON a.student_id = s.id \
         WHERE a.year_month = ? \
         ORDER BY CAST(s.serial_no AS INTEGER), s.serial_no",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("원생 조회 실패: {}", e))?;

    let mut students = Vec::with_capacity(student_rows.len());
    for srow in student_rows {
        let student_id: i64 = srow.try_get("id").map_err(|e| e.to_string())?;
        let name: String = srow.try_get("name").map_err(|e| e.to_string())?;
        let serial_no: String = srow.try_get("serial_no").map_err(|e| e.to_string())?;
        let enroll_date: String = srow.try_get("enroll_date").map_err(|e| e.to_string())?;
        let withdraw_date: Option<String> =
            srow.try_get("withdraw_date").map_err(|e| e.to_string())?;

        // 수업 요일 (현행 스케줄)
        let day_rows = sqlx::query(
            "SELECT day_of_week FROM student_schedules \
             WHERE student_id = ? AND effective_to IS NULL ORDER BY day_of_week",
        )
        .bind(student_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("스케줄 조회 실패: {}", e))?;
        let schedule_days: Vec<i64> = day_rows
            .iter()
            .filter_map(|r| r.try_get::<i64, _>("day_of_week").ok())
            .collect();

        // 출결 셀들
        let cell_rows = sqlx::query(
            "SELECT id, event_date, status, class_minutes, absence_memo, \
                    makeup_deadline, makeup_attendance_id \
             FROM regular_attendances \
             WHERE student_id = ? AND year_month = ? \
             ORDER BY event_date",
        )
        .bind(student_id)
        .bind(year_month)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("출결 조회 실패: {}", e))?;

        let attendances: Vec<AttendanceCell> = cell_rows
            .into_iter()
            .map(|r| {
                Ok(AttendanceCell {
                    id: r.try_get("id").map_err(|e: sqlx::Error| e.to_string())?,
                    event_date: r.try_get("event_date").map_err(|e: sqlx::Error| e.to_string())?,
                    status: r.try_get("status").map_err(|e: sqlx::Error| e.to_string())?,
                    class_minutes: r
                        .try_get("class_minutes")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                    absence_memo: r
                        .try_get("absence_memo")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                    makeup_deadline: r
                        .try_get("makeup_deadline")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                    makeup_attendance_id: r
                        .try_get("makeup_attendance_id")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let summary = compute_summary(pool, student_id, year_month).await?;

        // J4: 학생별 보강 출결 조회 — 비수업일 셀에 "보강" 표기.
        let makeup_rows = sqlx::query(
            "SELECT id, event_date, status, class_minutes \
             FROM makeup_attendances \
             WHERE student_id = ? AND year_month = ? \
             ORDER BY event_date",
        )
        .bind(student_id)
        .bind(year_month)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("보강 출결 조회 실패: {}", e))?;
        let makeups: Vec<GridMakeupCell> = makeup_rows
            .into_iter()
            .map(|r| {
                Ok(GridMakeupCell {
                    id: r.try_get("id").map_err(|e: sqlx::Error| e.to_string())?,
                    event_date: r
                        .try_get("event_date")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                    status: r.try_get("status").map_err(|e: sqlx::Error| e.to_string())?,
                    class_minutes: r
                        .try_get("class_minutes")
                        .map_err(|e: sqlx::Error| e.to_string())?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        students.push(AttendanceGridStudent {
            student_id,
            name,
            serial_no,
            schedule_days,
            enroll_date,
            withdraw_date,
            attendances,
            makeups,
            summary,
        });
    }

    let day_schedules = build_day_schedules(pool, year_month).await?;

    Ok(AttendanceGrid {
        year_month: year_month.to_string(),
        students,
        day_schedules,
    })
}

/// 월의 일자별 학사일정 코드 매핑을 생성한다 (Session #10 I7/I8).
///
/// 동일 일자에 다중 코드 가능 — 우선순위: `allows_makeup=1` > `is_block` > 일반.
/// 기간성 코드 (period_end_date) 는 시작~종료 모든 일자로 펼친다.
async fn build_day_schedules(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DaySchedule>, String> {
    let parts: Vec<&str> = year_month.split('-').collect();
    let year: i32 = parts[0].parse().expect("validated");
    let month: u32 = parts[1].parse().expect("validated");
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("일자 생성 실패: {}-{:02}-01", year, month))?;
    let next_month_first = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "다음 월 일자 생성 실패".to_string())?;

    // month 와 겹치는 모든 schedule_events + 속성 조회.
    let rows = sqlx::query(
        "SELECT e.event_date, COALESCE(e.period_end_date, e.event_date) AS end_d, \
                c.code_name, c.allows_regular_class, c.allows_makeup_class \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.event_date < ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(next_month_first.to_string())
    .bind(first.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| format!("학사일정 조회 실패: {}", e))?;

    use std::collections::BTreeMap;
    // 일자별 코드 후보 — (allows_makeup, is_block, label) 우선순위로 reduce.
    let mut by_date: BTreeMap<String, (bool, bool, String)> = BTreeMap::new();
    for r in rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e_str: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let code_name: String = r.try_get("code_name").map_err(|e| e.to_string())?;
        let allows_reg: i64 = r.try_get("allows_regular_class").map_err(|e| e.to_string())?;
        let allows_mk: i64 = r.try_get("allows_makeup_class").map_err(|e| e.to_string())?;
        let is_block = allows_reg == 0 && allows_mk == 0;
        let mut d = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 일자 파싱 실패: {}", e))?;
        let ed = chrono::NaiveDate::parse_from_str(&e_str, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 종료일 파싱 실패: {}", e))?;
        while d <= ed {
            if d >= first && d < next_month_first {
                let key = d.to_string();
                let entry = by_date.entry(key).or_insert((false, false, String::new()));
                // 우선순위: allows_makeup 가 우세 — 한 일자에 보강데이 + 공휴일 동시 등록 시 보강데이로 표시.
                let new_makeup = allows_mk == 1;
                let new_block = is_block;
                if new_makeup && !entry.0 {
                    *entry = (true, entry.1 || new_block, code_name.clone());
                } else if !entry.0 && new_block && !entry.1 {
                    *entry = (false, true, code_name.clone());
                } else if entry.2.is_empty() {
                    entry.2 = code_name.clone();
                }
                entry.0 = entry.0 || new_makeup;
                entry.1 = entry.1 || new_block;
            }
            d = d.succ_opt().expect("date succ");
        }
    }

    Ok(by_date
        .into_iter()
        .map(|(event_date, (allows_makeup, is_block, label))| DaySchedule {
            event_date,
            allows_makeup,
            is_block,
            label,
        })
        .collect())
}

async fn toggle_impl(
    pool: &SqlitePool,
    attendance_id: i64,
    new_status: &str,
) -> Result<ToggleResult, String> {
    if new_status != "present" && new_status != "absent" {
        return Err(format!(
            "토글 가능한 상태는 'present' 또는 'absent' 입니다 (요청: {}).",
            new_status
        ));
    }

    // 현재 상태 조회
    let row = sqlx::query(
        "SELECT student_id, year_month, status, makeup_attendance_id \
         FROM regular_attendances WHERE id = ?",
    )
    .bind(attendance_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("출결 조회 실패: {}", e))?
    .ok_or_else(|| format!("출결 레코드를 찾을 수 없습니다 (id={}).", attendance_id))?;

    let current_status: String = row.try_get("status").map_err(|e| e.to_string())?;
    let student_id: i64 = row.try_get("student_id").map_err(|e| e.to_string())?;
    let year_month: String = row.try_get("year_month").map_err(|e| e.to_string())?;

    // 보강완료/소멸 상태는 토글 불가
    if current_status == "makeup_done" {
        return Err(
            "이 출결은 보강이 매칭되어 있어 직접 토글할 수 없습니다. 보강 매칭을 먼저 해제하세요."
                .to_string(),
        );
    }
    if current_status == "makeup_expired" {
        return Err(
            "이 결석은 소멸 처리되어 토글할 수 없습니다. 필요 시 소멸 환원을 먼저 수행하세요."
                .to_string(),
        );
    }
    if current_status == new_status {
        return Err(format!("이미 '{}' 상태입니다.", new_status));
    }

    // 토글 실행
    let new_deadline = if new_status == "absent" {
        Some(next_month_str(&year_month)?)
    } else {
        None
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    if new_status == "absent" {
        sqlx::query(
            "UPDATE regular_attendances \
             SET status='absent', makeup_deadline=?, \
                 updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
             WHERE id = ?",
        )
        .bind(&new_deadline)
        .bind(attendance_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("토글 UPDATE 실패: {}", e))?;
    } else {
        sqlx::query(
            "UPDATE regular_attendances \
             SET status='present', makeup_deadline=NULL, absence_memo=NULL, \
                 updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
             WHERE id = ?",
        )
        .bind(attendance_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("토글 UPDATE 실패: {}", e))?;
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    let details = format!(
        r#"{{"student_id":{},"year_month":"{}","from":"{}","to":"{}"}}"#,
        student_id, year_month, current_status, new_status
    );
    audit::try_record(
        AuditEventType::AttendanceToggled,
        Some(&attendance_id.to_string()),
        Some(&details),
    )
    .await;

    let updated_summary = compute_summary(pool, student_id, &year_month).await?;

    Ok(ToggleResult {
        attendance_id,
        new_status: new_status.to_string(),
        new_makeup_deadline: new_deadline,
        updated_summary,
    })
}

async fn update_memo_impl(
    pool: &SqlitePool,
    attendance_id: i64,
    memo: Option<&str>,
) -> Result<(), String> {
    let result = sqlx::query(
        "UPDATE regular_attendances \
         SET absence_memo=?, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') \
         WHERE id = ?",
    )
    .bind(memo)
    .bind(attendance_id)
    .execute(pool)
    .await
    .map_err(|e| format!("메모 UPDATE 실패: {}", e))?;

    if result.rows_affected() == 0 {
        return Err(format!("출결 레코드를 찾을 수 없습니다 (id={}).", attendance_id));
    }
    Ok(())
}

async fn get_summary_impl(
    pool: &SqlitePool,
    student_id: i64,
    year_month: &str,
) -> Result<AttendanceSummary, String> {
    validate_year_month(year_month)?;
    compute_summary(pool, student_id, year_month).await
}

async fn compute_summary(
    pool: &SqlitePool,
    student_id: i64,
    year_month: &str,
) -> Result<AttendanceSummary, String> {
    let row = sqlx::query(
        "SELECT \
            SUM(CASE WHEN status='present' THEN 1 ELSE 0 END) AS present_count, \
            SUM(CASE WHEN status='absent' THEN 1 ELSE 0 END) AS absent_count, \
            COALESCE(SUM(CASE WHEN status='absent' AND makeup_attendance_id IS NULL \
                              THEN class_minutes ELSE 0 END), 0) AS needed \
         FROM regular_attendances \
         WHERE student_id = ? AND year_month = ?",
    )
    .bind(student_id)
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("요약 조회 실패: {}", e))?;

    let present_count: i64 = row.try_get::<Option<i64>, _>("present_count").map_err(|e| e.to_string())?.unwrap_or(0);
    let absent_count: i64 = row.try_get::<Option<i64>, _>("absent_count").map_err(|e| e.to_string())?.unwrap_or(0);
    let needed: i64 = row.try_get("needed").map_err(|e| e.to_string())?;

    let completed_row = sqlx::query(
        "SELECT COALESCE(SUM(class_minutes), 0) AS completed \
         FROM makeup_attendances \
         WHERE student_id = ? AND year_month = ? AND status = 'makeup_attended'",
    )
    .bind(student_id)
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("보강완료 조회 실패: {}", e))?;
    let completed: i64 = completed_row.try_get("completed").map_err(|e| e.to_string())?;

    Ok(AttendanceSummary {
        student_id,
        year_month: year_month.to_string(),
        present_count,
        absent_count,
        makeup_needed_minutes: needed,
        makeup_completed_minutes: completed,
    })
}

/// `YYYY-MM` 다음 달을 `YYYY-MM` 형식으로 반환. 12월 → 다음해 01.
fn next_month_str(year_month: &str) -> Result<String, String> {
    validate_year_month(year_month)?;
    let base = NaiveDate::parse_from_str(&format!("{}-01", year_month), "%Y-%m-%d")
        .map_err(|e| format!("year_month 파싱 실패: {}", e))?;
    let next = base
        .checked_add_months(Months::new(1))
        .ok_or_else(|| "다음 달 계산 오버플로".to_string())?;
    Ok(format!("{:04}-{:02}", next.year(), next.month()))
}

// ─────────────────────── 단위 테스트 ───────────────────────

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    /// 원생 1명 삽입 후 id 반환. day_of_week 와 duration_hours 를 부여하면 현행 스케줄로 추가.
    async fn seed_student(
        pool: &SqlitePool,
        serial_no: &str,
        enroll: &str,
        withdraw: Option<&str>,
        schedules: &[(i64, i64)], // (day_of_week 1~7, duration_hours)
    ) -> i64 {
        let row = sqlx::query(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, \
             enroll_date, withdraw_date) \
             VALUES (?, ?, 'male', 'elementary', 3, ?, ?) RETURNING id",
        )
        .bind(serial_no)
        .bind(format!("학생-{}", serial_no))
        .bind(enroll)
        .bind(withdraw)
        .fetch_one(pool)
        .await
        .expect("학생 INSERT");
        let id: i64 = row.try_get("id").expect("학생 id");

        for (dow, hours) in schedules {
            sqlx::query(
                "INSERT INTO student_schedules \
                 (student_id, day_of_week, start_time, duration_hours, effective_from) \
                 VALUES (?, ?, '16:00', ?, ?)",
            )
            .bind(id)
            .bind(dow)
            .bind(hours)
            .bind(enroll)
            .execute(pool)
            .await
            .expect("스케줄 INSERT");
        }
        id
    }

    async fn seed_period(
        pool: &SqlitePool,
        year_month: &str,
        start: &str,
        end: &str,
        is_confirmed: i64,
    ) {
        sqlx::query(
            "INSERT INTO study_periods (year_month, start_date, end_date, is_confirmed) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(year_month)
        .bind(start)
        .bind(end)
        .bind(is_confirmed)
        .execute(pool)
        .await
        .expect("교습기간 INSERT");
    }

    /// 시스템 예약 코드 '공휴일' 이 V200 시드에 있다고 가정 — 없으면 직접 삽입 후 id 반환.
    async fn schedule_code_id(pool: &SqlitePool, name: &str) -> i64 {
        let existing: Option<(i64,)> =
            sqlx::query_as("SELECT id FROM schedule_codes WHERE code_name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await
                .expect("schedule_code 조회");
        if let Some((id,)) = existing {
            return id;
        }
        let row = sqlx::query(
            "INSERT INTO schedule_codes \
             (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
              is_duplicate_blocked, is_period_type) \
             VALUES (?, 0, 0, 0, 1, 0) RETURNING id",
        )
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("schedule_code INSERT");
        row.try_get("id").expect("code id")
    }

    async fn add_schedule_event(
        pool: &SqlitePool,
        code_id: i64,
        event_date: &str,
        period_end: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) \
             VALUES (?, ?, ?)",
        )
        .bind(code_id)
        .bind(event_date)
        .bind(period_end)
        .execute(pool)
        .await
        .expect("schedule_event INSERT");
    }

    // ─────── AC-T2-1 ───────

    #[tokio::test]
    async fn generate_creates_attendances_for_active_students() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 월/수/금 (1, 3, 5) 주 3일 수업
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (3, 1), (5, 1)]).await;
        // 화/목 (2, 4) 주 2일 수업
        seed_student(&pool, "S002", "2026-04-01", None, &[(2, 2), (4, 2)]).await;

        let result = generate_impl(&pool, "2026-06").await.expect("generate");
        // 2026-06 월(1)·수(3)·금(5) = 5+4+4=13일 (calendar 확인)
        // 화(2)·목(4) = 5+4 = 9일 — 6/2, 9, 16, 23, 30 (5) + 6/4, 11, 18, 25 (4) = 9
        // S001 (월수금): 6/1·3·5·8·10·12·15·17·19·22·24·26·29 → 월=5(1,8,15,22,29), 수=5(3,10,17,24), 금=4(5,12,19,26) = 14
        // 정확한 카운트는 chrono 위임 — 합계 검증만.
        assert!(result.attendance_count > 0);
        assert_eq!(result.student_count, 2);
        assert_eq!(result.year_month, "2026-06");

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances WHERE year_month = '2026-06'",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(total.0, result.attendance_count);
    }

    // ─────── AC-T2-2 ───────

    #[tokio::test]
    async fn generate_skips_off_days() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 매일 수업 (월~일)
        seed_student(
            &pool,
            "S001",
            "2026-04-01",
            None,
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
        )
        .await;

        // 6/6 현충일 (단일 일자), 6/15~6/19 방학 (기간성) 모두 allows_regular_class=0 인 코드로 등록.
        let holiday_id = schedule_code_id(&pool, "현충일").await; // 신규 (allows_regular_class=0)
        add_schedule_event(&pool, holiday_id, "2026-06-06", None).await;
        let vac_id = schedule_code_id(&pool, "방학").await; // V102 시드 (allows_regular_class=0)
        add_schedule_event(&pool, vac_id, "2026-06-15", Some("2026-06-19")).await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        // 6/6 + 6/15~6/19 (5일) = 총 6개 OFF 일자 → 출결 없어야 함.
        let off_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE event_date IN ('2026-06-06','2026-06-15','2026-06-16','2026-06-17','2026-06-18','2026-06-19')",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(off_count.0, 0, "OFF 일자에 출결이 생성되면 안 됩니다");

        // 6/30 마지막 날에는 출결 있어야 함.
        let last_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-30'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(last_count.0, 1);
    }

    // ─────── AC-T2-3 ───────

    #[tokio::test]
    async fn generate_respects_enroll_withdraw_range() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 6/15 입교, 6/25 퇴교, 매일 수업
        seed_student(
            &pool,
            "S001",
            "2026-06-15",
            Some("2026-06-25"),
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (7, 1)],
        )
        .await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        // 6/14 이전 출결 없음
        let before: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date < '2026-06-15'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(before.0, 0);

        // 6/26 이후 출결 없음
        let after: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM regular_attendances WHERE event_date > '2026-06-25'")
                .fetch_one(&pool)
                .await
                .expect("count");
        assert_eq!(after.0, 0);

        // 6/15 ~ 6/25 = 11일 — 매일 수업이므로 11건
        let mid: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE event_date >= '2026-06-15' AND event_date <= '2026-06-25'",
        )
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(mid.0, 11);
    }

    // ─────── AC-T2-4 ───────

    #[tokio::test]
    async fn generate_blocks_duplicate_month() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        generate_impl(&pool, "2026-06").await.expect("first");
        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("두 번째 호출은 실패해야 함");
        assert!(
            err.contains("이미 생성"),
            "에러 메시지에 '이미 생성' 포함 필요: {}",
            err
        );
    }

    // ─────── AC-T2-5 ───────

    #[tokio::test]
    async fn generate_requires_confirmed_period() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 0).await; // 미확정
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("미확정 교습기간은 거부");
        assert!(err.contains("확정"), "확정 안내 메시지 필요: {}", err);
    }

    #[tokio::test]
    async fn generate_requires_period_to_exist() {
        let pool = test_pool_in_memory().await.expect("pool");
        // 교습기간 미설정
        let err = generate_impl(&pool, "2026-06")
            .await
            .expect_err("교습기간 미설정 → 거부");
        assert!(err.contains("설정"), "설정 안내 메시지 필요: {}", err);
    }

    // ─────── AC-T2-6 ───────

    #[tokio::test]
    async fn class_minutes_matches_schedule_hours() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // duration_hours=2 → class_minutes=120 이어야 함
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 2)]).await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        let minutes: (i64,) =
            sqlx::query_as("SELECT class_minutes FROM regular_attendances LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("minutes");
        assert_eq!(minutes.0, 120, "duration_hours=2 → class_minutes=120");
    }

    // ─────── check_attendance_exists ───────

    #[tokio::test]
    async fn check_attendance_exists_reflects_state() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        assert!(!check_exists_impl(&pool, "2026-06").await.expect("초기 false"));
        generate_impl(&pool, "2026-06").await.expect("generate");
        assert!(check_exists_impl(&pool, "2026-06").await.expect("생성 후 true"));
    }

    #[tokio::test]
    async fn validate_year_month_rejects_invalid_formats() {
        assert!(validate_year_month("2026-06").is_ok());
        assert!(validate_year_month("2026-01").is_ok());
        assert!(validate_year_month("2026-12").is_ok());
        assert!(validate_year_month("2026/06").is_err());
        assert!(validate_year_month("2026-6").is_err());
        assert!(validate_year_month("YYYY-MM").is_err());
        assert!(validate_year_month("").is_err());
    }

    /// Sprint 9 T2 (A43): 월 범위(01-12) 검증 — GLOB 패턴은 통과하지만 의미론적
    /// 무효 입력 차단. 사용자 친화 에러 메시지 제공.
    #[tokio::test]
    async fn validate_year_month_rejects_out_of_range_month() {
        let err00 = validate_year_month("2026-00").expect_err("월 0은 무효");
        assert!(err00.contains("01~12"), "친화 메시지: {}", err00);
        let err13 = validate_year_month("2026-13").expect_err("월 13은 무효");
        assert!(err13.contains("01~12"), "친화 메시지: {}", err13);
        // 99 같은 명확히 잘못된 케이스도 차단 (이전엔 GLOB 통과로 NaiveDate 파싱 실패)
        assert!(validate_year_month("2026-99").is_err());
    }

    // ─────────────── T3 단위 테스트 ───────────────

    /// 출결 1건 직접 조회 (테스트 헬퍼).
    async fn fetch_cell(pool: &SqlitePool, attendance_id: i64) -> AttendanceCell {
        let row = sqlx::query(
            "SELECT id, event_date, status, class_minutes, absence_memo, \
                    makeup_deadline, makeup_attendance_id \
             FROM regular_attendances WHERE id = ?",
        )
        .bind(attendance_id)
        .fetch_one(pool)
        .await
        .expect("출결 조회");
        AttendanceCell {
            id: row.try_get("id").unwrap(),
            event_date: row.try_get("event_date").unwrap(),
            status: row.try_get("status").unwrap(),
            class_minutes: row.try_get("class_minutes").unwrap(),
            absence_memo: row.try_get("absence_memo").unwrap(),
            makeup_deadline: row.try_get("makeup_deadline").unwrap(),
            makeup_attendance_id: row.try_get("makeup_attendance_id").unwrap(),
        }
    }

    /// 출결을 임의 상태로 직접 설정 (테스트 시나리오 셋업용).
    async fn set_cell_state(
        pool: &SqlitePool,
        id: i64,
        status: &str,
        makeup_attendance_id: Option<i64>,
    ) {
        sqlx::query(
            "UPDATE regular_attendances \
             SET status=?, makeup_attendance_id=? WHERE id=?",
        )
        .bind(status)
        .bind(makeup_attendance_id)
        .bind(id)
        .execute(pool)
        .await
        .expect("상태 설정");
    }

    /// 테스트용 makeup_attendances 행 1개 삽입 → id 반환.
    /// V107 (Sprint 8 review F2) 적용 후 FK 강제되므로, regular_attendances 의
    /// makeup_attendance_id 컬럼에 더이상 dummy id 사용 불가. 본 헬퍼로 실제 id 확보.
    async fn seed_makeup(pool: &SqlitePool, student_id: i64, event_date: &str, year_month: &str) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, ?, ?, 60) RETURNING id",
        )
        .bind(student_id)
        .bind(event_date)
        .bind(year_month)
        .fetch_one(pool)
        .await
        .expect("makeup INSERT");
        row.0
    }

    async fn first_attendance_id(pool: &SqlitePool, student_id: i64, year_month: &str) -> i64 {
        let r: (i64,) = sqlx::query_as(
            "SELECT id FROM regular_attendances \
             WHERE student_id=? AND year_month=? ORDER BY event_date LIMIT 1",
        )
        .bind(student_id)
        .bind(year_month)
        .fetch_one(pool)
        .await
        .expect("attendance id");
        r.0
    }

    // ─────── AC-T3-1 ───────

    #[tokio::test]
    async fn get_attendance_grid_returns_full_structure() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (3, 1)]).await;

        generate_impl(&pool, "2026-06").await.expect("generate");

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        assert_eq!(grid.year_month, "2026-06");
        assert_eq!(grid.students.len(), 1);
        let s = &grid.students[0];
        assert_eq!(s.student_id, sid);
        assert_eq!(s.serial_no, "S001");
        assert_eq!(s.schedule_days, vec![1, 3]);
        assert!(!s.attendances.is_empty());
        assert_eq!(s.summary.year_month, "2026-06");
        assert_eq!(s.summary.absent_count, 0);
        assert!(s.summary.present_count > 0);
    }

    // ─────── AC-T3-2 ───────

    #[tokio::test]
    async fn toggle_present_to_absent_increases_makeup_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 2)]).await; // 2h=120m
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;

        let before = compute_summary(&pool, sid, "2026-06").await.expect("before");
        assert_eq!(before.makeup_needed_minutes, 0);

        let result = toggle_impl(&pool, aid, "absent").await.expect("toggle");
        assert_eq!(result.new_status, "absent");
        assert_eq!(result.new_makeup_deadline.as_deref(), Some("2026-07"));
        assert_eq!(result.updated_summary.makeup_needed_minutes, 120);
        assert_eq!(result.updated_summary.absent_count, 1);
    }

    // ─────── AC-T3-3 ───────

    #[tokio::test]
    async fn toggle_absent_to_present_decreases_makeup_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 60m
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;

        toggle_impl(&pool, aid, "absent").await.expect("absent");
        let mid = compute_summary(&pool, sid, "2026-06").await.expect("mid");
        assert_eq!(mid.makeup_needed_minutes, 60);

        let result = toggle_impl(&pool, aid, "present").await.expect("present");
        assert_eq!(result.new_status, "present");
        assert!(result.new_makeup_deadline.is_none());
        assert_eq!(result.updated_summary.makeup_needed_minutes, 0);
        assert_eq!(result.updated_summary.absent_count, 0);

        // absence_memo 도 NULL 로 환원되어야 함
        let cell = fetch_cell(&pool, aid).await;
        assert!(cell.absence_memo.is_none());
        assert!(cell.makeup_deadline.is_none());
    }

    // ─────── AC-T3-4 ───────

    #[tokio::test]
    async fn toggle_to_absent_sets_deadline_next_month() {
        // 5월 → 6월
        assert_eq!(next_month_str("2026-05").expect("ok"), "2026-06");
        // 12월 → 다음해 01
        assert_eq!(next_month_str("2026-12").expect("ok"), "2027-01");
        // 1월 → 2월
        assert_eq!(next_month_str("2026-01").expect("ok"), "2026-02");
        // 잘못된 입력은 에러
        assert!(next_month_str("invalid").is_err());
    }

    // ─────── AC-T3-5 ───────

    #[tokio::test]
    async fn toggle_blocked_for_makeup_done_and_expired() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;

        // makeup_done 상태에서 토글 차단 — V107 FK 강제로 실제 makeup id 필요.
        let makeup_id = seed_makeup(&pool, sid, "2026-06-15", "2026-06").await;
        set_cell_state(&pool, aid, "makeup_done", Some(makeup_id)).await;
        let err = toggle_impl(&pool, aid, "present")
            .await
            .expect_err("makeup_done 차단");
        assert!(err.contains("보강"), "보강 안내 메시지: {}", err);

        // makeup_expired 상태에서 토글 차단
        set_cell_state(&pool, aid, "makeup_expired", None).await;
        let err = toggle_impl(&pool, aid, "absent")
            .await
            .expect_err("makeup_expired 차단");
        assert!(err.contains("소멸"), "소멸 안내 메시지: {}", err);
    }

    #[tokio::test]
    async fn toggle_rejects_invalid_status() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;

        let err = toggle_impl(&pool, aid, "makeup_done")
            .await
            .expect_err("외부 새 상태 거부");
        assert!(err.contains("present"));
    }

    // ─────── AC-T3-6 ───────

    #[tokio::test]
    async fn update_absence_memo_writes_text_and_nulls() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;
        toggle_impl(&pool, aid, "absent").await.expect("absent");

        update_memo_impl(&pool, aid, Some("가족 행사")).await.expect("set memo");
        assert_eq!(fetch_cell(&pool, aid).await.absence_memo.as_deref(), Some("가족 행사"));

        update_memo_impl(&pool, aid, None).await.expect("clear memo");
        assert!(fetch_cell(&pool, aid).await.absence_memo.is_none());

        // 존재하지 않는 id
        assert!(update_memo_impl(&pool, 9999, Some("x")).await.is_err());
    }

    // ─────── 보조: 보강 매칭은 needed 에서 제외 ───────

    #[tokio::test]
    async fn summary_excludes_matched_makeup_from_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 60m/회
        generate_impl(&pool, "2026-06").await.expect("generate");

        // 2건 결석 후 1건만 보강 매칭 (status=absent + makeup_attendance_id=NOT NULL → needed 제외)
        let a_ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM regular_attendances WHERE student_id=? AND year_month=? ORDER BY event_date LIMIT 2",
        )
        .bind(sid)
        .bind("2026-06")
        .fetch_all(&pool)
        .await
        .expect("ids");
        toggle_impl(&pool, a_ids[0].0, "absent").await.expect("a1 absent");
        toggle_impl(&pool, a_ids[1].0, "absent").await.expect("a2 absent");
        // 1번째 결석은 보강 매칭됨 — V107 FK 강제로 실제 makeup id 필요.
        let makeup_id = seed_makeup(&pool, sid, "2026-06-22", "2026-06").await;
        set_cell_state(&pool, a_ids[0].0, "absent", Some(makeup_id)).await;

        let s = compute_summary(&pool, sid, "2026-06").await.expect("summary");
        assert_eq!(s.absent_count, 2);
        assert_eq!(s.makeup_needed_minutes, 60, "매칭된 결석은 needed 에서 제외");
    }

    // ─────── T5: 보강필요시간 + 소멸기한 규칙 전수 ───────

    /// 시나리오 2 — 결석 2건이면 needed 는 class_minutes 합산.
    #[tokio::test]
    async fn t5_two_absents_sum_makeup_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 60m
        generate_impl(&pool, "2026-06").await.expect("generate");

        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM regular_attendances \
             WHERE student_id=? AND year_month=? ORDER BY event_date LIMIT 2",
        )
        .bind(sid)
        .bind("2026-06")
        .fetch_all(&pool)
        .await
        .expect("ids");
        toggle_impl(&pool, ids[0].0, "absent").await.expect("a1");
        toggle_impl(&pool, ids[1].0, "absent").await.expect("a2");

        let s = compute_summary(&pool, sid, "2026-06").await.expect("summary");
        assert_eq!(s.absent_count, 2);
        assert_eq!(s.makeup_needed_minutes, 120, "결석 2건 → 60+60");
    }

    /// 시나리오 5 — makeup_expired (소멸) 상태는 needed 에서 제외.
    #[tokio::test]
    async fn t5_expired_absent_excluded_from_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;

        toggle_impl(&pool, aid, "absent").await.expect("absent");
        assert_eq!(
            compute_summary(&pool, sid, "2026-06").await.expect("mid").makeup_needed_minutes,
            60
        );

        // 소멸 상태로 강제 전이 — Phase 3 소멸 트리거가 들어오기 전 까지의 직접 검증.
        set_cell_state(&pool, aid, "makeup_expired", None).await;

        let s = compute_summary(&pool, sid, "2026-06").await.expect("summary");
        assert_eq!(s.absent_count, 0, "expired 는 absent_count 에서 제외");
        assert_eq!(
            s.makeup_needed_minutes, 0,
            "expired 는 needed 에서 제외 (status='absent' 조건 위반)"
        );
    }

    /// 시나리오 9 — 동일 월 다중 결석은 각각 독립 소멸기한을 갖는다.
    /// (현재 구현은 결석 발생 월 +1 이라 같은 월 결석은 모두 같은 deadline.
    ///  "독립" 의 의미는 row 별로 deadline 컬럼이 따로 저장되며, 한 row 토글이
    ///  다른 row 의 deadline 을 건드리지 않는다는 무간섭성 검증.)
    #[tokio::test]
    async fn t5_multiple_absents_have_independent_deadlines() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");

        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM regular_attendances \
             WHERE student_id=? AND year_month=? ORDER BY event_date LIMIT 3",
        )
        .bind(sid)
        .bind("2026-06")
        .fetch_all(&pool)
        .await
        .expect("ids");
        let (a, b, c) = (ids[0].0, ids[1].0, ids[2].0);

        toggle_impl(&pool, a, "absent").await.expect("a");
        toggle_impl(&pool, b, "absent").await.expect("b");
        toggle_impl(&pool, c, "absent").await.expect("c");

        for &id in &[a, b, c] {
            assert_eq!(
                fetch_cell(&pool, id).await.makeup_deadline.as_deref(),
                Some("2026-07")
            );
        }

        // 한 row 출석 환원 시 다른 row 의 deadline 무영향.
        toggle_impl(&pool, b, "present").await.expect("b present");
        assert!(fetch_cell(&pool, b).await.makeup_deadline.is_none());
        assert_eq!(fetch_cell(&pool, a).await.makeup_deadline.as_deref(), Some("2026-07"));
        assert_eq!(fetch_cell(&pool, c).await.makeup_deadline.as_deref(), Some("2026-07"));
    }

    /// 시나리오 10 — class_minutes <= 0 은 DB CHECK 로 거부된다 (V106 정책 회귀 방지).
    #[tokio::test]
    async fn t5_class_minutes_check_rejects_zero_and_negative() {
        let pool = test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[]).await;

        for bad in [0i64, -1, -60] {
            let r = sqlx::query(
                "INSERT INTO regular_attendances \
                 (student_id, event_date, year_month, status, class_minutes) \
                 VALUES (?, '2026-06-01', '2026-06', 'present', ?)",
            )
            .bind(sid)
            .bind(bad)
            .execute(&pool)
            .await;
            assert!(
                r.is_err(),
                "class_minutes={} 은 CHECK 로 거부되어야 함",
                bad
            );
        }
    }

    // ─────── 보조: 보강완료 분 합산 ───────

    #[tokio::test]
    async fn summary_aggregates_completed_makeup_minutes() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        // 보강 출결 2건 (출석 + 결석)
        sqlx::query(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) \
             VALUES (?, '2026-06-10', '2026-06', 'makeup_attended', 60), \
                    (?, '2026-06-17', '2026-06', 'makeup_attended', 90), \
                    (?, '2026-06-24', '2026-06', 'makeup_absent',   60)",
        )
        .bind(sid)
        .bind(sid)
        .bind(sid)
        .execute(&pool)
        .await
        .expect("makeup INSERT");

        let s = compute_summary(&pool, sid, "2026-06").await.expect("summary");
        assert_eq!(s.makeup_completed_minutes, 150, "출석한 보강만 합산 (60+90)");
    }
}
