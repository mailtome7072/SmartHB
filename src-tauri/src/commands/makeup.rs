//! 보강 도메인 IPC (Sprint 9~10, PRD §4.5.4~6, §4.5.10).
//!
//! Phase 3 — 보강 등록(개별/일괄) + 매칭 + 취소/미등원 + 결석 이력.
//! 본 모듈은 attendance.rs 와 별개 도메인 — V107 FK
//! (`regular_attendances.makeup_attendance_id → makeup_attendances.id`) 를 통해 정규 출결과
//! 연결되지만, 보강 등록/취소/미등원 트랜잭션은 본 모듈이 담당한다.
//!
//! ## Sprint 9 진입점
//! - T2 (본 세션): `get_pending_absences`, `get_makeup_eligible_dates` IPC 2종
//! - T3: `create_makeup_with_absences` (트랜잭션 매칭)
//! - T4: `cancel_makeup`, `mark_makeup_absent`, `batch_create_makeups`
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
// 응답 구조체 (camelCase serde)
// ────────────────────────────────────────────────────────────────────

/// 원생의 미처리 결석 1건 — `status='absent' AND makeup_attendance_id IS NULL`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAbsence {
    pub id: i64,
    pub event_date: String,
    pub year_month: String,
    pub class_minutes: i64,
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

/// 보강데이 일괄 등록 — 학생 1명분 입력 (T4 `batch_create_makeups`).
///
/// 학생별로 `class_minutes` 다를 수 있어 entry 에 포함 (보강데이라도 학생마다 정규
/// 수업 시간 상이). `event_date` 는 batch 전체 공통.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchMakeupEntry {
    pub student_id: i64,
    pub class_minutes: i64,
    pub absence_ids: Vec<i64>,
}

/// 보강데이 일괄 등록 페이로드.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateMakeupsPayload {
    pub event_date: String,
    pub entries: Vec<BatchMakeupEntry>,
}

/// 일괄 등록 실패 1건 — 학생 id + 실패 사유 (사용자 친화 메시지).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFailure {
    pub student_id: i64,
    pub reason: String,
}

/// 일괄 등록 결과 — 학생별 독립 트랜잭션으로 부분 성공 처리.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub succeeded: Vec<MakeupResult>,
    pub failed: Vec<BatchFailure>,
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
    let rows = sqlx::query(
        "SELECT id, event_date, year_month, class_minutes, makeup_deadline, absence_memo \
         FROM regular_attendances \
         WHERE student_id = ? AND status = 'absent' AND makeup_attendance_id IS NULL \
         ORDER BY (makeup_deadline IS NULL), makeup_deadline ASC, event_date ASC",
    )
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
            makeup_deadline: r.try_get("makeup_deadline").map_err(|e| e.to_string())?,
            absence_memo: r.try_get("absence_memo").map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 가능 일자 조회
// ────────────────────────────────────────────────────────────────────

