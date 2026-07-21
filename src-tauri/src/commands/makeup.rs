//! 보강 도메인 IPC (Sprint 9~10, PRD §4.5.4~6, §4.5.10).
//!
//! Phase 3 — 보강 등록(개별) + 매칭 + 취소 + 결석 이력.
//! 본 모듈은 attendance.rs 와 별개 도메인 — V107 FK
//! (`regular_attendances.makeup_attendance_id → makeup_attendances.id`) 를 통해 정규 출결과
//! 연결되지만, 보강 등록/취소 트랜잭션은 본 모듈이 담당한다.
//!
//! ## IPC 목록
//! - `get_pending_absences`, `get_makeup_eligible_dates` (Sprint 9 T2)
//! - `create_makeup_with_absences` 트랜잭션 매칭 (Sprint 9 T3)
//! - `cancel_makeup` (Sprint 9 T4)
//! - `get_absence_history` (Sprint 9 T8)
//!
//! ## PI-02 결정 (사용자, 2026-05-24)
//! - 옵션 A 일 단위 매칭 — 보강 1일 = 결석 N일 충당. 시간값 비교 없음.
//! - 분 단위 전환은 T3 검증 3 활성/비활성 1줄 토글로 가능 (R58 추적).

use crate::commands::attendance::validate_year_month;
use crate::commands::audit::{self, AuditEventType};
use crate::commands::db;
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::BTreeMap;

// ────────────────────────────────────────────────────────────────────
// 잔여 보강필요분 계산 (ADR-011 분 단위 부분 차감)
// ────────────────────────────────────────────────────────────────────

/// 결석의 잔여 보강필요분을 계산하는 SQL 서브식을 만든다 (ADR-011).
///
/// `잔여 = class_minutes - SUM(makeup_allocations.allocated_minutes)`.
/// `alias` 는 `regular_attendances` 테이블(또는 그 별칭)이며 **코드 내부 상수만**
/// 전달한다 — 사용자 입력을 넘기면 안 된다(SQL 인젝션 방지, backend.md).
/// 보강 매칭 여부 판정·잔여 집계에 쓰는 8개 쿼리(T4)가 이 헬퍼로 통일된다(R139 완화).
pub(crate) fn remaining_minutes_expr(alias: &str) -> String {
    format!(
        "({a}.class_minutes - COALESCE((SELECT SUM(mal.allocated_minutes) \
          FROM makeup_allocations mal WHERE mal.absence_id = {a}.id), 0))",
        a = alias
    )
}

// ────────────────────────────────────────────────────────────────────
// 응답 구조체 (camelCase serde)
// ────────────────────────────────────────────────────────────────────

/// 원생의 미처리 결석 1건 — `status='absent'` 이면서 잔여 보강필요분 > 0 (ADR-011).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAbsence {
    pub id: i64,
    pub event_date: String,
    pub year_month: String,
    /// 원 결석 수업 시간(분) — 참고용.
    pub class_minutes: i64,
    /// 잔여 보강필요분 = class_minutes - 이미 배분된 보강분. UI 표시·합산은 이 값 사용.
    pub remaining_minutes: i64,
    pub makeup_deadline: Option<String>,
    pub absence_memo: Option<String>,
}

/// 보강 가능 일자 1건 — `schedule_codes.allows_makeup_class=1` 인 학사일정 일자.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EligibleDate {
    pub event_date: String,
    pub schedule_code_name: String,
}

/// 보강 등록 페이로드 — `create_makeup_with_absences` IPC 입력.
///
/// 단일 구조체로 묶어 Tauri IPC argument 직렬화 안정성 확보 (다중 i64 + Vec).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMakeupPayload {
    pub student_id: i64,
    pub event_date: String,
    pub class_minutes: i64,
    pub absence_ids: Vec<i64>,
}

/// 보강 등록 결과 — IPC 응답.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MakeupResult {
    pub makeup_id: i64,
    pub student_id: i64,
    pub event_date: String,
    pub matched_count: usize,
}

/// 결석 이력 1건 (T8) — `regular_attendances WHERE status IN ('absent', 'makeup_done', 'makeup_expired')`
/// + LEFT JOIN `makeup_attendances` 로 보강 일자/시간 포함.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceHistoryItem {
    pub id: i64,
    pub event_date: String,
    pub class_minutes: i64,
    /// 'absent' / 'makeup_done' / 'makeup_expired'
    pub status: String,
    pub makeup_deadline: Option<String>,
    pub absence_memo: Option<String>,
    /// `makeup_done` 인 경우 매칭된 보강의 event_date.
    pub makeup_event_date: Option<String>,
    /// 매칭된 보강의 class_minutes.
    pub makeup_class_minutes: Option<i64>,
}

// ────────────────────────────────────────────────────────────────────
// IPC: 미처리 결석 조회
// ────────────────────────────────────────────────────────────────────

/// 원생의 미처리 결석 목록 — 소멸기한 임박 순 정렬 (NULL 은 마지막).
///
/// PRD §4.5.4 보강 등록 다이얼로그가 충당 결석 선택지를 표시하기 위해 호출.
#[tauri::command]
pub async fn get_pending_absences(student_id: i64) -> Result<Vec<PendingAbsence>, String> {
    let pool = db::pool().map_err(String::from)?;
    get_pending_absences_impl(pool, student_id).await
}

