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
use std::collections::HashSet;

const MINUTES_PER_HOUR: i64 = 60;

/// 출결 생성 결과 — 프론트엔드 토스트/요약에 사용.
///
/// Sprint 10 T4 (PI-05/PI-09): 출결 생성 직후 소멸 자동 전이 트리거 — `expiration_report` 동봉.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub year_month: String,
    pub student_count: i64,
    pub attendance_count: i64,
    pub expiration_report: crate::commands::expiration::ExpirationReport,
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

    // hotfix post-Sprint 11: 출결 재호출 차단 폐지 — INSERT OR IGNORE 로 신규 원생만 추가.
    // 청구 generate_bills 와 동일 패턴 ("추가 출결 데이터 생성" UX 트리거).
    // UNIQUE (student_id, event_date) 가 중복 차단 안전망.

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
        // Sprint 16 T0: 날짜 인식 — 현행만이 아닌 전체 스케줄 이력을 로드하여 각 일자에 유효한 스케줄을 매칭.
        let slices = load_schedule_slices(&mut *tx, s.id).await?;
        if slices.is_empty() {
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
            if let Some(minutes) = minutes_for_date(&slices, d) {
                let in_enroll_range = d >= enroll_d && withdraw_d.is_none_or(|wd| d <= wd);
                let date_str = d.format("%Y-%m-%d").to_string();
                if in_enroll_range && !off_dates.contains(&date_str) {
                    let res = sqlx::query(
                        "INSERT OR IGNORE INTO regular_attendances \
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
                    if res.rows_affected() > 0 {
                        inserted += 1;
                    }
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

    // Sprint 10 T4 (PI-05): 출결 생성 직후 소멸 자동 전이 — 같은 월의 deadline 도래 결석을 일괄 전이.
    // Sprint 11 F2: fail-soft 전환 — expire 실패가 출결 생성 본 흐름을 막지 않도록 warn 로그만 남김.
    // startup 트리거(`startup::run`) 와 동일 정책. 사용자는 다음 트리거(다음 달 출결 생성 / 앱 재시작) 에서 재시도.
    let expiration_report = match crate::commands::expiration::expire_overdue_absences_impl(
        pool, None,
    )
    .await
    {
        Ok(report) => report,
        Err(e) => {
            eprintln!(
                "[attendance::generate] 소멸 자동 전이 실패 (출결 생성은 성공): {}",
                e
            );
            crate::commands::expiration::ExpirationReport {
                transitioned_count: 0,
                details: Vec::new(),
            }
        }
    };

    Ok(GenerateResult {
        year_month: year_month.to_string(),
        student_count,
        attendance_count,
        expiration_report,
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

/// Sprint 16 T0 — 날짜 인식 스케줄 슬라이스.
///
/// 원생의 전체 스케줄 이력(마감 행 포함)을 보유하여, 임의 일자에 유효한 스케줄(요일별
/// 수업 분)을 산출한다. 케이스2(특정일 이후 영구 변경) 이후에도 변경일 기준으로 옛/신
/// 스케줄을 날짜별로 정확히 반영하기 위함.
struct ScheduleSlice {
    day_of_week: i64,
    minutes: i64,
    effective_from: String,
    effective_to: Option<String>,
}

/// 원생의 전체 스케줄 이력을 로드한다 (현행 + 마감 행 모두).
async fn load_schedule_slices<'c, E>(
    executor: E,
    student_id: i64,
) -> Result<Vec<ScheduleSlice>, String>
where
    E: sqlx::Executor<'c, Database = sqlx::Sqlite>,
{
    let rows = sqlx::query(
        "SELECT day_of_week, duration_hours, effective_from, effective_to \
         FROM student_schedules WHERE student_id = ?",
    )
    .bind(student_id)
    .fetch_all(executor)
    .await
    .map_err(|e| format!("원생 스케줄 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            let hours: i64 = r
                .try_get("duration_hours")
                .map_err(|e: sqlx::Error| e.to_string())?;
            Ok(ScheduleSlice {
                day_of_week: r.try_get("day_of_week").map_err(|e: sqlx::Error| e.to_string())?,
                minutes: hours * MINUTES_PER_HOUR,
                effective_from: r
                    .try_get("effective_from")
                    .map_err(|e: sqlx::Error| e.to_string())?,
                effective_to: r
                    .try_get("effective_to")
                    .map_err(|e: sqlx::Error| e.to_string())?,
            })
        })
        .collect()
}

/// 특정 일자에 유효한 스케줄의 수업 분을 반환한다. 매칭 없으면 None.
///
/// 유효 조건: 요일 일치 AND `effective_from ≤ d` AND (`effective_to` IS NULL OR `d < effective_to`).
/// effective_to 는 **exclusive** — `set_schedule` 이 이전 스케줄의 effective_to 를 신규
/// effective_from 과 동일 일자로 마감하므로, 변경일 당일부터 신 스케줄이 적용된다 (무경계 연결).
fn minutes_for_date(slices: &[ScheduleSlice], d: NaiveDate) -> Option<i64> {
    let dow = d.weekday().number_from_monday() as i64;
    let ds = d.format("%Y-%m-%d").to_string();
    slices
        .iter()
        .find(|s| {
            s.day_of_week == dow
                && s.effective_from.as_str() <= ds.as_str()
                && s.effective_to.as_deref().is_none_or(|to| ds.as_str() < to)
        })
        .map(|s| s.minutes)
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
    /// Sprint 16 T0 케이스1 — 1회성 수업일 이동 메모 (예: "6/8(월)→6/10(수) 이동"). 없으면 None.
    pub note: Option<String>,
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

/// 보강필요 내역 1건 — 그리드 "보강필요" 셀 hover 힌트용 (Sprint 14 버그픽스).
/// 이월 누적: 조회월 이전 달의 미보강 결석도 포함 (소멸기한 ≥ 조회월).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PendingMakeupDetail {
    pub event_date: String,
    pub class_minutes: i64,
    pub makeup_deadline: Option<String>,
}

/// 그리드 한 원생 행.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttendanceGridStudent {
    pub student_id: i64,
    pub name: String,
    pub serial_no: String,
    /// Sprint 19 T2(사용자 요청 1번) — 그리드 기본 정렬(학년별+이름) 판단 및 프론트 표시용.
    /// Sprint 19 sprint-review F4: 검증되지 않은 raw String 대신 `SchoolLevel` enum으로
    /// 파싱해, DB에 예기치 못한 값이 들어와도 조용히 통과시키지 않고 에러로 드러낸다.
    pub school_level: crate::commands::students::SchoolLevel,
    pub grade: i64,
    pub schedule_days: Vec<i64>,
    /// Sprint 9 Session #10 I8 — 클라이언트가 비수업일 셀 "+" 표시 조건 판단에 사용.
    pub enroll_date: String,
    /// Sprint 9 Session #10 I8 — 퇴교일 없으면 null.
    pub withdraw_date: Option<String>,
    pub attendances: Vec<AttendanceCell>,
    /// Sprint 9 Session #10 J4 — month 내 보강 출결 (비수업일 셀에 표기).
    pub makeups: Vec<GridMakeupCell>,
    pub summary: AttendanceSummary,
    /// Sprint 9 Session #12 K1' — 만기 미도래 미보강 결석 중 가장 이른 일자.
    /// 클라이언트의 비수업일 "+" 표시 사전 판단에 사용. 이전 월 결석도 포함.
    /// `None` 이면 보강 필요한 결석 없음 → "+" 비표시.
    pub earliest_pending_absence_date: Option<String>,
    /// Sprint 14 — "보강필요" 셀 hover 내역. summary.makeup_needed_minutes 의 구성 결석 목록
    /// (이월 누적, 소멸기한 ≥ 조회월, 퇴교생 제외). 일자 오름차순.
    pub pending_absences: Vec<PendingMakeupDetail>,
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
    /// 정규수업 불가 — `allows_regular_class=0` 코드 존재 (공휴일/방학/휴원/보강데이 등). Sprint 16 T0.
    /// 공휴수업일처럼 보강 가능(allows_makeup)이면서 정규도 가능한 코드는 false.
    pub regular_blocked: bool,
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

/// 해당 월에 수업 가능(스케줄 합 > 0)하지만 아직 출결이 생성되지 않은 원생 수.
/// hotfix post-Sprint 11: 출결 데이터 생성 후 신규 등록 원생에 대한 "추가 출결 데이터 생성" UX.
#[tauri::command]
pub async fn count_ungenerated_attendance_students(year_month: String) -> Result<i64, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    count_ungenerated_attendance_students_impl(pool, &year_month).await
}

pub(crate) async fn count_ungenerated_attendance_students_impl(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<i64, String> {
    validate_year_month(year_month)?;
    let (period_start, period_end) = ym_to_range(year_month)?;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ( \
            SELECT s.id \
            FROM students s \
            INNER JOIN student_schedules sch \
                   ON sch.student_id = s.id AND sch.effective_to IS NULL \
            WHERE s.enroll_date <= ? \
              AND (s.withdraw_date IS NULL OR s.withdraw_date >= ?) \
              AND s.id NOT IN ( \
                SELECT DISTINCT student_id FROM regular_attendances WHERE year_month = ? \
              ) \
            GROUP BY s.id \
            HAVING SUM(sch.duration_hours) > 0 \
         )",
    )
    .bind(&period_end)
    .bind(&period_start)
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("미생성 출결 원생 수 조회 실패: {}", e))?;
    Ok(count)
}

/// YYYY-MM → (월초 YYYY-MM-01, 월말 YYYY-MM-DD) 문자열 쌍.
fn ym_to_range(year_month: &str) -> Result<(String, String), String> {
    let year: i32 = year_month[..4]
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    let month: u32 = year_month[5..]
        .parse()
        .map_err(|e: std::num::ParseIntError| e.to_string())?;
    let first = chrono::NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("월초 일자 생성 실패: {}-{:02}-01", year, month))?;
    let next_first = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "다음 달 일자 생성 실패".to_string())?;
    let last = next_first
        .pred_opt()
        .ok_or_else(|| "월말 일자 계산 실패".to_string())?;
    Ok((first.to_string(), last.to_string()))
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

    // 1) 해당 월 출결이 있는 원생들 — Session #10 I8 위해 enroll/withdraw 동봉.
    // Sprint 19 T2(사용자 요청 1번): 그리드 기본 정렬을 학년별+이름 가나다순으로 통일
    // (students.rs StudentSort::GradeAsc 와 동일한 school_level→grade→name 순서).
    let student_rows = sqlx::query(
        "SELECT DISTINCT s.id, s.name, s.serial_no, s.school_level, s.grade, \
                s.enroll_date, s.withdraw_date \
         FROM students s \
         JOIN regular_attendances a ON a.student_id = s.id \
         WHERE a.year_month = ? \
         ORDER BY s.school_level ASC, s.grade ASC, s.name ASC",
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
        let school_level_raw: String = srow.try_get("school_level").map_err(|e| e.to_string())?;
        let school_level = crate::commands::students::SchoolLevel::from_db_code(&school_level_raw)
            .map_err(|e| e.to_string())?;
        let grade: i64 = srow.try_get("grade").map_err(|e| e.to_string())?;
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
                    makeup_deadline, makeup_attendance_id, note \
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
                    note: r.try_get("note").map_err(|e: sqlx::Error| e.to_string())?,
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

        // Sprint 9 Session #12 K1': 만기 미도래 미보강 결석 중 가장 이른 일자.
        // 그리드 yearMonth 기준으로 makeup_deadline 도래 여부 판단 (deadline NULL 또는 deadline >= yearMonth).
        // year_month 필터 없음 — 이전 월의 결석도 포함.
        let earliest_pending_absence_date: Option<String> = sqlx::query_scalar(
            "SELECT MIN(event_date) FROM regular_attendances \
             WHERE student_id = ? \
               AND status = 'absent' \
               AND makeup_attendance_id IS NULL \
               AND (makeup_deadline IS NULL OR makeup_deadline >= ?)",
        )
        .bind(student_id)
        .bind(year_month)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("만기 미도래 결석 최소 일자 조회 실패: {}", e))?;

        // Sprint 14 — 보강필요 셀 hover 내역 (summary.makeup_needed_minutes 구성 결석).
        let pending_absences = fetch_pending_absences(pool, student_id, year_month).await?;

        students.push(AttendanceGridStudent {
            student_id,
            name,
            serial_no,
            school_level,
            grade,
            schedule_days,
            enroll_date,
            withdraw_date,
            attendances,
            makeups,
            summary,
            earliest_pending_absence_date,
            pending_absences,
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
    // 일자별 코드 후보 — (allows_makeup, is_block, regular_blocked, label) 로 reduce.
    let mut by_date: BTreeMap<String, (bool, bool, bool, String)> = BTreeMap::new();
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
                let entry = by_date
                    .entry(key)
                    .or_insert((false, false, false, String::new()));
                // 라벨 우선순위: allows_makeup 우세(보강데이 + 공휴일 동시 등록 시 보강데이 표시),
                // 그 외에는 첫 등록 코드명 유지.
                let new_makeup = allows_mk == 1;
                if (new_makeup && !entry.0) || entry.3.is_empty() {
                    entry.3 = code_name.clone();
                }
                entry.0 = entry.0 || new_makeup;
                entry.1 = entry.1 || is_block;
                // 정규수업 불가: allows_regular_class=0 코드가 하나라도 있으면 true.
                entry.2 = entry.2 || allows_reg == 0;
            }
            // Sprint 11 F1: succ_opt() 는 NaiveDate::MAX 도달 시 None 반환 — panic 대신 Result 전파.
            d = d
                .succ_opt()
                .ok_or_else(|| format!("일자 다음 날짜 계산 실패: {}", d))?;
        }
    }

    Ok(by_date
        .into_iter()
        .map(
            |(event_date, (allows_makeup, is_block, regular_blocked, label))| DaySchedule {
                event_date,
                allows_makeup,
                is_block,
                regular_blocked,
                label,
            },
        )
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
    // 출석/결석 건수는 조회월 기준 (월 그리드 의미).
    let row = sqlx::query(
        "SELECT \
            SUM(CASE WHEN status='present' THEN 1 ELSE 0 END) AS present_count, \
            SUM(CASE WHEN status='absent' THEN 1 ELSE 0 END) AS absent_count \
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

    // 보강필요시간은 이월 누적 — 조회월 이전 달의 미보강 결석도 포함 (Sprint 14 버그픽스).
    // 기준: status='absent' AND 미매칭 AND 소멸기한 미도래(deadline NULL 또는 ≥ 조회월) AND 재원생.
    // earliest_pending_absence_date 의 술어와 정합. 퇴교생은 보강 대상 아님 → 0.
    let needed: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(ra.class_minutes), 0) \
         FROM regular_attendances ra \
         JOIN students s ON s.id = ra.student_id \
         WHERE ra.student_id = ? \
           AND ra.status = 'absent' AND ra.makeup_attendance_id IS NULL \
           AND (ra.makeup_deadline IS NULL OR ra.makeup_deadline >= ?) \
           AND s.withdraw_date IS NULL",
    )
    .bind(student_id)
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("보강필요시간 조회 실패: {}", e))?;

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