/// 원생의 보강 가능 일자 조회 — year_month 내 `allows_makeup_class=1` 학사일정 일자.
///
/// 알고리즘:
/// 1. year_month 의 모든 일자 펼침
/// 2. `schedule_events JOIN schedule_codes WHERE allows_makeup_class=1` 의 단일/기간 일자 펼침
/// 3. 학생 입교일 이전 / 퇴교일 이후 일자 제외
/// 4. 동일 일자 중복 학사일정은 첫 코드명으로 통합 (BTreeMap)
///
/// 정규 수업 요일 필터는 본 IPC 가 아닌 T3 `create_makeup_with_absences` 트랜잭션 검증
/// 단계에서 적용 — 책임 분담 단순화 (학생이 이미 결석한 정규 수업일에 같은 학생의 보강을
/// 등록하는 시나리오는 드물고, 본 IPC 는 "후보 일자" 목록 제공이 책임).
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
    // 1. 학생 입퇴교 범위 조회 (정규 수업 요일은 본 IPC 책임 외).
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

    // 3. 학사일정 중 allows_makeup_class=1 인 일자/기간을 month 와 겹치는 범위로 조회.
    // attendance.rs::load_off_dates 와 동일 패턴 — 단일 쿼리 + Rust 측 펼침.
    let makeup_rows = sqlx::query(
        "SELECT e.event_date, COALESCE(e.period_end_date, e.event_date) AS end_d, c.code_name \
         FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_makeup_class = 1 \
           AND e.event_date < ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(next_month_first.to_string())
    .bind(first.to_string())
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강 가능 학사일정 조회 실패: {}", e))?;

    // 4. 일자 펼침 + month 범위 내만 + 학생 입퇴교 범위 필터.
    // BTreeMap 으로 동일 일자 중복 코드 회피 + event_date 정렬 자동.
    let mut eligible: BTreeMap<String, String> = BTreeMap::new();
    for r in makeup_rows {
        let s: String = r.try_get("event_date").map_err(|e| e.to_string())?;
        let e_str: String = r.try_get("end_d").map_err(|e| e.to_string())?;
        let code_name: String = r.try_get("code_name").map_err(|e| e.to_string())?;
        let mut d = NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 일자 파싱 실패: {}", e))?;
        let ed = NaiveDate::parse_from_str(&e_str, "%Y-%m-%d")
            .map_err(|e| format!("이벤트 종료일 파싱 실패: {}", e))?;
        while d <= ed {
            if d >= first && d < next_month_first && d >= enroll_d {
                let in_withdraw_range = withdraw_d.is_none_or(|wd| d <= wd);
                if in_withdraw_range {
                    eligible
                        .entry(d.to_string())
                        .or_insert_with(|| code_name.clone());
                }
            }
            d = d.succ_opt().expect("date succ");
        }
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
/// 트랜잭션 내 검증 5종:
/// 1. **이벤트 일자 보강 가능** — `event_date` 에 `allows_makeup_class=1` 학사일정 존재
/// 2. **학생 일관성** — 학생 존재 + 입퇴교 범위 내 `event_date`
/// 3. **정규 수업 요일 차단** — `event_date` 가 학생의 정규 수업 요일이면 거부
///    (해당 요일은 정규 출결 대상이므로 보강 등록은 비수업일 한정 — PRD §4.5.4)
/// 4. **결석 유효성** — `absence_ids` 모두 해당 학생 + `status='absent'` + 미매칭
/// 5. **PI-02 시간값** — 옵션 A (일 단위) 채택: 검증 생략. 분 단위 전환 시 본 함수 내
///    "PI-02 분 단위 활성 위치" 주석 위치에서 1줄 추가만으로 활성화 가능.
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

    // 검증 1: event_date 가 보강 가능 학사일정 일자인지 (allows_makeup_class=1 + 기간 매칭).
    let makeup_eligible: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.allows_makeup_class = 1 \
           AND e.event_date <= ? AND COALESCE(e.period_end_date, e.event_date) >= ?",
    )
    .bind(&payload.event_date)
    .bind(&payload.event_date)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("보강 가능 일자 검증 실패: {}", e))?;
    if makeup_eligible.0 == 0 {
        return Err(format!(
            "{} 은 보강 가능 일자가 아닙니다 (학사일정에서 '보강 진행 가능' 코드가 활성된 일자에만 등록 가능합니다).",
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

    // 검증 3: 정규 수업 요일 차단 — 학생의 student_schedules.day_of_week 와 일치하면 거부.
    let weekday = event_d.weekday().number_from_monday() as i64;
    let regular_match: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM student_schedules WHERE student_id = ? AND day_of_week = ?",
    )
    .bind(payload.student_id)
    .bind(weekday)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("정규 수업 요일 검증 실패: {}", e))?;
    if regular_match.0 > 0 {
        return Err("학생의 정규 수업 요일에는 보강을 등록할 수 없습니다 (비수업일에만 가능).".to_string());
    }

    // 검증 4: 결석 유효성 — 모두 본 학생 + status='absent' + 미매칭.
    // SQL IN 절은 동적 placeholder 가 sqlx 에서 까다로워 — Rust 측 루프로 단건 검증.
    for &aid in &payload.absence_ids {
        let row = sqlx::query(
            "SELECT student_id, status, makeup_attendance_id \
             FROM regular_attendances WHERE id = ?",
        )
        .bind(aid)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("결석 조회 실패 (id={}): {}", aid, e))?;
        let row = row.ok_or_else(|| format!("결석을 찾을 수 없습니다 (id={}).", aid))?;
        let sid: i64 = row.try_get("student_id").map_err(|e| e.to_string())?;
        let status: String = row.try_get("status").map_err(|e| e.to_string())?;
        let matched: Option<i64> = row
            .try_get("makeup_attendance_id")
            .map_err(|e| e.to_string())?;
        if sid != payload.student_id {
            return Err(format!("결석 id={} 가 다른 학생의 것입니다.", aid));
        }
        // matched 체크를 status 보다 먼저 — 정상 매칭된 결석(status='makeup_done', matched=Some)
        // 케이스에 "이미 다른 보강" 메시지가 더 정확. status 분기는 makeup_expired 같은
        // 예외 상태(matched=None) 케이스를 잡는다.
        if matched.is_some() {
            return Err(format!(
                "결석 id={} 는 이미 다른 보강에 매칭되어 있습니다.",
                aid
            ));
        }
        if status != "absent" {
            return Err(format!(
                "결석 id={} 의 상태가 '{}' 입니다 — 미처리 결석(absent) 만 매칭 가능합니다.",
                aid, status
            ));
        }
    }

    // 검증 5 (PI-02 분 단위 활성 위치): 옵션 A 일 단위 채택으로 생략.
    // 분 단위 전환 시 아래 주석 해제:
    // let total: (i64,) = sqlx::query_as(
    //     "SELECT COALESCE(SUM(class_minutes),0) FROM regular_attendances WHERE id IN (...)"
    // ).fetch_one(pool).await...;
    // if payload.class_minutes < total.0 { return Err("보강 시간이 결석 합계보다 적습니다."); }

    // 실행: 단일 트랜잭션 — INSERT makeup → UPDATE absences.
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

    let mut matched_count = 0usize;
    for &aid in &payload.absence_ids {
        let res = sqlx::query(
            "UPDATE regular_attendances \
             SET status = 'makeup_done', makeup_attendance_id = ?, \
                 updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
             WHERE id = ? AND status = 'absent' AND makeup_attendance_id IS NULL",
        )
        .bind(makeup_id)
        .bind(aid)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("결석 매칭 UPDATE 실패 (id={}): {}", aid, e))?;
        if res.rows_affected() != 1 {
            // 검증 4 통과 후 race 가 발생한 경우 — 트랜잭션 롤백.
            return Err(format!(
                "결석 id={} 매칭 실패 (검증 후 상태 변경 추정). 트랜잭션 롤백.",
                aid
            ));
        }
        matched_count += 1;
    }

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(MakeupResult {
        makeup_id,
        student_id: payload.student_id,
        event_date: payload.event_date.clone(),
        matched_count,
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

    let revert_res = sqlx::query(
        "UPDATE regular_attendances \
         SET makeup_attendance_id = NULL, status = 'absent', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE makeup_attendance_id = ?",
    )
    .bind(makeup_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("결석 환원 실패: {}", e))?;

    sqlx::query("DELETE FROM makeup_attendances WHERE id = ?")
        .bind(makeup_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("보강 DELETE 실패: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(revert_res.rows_affected() as usize)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강 미등원 (Sprint 9 T4)
// ────────────────────────────────────────────────────────────────────

/// 보강 약속에 학생이 등원하지 않은 경우 — 보강 상태를 `makeup_absent` 로 마킹하고
/// 연결된 결석은 `absent` 로 환원 (결석 상태 유지, 새 결석 미생성).
///
/// 보강 행은 보존 (DELETE 안 함) — 미등원 이력 추적 + 차후 분석.
/// 연결된 결석은 다음 보강 매칭 대상으로 재진입 가능 (`status='absent'` + `makeup_attendance_id=NULL`).
#[tauri::command]
pub async fn mark_makeup_absent(makeup_id: i64) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let reverted = mark_makeup_absent_impl(pool, makeup_id).await?;
    audit::try_record(
        AuditEventType::MakeupAbsent,
        Some(&makeup_id.to_string()),
        Some(&format!(r#"{{"revertedAbsences":{}}}"#, reverted)),
    )
    .await;
    Ok(())
}

async fn mark_makeup_absent_impl(pool: &SqlitePool, makeup_id: i64) -> Result<usize, String> {
    // 보강 존재 + 현재 상태 확인 — 이미 makeup_absent 이면 멱등 처리.
    let row = sqlx::query("SELECT status FROM makeup_attendances WHERE id = ?")
        .bind(makeup_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("보강 조회 실패: {}", e))?
        .ok_or_else(|| format!("보강 id={} 를 찾을 수 없습니다.", makeup_id))?;
    let status: String = row.try_get("status").map_err(|e| e.to_string())?;
    if status == "makeup_absent" {
        return Ok(0); // 이미 미등원 처리됨 — 멱등.
    }

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| format!("트랜잭션 시작 실패: {}", e))?;

    sqlx::query(
        "UPDATE makeup_attendances SET status = 'makeup_absent', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE id = ?",
    )
    .bind(makeup_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("보강 미등원 마킹 실패: {}", e))?;

    let revert_res = sqlx::query(
        "UPDATE regular_attendances \
         SET makeup_attendance_id = NULL, status = 'absent', \
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') \
         WHERE makeup_attendance_id = ?",
    )
    .bind(makeup_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| format!("결석 환원 실패: {}", e))?;

    tx.commit()
        .await
        .map_err(|e| format!("트랜잭션 커밋 실패: {}", e))?;

    Ok(revert_res.rows_affected() as usize)
}

// ────────────────────────────────────────────────────────────────────
// IPC: 보강데이 일괄 등록 (Sprint 9 T4)
// ────────────────────────────────────────────────────────────────────

/// 보강데이 — 같은 일자에 여러 학생을 일괄 보강 등록.
///
/// **학생별 독립 트랜잭션** — 한 학생 실패해도 다른 학생은 계속 진행 (부분 성공 처리).
/// 단일 트랜잭션이 아닌 이유: PRD §4.5.5 "실패 원생은 건너뛰고 성공/실패 결과 반환".
/// 한 학생 실패 (예: 정규 수업 요일) 가 다른 학생들의 정상 등록을 차단하면 UX 손상.
///
/// 학생별로 `create_makeup_with_absences_impl` 재사용 — 검증 5종 동일 적용.
/// 실패 시 사용자 친화 메시지를 `BatchFailure.reason` 에 누적.
#[tauri::command]
pub async fn batch_create_makeups(
    payload: BatchCreateMakeupsPayload,
) -> Result<BatchResult, String> {
    let pool = db::pool().map_err(String::from)?;
    batch_create_makeups_impl(pool, &payload).await
}

async fn batch_create_makeups_impl(
    pool: &SqlitePool,
    payload: &BatchCreateMakeupsPayload,
) -> Result<BatchResult, String> {
    if payload.entries.is_empty() {
        return Err("보강 등록할 원생을 1명 이상 선택해야 합니다.".to_string());
    }

    let mut succeeded: Vec<MakeupResult> = Vec::with_capacity(payload.entries.len());
    let mut failed: Vec<BatchFailure> = Vec::new();

    for entry in &payload.entries {
        let single_payload = CreateMakeupPayload {
            student_id: entry.student_id,
            event_date: payload.event_date.clone(),
            class_minutes: entry.class_minutes,
            absence_ids: entry.absence_ids.clone(),
        };
        match create_makeup_with_absences_impl(pool, &single_payload).await {
            Ok(result) => {
                // 학생별 성공 시 audit 기록 — 일괄도 학생별 fire-and-forget.
                audit::try_record(
                    AuditEventType::MakeupCreated,
                    Some(&result.makeup_id.to_string()),
                    Some(&format!(
                        r#"{{"studentId":{},"eventDate":"{}","matchedCount":{},"batch":true}}"#,
                        result.student_id, result.event_date, result.matched_count
                    )),
                )
                .await;
                succeeded.push(result);
            }
            Err(reason) => {
                failed.push(BatchFailure {
                    student_id: entry.student_id,
                    reason,
                });
            }
        }
    }

    Ok(BatchResult { succeeded, failed })
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

    /// AC-T2-2: allows_makeup_class=1 인 학사일정이 있는 일자만 반환.
    /// V301 시드 — "공휴수업일" 코드(allows_makeup_class=1).
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
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-15", "2026-06-22"]);
    }

    /// AC-T2-2 보강: allows_makeup_class=0 인 학사일정은 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_makeup_off_codes() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", None, &[]).await;
        // 방학 코드: allows_makeup_class=0 (V102 시드)
        let vac_code = schedule_code_id(&pool, "방학").await;
        insert_schedule_event(&pool, vac_code, "2026-06-15", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        assert!(list.is_empty(), "방학(allows_makeup_class=0)은 보강 가능 아님");
    }

    /// AC-T2-3: 학생 입교일 이전 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_before_enroll() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        // 6/20 입교 학생 — 6/15 학사일정은 입교 전이라 제외
        let sid = seed_student(&pool, "S001", "2026-06-20", None, &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-22"], "입교 전 6/15 제외");
    }

    /// AC-T2-3 보강: 학생 퇴교일 이후 일자 제외.
    #[tokio::test]
    async fn eligible_dates_excludes_after_withdraw() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let sid = seed_student(&pool, "S001", "2026-01-01", Some("2026-06-18"), &[]).await;
        let makeup_code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, makeup_code, "2026-06-15", None).await;
        insert_schedule_event(&pool, makeup_code, "2026-06-22", None).await;

        let list = get_makeup_eligible_dates_impl(&pool, sid, "2026-06")
            .await
            .expect("조회");
        let dates: Vec<String> = list.iter().map(|e| e.event_date.clone()).collect();
        assert_eq!(dates, vec!["2026-06-15"], "퇴교 후 6/22 제외");
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
        assert_eq!(dates, vec!["2026-06-14", "2026-06-15", "2026-06-16"]);
    }

    // validate_year_month 자체 검증은 attendance.rs 의 단위 테스트
    // (`validate_year_month_rejects_out_of_range_month`) 에서 보장됨. 본 모듈은 호출만 위임.

    // ─────────────── T3: create_makeup_with_absences (트랜잭션 매칭) ───────────────

    /// T3 픽스처 — 학생 1명 (월~금 정규 수업 5요일) + 보강 가능 코드 (공휴수업일) 일자
    /// + 미처리 결석 N건. event_date 는 토요일(2026-06-13) 또는 일요일(2026-06-14)로
    /// 정규 수업 요일 아닌 일자.
    async fn fixture_student_with_absences(
        pool: &SqlitePool,
        absence_dates: &[&str],
    ) -> (i64, Vec<i64>) {
        let sid = seed_student(
            &pool,
            "S001",
            "2026-01-01",
            None,
            // 월(1)~금(5) 정규 수업
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
        )
        .await;
        let mut absence_ids = Vec::with_capacity(absence_dates.len());
        for date in absence_dates {
            let aid = insert_absence(&pool, sid, date, "2026-06", 60, Some("2026-07")).await;
            absence_ids.push(aid);
        }
        (sid, absence_ids)
    }

    /// T3 픽스처 — event_date 에 보강 가능 학사일정 1건 등록 (공휴수업일).
    async fn fixture_makeup_eligible_date(pool: &SqlitePool, event_date: &str) {
        let code = schedule_code_id(&pool, "공휴수업일").await;
        insert_schedule_event(&pool, code, event_date, None).await;
    }

    fn payload(student_id: i64, event_date: &str, absence_ids: Vec<i64>) -> CreateMakeupPayload {
        CreateMakeupPayload {
            student_id,
            event_date: event_date.to_string(),
            class_minutes: 60,
            absence_ids,
        }
    }

    /// AC-T3-1: 정상 매칭 — 결석 2건 → makeup_id 발급 + 2건 모두 makeup_done 전이.
    #[tokio::test]
    async fn create_makeup_matches_absences_atomically() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) =
            fixture_student_with_absences(&pool, &["2026-06-15", "2026-06-16"]).await;
        // 2026-06-13 = 토요일 (정규 수업 요일 아님)
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let result = create_makeup_with_absences_impl(
            &pool,
            &payload(sid, "2026-06-13", absences.clone()),
        )
        .await
        .expect("정상 매칭");
        assert_eq!(result.matched_count, 2);
        assert_eq!(result.student_id, sid);
        assert!(result.makeup_id > 0);

        // 결석 2건 모두 makeup_done + makeup_attendance_id 설정 확인
        for aid in absences {
            let row: (String, Option<i64>) = sqlx::query_as(
                "SELECT status, makeup_attendance_id FROM regular_attendances WHERE id=?",
            )
            .bind(aid)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(row.0, "makeup_done");
            assert_eq!(row.1, Some(result.makeup_id));
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

    /// AC-T3-4: 정규 수업 요일 차단 — 학생의 월(1) 수업일에 보강 등록 시도.
    #[tokio::test]
    async fn create_makeup_blocks_regular_class_weekday() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        // 2026-06-15 는 월요일 (정규 수업) — 학사일정도 등록해서 검증 1 통과 후 검증 3 차단 확인
        fixture_makeup_eligible_date(&pool, "2026-06-15").await;

        let err = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-15", absences))
            .await
            .expect_err("정규 수업 요일 차단");
        assert!(err.contains("정규 수업 요일"), "친화 메시지: {}", err);
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

        // 같은 결석으로 두 번째 보강 시도 → 이미 매칭됨 거부
        let err = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-20", absences))
            .await
            .expect_err("이미 매칭된 결석 거부");
        assert!(err.contains("이미 다른 보강"));
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
        let r = create_makeup_with_absences_impl(
            &pool,
            &payload(sid, "2026-06-13", absences.clone()),
        )
        .await
        .expect("등록");

        let reverted = cancel_makeup_impl(&pool, r.makeup_id)
            .await
            .expect("취소");
        assert_eq!(reverted, 2);

        // 결석 2건 모두 absent + makeup_attendance_id NULL
        for aid in &absences {
            let row: (String, Option<i64>) = sqlx::query_as(
                "SELECT status, makeup_attendance_id FROM regular_attendances WHERE id=?",
            )
            .bind(aid)
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(row.0, "absent");
            assert_eq!(row.1, None);
        }
        // makeup_attendances 0건
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM makeup_attendances")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 0);
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

    // ─────────────── T4: mark_makeup_absent ───────────────

    /// 보강 등록 후 미등원 → 보강 status='makeup_absent' + 결석 absent 환원.
    /// 결석은 다음 보강 매칭 대상으로 재진입 가능 (makeup_attendance_id=NULL).
    #[tokio::test]
    async fn mark_makeup_absent_preserves_makeup_but_reverts_absence() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        let r =
            create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", absences.clone()))
                .await
                .expect("등록");

        mark_makeup_absent_impl(&pool, r.makeup_id)
            .await
            .expect("미등원 처리");

        // 보강 행 보존 + status='makeup_absent'
        let m_status: (String,) =
            sqlx::query_as("SELECT status FROM makeup_attendances WHERE id=?")
                .bind(r.makeup_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(m_status.0, "makeup_absent");

        // 결석은 absent 환원 + 매칭 NULL (재매칭 가능)
        let a: (String, Option<i64>) = sqlx::query_as(
            "SELECT status, makeup_attendance_id FROM regular_attendances WHERE id=?",
        )
        .bind(absences[0])
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(a.0, "absent");
        assert_eq!(a.1, None);
    }

    /// 이미 makeup_absent 상태인 보강에 재호출 — 멱등 (변경 없음, 에러 없음).
    #[tokio::test]
    async fn mark_makeup_absent_is_idempotent() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (sid, absences) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;
        let r = create_makeup_with_absences_impl(&pool, &payload(sid, "2026-06-13", absences))
            .await
            .expect("등록");

        mark_makeup_absent_impl(&pool, r.makeup_id).await.expect("1st");
        let second = mark_makeup_absent_impl(&pool, r.makeup_id)
            .await
            .expect("2nd 멱등");
        assert_eq!(second, 0, "이미 미등원이면 환원할 결석 없음");
    }

    // ─────────────── T4: batch_create_makeups ───────────────

    /// 다중 학생 일괄 보강 — 전원 성공 시 succeeded N건, failed 0건.
    #[tokio::test]
    async fn batch_create_all_succeed() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (s1, a1) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        // 두 번째 학생은 별도 fixture
        let s2 = seed_student(
            &pool,
            "S002",
            "2026-01-01",
            None,
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
        )
        .await;
        let a2 = insert_absence(&pool, s2, "2026-06-16", "2026-06", 90, Some("2026-07")).await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let result = batch_create_makeups_impl(
            &pool,
            &BatchCreateMakeupsPayload {
                event_date: "2026-06-13".to_string(),
                entries: vec![
                    BatchMakeupEntry {
                        student_id: s1,
                        class_minutes: 60,
                        absence_ids: a1,
                    },
                    BatchMakeupEntry {
                        student_id: s2,
                        class_minutes: 90,
                        absence_ids: vec![a2],
                    },
                ],
            },
        )
        .await
        .expect("batch");

        assert_eq!(result.succeeded.len(), 2);
        assert_eq!(result.failed.len(), 0);
    }

    /// 일부 학생 실패 — succeeded/failed 분리 + 성공 학생만 makeup_done.
    /// 시나리오: s1 정상, s2 의 absence_ids 무효 (미존재).
    #[tokio::test]
    async fn batch_create_partial_failure() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let (s1, a1) = fixture_student_with_absences(&pool, &["2026-06-15"]).await;
        let s2 = seed_student(
            &pool,
            "S002",
            "2026-01-01",
            None,
            &[(1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
        )
        .await;
        fixture_makeup_eligible_date(&pool, "2026-06-13").await;

        let result = batch_create_makeups_impl(
            &pool,
            &BatchCreateMakeupsPayload {
                event_date: "2026-06-13".to_string(),
                entries: vec![
                    BatchMakeupEntry {
                        student_id: s1,
                        class_minutes: 60,
                        absence_ids: a1.clone(),
                    },
                    BatchMakeupEntry {
                        student_id: s2,
                        class_minutes: 60,
                        absence_ids: vec![99999], // 미존재
                    },
                ],
            },
        )
        .await
        .expect("batch");

        assert_eq!(result.succeeded.len(), 1);
        assert_eq!(result.failed.len(), 1);
        assert_eq!(result.failed[0].student_id, s2);
        assert!(result.failed[0].reason.contains("찾을 수 없습니다"));

        // s1 의 결석은 makeup_done 으로 정상 처리됨 — s2 실패가 s1 에 영향 없음.
        let row: (String,) = sqlx::query_as("SELECT status FROM regular_attendances WHERE id=?")
            .bind(a1[0])
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, "makeup_done");
    }

    /// 빈 entries 거부.
    #[tokio::test]
    async fn batch_create_rejects_empty_entries() {
        let pool = db::test_pool_in_memory().await.expect("pool");
        let err = batch_create_makeups_impl(
            &pool,
            &BatchCreateMakeupsPayload {
                event_date: "2026-06-13".to_string(),
                entries: vec![],
            },
        )
        .await
        .expect_err("빈 entries 거부");
        assert!(err.contains("1명 이상"));
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
}