async fn get_pending_absences_impl(
    pool: &SqlitePool,
    student_id: i64,
) -> Result<Vec<PendingAbsence>, String> {
    // ADR-011: 미매칭(makeup_attendance_id) 대신 "잔여분 > 0" 기준 — 부분 소진 결석도 포함.
    let remaining = remaining_minutes_expr("regular_attendances");
    let sql = format!(
        "SELECT id, event_date, year_month, class_minutes, \
                {remaining} AS remaining_minutes, makeup_deadline, absence_memo \
         FROM regular_attendances \
         WHERE student_id = ? AND status = 'absent' AND {remaining} > 0 \
         ORDER BY (makeup_deadline IS NULL), makeup_deadline ASC, event_date ASC",
        remaining = remaining
    );
    let rows = sqlx::query(&sql)
        .bind(student_id)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("미처리 결석 조회 실패: {}", e))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        result.push(PendingAbsence {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            event_date: r.try_get("event_date").map_err(|e| e.to_string())?,
            year_month: r.try_get("year_month").map_err(|e| e.to_string())?,
            class_minutes: r.try_get("class_minutes").map_err(|e| e.to_string())?,
            remaining_minutes: r.try_get("remaining_minutes").map_err(|e| e.to_string())?,
            makeup_deadline: r.try_get("makeup_deadline").map_err(|e| e.to_string())?,
            absence_memo: r.try_get("absence_memo").map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 가능 일자 조회
// ────────────────────────────────────────────────────────────────────

/// 원생의 보강 가능 일자 조회 — year_month 내 보강이 가능한 일자.
///
/// 사용자 룰 (Session #10, 2026-05-24):
/// - 케이스 A: 평일(월~금) AND 보강불가 코드 없음
///   (보강불가 코드 = `allows_regular_class=0 AND allows_makeup_class=0` — 공휴일/방학/휴원일)
/// - OR 케이스 B: `allows_makeup_class=1` 명시 코드 (보강데이/단원평가 응시일/공휴수업일)
///   — 요일 무관
///
/// `study_periods` 범위 제약은 없음 (소멸기한 기준 + 학생 입퇴교 범위만 제약).
/// 학생의 정규 수업 요일에도 보강 등록 가능 (사용자 결정 — Session #10).
///
/// 응답 `schedule_code_name`:
/// - 케이스 B 우선 — 보강 가능 코드명 (예: "보강데이")
/// - 케이스 A — "정규수업일" (학사코드 없는 평일)
#[tauri::command]
pub async fn get_makeup_eligible_dates(
    student_id: i64,
    year_month: String,
) -> Result<Vec<EligibleDate>, String> {
    validate_year_month(&year_month)?;
    let pool = db::pool().map_err(String::from)?;
    get_makeup_eligible_dates_impl(pool, student_id, &year_month).await
}

async fn get_makeup_eligible_dates_impl(
    pool: &SqlitePool,
    student_id: i64,
    year_month: &str,
) -> Result<Vec<EligibleDate>, String> {
    // 1. 학생 입퇴교 범위 조회.
    let student_row = sqlx::query("SELECT enroll_date, withdraw_date FROM students WHERE id = ?")
        .bind(student_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("학생 조회 실패: {}", e))?
        .ok_or_else(|| "학생을 찾을 수 없습니다.".to_string())?;
    let enroll: String = student_row
        .try_get("enroll_date")
        .map_err(|e| e.to_string())?;
    let withdraw: Option<String> = student_row
        .try_get("withdraw_date")
        .map_err(|e| e.to_string())?;
    let enroll_d = NaiveDate::parse_from_str(&enroll, "%Y-%m-%d")
        .map_err(|e| format!("입교일 파싱 실패: {}", e))?;
    let withdraw_d = withdraw
        .as_deref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| format!("퇴교일 파싱 실패: {}", e))?;

    // 2. year_month 범위 (validate 통과 후라 unwrap 안전).
    let parts: Vec<&str> = year_month.split('-').collect();
    let year: i32 = parts[0].parse().expect("validated");
    let month: u32 = parts[1].parse().expect("validated");
    let first = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| format!("일자 생성 실패: {}-{:02}-01", year, month))?;
    let next_month_first = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| "다음 월 일자 생성 실패".to_string())?;

    // 3. month 와 겹치는 모든 schedule_events + 코드 속성 조회 — 단일 쿼리.
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

    // 4. 일자별 학사코드 매핑 펼침 (한 일자에 다중 코드 가능).
    // 튜플: (code_name, allows_regular=1, allows_makeup=1)
    let mut codes_by_date: BTreeMap<String, Vec<(String, bool, bool)>> = BTreeMap::new();
    for r in rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e_str: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let code_name: String = r.try_get("code_name").map_err(|e| e.to_string())?;
        let allows_reg: i64 = r.try_get("allows_regular_class").map_err(|e| e.to_string())?;
        let allows_mk: i64 = r.try_get("allows_makeup_class").map_err(|e| e.to_string())?;
        let mut d = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 일자 파싱 실패: {}", e))?;
        let ed = NaiveDate::parse_from_str(&e_str, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 종료일 파싱 실패: {}", e))?;
        while d <= ed {
            if d >= first && d < next_month_first {
                codes_by_date
                    .entry(d.to_string())
                    .or_default()
                    .push((code_name.clone(), allows_reg == 1, allows_mk == 1));
            }
            d = d.succ_opt().expect("date succ");
        }
    }

    // 5. month 의 모든 일자를 순회하면서 룰 적용.
    let mut eligible: BTreeMap<String, String> = BTreeMap::new();
    let mut d = first;
    while d < next_month_first {
        // 학생 입퇴교 범위 외 제외.
        if d < enroll_d {
            d = d.succ_opt().expect("date succ");
            continue;
        }
        if let Some(wd) = withdraw_d {
            if d > wd {
                d = d.succ_opt().expect("date succ");
                continue;
            }
        }

        let date_str = d.to_string();
        let codes = codes_by_date.get(&date_str);

        // 케이스 B: allows_makeup_class=1 코드 우선.
        let case_b = codes.and_then(|cs| {
            cs.iter()
                .find(|(_, _, mk)| *mk)
                .map(|(name, _, _)| name.clone())
        });

        if let Some(name) = case_b {
            eligible.insert(date_str, name);
        } else {
            // 케이스 A: 평일(월~금=1~5) + 보강불가 코드 없음.
            let weekday = d.weekday().number_from_monday(); // 1..=7
            let is_weekday = weekday <= 5;
            let has_block = codes.is_some_and(|cs| cs.iter().any(|(_, reg, mk)| !*reg && !*mk));
            if is_weekday && !has_block {
                eligible.insert(date_str, "정규수업일".to_string());
            }
        }

        d = d.succ_opt().expect("date succ");
    }

    Ok(eligible
        .into_iter()
        .map(|(event_date, schedule_code_name)| EligibleDate {
            event_date,
            schedule_code_name,
        })
        .collect())
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 등록 + 매칭 (Sprint 9 T3 핵심 트랜잭션)
// ────────────────────────────────────────────────────────────────────

/// 보강 1건을 등록하고 미처리 결석 N건을 "보강완료" 로 매칭한다.
///
/// 트랜잭션 내 검증 4종 (Session #10 — 검증 3 정규 수업 요일 차단 폐기):
/// 1. **이벤트 일자 보강 가능** — Session #10 룰: 평일+보강불가코드없음 OR `allows_makeup_class=1`
/// 2. **학생 일관성** — 학생 존재 + 입퇴교 범위 내 `event_date`
/// 3. **결석 유효성** — `absence_ids` 모두 해당 학생 + `status='absent'` + 미매칭
/// 4. **PI-02 시간값** — 옵션 A (일 단위) 채택: 검증 생략. 분 단위 전환 시 본 함수 내
///    "PI-02 분 단위 활성 위치" 주석 위치에서 1줄 추가만으로 활성화 가능.
///
/// 정규 수업 요일에도 보강 등록 허용 — 사용자 결정 Session #10 ("수업 요일에 추가
/// 시간 써서 수업 완료 후 보강 진행 가능"). 기존 차단 정책 폐기.
///
/// 실행 순서 (단일 트랜잭션):
/// - INSERT makeup_attendances → makeup_id 발급
/// - UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=makeup_id
///   WHERE id IN absence_ids
///
/// audit `MakeupCreated` 기록 (트랜잭션 커밋 후 fire-and-forget — pool 의존이라 silent skip
/// 가능, 그러나 본 IPC 는 unlock 후 호출되므로 정상 기록).
#[tauri::command]
pub async fn create_makeup_with_absences(
    payload: CreateMakeupPayload,
) -> Result<MakeupResult, String> {
    let pool = db::pool().map_err(String::from)?;
    let result = create_makeup_with_absences_impl(pool, &payload).await?;
    audit::try_record(
        AuditEventType::MakeupCreated,
        Some(&result.makeup_id.to_string()),
        Some(&format!(
            r#"{{"studentId":{},"eventDate":"{}","matchedCount":{}}}"#,
            result.student_id, result.event_date, result.matched_count
        )),
    )
    .await;
    Ok(result)
}