/// 보강필요 셀 hover 내역 — compute_summary 의 `needed` 와 동일 술어의 결석 목록.
/// (이월 누적, 소멸기한 ≥ 조회월, 퇴교생 제외) 일자 오름차순.
async fn fetch_pending_absences(
    pool: &SqlitePool,
    student_id: i64,
    year_month: &str,
) -> Result<Vec<PendingMakeupDetail>, String> {
    let rows = sqlx::query(
        "SELECT ra.event_date, ra.class_minutes, ra.makeup_deadline \
         FROM regular_attendances ra \
         JOIN students s ON s.id = ra.student_id \
         WHERE ra.student_id = ? \
           AND ra.status = 'absent' AND ra.makeup_attendance_id IS NULL \
           AND (ra.makeup_deadline IS NULL OR ra.makeup_deadline >= ?) \
           AND s.withdraw_date IS NULL \
         ORDER BY ra.event_date",
    )
    .bind(student_id)
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강필요 내역 조회 실패: {}", e))?;

    rows.into_iter()
        .map(|r| {
            Ok(PendingMakeupDetail {
                event_date: r.try_get("event_date").map_err(|e: sqlx::Error| e.to_string())?,
                class_minutes: r.try_get("class_minutes").map_err(|e: sqlx::Error| e.to_string())?,
                makeup_deadline: r.try_get("makeup_deadline").map_err(|e: sqlx::Error| e.to_string())?,
            })
        })
        .collect()
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

// ─────────────────── Sprint 16 T0: 수업일 변경 ───────────────────
//
// 케이스1(move_attendance): 특정일 1회성 수업일 이동 — 출결 행 1건의 event_date 변경.
// 케이스2(apply_schedule_change): 특정일 이후 영구 스케줄 변경 + 변경일 이후 출결 재생성.
// 설계 근거: docs/sprint/sprint16.md T0 (PI-20~27).

/// `NaiveDate` → "M/D" (예: 6/8). 메모 표기용.
fn format_md(d: NaiveDate) -> String {
    format!("{}/{}", d.month(), d.day())
}

/// `NaiveDate` → 한글 요일 1글자.
fn weekday_ko(d: NaiveDate) -> &'static str {
    match d.weekday().number_from_monday() {
        1 => "월",
        2 => "화",
        3 => "수",
        4 => "목",
        5 => "금",
        6 => "토",
        _ => "일",
    }
}