async fn create_makeup_with_absences_impl(
    pool: &SqlitePool,
    payload: &CreateMakeupPayload,
) -> Result<MakeupResult, String> {
    if payload.absence_ids.is_empty() {
        return Err("충당할 결석을 1건 이상 선택해야 합니다.".to_string());
    }
    if payload.class_minutes <= 0 {
        return Err("수업 시간(분)은 양수여야 합니다.".to_string());
    }

    let event_d = NaiveDate::parse_from_str(&payload.event_date, "%Y-%m-%d")
        .map_err(|e| format!("이벤트 일자 파싱 실패 ({}): {}", payload.event_date, e))?;
    let year_month = format!("{}-{:02}", event_d.year(), event_d.month());

    // 검증 1: event_date 가 보강 가능 일자인지 (Session #10 룰).
    // - 케이스 B: allows_makeup_class=1 코드가 명시된 일자 (요일 무관)
    // - 케이스 A: 평일(월~금) + 보강불가 코드(allows_regular=0 AND allows_makeup=0) 없음
    let codes: Vec<(i64, i64)> = sqlx::query_as(
        "SELECT c.allows_regular_class, c.allows_makeup_class \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(&payload.event_date)
    .bind(&payload.event_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강 가능 일자 검증 실패: {}", e))?;
    let case_b = codes.iter().any(|(_, mk)| *mk == 1);
    let weekday = event_d.weekday().number_from_monday();
    let is_weekday = weekday <= 5;
    let has_block = codes.iter().any(|(reg, mk)| *reg == 0 && *mk == 0);
    let case_a = is_weekday && !has_block;
    if !case_b && !case_a {
        return Err(format!(
            "{} 은 보강 가능 일자가 아닙니다 (공휴일/방학/휴원일 또는 주말+보강데이 미설정 일자입니다).",
            payload.event_date
        ));
    }

    // 검증 2: 학생 존재 + 입퇴교 범위.
    let student_row = sqlx::query("SELECT enroll_date, withdraw_date FROM students WHERE id = ?")
        .bind(payload.student_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("학생 조회 실패: {}", e))?
        .ok_or_else(|| "학생을 찾을 수 없습니다.".to_string())?;
    let enroll: String = student_row
        .try_get("enroll_date")
        .map_err(|e| e.to_string())?;
    let withdraw: Option<String> = student_row
        .try_get("withdraw_date")
        .map_err(|e| e.to_string())?;
    let enroll_d = NaiveDate::parse_from_str(&enroll, "%Y-%m-%d")
        .map_err(|e| format!("입교일 파싱 실패: {}", e))?;
    if event_d < enroll_d {
        return Err("입교일 이전 일자에는 보강을 등록할 수 없습니다.".to_string());
    }
    if let Some(wd_str) = withdraw.as_deref() {
        let wd = NaiveDate::parse_from_str(wd_str, "%Y-%m-%d")
            .map_err(|e| format!("퇴교일 파싱 실패: {}", e))?;
        if event_d > wd {
            return Err("퇴교일 이후 일자에는 보강을 등록할 수 없습니다.".to_string());
        }
    }

    // 검증 3 폐기 (Session #10) — 정규 수업 요일에도 보강 등록 허용.

    // 검증 3: 결석 유효성 + 잔여분 수집 (ADR-011 분 단위 부분 차감).
    // - 모두 본 학생 소속
    // - status='absent' (makeup_done=잔여 0 / makeup_expired 는 매칭 불가)
    // - 잔여분(class_minutes - 기존 배분합) > 0
    // SQL IN 절은 sqlx 동적 placeholder 가 까다로워 — Rust 측 루프로 단건 검증.
    struct AbsenceInfo {
        id: i64,
        deadline: Option<String>,
        event_date: String,
        remaining: i64,
    }
    let mut infos: Vec<AbsenceInfo> = Vec::with_capacity(payload.absence_ids.len());
    for &aid in &payload.absence_ids {
        let row = sqlx::query(
            "SELECT student_id, status, event_date, makeup_deadline, class_minutes, \
                    COALESCE((SELECT SUM(mal.allocated_minutes) FROM makeup_allocations mal \
                              WHERE mal.absence_id = regular_attendances.id), 0) AS allocated \
             FROM regular_attendances WHERE id = ?",
        )
        .bind(aid)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("결석 조회 실패 (id={}): {}", aid, e))?;
        let row = row.ok_or_else(|| format!("결석을 찾을 수 없습니다 (id={}).", aid))?;
        let sid: i64 = row.try_get("student_id").map_err(|e| e.to_string())?;
        let status: String = row.try_get("status").map_err(|e| e.to_string())?;
        if sid != payload.student_id {
            return Err(format!("결석 id={} 가 다른 학생의 것입니다.", aid));
        }
        if status != "absent" {
            return Err(format!(
                "결석 id={} 의 상태가 '{}' 입니다 — 미처리 결석(absent) 만 매칭 가능합니다.",
                aid, status
            ));
        }
        let class_minutes: i64 = row.try_get("class_minutes").map_err(|e| e.to_string())?;
        let allocated: i64 = row.try_get("allocated").map_err(|e| e.to_string())?;
        let remaining = class_minutes - allocated;
        if remaining <= 0 {
            return Err(format!(
                "결석 id={} 는 이미 보강이 모두 채워졌습니다.",
                aid
            ));
        }
        let event_date: String = row.try_get("event_date").map_err(|e| e.to_string())?;
        let deadline: Option<String> = row.try_get("makeup_deadline").map_err(|e| e.to_string())?;
        infos.push(AbsenceInfo {
            id: aid,
            deadline,
            event_date,
            remaining,
        });
    }

    // 검증 4 (ADR-011): 보강 시간이 선택 결석들의 잔여 합계를 초과하지 않아야 한다.
    // 초과분은 기록할 결석이 없어 데이터 모순 — 신규 등록은 엄격히 차단한다.
    // (과거 데이터 백필 V312 는 초과분 버림으로 관대 처리 — 구분)
    let total_remaining: i64 = infos.iter().map(|i| i.remaining).sum();
    if payload.class_minutes > total_remaining {
        return Err(format!(
            "보강 시간({}분)이 선택한 결석의 잔여 보강필요시간({}분)보다 많습니다.",
            payload.class_minutes, total_remaining
        ));
    }

    // 배분 순서: 소멸기한 임박(오래된) 순 — deadline ASC (NULL 마지막), event_date ASC.
    infos.sort_by(|a, b| {
        use std::cmp::Ordering;
        let d = match (&a.deadline, &b.deadline) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        d.then_with(|| a.event_date.cmp(&b.event_date))
    });

    // 실행: 단일 트랜잭션 — INSERT makeup → 배분(makeup_allocations) → 완전 소진 결석 makeup_done.
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    let makeup_row: (i64,) = sqlx::query_as(
        "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes, status) \
         VALUES (?, ?, ?, ?, 'makeup_attended') RETURNING id",
    )
    .bind(payload.student_id)
    .bind(&payload.event_date)
    .bind(&year_month)
    .bind(payload.class_minutes)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| format!("보강 INSERT 실패: {}", e))?;
    let makeup_id = makeup_row.0;

    // 소멸기한 임박순으로 보강분을 결석에 배분 (min(잔여 보강분, 결석 잔여분)).
    let mut remaining_makeup = payload.class_minutes;
    let mut allocated_count = 0usize;
    for info in &infos {
        if remaining_makeup <= 0 {
            break;
        }
        let alloc = remaining_makeup.min(info.remaining);
        if alloc <= 0 {
            continue;
        }
        sqlx::query(
            "INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) \
             VALUES (?, ?, ?)",
        )
        .bind(makeup_id)
        .bind(info.id)
        .bind(alloc)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("보강 배분 INSERT 실패 (absence_id={}): {}", info.id, e))?;
        allocated_count += 1;
        remaining_makeup -= alloc;

        // 잔여가 모두 소진되면 makeup_done 전이 (makeup_attendance_id 는 레거시 — 설정하지 않음).
        if info.remaining - alloc == 0 {
            sqlx::query(
                "UPDATE regular_attendances \
                 SET status = 'makeup_done', \
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                 WHERE id = ? AND status = 'absent'",
            )
            .bind(info.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("결석 소진 UPDATE 실패 (id={}): {}", info.id, e))?;
        }
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(MakeupResult {
        makeup_id,
        student_id: payload.student_id,
        event_date: payload.event_date.clone(),
        matched_count: allocated_count,
    })
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 취소 (Sprint 9 T4)
// ────────────────────────────────────────────────────────────────────

/// 보강 1건을 취소하고 매칭된 결석을 모두 `absent` 상태로 환원한다.
///
/// 트랜잭션 순서 (V107 FK 위반 회피):
/// 1. `regular_attendances` 연결 결석 환원 — `makeup_attendance_id=NULL, status='absent'`
/// 2. `makeup_attendances` DELETE
///
/// 1→2 순서 — FK NULL 처리가 DELETE 전이라야 무결성 위반 없음.
/// audit `MakeupCancelled` 기록 (커밋 후 fire-and-forget).
#[tauri::command]
pub async fn cancel_makeup(makeup_id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let reverted = cancel_makeup_impl(pool, makeup_id).await?;
    audit::try_record(
        AuditEventType::MakeupCancelled,
        Some(&makeup_id.to_string()),
        Some(&format!(r#"{{"revertedAbsences":{}}}"#, reverted)),
    )
    .await;
    Ok(())
}

async fn cancel_makeup_impl(pool: &SqlitePool, makeup_id: i64) -> Result<usize, String> {
    // 존재 여부 사전 확인 — 친화 메시지 + 후속 트랜잭션 단순화.
    let exists: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances WHERE id = ?")
            .bind(makeup_id)
            .fetch_one(pool)
            .await
            .map_err(|e| format!("보강 조회 실패: {}", e))?;
    if exists.0 == 0 {
        return Err(format!("보강 id={} 를 찾을 수 없습니다.", makeup_id));
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    // 1. 이 보강이 배분된 결석 id 수집 (환원 판정 대상).
    let absence_rows =
        sqlx::query("SELECT absence_id FROM makeup_allocations WHERE makeup_id = ?")
            .bind(makeup_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| format!("배분 조회 실패: {}", e))?;
    let mut affected: Vec<i64> = Vec::with_capacity(absence_rows.len());
    for r in &absence_rows {
        affected.push(r.try_get("absence_id").map_err(|e| e.to_string())?);
    }

    // 2. 이 보강의 배분 레코드 삭제 (makeup DELETE 전 — FK 순서).
    sqlx::query("DELETE FROM makeup_allocations WHERE makeup_id = ?")
        .bind(makeup_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("배분 삭제 실패: {}", e))?;

    // 3. 배분이 사라진 결석의 잔여를 재계산 — 잔여가 생겼는데 makeup_done 이면 absent 환원.
    //    부분 소진 중이던(status='absent') 결석은 잔여만 늘고 상태 유지 → 환원 카운트 제외.
    let mut reverted = 0usize;
    for aid in &affected {
        let row: (i64, i64, String) = sqlx::query_as(
            "SELECT class_minutes, \
                    COALESCE((SELECT SUM(allocated_minutes) FROM makeup_allocations \
                              WHERE absence_id = ?), 0), \
                    status \
             FROM regular_attendances WHERE id = ?",
        )
        .bind(aid)
        .bind(aid)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("잔여 재계산 실패 (id={}): {}", aid, e))?;
        let remaining = row.0 - row.1;
        if remaining > 0 && row.2 == "makeup_done" {
            sqlx::query(
                "UPDATE regular_attendances \
                 SET status = 'absent', \
                     updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
                 WHERE id = ?",
            )
            .bind(aid)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("결석 환원 실패 (id={}): {}", aid, e))?;
            reverted += 1;
        }
    }

    // 4. 보강 레코드 삭제.
    sqlx::query("DELETE FROM makeup_attendances WHERE id = ?")
        .bind(makeup_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("보강 DELETE 실패: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(reverted)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 결석 이력 조회 (Sprint 9 T8)
// ────────────────────────────────────────────────────────────────────

/// 원생의 결석 이력 — 미처리/보강완료/보강소멸 모두 포함. PRD §4.5.10.
///
/// `LEFT JOIN makeup_attendances` 로 `makeup_done` 행의 보강 일자/시간 포함.
/// 정렬: `event_date DESC` (최신순). 출석/`makeup_attended` 단순 보강 등록 행은 제외.
#[tauri::command]
pub async fn get_absence_history(student_id: i64) -> Result<Vec<AbsenceHistoryItem>, String> {
    let pool = db::pool().map_err(String::from)?;
    get_absence_history_impl(pool, student_id).await
}

async fn get_absence_history_impl(
    pool: &SqlitePool,
    student_id: i64,
) -> Result<Vec<AbsenceHistoryItem>, String> {
    let rows = sqlx::query(
        "SELECT r.id, r.event_date, r.class_minutes, r.status, \
                r.makeup_deadline, r.absence_memo, \
                m.event_date AS makeup_event_date, \
                m.class_minutes AS makeup_class_minutes \
         FROM regular_attendances r \
         LEFT JOIN makeup_attendances m ON r.makeup_attendance_id = m.id \
         WHERE r.student_id = ? \
           AND r.status IN ('absent', 'makeup_done', 'makeup_expired') \
         ORDER BY r.event_date DESC",
    )
    .bind(student_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("결석 이력 조회 실패: {}", e))?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        result.push(AbsenceHistoryItem {
            id: r.try_get("id").map_err(|e| e.to_string())?,
            event_date: r.try_get("event_date").map_err(|e| e.to_string())?,
            class_minutes: r.try_get("class_minutes").map_err(|e| e.to_string())?,
            status: r.try_get("status").map_err(|e| e.to_string())?,
            makeup_deadline: r.try_get("makeup_deadline").map_err(|e| e.to_string())?,
            absence_memo: r.try_get("absence_memo").map_err(|e| e.to_string())?,
            makeup_event_date: r
                .try_get("makeup_event_date")
                .map_err(|e| e.to_string())?,
            makeup_class_minutes: r
                .try_get("makeup_class_minutes")
                .map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

// ────────────────────────────────────────────────────────────────────
// 단위 테스트 (cipher off 인메모리 풀)
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(not(feature = "cipher"))]
mod tests {
    use super::*;

    /// 테스트 학생 1명 + 정규 수업 스케줄 N 요일 삽입.
    ///
    /// schedules: `&[(day_of_week 1~7, duration_hours)]` — 비어있으면 student_schedules 미삽입
    /// (T2 IPC 들은 schedules 의존성 없음). T3 정규 수업 요일 차단 검증에는 schedules 필요.
    async fn seed_student(
        pool: &SqlitePool,
        serial_no: &str,
        enroll: &str,
        withdraw: Option<&str>,
        schedules: &[(i64, i64)],
    ) -> i64 {
        let row: (i64,) = sqlx::query_as(
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
        let sid = row.0;
        for &(dow, hours) in schedules {
            sqlx::query(
                "INSERT INTO student_schedules \
                 (student_id, day_of_week, start_time, duration_hours, effective_from) \
                 VALUES (?, ?, '15:00', ?, ?)",
            )
            .bind(sid)
            .bind(dow)
            .bind(hours)
            .bind(enroll)
            .execute(pool)
            .await
            .expect("schedule INSERT");
        }
        sid
    }

    /// 보강 가능 학사일정 코드 id 조회 (V102 시드 사용).
    async fn schedule_code_id(pool: &SqlitePool, name: &str) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM schedule_codes WHERE code_name = ?")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("코드 id 조회");
        row.0
    }

    async fn insert_schedule_event(
        pool: &SqlitePool,
        code_id: i64,
        event_date: &str,
        period_end_date: Option<&str>,
    ) {
        sqlx::query(
            "INSERT INTO schedule_events (code_id, event_date, period_end_date) VALUES (?, ?, ?)",
        )
        .bind(code_id)
        .bind(event_date)
        .bind(period_end_date)
        .execute(pool)
        .await
        .expect("이벤트 INSERT");
    }

    async fn insert_absence(
        pool: &SqlitePool,
        student_id: i64,
        event_date: &str,
        year_month: &str,
        class_minutes: i64,
        makeup_deadline: Option<&str>,
    ) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO regular_attendances \
             (student_id, event_date, year_month, status, class_minutes, makeup_deadline) \
             VALUES (?, ?, ?, 'absent', ?, ?) RETURNING id",
        )
        .bind(student_id)
        .bind(event_date)
        .bind(year_month)
        .bind(class_minutes)
        .bind(makeup_deadline)
        .fetch_one(pool)
        .await
        .expect("결석 INSERT");
        row.0
    }

    // ─────────────── get_pending_absences ───────────────

    /// AC-T2-1: 미처리 결석만 조회 (status='absent' AND makeup_attendance_id IS NULL).
    /// 소멸기한 임박 순 정렬, NULL 은 마지막.
    #[tokio::test]
    async fn pending_absences_sorts_by_makeup_deadline_nulls_last() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;

        // 3건 결석: 6/15(deadline=07), 6/10(deadline=NULL), 6/20(deadline=07)
        // 기대 순서: 6/15(07), 6/20(07), 6/10(NULL)
        insert_absence(&pool, sid, "2026-06-15", "2026-06", 90, Some("2026-07")).await;
        insert_absence(&pool, sid, "2026-06-10", "2026-06", 90, None).await;
        insert_absence(&pool, sid, "2026-06-20", "2026-06", 90, Some("2026-07")).await;

        let list = get_pending_absences_impl(&pool, sid)
            .await
            .expect("미처리 결석 조회");
        let dates: Vec<String> = list.iter().map(|p| p.event_date.clone()).collect();
        assert_eq!(
            dates,
            vec!["2026-06-15", "2026-06-20", "2026-06-10"],
            "deadline 임박순 + NULL 마지막"
        );
    }

    /// AC-T2-1 보강: 매칭된 결석 (makeup_attendance_id NOT NULL) 은 제외.
    #[tokio::test]
    async fn pending_absences_excludes_matched_absences() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let aid = insert_absence(&pool, sid, "2026-06-15", "2026-06", 90, Some("2026-07")).await;

        // 보강 행 + 매칭 설정 — V107 FK 강제로 실제 makeup_id 필요.
        let makeup_id: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-22', '2026-06', 90) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("makeup INSERT");
        sqlx::query(
            "UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=? WHERE id=?",
        )
        .bind(makeup_id.0)
        .bind(aid)
        .execute(&pool)
        .await
        .expect("매칭 설정");

        let list = get_pending_absences_impl(&pool, sid).await.expect("조회");
        assert!(list.is_empty(), "이미 매칭된 결석은 미처리에서 제외");
    }

    /// AC-T2-1 보강: 출석 상태(present) 는 제외.
    #[tokio::test]
    async fn pending_absences_excludes_present_status() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, status) \
             VALUES (?, '2026-06-15', '2026-06', 90, 'present')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .expect("출석 INSERT");

        let list = get_pending_absences_impl(&pool, sid).await.expect("조회");
        assert!(list.is_empty(), "출석 상태는 미처리 결석 아님");
    }

    // ─────────────── get_makeup_eligible_dates ───────────────

    /// AC-T2-2: allows_makeup_class=1 인 학사일정이 있는 일자는 케이스 B 로 반환 (요일 무관).
    /// V301 보정 후 "공휴수업일" 코드도 allows_makeup_class=1.
    #[tokio::test]
    async fn eligible_dates_returns_makeup_class_dates() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let by_date: BTreeMap<String, String> = list
            .into_iter()
            .map(|e| (e.event_date, e.schedule_code_name))
            .collect();
        // 케이스 B 우선 — 코드명 노출
        assert_eq!(by_date.get("2026-06-15"), Some(&"공휴수업일".to_string()));
        assert_eq!(by_date.get("2026-06-22"), Some(&"공휴수업일".to_string()));
    }

    /// AC-T2-2 보강: allows_regular=0 AND allows_makeup=0 (방학) 인 일자는 케이스 A 차단.
    #[tokio::test]
    async fn eligible_dates_excludes_makeup_off_codes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        // 방학 코드: allows_regular=0 AND allows_makeup=0
        let vac_code = schedule_code_id(&pool, "방학").await;
        insert_schedule_event(&pool, vac_code, "2026-06-15", None).await; // 월요일

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert!(
            !dates.contains(&"2026-06-15".to_string()),
            "방학(보강불가) 일자 미포함"
        );
        // 다른 평일은 케이스 A 로 정상 가능 — Session #10 새 룰
        assert!(
            dates.contains(&"2026-06-16".to_string()),
            "다른 평일은 정상 포함"
        );
    }

    /// AC-T2-3: 학생 입교일 이전 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_before_enroll() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 6/20 입교 학생 — 6/15(월) 학사코드 있어도 입교 전이라 제외
        let sid = seed_student(&pool, "S001", "2026-06-20", None, &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert!(
            !dates.contains(&"2026-06-15".to_string()),
            "입교 전 6/15 제외"
        );
        // 입교 후 평일은 케이스 A 로 정상 가능
        assert!(
            dates.contains(&"2026-06-22".to_string()),
            "입교 후 평일 포함"
        );
    }

    /// AC-T2-3 보강: 학생 퇴교일 이후 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_after_withdraw() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", Some("2026-06-18"), &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert!(
            !dates.contains(&"2026-06-22".to_string()),
            "퇴교 후 6/22 제외"
        );
        assert!(
            dates.contains(&"2026-06-15".to_string()),
            "퇴교 전 평일 포함"
        );
    }

    /// AC-T2-2 기간성 코드: period_end_date 가 있으면 시작~종료 모든 일자 펼침.
    /// month 와 겹치는 부분만 반환.
    #[tokio::test]
    async fn eligible_dates_expands_period_codes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        // 6/14 ~ 6/16 (3일) 기간성 보강 가능일
        insert_schedule_event(&pool, makeup_code, "2026-06-14", Some("2026-06-16")).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        // 6/14(일)도 보강데이 코드라 케이스 B 로 가능 / 6/15(월)·6/16(화)도 평일이라 케이스 A 로 가능
        assert!(dates.contains(&"2026-06-14".to_string()));
        assert!(dates.contains(&"2026-06-15".to_string()));
        assert!(dates.contains(&"2026-06-16".to_string()));
    }

    // ─────────────── Session #10 신규: 케이스 A/B 분리 검증 ───────────────

    /// Session #10: 평일 + 학사코드 없음 → 케이스 A 로 가능 (정규수업일 라벨).
    /// 2026-06-15 = 월요일.
    #[tokio::test]
    async fn eligible_dates_includes_weekdays_without_schedule_code() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        // 모든 평일이 반환되어야 함 — 22개 평일 (2026-06)
        assert!(
            dates.contains(&"2026-06-15".to_string()),
            "월요일 평일 포함"
        );
        assert!(
            dates.contains(&"2026-06-16".to_string()),
            "화요일 평일 포함"
        );
        assert!(
            !dates.contains(&"2026-06-13".to_string()),
            "토요일 미포함 (학사코드 없는 주말 불가)"
        );
        // 라벨 — "정규수업일"
        let mon = list.iter().find(|e| e.event_date == "2026-06-15").unwrap();
        assert_eq!(mon.schedule_code_name, "정규수업일");
    }

    /// Session #10: 토/일 + 보강데이 코드 없음 → 불가.
    /// 2026-06-13(토)/14(일).
    #[tokio::test]
    async fn eligible_dates_excludes_weekends_without_makeup_code() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert!(
            !dates.contains(&"2026-06-13".to_string()),
            "토요일 미포함"
        );
        assert!(
            !dates.contains(&"2026-06-14".to_string()),
            "일요일 미포함"
        );
    }

    /// Session #10: 평일 + 공휴일 코드 → 케이스 A 차단 (보강불가 코드).
    #[tokio::test]
    async fn eligible_dates_excludes_holiday_code() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        // 2026-06-15(월)에 "공휴일" 코드 (allows_regular=0, allows_makeup=0)
        let holiday = schedule_code_id(&pool, "공휴일").await;
        insert_schedule_event(&pool, holiday, "2026-06-15", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert!(
            !dates.contains(&"2026-06-15".to_string()),
            "공휴일 코드 평일 미포함"
        );
        assert!(
            dates.contains(&"2026-06-16".to_string()),
            "화요일은 정상 평일"
        );
    }

    /// Session #10: 토/일 + 보강데이 코드 명시 → 케이스 B 로 가능 (요일 무관).
    #[tokio::test]
    async fn eligible_dates_includes_weekends_with_makeup_code() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let makeup_day = schedule_code_id(&pool, "보강데이").await;
        insert_schedule_event(&pool, makeup_day, "2026-06-13", None).await; // 토요일

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let sat = list.iter().find(|e| e.event_date == "2026-06-13").unwrap();
        assert_eq!(sat.schedule_code_name, "보강데이");
    }

    // validate_year_month 자체 검증은 attendance.rs 의 단위 테스트
    // (`validate_year_month_rejects_out_of_range_month`) 에서 보장됨. 본 모듈은 호출만 위임.

    // ─────────────── T3: create_makeup_with_absences (트랜잭션 매칭) ───────────────

    /// T3 픽스처 — 학생 1명 (월~금 정규 수업 5요일) + 보강 가능 코드(공휴수업일) 일자 +
    /// 미처리 결석 N건. event_date 는 토요일(2026-06-13) 또는 일요일(2026-06-14)로 정규 수업
    /// 요일이 아닌 일자.
    async fn fixture_student_with_absences(
        pool: &SqlitePool,
        absence_dates: &[&str],
    ) -> (i64, Vec<i64>) {
        let sid = seed_student(
            pool,
            "S001",
            "2026-01-01",
            None,
            // 월(1)~금(5) 정규 수업
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
        )
        .await;
        let mut absence_ids = Vec::with_capacity(absence_dates.len());
        for date in absence_dates {
            let aid = insert_absence(pool, sid, date, "2026-06", 60, Some("2026-07")).await;
            absence_ids.push(aid);
        }
        (sid, absence_ids)
    }

    /// T3 픽스처 — event_date 에 보강 가능 학사일정 1건 등록 (공휴수업일).
    async fn fixture_makeup_eligible_date(pool: &SqlitePool, event_date: &str) {
        let code = schedule_code_id(pool, "공휴수업일").await;
        insert_schedule_event(pool, code, event_date, None).await;
    }

    fn payload(student_id: i64, event_date: &str, absence_ids: Vec<i64>) -> CreateMakeupPayload {
        payload_minutes(student_id, event_date, absence_ids, 60)
    }

    fn payload_minutes(
        student_id: i64,
        event_date: &str,
        absence_ids: Vec<i64>,
        class_minutes: i64,
    ) -> CreateMakeupPayload {
        CreateMakeupPayload {
            student_id,
            event_date: event_date.to_string(),
            class_minutes,
            absence_ids,
        }
    }

    /// 결석의 현재 잔여 보강필요분 (class_minutes - 배분합).
    async fn remaining_of(pool: &SqlitePool, absence_id: i64) -> i64 {
        let r: (i64,) = sqlx::query_as(
            "SELECT class_minutes - COALESCE((SELECT SUM(allocated_minutes) \
                    FROM makeup_allocations WHERE absence_id = ?), 0) \
             FROM regular_attendances WHERE id = ?",
        )
        .bind(absence_id)
        .bind(absence_id)
        .fetch_one(pool)
        .await
        .expect("잔여 조회");
        r.0
    }

    /// 결석의 현재 status.
    async fn status_of(pool: &SqlitePool, absence_id: i64) -> String {
        let r: (String,) =
            sqlx::query_as("SELECT status FROM regular_attendances WHERE id = ?")
                .bind(absence_id)
                .fetch_one(pool)
                .await
                .expect("상태 조회");
        r.0
    }

    /// 특정 보강(makeup_id)이 특정 결석에 배분한 분(없으면 0).
    async fn alloc_of(pool: &SqlitePool, makeup_id: i64, absence_id: i64) -> i64 {
        let r: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(allocated_minutes), 0) FROM makeup_allocations \
             WHERE makeup_id = ? AND absence_id = ?",
        )
        .bind(makeup_id)
        .bind(absence_id)
        .fetch_one(pool)
        .await
        .expect("배분 조회");
        r.0
    }

    /// AC-T3-1 (신규 분 단위): 결석 2건(각 60분)에 120분 보강 → 2건 모두 완전 소진.
    #[tokio::test]
    async fn create_makeup_matches_absences_atomically() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) =
            fixture_student_with_absences(&pool, &["2026-06-15", "2026-06-16"]).await;
        // 2026-06-13 = 토요일 (정규 수업 요일 아님)
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let result = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", absences.clone(), 120),
        )
        .await
        .expect("정상 매칭");
        assert_eq!(result.matched_count, 2);
        assert_eq!(result.student_id, sid);
        assert!(result.makeup_id > 0);

        // 결석 2건 모두 makeup_done + 잔여 0 + 각 60분 배분
        for aid in absences {
            assert_eq!(status_of(&pool, aid).await, "makeup_done");
            assert_eq!(remaining_of(&pool, aid).await, 0);
            assert_eq!(alloc_of(&pool, result.makeup_id, aid).await, 60);
        }
    }

    /// AC-T3-2: 빈 absence_ids 거부.
    #[tokio::test]
    async fn create_makeup_rejects_empty_absences() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, _) = fixture_student_with_absences(&pool, &[]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        let err = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", vec![]))
            .await
            .expect_err("빈 absence_ids 거부");
        assert!(err.contains("1건 이상"), "친화 메시지: {}", err);
    }

    /// AC-T3-3: 보강 불가 일자 차단 — event_date 에 allows_makeup_class 학사일정 없음.
    #[tokio::test]
    async fn create_makeup_blocks_when_event_date_not_makeup_eligible() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        // 학사일정 미등록 → 2026-06-13 은 보강 가능 아님
        let err = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", absences))
            .await
            .expect_err("보강 불가 일자 차단");
        assert!(err.contains("보강 가능 일자가 아닙니다"), "친화 메시지: {}", err);
    }

    /// AC-T3-4 (Session #10 정책 전환): 정규 수업 요일에도 보강 등록 허용.
    /// 사용자 결정 — "수업 요일에 추가 시간 써서 수업 완료 후 보강 진행 가능".
    #[tokio::test]
    async fn create_makeup_allows_regular_class_weekday() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 2026-06-15 는 월요일 — 학생의 정규 수업 요일이지만 보강 허용
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-16"]).await;
        // 학사일정 없어도 평일이라 케이스 A 로 가능 — 다만 결석일과 다른 일자여야 함
        let r = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-15", absences.clone()))
            .await
            .expect("정규 수업 요일에도 보강 허용");
        assert_eq!(r.matched_count, 1);
        // 60분 결석에 60분 보강 → 완전 소진 (makeup_attendance_id 는 레거시 — 검증하지 않음)
        assert_eq!(status_of(&pool, absences[0]).await, "makeup_done");
        assert_eq!(remaining_of(&pool, absences[0]).await, 0);
        assert_eq!(alloc_of(&pool, r.makeup_id, absences[0]).await, 60);
    }

    /// AC-T3-5: 무효 absence_id (미존재) 거부.
    #[tokio::test]
    async fn create_makeup_rejects_nonexistent_absence_id() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, _) = fixture_student_with_absences(&pool, &[]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        let err =
            create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", vec![99999]))
                .await
                .expect_err("미존재 결석 거부");
        assert!(err.contains("찾을 수 없습니다"));
    }

    /// AC-T3-6: 다른 학생의 결석 거부 — 학생 일관성 검증.
    #[tokio::test]
    async fn create_makeup_rejects_other_students_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (s1, _) = fixture_student_with_absences(&pool, &[]).await;
        let s2 = seed_student(&pool, "S002", "2026-01-01", None, &[]).await;
        let other_aid = insert_absence(&pool, s2, "2026-06-15", "2026-06", 60, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let err = create_makeup_with_absences_impl(
            &pool,
            &payload(s1, "2026-06-13", vec![other_aid]),
        )
        .await
        .expect_err("다른 학생 결석 거부");
        assert!(err.contains("다른 학생"));
    }

    /// AC-T3-7: 이미 매칭된 결석 거부 — makeup_attendance_id NOT NULL.
    #[tokio::test]
    async fn create_makeup_rejects_already_matched_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        fixture_makeup_eligible_date(&pool, "2026-06-20").await;

        // 첫 보강 등록 — 정상 매칭
        let r1 =
            create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", absences.clone()))
                .await
                .expect("첫 매칭");
        assert_eq!(r1.matched_count, 1);

        // 같은 결석(60분 완전 소진 → makeup_done)으로 두 번째 보강 시도 → 미처리(absent) 아님 거부
        let err = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-20", absences))
            .await
            .expect_err("완전 소진된 결석 재매칭 거부");
        assert!(err.contains("미처리 결석"), "친화 메시지: {}", err);
    }

    /// AC-T3-8: 트랜잭션 원자성 — 일부 결석 유효성 검증 실패 시 makeup INSERT 도 롤백.
    #[tokio::test]
    async fn create_makeup_rolls_back_on_validation_failure() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        // 유효한 결석 + 미존재 id 혼합 → 트랜잭션 전체 롤백
        let mut mixed = absences.clone();
        mixed.push(99999);

        let err =
            create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", mixed)).await;
        assert!(err.is_err(), "혼합 입력 거부");

        // 검증 4 가 트랜잭션 시작 전에 실행되므로 makeup_attendances 도 0건
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0, "롤백 후 makeup_attendances 0건");

        // 유효 결석도 여전히 absent 상태 유지 (UPDATE 안 됨)
        let row: (String,) =
            sqlx::query_as("SELECT status FROM regular_attendances WHERE id=?")
                .bind(absences[0])
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(row.0, "absent");
    }

    /// AC-T3-9: 입교일 이전 event_date 거부.
    #[tokio::test]
    async fn create_makeup_rejects_before_enroll_date() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-06-20", None, &[(1, 1)]).await;
        let aid =
            insert_absence(&pool, sid, "2026-06-22", "2026-06", 60, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let err = create_makeup_with_absences_impl(
            &pool,
            &payload(sid, "2026-06-13", vec![aid]),
        )
        .await
        .expect_err("입교일 이전 거부");
        assert!(err.contains("입교일 이전"));
    }

    // ─────────────── T4: cancel_makeup ───────────────

    /// 보강 등록 후 cancel → 결석 absent 환원 + makeup_attendances 0건.
    #[tokio::test]
    async fn cancel_makeup_reverts_absences_and_deletes_makeup() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) =
            fixture_student_with_absences(&pool, &["2026-06-15", "2026-06-16"]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        // 결석 2건(각 60분)에 120분 보강 → 2건 완전 소진
        let r = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", absences.clone(), 120),
        )
        .await
        .expect("등록");

        let reverted = cancel_makeup_impl(&pool, r.makeup_id)
            .await
            .expect("취소");
        assert_eq!(reverted, 2);

        // 결석 2건 모두 absent 환원 + 잔여 60 복원
        for aid in &absences {
            assert_eq!(status_of(&pool, *aid).await, "absent");
            assert_eq!(remaining_of(&pool, *aid).await, 60);
        }
        // makeup_attendances 0건 + makeup_allocations 0건
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
        let acount: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_allocations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(acount.0, 0);
    }

    /// 존재하지 않는 makeup_id 취소 — 친화 에러.
    #[tokio::test]
    async fn cancel_makeup_rejects_nonexistent_id() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let err = cancel_makeup_impl(&pool, 99999)
            .await
            .expect_err("미존재 거부");
        assert!(err.contains("찾을 수 없습니다"));
    }

    // ─────────────── T8: get_absence_history ───────────────

    /// 결석 이력 — absent/makeup_done/makeup_expired 모두 포함, 출석은 제외.
    /// event_date DESC 정렬.
    #[tokio::test]
    async fn absence_history_includes_three_states_in_desc_order() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        // 출석 (제외 대상)
        sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, status) \
             VALUES (?, '2026-06-10', '2026-06', 60, 'present')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .unwrap();
        // 미처리 결석 + 보강소멸 — JOIN 대상 아님
        insert_absence(&pool, sid, "2026-06-15", "2026-06", 60, Some("2026-07")).await;
        sqlx::query(
            "INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes, status, makeup_deadline) \
             VALUES (?, '2026-06-05', '2026-06', 60, 'makeup_expired', '2026-06')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .unwrap();
        // 보강완료 — JOIN으로 makeup 정보 포함
        let aid =
            insert_absence(&pool, sid, "2026-06-20", "2026-06", 60, Some("2026-07")).await;
        let mid: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-25', '2026-06', 90) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        sqlx::query(
            "UPDATE regular_attendances SET status='makeup_done', makeup_attendance_id=? WHERE id=?",
        )
        .bind(mid.0)
        .bind(aid)
        .execute(&pool)
        .await
        .unwrap();

        let history = get_absence_history_impl(&pool, sid).await.expect("이력");
        // 출석 제외 → 3건. DESC 정렬: 06-20, 06-15, 06-05
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].event_date, "2026-06-20");
        assert_eq!(history[0].status, "makeup_done");
        assert_eq!(history[0].makeup_event_date, Some("2026-06-25".to_string()));
        assert_eq!(history[0].makeup_class_minutes, Some(90));

        assert_eq!(history[1].event_date, "2026-06-15");
        assert_eq!(history[1].status, "absent");
        assert_eq!(history[1].makeup_event_date, None);

        assert_eq!(history[2].event_date, "2026-06-05");
        assert_eq!(history[2].status, "makeup_expired");
    }

    /// 결석 이력이 없는 학생 — 빈 vec.
    #[tokio::test]
    async fn absence_history_returns_empty_when_no_absences() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let history = get_absence_history_impl(&pool, sid).await.expect("빈 이력");
        assert!(history.is_empty());
    }

    /// 다른 학생의 결석은 제외 (student_id 필터 확인).
    #[tokio::test]
    async fn absence_history_filters_by_student_id() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let s1 = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        let s2 = seed_student(&pool, "S002", "2026-01-01", None, &[]).await;
        insert_absence(&pool, s1, "2026-06-15", "2026-06", 60, Some("2026-07")).await;
        insert_absence(&pool, s2, "2026-06-16", "2026-06", 60, Some("2026-07")).await;

        let h1 = get_absence_history_impl(&pool, s1).await.expect("s1");
        let h2 = get_absence_history_impl(&pool, s2).await.expect("s2");
        assert_eq!(h1.len(), 1);
        assert_eq!(h1[0].event_date, "2026-06-15");
        assert_eq!(h2.len(), 1);
        assert_eq!(h2[0].event_date, "2026-06-16");
    }

    // ─────────────── Sprint 10 T7 — 선행 수업 (PRD §4.2.3) ───────────────

    /// 선행 수업 시나리오 — 미래 결석(6/20)을 현재 보강(6/13)이 충당.
    /// 백엔드는 보강일 < 결석일 순서 검증을 하지 않으므로 PRD §4.2.3 시나리오 자연스럽게 지원.
    /// UI 필터(MakeupRegisterDialog::filteredPending) 는 별도 — UI 차원의 제약.
    #[tokio::test]
    async fn create_makeup_supports_future_absence_for_advance_class() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 미래 일자 6/20 에 결석 사전 등록 (학부모 통보로 셀 토글한 상태 가정).
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-20"]).await;
        // 보강 일자 6/13 (토요일, 공휴수업일 코드 등록)
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        // 보강일(6/13) < 결석일(6/20) — 선행 수업 시나리오.
        let result = create_makeup_with_absences_impl(
            &pool,
            &payload(sid, "2026-06-13", absences.clone()),
        )
        .await
        .expect("선행 수업 보강 등록 성공");

        assert_eq!(result.matched_count, 1);

        // 미래 결석이 makeup_done 으로 전이 + 60분 배분됨 (선행 수업 시나리오).
        assert_eq!(status_of(&pool, absences[0]).await, "makeup_done");
        assert_eq!(remaining_of(&pool, absences[0]).await, 0);
        assert_eq!(alloc_of(&pool, result.makeup_id, absences[0]).await, 60);
    }

    // ─────────────── T1: makeup_allocations 스키마 (V311, ADR-011) ───────────────

    /// V311 마이그레이션이 인메모리 풀에 적용되어 유효 배분 행을 INSERT 할 수 있다.
    #[tokio::test]
    async fn makeup_allocations_accepts_valid_row() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        let mid: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-13', '2026-06', 60) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("makeup INSERT");
        sqlx::query(
            "INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) \
             VALUES (?, ?, 60)",
        )
        .bind(mid.0)
        .bind(absences[0])
        .execute(&pool)
        .await
        .expect("유효 배분 INSERT 성공");
        let cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_allocations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(cnt.0, 1);
    }

    /// allocated_minutes <= 0 은 CHECK 제약 위반.
    #[tokio::test]
    async fn makeup_allocations_rejects_non_positive_minutes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        let mid: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-13', '2026-06', 60) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("makeup INSERT");
        let err = sqlx::query(
            "INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) \
             VALUES (?, ?, 0)",
        )
        .bind(mid.0)
        .bind(absences[0])
        .execute(&pool)
        .await;
        assert!(err.is_err(), "allocated_minutes=0 은 CHECK 위반");
    }

    /// (makeup_id, absence_id) 쌍은 UNIQUE — 같은 보강이 같은 결석에 중복 배분 불가.
    #[tokio::test]
    async fn makeup_allocations_rejects_duplicate_pair() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        let mid: (i64,) = sqlx::query_as(
            "INSERT INTO makeup_attendances (student_id, event_date, year_month, class_minutes) \
             VALUES (?, '2026-06-13', '2026-06', 120) RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .expect("makeup INSERT");
        sqlx::query(
            "INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) VALUES (?, ?, 30)",
        )
        .bind(mid.0)
        .bind(absences[0])
        .execute(&pool)
        .await
        .expect("첫 배분");
        let err = sqlx::query(
            "INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) VALUES (?, ?, 30)",
        )
        .bind(mid.0)
        .bind(absences[0])
        .execute(&pool)
        .await;
        assert!(err.is_err(), "(makeup_id, absence_id) 중복 배분 거부");
    }

    // ─────────────── T2 신규: 분 단위 부분 차감 ───────────────

    /// 120분 결석에 60분 보강 → 부분 소진 (status='absent' 유지, 잔여 60분).
    #[tokio::test]
    async fn create_makeup_partial_leaves_remaining() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 2)]).await;
        let aid = insert_absence(&pool, sid, "2026-06-16", "2026-06", 120, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let r = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", vec![aid], 60),
        )
        .await
        .expect("부분 보강 등록");
        assert_eq!(r.matched_count, 1);
        // 잔여 60분 → 여전히 미처리(absent) → 보강 대상 유지
        assert_eq!(status_of(&pool, aid).await, "absent");
        assert_eq!(remaining_of(&pool, aid).await, 60);
        assert_eq!(alloc_of(&pool, r.makeup_id, aid).await, 60);
    }

    /// 120분 결석에 60분 보강 2회 → 완전 소진.
    #[tokio::test]
    async fn create_makeup_two_partials_complete() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 2)]).await;
        let aid = insert_absence(&pool, sid, "2026-06-16", "2026-06", 120, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        fixture_makeup_eligible_date(&pool, "2026-06-20").await;

        // 1차 60분
        create_makeup_with_absences_impl(&pool, &payload_minutes(sid, "2026-06-13", vec![aid], 60))
            .await
            .expect("1차 부분 보강");
        assert_eq!(status_of(&pool, aid).await, "absent");
        assert_eq!(remaining_of(&pool, aid).await, 60);

        // 2차 60분 → 완전 소진
        create_makeup_with_absences_impl(&pool, &payload_minutes(sid, "2026-06-20", vec![aid], 60))
            .await
            .expect("2차 부분 보강");
        assert_eq!(status_of(&pool, aid).await, "makeup_done");
        assert_eq!(remaining_of(&pool, aid).await, 0);
    }

    /// 보강 시간이 선택 결석 잔여 합계를 초과하면 거부 (신규 등록은 엄격).
    #[tokio::test]
    async fn create_makeup_rejects_over_allocation() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 1)]).await;
        let aid = insert_absence(&pool, sid, "2026-06-16", "2026-06", 60, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        // 결석 잔여 60분인데 120분 보강 시도 → 거부
        let err = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", vec![aid], 120),
        )
        .await
        .expect_err("초과 배분 거부");
        assert!(err.contains("잔여 보강필요시간"), "친화 메시지: {}", err);
        // 롤백 확인 — makeup 0건
        let mc: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(mc.0, 0);
    }

    /// 소멸기한 임박 순으로 배분 — 60분 보강은 마감 임박 결석부터 채운다.
    #[tokio::test]
    async fn create_makeup_allocates_by_deadline_first() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 1)]).await;
        // A: 마감 2026-07(임박), B: 마감 2026-08 — 각 60분
        let a_urgent =
            insert_absence(&pool, sid, "2026-06-10", "2026-06", 60, Some("2026-07")).await;
        let b_later =
            insert_absence(&pool, sid, "2026-06-11", "2026-06", 60, Some("2026-08")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        // 60분 보강 → 임박한 A 부터 채움 (입력 순서를 B,A 로 줘도 정렬로 A 우선)
        let r = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", vec![b_later, a_urgent], 60),
        )
        .await
        .expect("배분");
        assert_eq!(r.matched_count, 1, "임박 결석 1건만 배분");
        assert_eq!(status_of(&pool, a_urgent).await, "makeup_done");
        assert_eq!(remaining_of(&pool, a_urgent).await, 0);
        assert_eq!(status_of(&pool, b_later).await, "absent");
        assert_eq!(remaining_of(&pool, b_later).await, 60);
    }

    // ─────────────── T3 신규: 부분 차감 취소 ───────────────

    /// 부분 소진 보강 취소 → 잔여 복원 (배분만 제거, status='absent' 유지 → 환원 카운트 0).
    #[tokio::test]
    async fn cancel_partial_makeup_restores_remaining() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 2)]).await;
        let aid = insert_absence(&pool, sid, "2026-06-16", "2026-06", 120, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        let r = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", vec![aid], 60),
        )
        .await
        .expect("부분 보강");
        assert_eq!(remaining_of(&pool, aid).await, 60);

        let reverted = cancel_makeup_impl(&pool, r.makeup_id).await.expect("취소");
        assert_eq!(reverted, 0, "부분 소진(absent)은 환원 카운트 제외");
        assert_eq!(status_of(&pool, aid).await, "absent");
        assert_eq!(remaining_of(&pool, aid).await, 120);
    }

    /// 다중 보강 중 1건 취소 → 나머지 배분은 영향 없음.
    #[tokio::test]
    async fn cancel_one_of_multiple_makeups_keeps_others() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[(1, 2)]).await;
        let aid = insert_absence(&pool, sid, "2026-06-16", "2026-06", 120, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        fixture_makeup_eligible_date(&pool, "2026-06-20").await;

        let r1 = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-13", vec![aid], 60),
        )
        .await
        .expect("1차");
        let r2 = create_makeup_with_absences_impl(
            &pool,
            &payload_minutes(sid, "2026-06-20", vec![aid], 60),
        )
        .await
        .expect("2차");
        assert_eq!(status_of(&pool, aid).await, "makeup_done");

        // 2차 보강 취소 → 잔여 60 복원 + makeup_done→absent 환원
        let reverted = cancel_makeup_impl(&pool, r2.makeup_id).await.expect("취소");
        assert_eq!(reverted, 1);
        assert_eq!(status_of(&pool, aid).await, "absent");
        assert_eq!(remaining_of(&pool, aid).await, 60);
        // 1차 배분은 그대로 유지
        assert_eq!(alloc_of(&pool, r1.makeup_id, aid).await, 60);
    }
}