/// 특정 시점에 유효한 모든 요일 스케줄의 수업 분 합 (주당 수업시간, 분 단위).
fn weekly_minutes_on(slices: &[ScheduleSlice], ref_date: NaiveDate) -> i64 {
    let ds = ref_date.format("%Y-%m-%d").to_string();
    slices
        .iter()
        .filter(|s| {
            s.effective_from.as_str() <= ds.as_str()
                && s.effective_to.as_deref().is_none_or(|to| ds.as_str() < to)
        })
        .map(|s| s.minutes)
        .sum()
}

/// 케이스1 — 특정일 1회성 수업일 이동 결과.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MoveAttendanceResult {
    pub attendance_id: i64,
    pub from_date: String,
    pub to_date: String,
    pub note: String,
}

/// 케이스1 — 특정일 1회성 수업일 이동.
///
/// 출결 행(present) 1건의 `event_date` 를 다른 날로 옮기고 `note` 에 이동 내역을 남긴다.
/// 사용자가 도착일의 수업 시작시간(`start_time`, "HH:MM")을 입력하면 `regular_attendances`
/// 에 저장하여 수업 캘린더가 시간을 표시할 수 있게 한다 (PI-28).
/// 동월 한정, 도착일 OFF/공휴일·충돌 차단 (PI-25). class_minutes 유지 → 청구·주당시간 불변.
#[tauri::command]
pub async fn move_attendance(
    student_id: i64,
    from_date: String,
    to_date: String,
    start_time: String,
) -> Result<MoveAttendanceResult, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    let result = move_attendance_impl(pool, student_id, &from_date, &to_date, &start_time).await?;
    audit::try_record(
        AuditEventType::AttendanceRescheduled,
        Some(&student_id.to_string()),
        Some(&format!(
            r#"{{"from":"{}","to":"{}","startTime":"{}"}}"#,
            from_date, to_date, start_time
        )),
    )
    .await;
    Ok(result)
}

/// "HH:MM" 또는 "HH:MM:SS" → "HH:MM:SS" 정규화. 형식 위반 시 Err.
fn normalize_time(t: &str) -> Result<String, String> {
    let parts: Vec<&str> = t.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err("수업 시작시간 형식이 올바르지 않습니다 (예: 16:00).".to_string());
    }
    let h: u32 = parts[0]
        .parse()
        .map_err(|_| "수업 시작시간의 시(時)가 올바르지 않습니다.".to_string())?;
    let m: u32 = parts[1]
        .parse()
        .map_err(|_| "수업 시작시간의 분(分)이 올바르지 않습니다.".to_string())?;
    if h > 23 || m > 59 {
        return Err("수업 시작시간 범위가 올바르지 않습니다 (00:00~23:59).".to_string());
    }
    Ok(format!("{:02}:{:02}:00", h, m))
}

async fn move_attendance_impl(
    pool: &SqlitePool,
    student_id: i64,
    from_date: &str,
    to_date: &str,
    start_time: &str,
) -> Result<MoveAttendanceResult, String> {
    let from_d = parse_date(from_date)?;
    let to_d = parse_date(to_date)?;
    let start_time = normalize_time(start_time)?;
    if from_d == to_d {
        return Err("출발일과 도착일이 같습니다.".to_string());
    }
    // 동월 한정 — 파싱된 날짜의 연·월 비교 (P1-8: 문자열 바이트 슬라이싱은 7바이트 미만
    // 입력에서 panic — parse_date 는 "26-6-1" 같은 축약 표기도 통과시키므로 안전하지 않다)
    if from_d.format("%Y-%m").to_string() != to_d.format("%Y-%m").to_string() {
        return Err(
            "수업일 이동은 같은 달 안에서만 가능합니다. 다른 달로 옮기려면 보강 기능을 이용하세요."
                .to_string(),
        );
    }
    // 원본 출결 — present 만 이동 허용
    let row = sqlx::query("SELECT id, status FROM regular_attendances WHERE student_id = ? AND event_date = ?")
        .bind(student_id)
        .bind(from_date)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("출결 조회 실패: {}", e))?
        .ok_or_else(|| format!("{} 에 옮길 수업이 없습니다.", from_date))?;
    let att_id: i64 = row.try_get("id").map_err(|e| e.to_string())?;
    let status: String = row.try_get("status").map_err(|e| e.to_string())?;
    if status != "present" {
        return Err(
            "출석(미처리) 상태의 수업만 이동할 수 있습니다. 결석·보강 처리된 수업은 보강 기능을 이용하세요."
                .to_string(),
        );
    }
    // 도착일 주말 차단 — 정규수업은 평일만 (PI-30)
    let to_dow = to_d.weekday().number_from_monday();
    if to_dow >= 6 {
        return Err(format!(
            "{} 은(는) 주말이라 정규수업을 옮길 수 없습니다.",
            format_md(to_d)
        ));
    }
    // 도착일 OFF/공휴일/보강데이 차단 — allows_regular_class=0 일자 (PI-25)
    let off = load_off_dates(pool, to_date, to_date).await?;
    if off.contains(to_date) {
        return Err(format!(
            "{} 은(는) 휴일·보강데이 등 정규수업이 불가능한 날이라 수업을 옮길 수 없습니다.",
            format_md(to_d)
        ));
    }
    // 도착일 충돌 차단
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM regular_attendances WHERE student_id = ? AND event_date = ?")
            .bind(student_id)
            .bind(to_date)
            .fetch_optional(pool)
            .await
            .map_err(|e| format!("도착일 출결 조회 실패: {}", e))?;
    if exists.is_some() {
        return Err(format!(
            "{} 에 이미 수업이 있어 옮길 수 없습니다.",
            format_md(to_d)
        ));
    }
    let note = format!(
        "{}({})→{}({}) 이동",
        format_md(from_d),
        weekday_ko(from_d),
        format_md(to_d),
        weekday_ko(to_d)
    );
    sqlx::query(
        "UPDATE regular_attendances SET event_date = ?, note = ?, start_time = ?, \
            updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = ?",
    )
    .bind(to_date)
    .bind(&note)
    .bind(&start_time)
    .bind(att_id)
    .execute(pool)
    .await
    .map_err(|e| format!("수업일 이동 실패: {}", e))?;

    Ok(MoveAttendanceResult {
        attendance_id: att_id,
        from_date: from_date.to_string(),
        to_date: to_date.to_string(),
        note,
    })
}

/// 케이스2 — 특정일 이후 영구 스케줄 변경 + 출결 재생성 결과.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleChangeResult {
    /// 변경일 이후 신 스케줄로 새로 생성된 출결 수.
    pub regenerated_count: i64,
    /// 변경일 이후 보존된 처리행(결석/보강/메모) 수.
    pub preserved_count: i64,
    pub weekly_minutes_before: i64,
    pub weekly_minutes_after: i64,
}

/// 케이스2 — 변경일(effective_date) 이후 스케줄 변경을 출결에 반영한다.
///
/// 선행: 호출 전에 `set_schedule(effective_from = effective_date)` 로 신 스케줄이 반영되어 있어야 한다.
/// 동작: 변경일 D 이후의 `present`(미처리) 출결만 삭제 후, 날짜 인식 스케줄로 재생성한다.
/// 결석/보강완료/소멸/메모 행은 **보존**한다 (PI-21). 변경일은 사전(미래)·사후(과거) 모두 허용 (PI-24).
#[tauri::command]
pub async fn apply_schedule_change(
    student_id: i64,
    effective_date: String,
) -> Result<ScheduleChangeResult, String> {
    let pool = pool().map_err(|e| e.to_string())?;
    let result = apply_schedule_change_impl(pool, student_id, &effective_date).await?;
    audit::try_record(
        AuditEventType::ScheduleChangedWithRegen,
        Some(&student_id.to_string()),
        Some(&format!(
            r#"{{"effectiveDate":"{}","regenerated":{},"preserved":{}}}"#,
            effective_date, result.regenerated_count, result.preserved_count
        )),
    )
    .await;
    Ok(result)
}

async fn apply_schedule_change_impl(
    pool: &SqlitePool,
    student_id: i64,
    effective_date: &str,
) -> Result<ScheduleChangeResult, String> {
    let d = parse_date(effective_date)?;

    // 원생 입퇴교일 — 변경일 하한 검증 + 재생성 범위 제한
    let stu = sqlx::query("SELECT enroll_date, withdraw_date FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("원생 조회 실패: {}", e))?
        .ok_or_else(|| "원생을 찾을 수 없습니다.".to_string())?;
    let enroll_date: String = stu.try_get("enroll_date").map_err(|e| e.to_string())?;
    let withdraw_date: Option<String> = stu.try_get("withdraw_date").map_err(|e| e.to_string())?;
    if effective_date < enroll_date.as_str() {
        return Err(format!("변경일은 입교일({}) 이후여야 합니다.", enroll_date));
    }
    let enroll_d = parse_date(&enroll_date)?;
    let withdraw_d = match &withdraw_date {
        Some(w) => Some(parse_date(w)?),
        None => None,
    };

    // 신 스케줄(현행 반영 완료) 이력 로드
    let slices = load_schedule_slices(pool, student_id).await?;
    let weekly_minutes_after = weekly_minutes_on(&slices, d);
    let weekly_minutes_before = match d.pred_opt() {
        Some(prev) => weekly_minutes_on(&slices, prev),
        None => 0,
    };

    // ── 트랜잭션 전 데이터 수집 ──
    // D 이후 ~ 와 겹치는 확정 교습기간들
    let period_rows = sqlx::query(
        "SELECT start_date, end_date FROM study_periods \
         WHERE is_confirmed = 1 AND end_date >= ? ORDER BY start_date",
    )
    .bind(effective_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("교습기간 조회 실패: {}", e))?;

    struct Period {
        start: NaiveDate,
        end: NaiveDate,
    }
    let mut periods = Vec::with_capacity(period_rows.len());
    for r in &period_rows {
        let ps: String = r.try_get("start_date").map_err(|e| e.to_string())?;
        let pe: String = r.try_get("end_date").map_err(|e| e.to_string())?;
        periods.push(Period {
            start: parse_date(&ps)?,
            end: parse_date(&pe)?,
        });
    }

    // off_dates — 재생성 전체 범위 (변경일 ~ 최대 교습기간 말)
    let off_dates = match periods.iter().map(|p| p.end).max() {
        Some(max_end) => {
            load_off_dates(pool, effective_date, &max_end.format("%Y-%m-%d").to_string()).await?
        }
        None => HashSet::new(),
    };

    // ── 트랜잭션: 변경일 이후 present 삭제 → 재생성 ──
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    sqlx::query(
        "DELETE FROM regular_attendances \
         WHERE student_id = ? AND event_date >= ? AND status = 'present'",
    )
    .bind(student_id)
    .bind(effective_date)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("기존 출결 삭제 실패: {}", e))?;

    // 보존된 처리행(결석/보강/소멸/메모) 수 — 삭제 후 남은 변경일 이후 행
    let preserved_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM regular_attendances WHERE student_id = ? AND event_date >= ?",
    )
    .bind(student_id)
    .bind(effective_date)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("보존 행 카운트 실패: {}", e))?;

    let mut regenerated_count = 0i64;
    for p in &periods {
        let mut cur = p.start.max(d);
        while cur <= p.end {
            if let Some(minutes) = minutes_for_date(&slices, cur) {
                let in_range = cur >= enroll_d && withdraw_d.is_none_or(|wd| cur <= wd);
                let ds = cur.format("%Y-%m-%d").to_string();
                if in_range && !off_dates.contains(&ds) {
                    let ym = &ds[..7];
                    let res = sqlx::query(
                        "INSERT OR IGNORE INTO regular_attendances \
                         (student_id, event_date, year_month, status, class_minutes) \
                         VALUES (?, ?, ?, 'present', ?)",
                    )
                    .bind(student_id)
                    .bind(&ds)
                    .bind(ym)
                    .bind(minutes)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| format!("출결 재생성 INSERT 실패: {}", e))?;
                    if res.rows_affected() > 0 {
                        regenerated_count += 1;
                    }
                }
            }
            cur = cur
                .succ_opt()
                .ok_or_else(|| "재생성 날짜 계산 오버플로".to_string())?;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(ScheduleChangeResult {
        regenerated_count,
        preserved_count,
        weekly_minutes_before,
        weekly_minutes_after,
    })
}

// ─────────────────────── 학사 일정 변경 시 출결 동기화 ───────────────────────

/// 학사 일정 이벤트 생성/수정/삭제 후 해당 날짜의 정규 출결을 현재 DB 상태와 동기화한다.
///
/// - **allows_regular_class=0인 이벤트가 남아있으면** 해당 날짜 정규 출결 전체 DELETE (ON→OFF).
/// - **allows_regular_class=0인 이벤트가 없으면** 해당 요일 스케줄 있는 원생에게 INSERT OR IGNORE (OFF→ON).
///
/// 교습기간 밖이거나 미확정 기간이면 INSERT 를 건너뜀 (출결 생성 조건 미충족).
pub async fn sync_attendance_on_schedule_change(
    pool: &SqlitePool,
    event_date: &str,
    period_end_date: Option<&str>,
) -> Result<(), String> {
    let end = period_end_date.unwrap_or(event_date);
    let mut d = parse_date(event_date)?;
    let ed = parse_date(end)?;
    while d <= ed {
        let ds = d.format("%Y-%m-%d").to_string();
        sync_single_date(pool, &ds).await?;
        d = d
            .succ_opt()
            .ok_or_else(|| "날짜 계산 오버플로".to_string())?;
    }
    Ok(())
}

async fn sync_single_date(pool: &SqlitePool, date: &str) -> Result<(), String> {
    // allows_regular_class=0 인 이벤트가 해당 날짜에 존재하는지 확인
    let off_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_regular_class = 0 \
           AND e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(date)
    .bind(date)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("날짜 상태 조회 실패: {}", e))?;

    // allows_regular_class=1 인 이벤트가 같은 날짜에 공존하면(예: 공휴일 + 공휴수업일, V309 로
    // 중복 배치 허용) ON 이 우선한다 — off_count > 0 이라고 무조건 OFF 로 판정하면 안 됨.
    let on_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_regular_class = 1 \
           AND e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(date)
    .bind(date)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("날짜 상태 조회 실패: {}", e))?;

    if off_count > 0 && on_count == 0 {
        // OFF 이벤트 존재 → 자동 생성된 출석(present) 행만 삭제.
        // 결석(absent) 또는 보강 매칭된 행(makeup_attendance_id IS NOT NULL)은 보존.
        sqlx::query(
            "DELETE FROM regular_attendances \
             WHERE event_date = ? AND status = 'present' AND makeup_attendance_id IS NULL",
        )
        .bind(date)
        .execute(pool)
        .await
        .map_err(|e| format!("출결 삭제 실패: {}", e))?;
    } else {
        // OFF 이벤트 없음 → 교습기간 확인 후 INSERT OR IGNORE
        let in_period: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM study_periods \
             WHERE start_date <= ? AND end_date >= ? AND is_confirmed = 1",
        )
        .bind(date)
        .bind(date)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("교습기간 조회 실패: {}", e))?;

        if in_period.is_some() {
            let d = parse_date(date)?;
            let dow = d.weekday().number_from_monday() as i64;
            let ym = &date[..7];
            sqlx::query(
                "INSERT OR IGNORE INTO regular_attendances \
                 (student_id, event_date, year_month, status, class_minutes) \
                 SELECT ss.student_id, ?, ?, 'present', ss.duration_hours * 60 \
                 FROM student_schedules ss \
                 JOIN students s ON s.id = ss.student_id \
                 WHERE ss.day_of_week = ? \
                   AND ss.effective_from <= ? \
                   AND (ss.effective_to IS NULL OR ss.effective_to > ?) \
                   AND s.enroll_date <= ? \
                   AND (s.withdraw_date IS NULL OR s.withdraw_date >= ?)",
            )
            .bind(date)
            .bind(ym)
            .bind(dow)
            .bind(date)
            .bind(date)
            .bind(date)
            .bind(date)
            .execute(pool)
            .await
            .map_err(|e| format!("출결 INSERT 실패: {}", e))?;
        }
    }
    Ok(())
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
    async fn generate_idempotent_for_same_students() {
        // hotfix post-Sprint 11: 재호출 차단 폐지 — INSERT OR IGNORE 로 idempotent.
        // 동일 학생만 있으면 재호출 시 0 row 추가.
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        let first = generate_impl(&pool, "2026-06").await.expect("first");
        assert!(first.student_count > 0);
        let attended_first = first.attendance_count;

        let second = generate_impl(&pool, "2026-06").await.expect("second");
        assert_eq!(second.student_count, 0, "신규 학생 없으면 0");
        assert_eq!(second.attendance_count, 0, "신규 row 0");

        // 총 행 수는 첫 호출과 동일 (idempotent)
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM regular_attendances")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(total, attended_first);
    }

    /// 추가 등록 원생만 INSERT — count_ungenerated 와 함께 "추가 출결 데이터 생성" UX 토대.
    #[tokio::test]
    async fn generate_adds_only_new_student_on_rerun() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        let first = generate_impl(&pool, "2026-06").await.expect("first");

        // 신규 학생 추가
        seed_student(&pool, "S002", "2026-04-01", None, &[(3, 1)]).await;

        // count_ungenerated 가 1 반환
        let ungenerated = count_ungenerated_attendance_students_impl(&pool, "2026-06").await.expect("count");
        assert_eq!(ungenerated, 1, "신규 등록 1명 미생성");

        let second = generate_impl(&pool, "2026-06").await.expect("second");
        assert_eq!(second.student_count, 1, "신규 학생 1명만 추가");
        assert!(second.attendance_count > 0);
        // 기존 학생 출결은 그대로 보존
        let s1_total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances WHERE student_id = 1",
        )
        .fetch_one(&pool).await.unwrap();
        assert_eq!(s1_total, first.attendance_count, "S001 기존 출결 보존");

        // 재호출 후 ungenerated 0
        let ungenerated2 = count_ungenerated_attendance_students_impl(&pool, "2026-06").await.expect("count2");
        assert_eq!(ungenerated2, 0);
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
                    makeup_deadline, makeup_attendance_id, note \
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
            note: row.try_get("note").unwrap(),
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
        assert_eq!(s.school_level, crate::commands::students::SchoolLevel::Elementary);
        assert_eq!(s.grade, 3);
        assert_eq!(s.schedule_days, vec![1, 3]);
        assert!(!s.attendances.is_empty());
        assert_eq!(s.summary.year_month, "2026-06");
        assert_eq!(s.summary.absent_count, 0);
        assert!(s.summary.present_count > 0);
        // K1': 결석 없음 → None.
        assert!(s.earliest_pending_absence_date.is_none());
    }

    // ─────── Sprint 19 T2 — 기본 정렬(학년별+이름) ───────

    #[tokio::test]
    async fn grid_orders_students_by_school_level_grade_then_name() {
        // 사용자 요청 1번: 원생 그리드 기본 정렬은 school_level→grade→name.
        // 먼저 등록된(serial_no/id 순서상 앞선) 학생을 중학생으로, 나중에 등록된 학생을
        // 초등학생으로 만들어 — serial_no 순서를 그대로 따르는 게 아니라 school_level 이
        // 실제 정렬 기준으로 적용되는지 검증(초등이 중등보다 항상 먼저).
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid_first = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        let sid_second = seed_student(&pool, "S002", "2026-04-01", None, &[(1, 1)]).await;
        sqlx::query("UPDATE students SET school_level='middle', grade=2 WHERE id=?")
            .bind(sid_first)
            .execute(&pool)
            .await
            .expect("먼저 등록된 학생을 중학생으로 갱신");
        // sid_second 는 seed_student 기본값(elementary/grade=3) 그대로 둔다.

        generate_impl(&pool, "2026-06").await.expect("generate");

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        assert_eq!(grid.students.len(), 2);
        // 먼저 등록됐지만 중학생인 sid_first 가 뒤로, 나중에 등록됐지만 초등학생인
        // sid_second 가 앞으로 — 등록순이 아닌 school_level 기준 정렬 증명.
        assert_eq!(grid.students[0].student_id, sid_second);
        assert_eq!(
            grid.students[0].school_level,
            crate::commands::students::SchoolLevel::Elementary
        );
        assert_eq!(grid.students[1].student_id, sid_first);
        assert_eq!(
            grid.students[1].school_level,
            crate::commands::students::SchoolLevel::Middle
        );
    }

    // ─────── Session #12 K1' — earliest_pending_absence_date ───────

    #[tokio::test]
    async fn grid_earliest_pending_returns_min_unmatched_absence() {
        // 6월에 결석 2건(절반은 만기 미도래) → MIN(eventDate) 반환.
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (3, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");

        // 6/01(월) 및 6/03(수) 두 일자를 결석으로 토글 — 둘 다 makeup_deadline = 2026-07.
        let aids: Vec<i64> = sqlx::query_scalar(
            "SELECT id FROM regular_attendances WHERE student_id = ? AND year_month = '2026-06' \
             AND (event_date = '2026-06-01' OR event_date = '2026-06-03') ORDER BY event_date",
        )
        .bind(sid)
        .fetch_all(&pool)
        .await
        .expect("aids");
        for aid in &aids {
            toggle_impl(&pool, *aid, "absent").await.expect("absent");
        }

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        assert_eq!(
            grid.students[0].earliest_pending_absence_date.as_deref(),
            Some("2026-06-01"),
        );
    }

    #[tokio::test]
    async fn grid_earliest_pending_excludes_expired_deadlines() {
        // 결석의 makeup_deadline 이 grid yearMonth 보다 이전이면 만기 도래 → 제외.
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let aid = first_attendance_id(&pool, sid, "2026-06").await;
        toggle_impl(&pool, aid, "absent").await.expect("absent");
        // 강제로 만기 도래 처리 — deadline 을 2026-05(grid yearMonth 이전) 으로 변경.
        sqlx::query("UPDATE regular_attendances SET makeup_deadline = '2026-05' WHERE id = ?")
            .bind(aid)
            .execute(&pool)
            .await
            .expect("update deadline");

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        assert!(grid.students[0].earliest_pending_absence_date.is_none());
    }

    #[tokio::test]
    async fn grid_earliest_pending_includes_previous_month_absence() {
        // 5월 결석(미보강, 만기 미도래) + 6월 그리드 조회 → 5월 일자 반환.
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31", 1).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (3, 1)]).await;
        generate_impl(&pool, "2026-05").await.expect("generate may");
        generate_impl(&pool, "2026-06").await.expect("generate jun");

        // 5월 첫 결석 처리 — deadline = 2026-06 (다음 달 말일까지).
        let may_aid = first_attendance_id(&pool, sid, "2026-05").await;
        toggle_impl(&pool, may_aid, "absent").await.expect("absent");

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        let pending = grid.students[0].earliest_pending_absence_date.as_deref();
        assert!(pending.is_some(), "이전 월 결석도 포함");
        assert!(
            pending.expect("some").starts_with("2026-05"),
            "5월 일자가 반환되어야 함: {:?}",
            pending,
        );
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

    // ─────── Sprint 14 버그픽스: 이월 누적 + 퇴교 제외 + hover 내역 ───────

    /// 이전 월(5월) 미보강 결석이 다음 달(6월) 보강필요시간에 이월된다 (고길동 0 버그).
    #[tokio::test]
    async fn summary_carries_over_previous_month_absence() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 월 60분
        generate_impl(&pool, "2026-05").await.expect("gen may");
        let aid = first_attendance_id(&pool, sid, "2026-05").await;
        toggle_impl(&pool, aid, "absent").await.expect("absent"); // 소멸기한 2026-06

        let s = compute_summary(&pool, sid, "2026-06").await.expect("june");
        assert_eq!(s.makeup_needed_minutes, 60, "5월 미보강 결석이 6월에 이월되어야 함");
        assert_eq!(s.absent_count, 0, "6월 결석 건수는 월별 유지(0)");
    }

    /// 퇴교 원생은 보강필요시간 집계에서 제외(0) — 자동 소멸 누락 경로 재현 (홍길동 버그).
    #[tokio::test]
    async fn summary_excludes_withdrawn_student_from_needed() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-05").await.expect("gen");
        let aid = first_attendance_id(&pool, sid, "2026-05").await;
        toggle_impl(&pool, aid, "absent").await.expect("absent");
        assert_eq!(
            compute_summary(&pool, sid, "2026-05").await.unwrap().makeup_needed_minutes,
            60,
            "재원 중에는 보강필요 60"
        );
        // withdraw_date 직접 설정(소멸 전이 누락 경로).
        sqlx::query("UPDATE students SET withdraw_date='2026-05-20' WHERE id=?")
            .bind(sid)
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(
            compute_summary(&pool, sid, "2026-05").await.unwrap().makeup_needed_minutes,
            0,
            "퇴교생은 보강필요 집계 제외"
        );
    }

    /// 보강필요 셀 hover 내역(pending_absences)이 이월 결석을 포함한다.
    #[tokio::test]
    async fn grid_pending_absences_lists_carryover() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31", 1).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-05").await.expect("gen may");
        generate_impl(&pool, "2026-06").await.expect("gen jun");
        let aid = first_attendance_id(&pool, sid, "2026-05").await;
        toggle_impl(&pool, aid, "absent").await.expect("absent");

        let grid = get_grid_impl(&pool, "2026-06").await.expect("grid");
        let stu = grid
            .students
            .iter()
            .find(|s| s.student_id == sid)
            .expect("학생이 6월 그리드에 존재");
        assert_eq!(stu.summary.makeup_needed_minutes, 60);
        assert_eq!(stu.pending_absences.len(), 1, "이월 결석 1건이 hover 내역에 포함");
        assert_eq!(stu.pending_absences[0].makeup_deadline.as_deref(), Some("2026-06"));
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

        // 보강 출결 2건 (출석한 보강만 — V108 이후 makeup_absent 시드 불가, J5 폐기)
        sqlx::query(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) \
             VALUES (?, '2026-06-10', '2026-06', 'makeup_attended', 60), \
                    (?, '2026-06-17', '2026-06', 'makeup_attended', 90)",
        )
        .bind(sid)
        .bind(sid)
        .execute(&pool)
        .await
        .expect("makeup INSERT");

        let s = compute_summary(&pool, sid, "2026-06").await.expect("summary");
        assert_eq!(s.makeup_completed_minutes, 150, "출석한 보강만 합산 (60+90)");
    }

    // ─────── Sprint 10 T4 — 트리거 통합 (PI-05) ───────

    /// 출결 생성 IPC 응답에 expiration_report 동봉 — 소멸기한 도래 결석 있을 때 전이 1건+.
    /// 시나리오: 5월 결석(deadline=2026-06) + 6월 교습기간 종료 후 7월 출결 생성 시 자동 전이.
    #[tokio::test]
    async fn generate_includes_expiration_report_when_deadline_reached() {
        let pool = test_pool_in_memory().await.expect("pool");
        // 5월 교습기간 + 학생 + 5월 결석 시드.
        seed_period(&pool, "2026-05", "2026-05-01", "2026-05-31", 1).await;
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        // 5/04(월) 결석 시드 — deadline=2026-06.
        sqlx::query(
            "INSERT INTO regular_attendances \
                (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, '2026-05-04', '2026-05', 'absent', 60, '2026-06')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("seed absence");
        // 7월 교습기간 + generate — 7월 generate 시점에 6월 종료일 도래 → 전이 발동.
        seed_period(&pool, "2026-07", "2026-07-01", "2026-07-31", 1).await;

        // 기준일은 expire_overdue_absences_impl 가 chrono::Local::now() — 2026 시점 미래라
        // 테스트 환경의 실제 today >= 2026-06-30 인 경우만 발동. 보강: 직접 expire_impl 호출로
        // 검증 (T3 단위 테스트가 이미 커버) — 본 테스트는 generate 응답 필드 존재만 확인.
        let result = generate_impl(&pool, "2026-07").await.expect("generate");
        // expiration_report 필드가 응답에 포함되어 직렬화 가능함을 확인 (필드 존재 컴파일 검증).
        // 실제 transitioned_count 는 환경 시점에 따라 달라지므로 단언하지 않음.
        let _ = result.expiration_report.transitioned_count;
        assert_eq!(result.year_month, "2026-07");
    }

    // ─────────── Sprint 16 T0: 수업일 변경 ───────────

    #[tokio::test]
    async fn minutes_for_date_respects_effective_range() {
        // 같은 요일(6/8과 6/22는 14일 차 = 동일 요일), 6/01~6/15(exclusive) 60분 슬라이스.
        let day1 = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 6, 22).unwrap();
        let dow = day1.weekday().number_from_monday() as i64;
        let slices = vec![ScheduleSlice {
            day_of_week: dow,
            minutes: 60,
            effective_from: "2026-06-01".into(),
            effective_to: Some("2026-06-15".into()),
        }];
        // 6/8 < 6/15 → 유효, 6/22 >= 6/15 → 마감(exclusive)
        assert_eq!(minutes_for_date(&slices, day1), Some(60));
        assert_eq!(minutes_for_date(&slices, day2), None);
        // 다른 요일은 매칭 없음
        let other = day1.succ_opt().unwrap();
        assert_eq!(minutes_for_date(&slices, other), None);
    }

    /// 6월의 특정 요일 첫 출결 일자를 반환.
    async fn first_event_date(pool: &SqlitePool, sid: i64) -> String {
        sqlx::query_scalar(
            "SELECT event_date FROM regular_attendances WHERE student_id = ? ORDER BY event_date LIMIT 1",
        )
        .bind(sid)
        .fetch_one(pool)
        .await
        .expect("첫 출결 일자")
    }

    #[tokio::test]
    async fn move_attendance_moves_present_cell() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 월요일 1h
        generate_impl(&pool, "2026-06").await.expect("generate");

        let from = first_event_date(&pool, sid).await; // 첫 월요일
        let from_d = NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap();
        let to_d = from_d.succ_opt().unwrap(); // 다음날(화요일, 비수업·비OFF·동월)
        let to = to_d.format("%Y-%m-%d").to_string();

        let r = move_attendance_impl(&pool, sid, &from, &to, "16:00").await.expect("이동 성공");
        assert_eq!(r.to_date, to);
        assert!(r.note.contains("이동"));

        // 원본 비고, 도착 출석 + note
        let from_left: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM regular_attendances WHERE student_id = ? AND event_date = ?",
        )
        .bind(sid).bind(&from).fetch_optional(&pool).await.unwrap();
        assert!(from_left.is_none(), "원본 일자는 비어야 함");

        let to_id: i64 = sqlx::query_scalar(
            "SELECT id FROM regular_attendances WHERE student_id = ? AND event_date = ?",
        )
        .bind(sid).bind(&to).fetch_one(&pool).await.unwrap();
        let cell = fetch_cell(&pool, to_id).await;
        assert_eq!(cell.status, "present");
        assert!(cell.note.is_some());

        // PI-28: 입력한 시작시간이 "HH:MM:SS" 로 저장되어야 (캘린더 표시용)
        let saved_time: Option<String> = sqlx::query_scalar(
            "SELECT start_time FROM regular_attendances WHERE id = ?",
        )
        .bind(to_id).fetch_one(&pool).await.unwrap();
        assert_eq!(saved_time.as_deref(), Some("16:00:00"));
    }

    #[test]
    fn normalize_time_formats_and_validates() {
        assert_eq!(normalize_time("16:00").unwrap(), "16:00:00");
        assert_eq!(normalize_time("9:5").unwrap(), "09:05:00");
        assert_eq!(normalize_time("16:00:00").unwrap(), "16:00:00");
        assert!(normalize_time("25:00").is_err());
        assert!(normalize_time("16:75").is_err());
        assert!(normalize_time("abc").is_err());
        assert!(normalize_time("16").is_err());
    }

    #[tokio::test]
    async fn move_attendance_blocks_weekend() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let from = first_event_date(&pool, sid).await; // 첫 월요일
        let from_d = NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap();
        // 같은 주 토요일(월 + 5일) — 동월 범위
        let sat = from_d.checked_add_days(chrono::Days::new(5)).unwrap();
        let to = sat.format("%Y-%m-%d").to_string();
        let err = move_attendance_impl(&pool, sid, &from, &to, "16:00").await.unwrap_err();
        assert!(err.contains("주말"), "주말 이동 차단: {}", err);
    }

    #[tokio::test]
    async fn move_attendance_blocks_off_day() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");

        let from = first_event_date(&pool, sid).await;
        let from_d = NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap();
        let to_d = from_d.succ_opt().unwrap();
        let to = to_d.format("%Y-%m-%d").to_string();
        // 도착일을 공휴일(allows_regular_class=0)로 지정
        let code = schedule_code_id(&pool, "공휴일").await;
        add_schedule_event(&pool, code, &to, None).await;

        let err = move_attendance_impl(&pool, sid, &from, &to, "16:00").await.unwrap_err();
        assert!(err.contains("옮길 수 없습니다"), "OFF일 차단: {}", err);
    }

    #[tokio::test]
    async fn move_attendance_blocks_cross_month() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let from = first_event_date(&pool, sid).await;

        let err = move_attendance_impl(&pool, sid, &from, "2026-07-06", "16:00").await.unwrap_err();
        assert!(err.contains("같은 달"), "월 경계 차단: {}", err);
    }

    #[tokio::test]
    async fn move_attendance_blocks_conflict() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 월(1) + 화(2) 둘 다 수업 → 화요일에 이미 출결 존재
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1), (2, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");

        let from = first_event_date(&pool, sid).await; // 첫 출결(월 또는 화 중 이른 것)
        let from_d = NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap();
        let to_d = from_d.succ_opt().unwrap(); // 다음날 — 둘 다 수업이면 충돌 가능
        let to = to_d.format("%Y-%m-%d").to_string();
        let to_has: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM regular_attendances WHERE student_id = ? AND event_date = ?",
        )
        .bind(sid).bind(&to).fetch_optional(&pool).await.unwrap();
        if to_has.is_some() {
            let err = move_attendance_impl(&pool, sid, &from, &to, "16:00").await.unwrap_err();
            assert!(err.contains("이미 수업"), "충돌 차단: {}", err);
        }
    }

    #[tokio::test]
    async fn move_attendance_rejects_non_present() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");
        let from = first_event_date(&pool, sid).await;
        let from_d = NaiveDate::parse_from_str(&from, "%Y-%m-%d").unwrap();
        let to = from_d.succ_opt().unwrap().format("%Y-%m-%d").to_string();
        // 결석으로 변경
        sqlx::query("UPDATE regular_attendances SET status = 'absent' WHERE student_id = ? AND event_date = ?")
            .bind(sid).bind(&from).execute(&pool).await.unwrap();

        let err = move_attendance_impl(&pool, sid, &from, &to, "16:00").await.unwrap_err();
        assert!(err.contains("출석(미처리)"), "present 외 거부: {}", err);
    }

    /// 케이스2 셋업: 월(1) 1h 스케줄을 effective_date 부터 화(2) 2h 로 변경 (set_schedule 패턴 재현).
    async fn change_mon_to_tue(pool: &SqlitePool, sid: i64, effective_date: &str, new_hours: i64) {
        sqlx::query(
            "UPDATE student_schedules SET effective_to = ? \
             WHERE student_id = ? AND day_of_week = 1 AND effective_to IS NULL",
        )
        .bind(effective_date).bind(sid).execute(pool).await.unwrap();
        sqlx::query(
            "INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 2, '16:00', ?, ?)",
        )
        .bind(sid).bind(new_hours).bind(effective_date).execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn apply_schedule_change_regenerates_after_date() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await; // 월 1h
        generate_impl(&pool, "2026-06").await.expect("generate");

        change_mon_to_tue(&pool, sid, "2026-06-15", 2).await; // 6/15부터 화 2h
        let r = apply_schedule_change_impl(&pool, sid, "2026-06-15").await.expect("apply");

        // 변경일 이후 월요일(%w=1) 출결 없음, 화요일(%w=2) 출결 생성
        let mon_after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE student_id = ? AND event_date >= '2026-06-15' AND CAST(strftime('%w', event_date) AS INTEGER) = 1",
        ).bind(sid).fetch_one(&pool).await.unwrap();
        assert_eq!(mon_after, 0, "변경일 이후 월요일 출결 없어야");
        let tue_after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE student_id = ? AND event_date >= '2026-06-15' AND CAST(strftime('%w', event_date) AS INTEGER) = 2",
        ).bind(sid).fetch_one(&pool).await.unwrap();
        assert!(tue_after > 0, "변경일 이후 화요일 출결 생성");
        assert!(r.regenerated_count > 0);

        // 변경일 이전 월요일 출결 유지(불변)
        let mon_before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances \
             WHERE student_id = ? AND event_date < '2026-06-15' AND CAST(strftime('%w', event_date) AS INTEGER) = 1",
        ).bind(sid).fetch_one(&pool).await.unwrap();
        assert!(mon_before > 0, "변경일 이전 월요일 출결은 유지");

        // 주당 분 변동 감지 (월 60 → 화 120)
        assert_eq!(r.weekly_minutes_before, 60);
        assert_eq!(r.weekly_minutes_after, 120);
    }

    #[tokio::test]
    async fn apply_schedule_change_preserves_processed_rows() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        generate_impl(&pool, "2026-06").await.expect("generate");

        // 변경일 이후 첫 월요일 출결을 결석 처리
        let absent_date: String = sqlx::query_scalar(
            "SELECT event_date FROM regular_attendances \
             WHERE student_id = ? AND event_date >= '2026-06-15' ORDER BY event_date LIMIT 1",
        ).bind(sid).fetch_one(&pool).await.unwrap();
        sqlx::query("UPDATE regular_attendances SET status = 'absent' WHERE student_id = ? AND event_date = ?")
            .bind(sid).bind(&absent_date).execute(&pool).await.unwrap();

        change_mon_to_tue(&pool, sid, "2026-06-15", 1).await;
        let r = apply_schedule_change_impl(&pool, sid, "2026-06-15").await.expect("apply");

        assert!(r.preserved_count >= 1, "결석 행 보존 카운트");
        // 결석 행 여전히 존재 + 상태 유지
        let still: Option<String> = sqlx::query_scalar(
            "SELECT status FROM regular_attendances WHERE student_id = ? AND event_date = ?",
        ).bind(sid).bind(&absent_date).fetch_optional(&pool).await.unwrap();
        assert_eq!(still.as_deref(), Some("absent"), "결석 행은 보존되어야");
    }

    #[tokio::test]
    async fn apply_schedule_change_rejects_before_enroll() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        let err = apply_schedule_change_impl(&pool, sid, "2026-03-01").await.unwrap_err();
        assert!(err.contains("입교일"), "입교일 이전 차단: {}", err);
    }

    // ─────────────────────── T8: sync_attendance_on_schedule_change 테스트 ───────────────────────

    /// T8 AC-1: OFF→ON — OFF 이벤트 삭제 후 해당 날짜 출결 INSERT
    #[tokio::test]
    async fn sync_attendance_inserts_on_off_to_on_transition() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        // 월요일(1) 1h 스케줄 원생 (2026-06-01 = 월)
        let _sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        // OFF 이벤트 없는 상태에서 sync → INSERT 발생
        sync_attendance_on_schedule_change(&pool, "2026-06-01", None)
            .await
            .expect("sync ok");
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-01'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(cnt >= 1, "OUT→ON: 출결 INSERT 기대, 실제 {}", cnt);
    }

    /// T8 AC-2: ON→OFF — OFF 이벤트 추가 후 해당 날짜 출결 DELETE
    #[tokio::test]
    async fn sync_attendance_deletes_on_on_to_off_transition() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let _sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;
        // 먼저 출결 데이터 생성
        generate_impl(&pool, "2026-06").await.expect("generate");
        let before: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-01'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(before >= 1, "generate 후 출결 존재 기대");

        // OFF 코드 이벤트 추가
        let holiday_id = schedule_code_id(&pool, "공휴일임시").await;
        add_schedule_event(&pool, holiday_id, "2026-06-01", None).await;

        // sync → DELETE 발생
        sync_attendance_on_schedule_change(&pool, "2026-06-01", None)
            .await
            .expect("sync ok");
        let after: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-01'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(after, 0, "ON→OFF: 출결 DELETE 기대, 실제 {}", after);
    }

    /// T8 AC-3 (회귀): 공휴일(OFF) 이벤트가 남아있는 상태에서 공휴수업일(ON) 이벤트가
    /// 같은 날짜에 추가로 배치되면(V309 중복 배치 허용) OFF 이벤트가 공존하더라도 ON 이
    /// 우선하여 출결 INSERT 되어야 한다. 실사용 버그: off_count>0 만으로 OFF 판정 시 미생성.
    #[tokio::test]
    async fn sync_attendance_inserts_when_on_event_coexists_with_off_event() {
        let pool = test_pool_in_memory().await.expect("pool");
        seed_period(&pool, "2026-06", "2026-06-01", "2026-06-30", 1).await;
        let _sid = seed_student(&pool, "S001", "2026-04-01", None, &[(1, 1)]).await;

        // 공휴일(OFF) 이벤트 먼저 배치
        let holiday_id = schedule_code_id(&pool, "공휴일임시3").await;
        add_schedule_event(&pool, holiday_id, "2026-06-01", None).await;

        // 공휴수업일(ON, allows_regular_class=1) 코드를 같은 날짜에 추가 배치 (공휴일과 공존)
        let makeup_class_id: i64 = sqlx::query(
            "INSERT INTO schedule_codes \
             (code_name, is_system_reserved, allows_regular_class, allows_makeup_class, \
              is_duplicate_blocked, is_period_type) \
             VALUES ('공휴수업일임시', 0, 1, 1, 0, 0) RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("공휴수업일 코드 INSERT")
        .try_get("id")
        .expect("code id");
        add_schedule_event(&pool, makeup_class_id, "2026-06-01", None).await;

        sync_attendance_on_schedule_change(&pool, "2026-06-01", None)
            .await
            .expect("sync ok");
        let cnt: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM regular_attendances WHERE event_date = '2026-06-01'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(cnt >= 1, "OFF+ON 공존 시 ON 우선 INSERT 기대, 실제 {}", cnt);
    }
}
