//! 데이터 자가 진단 IPC (Sprint 14 T1, PRD §6.6).
//!
//! 매월 1일 첫 실행 시 자동 + 사용자 수동 실행으로 7종 무결성 검사를 수행하고,
//! 결과를 `diagnosis_history` (V303) 에 보관한다 (최근 12개월, 초과분 자동 정리).
//!
//! ## 설계
//!
//! - 각 검사는 `&SqlitePool` 을 받는 내부 async 함수 — 인메모리 테스트로 독립 검증 가능.
//! - IPC 커맨드는 전역 `db::pool()` 을 조회해 내부 함수에 위임하는 얇은 래퍼.
//! - 검사는 발견 항목(`DiagnosisIssue`) 목록을 반환하며, 빈 목록이면 이상 없음.
//!
//! ## 검사 7종 (PRD §6.6.1) — 실제 스키마 컬럼 기준
//!
//! 1. 보강필요시간 음수 — 결석 누적분보다 보강 출석분이 많은 원생 (집계 휴리스틱)
//! 2. 재원중 원생 당월 출결 미생성 — 현행 스케줄 보유 재원생 중 당월 정규출결 0건
//! 3. 재원중 원생 당월 청구 미생성 — 재원생 중 당월 bills 0건
//! 4. 수업 스케줄 ↔ 출결 요일 불일치 — 현행 스케줄에 없는 요일의 당월 정규출결
//! 5. 결석 소멸기한 미설정 — status='absent' AND makeup_deadline IS NULL
//! 6. 고아 보강 데이터 — 어떤 정규출결에서도 참조하지 않는 makeup_attendances
//! 7. 수납 정합성 — is_paid=1 인데 결제수단 누락 / 카드결제인데 카드사 누락

use crate::commands::db::pool;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// 총 검사 항목 수 (PRD §6.6.1).
const TOTAL_CHECKS: i64 = 7;

/// 월 1회 자동진단 추적 키 (app_settings) — 이상 0건이면 이력을 남기지 않으므로(완전 0건 정책),
/// "이번 달 자동진단 실행 여부"는 이력이 아닌 이 설정값으로 판정한다 (AC-6.6-1).
const LAST_AUTO_DIAGNOSIS_KEY: &str = "last_auto_diagnosis";

/// 자가 진단에서 발견된 개별 이상 항목. `diagnosis_history.details` JSON 배열의 원소.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DiagnosisIssue {
    /// 검사 식별자 (예: "negative_makeup_minutes").
    pub check_id: String,
    /// 심각도: "error" | "warning".
    pub severity: String,
    /// 50대 친화 한글 설명 메시지.
    pub message: String,
    /// 관련 테이블명 (이동 링크 구성용, 없으면 None).
    pub target_table: Option<String>,
    /// 관련 행 id (없으면 None — 원생 단위 요약 등).
    pub target_id: Option<i64>,
}

/// `run_diagnosis` 반환 — 1회 실행 결과 요약 + 발견 항목 전체.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnosisResult {
    pub run_date: String,
    pub run_type: String,
    pub total_checks: i64,
    pub issues_found: i64,
    pub issues: Vec<DiagnosisIssue>,
}

/// 진단 이력 1건 — `get_diagnosis_history` / `get_latest_diagnosis` 반환.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiagnosisHistoryRow {
    pub id: i64,
    pub run_date: String,
    pub run_type: String,
    pub total_checks: i64,
    pub issues_found: i64,
    /// details 컬럼(JSON)을 파싱한 발견 항목.
    pub issues: Vec<DiagnosisIssue>,
    pub created_at: String,
}

/// 오늘 날짜 (YYYY-MM-DD). academic.rs 의 current_year_month 와 동일 chrono 기반.
fn current_date() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// 당월 (YYYY-MM).
fn current_year_month() -> String {
    chrono::Local::now().format("%Y-%m").to_string()
}

// ----------------------------------------------------------------------------
// 검사 7종 (내부 함수 — pool 주입으로 테스트 가능)
// ----------------------------------------------------------------------------

/// 1. 보강 배분 초과(이상) — 결석의 배분 보강분(makeup_allocations 합)이 결석 시간을 초과.
///
/// ADR-011: 잔여 음수(과보강)는 개별 결석 레벨에서 발생하며, 정상 경로는
/// `(makeup_id, absence_id)` UNIQUE + 등록 시 초과 거부로 방지된다. 백필 오류·직접 DB 조작으로
/// 어떤 결석의 배분합이 class_minutes 를 넘으면 이 검사가 잡는다. 부분 보강으로 잔여가 남는
/// 것(잔여 > 0)은 정상이므로 플래그하지 않는다.
async fn check_negative_makeup_minutes(
    pool: &SqlitePool,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id AS student_id, s.name AS name, \
            SUM(alloc.total - r.class_minutes) AS over_minutes \
         FROM students s \
         JOIN regular_attendances r ON r.student_id = s.id \
         JOIN (SELECT absence_id, SUM(allocated_minutes) AS total \
               FROM makeup_allocations GROUP BY absence_id) alloc \
              ON alloc.absence_id = r.id \
         WHERE alloc.total > r.class_minutes \
         GROUP BY s.id, s.name",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    let mut issues = Vec::new();
    for r in rows {
        let over: i64 = r.try_get("over_minutes").map_err(AppError::Db)?;
        let student_id: i64 = r.try_get("student_id").map_err(AppError::Db)?;
        let name: String = r.try_get("name").map_err(AppError::Db)?;
        issues.push(DiagnosisIssue {
            check_id: "negative_makeup_minutes".to_string(),
            severity: "error".to_string(),
            message: format!(
                "{} 원생의 보강 배분이 결석 시간을 초과합니다 ({}분 초과). 출결/보강 기록을 확인해주세요.",
                name, over
            ),
            target_table: Some("students".to_string()),
            target_id: Some(student_id),
        });
    }
    Ok(issues)
}

/// 2. 재원중 원생 당월 출결 미생성 — 현행 스케줄 보유 재원생 중 당월 정규출결 0건.
async fn check_active_students_missing_attendance(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id AS student_id, s.name AS name FROM students s \
         WHERE s.withdraw_date IS NULL \
           AND EXISTS (SELECT 1 FROM student_schedules ss \
                       WHERE ss.student_id = s.id AND ss.effective_to IS NULL) \
           AND NOT EXISTS (SELECT 1 FROM regular_attendances ra \
                           WHERE ra.student_id = s.id AND ra.year_month = ?)",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let student_id: i64 = r.get("student_id");
            let name: String = r.get("name");
            DiagnosisIssue {
                check_id: "missing_attendance".to_string(),
                severity: "warning".to_string(),
                message: format!("{} 원생의 {} 정규 출결이 생성되지 않았습니다.", name, year_month),
                target_table: Some("regular_attendances".to_string()),
                target_id: Some(student_id),
            }
        })
        .collect())
}

/// 3. 재원중 원생 당월 청구 미생성 — **청구 대상 재원생** 중 당월 bills 0건.
///
/// P2-10 (2026-06 코드리뷰): 청구 생성(`generate_bills_impl`)은 주당 수업시간이 0이거나
/// 표준 교습비 미매핑 원생을 **의도적으로 skip** 한다. 따라서 스케줄 없는 재원생은 청구가
/// 영원히 생성되지 않는 것이 정상인데, 과거에는 전체 재원생을 대상으로 해 매월 무의미한
/// 경고(만성 오탐)를 띄웠다. 검사 2(`missing_attendance`)와 동일하게 현행 스케줄 보유 +
/// 주당 수업시간 > 0 인 원생만 대상으로 한다 (표준교습비 미매핑은 드문 설정 오류라 경고 유지).
async fn check_active_students_missing_billing(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id AS student_id, s.name AS name FROM students s \
         WHERE s.withdraw_date IS NULL \
           AND EXISTS (SELECT 1 FROM student_schedules ss \
                       WHERE ss.student_id = s.id AND ss.effective_to IS NULL \
                       GROUP BY ss.student_id HAVING SUM(ss.duration_hours) > 0) \
           AND NOT EXISTS (SELECT 1 FROM bills b \
                           WHERE b.student_id = s.id AND b.bill_year_month = ?)",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let student_id: i64 = r.get("student_id");
            let name: String = r.get("name");
            DiagnosisIssue {
                check_id: "missing_billing".to_string(),
                severity: "warning".to_string(),
                message: format!("{} 원생의 {} 청구가 생성되지 않았습니다.", name, year_month),
                target_table: Some("bills".to_string()),
                target_id: Some(student_id),
            }
        })
        .collect())
}

/// 4. 수업 스케줄 ↔ 출결 요일 불일치 — 현행 스케줄에 없는 요일의 당월 정규출결.
///
/// SQLite `strftime('%w', date)` 는 0=일~6=토. 스케줄 day_of_week 는 ISO 1=월~7=일.
/// 변환: %w='0' → 7(일), 그 외 → 그대로. 변환식을 SQL 안에서 비교한다.
async fn check_schedule_attendance_mismatch(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT ra.id AS att_id, s.name AS name, ra.event_date AS event_date \
         FROM regular_attendances ra \
         JOIN students s ON s.id = ra.student_id \
         WHERE ra.year_month = ? \
           AND NOT EXISTS ( \
             SELECT 1 FROM student_schedules ss \
             WHERE ss.student_id = ra.student_id AND ss.effective_to IS NULL \
               AND ss.day_of_week = \
                   CASE WHEN CAST(strftime('%w', ra.event_date) AS INTEGER) = 0 \
                        THEN 7 ELSE CAST(strftime('%w', ra.event_date) AS INTEGER) END)",
    )
    .bind(year_month)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let att_id: i64 = r.get("att_id");
            let name: String = r.get("name");
            let event_date: String = r.get("event_date");
            DiagnosisIssue {
                check_id: "schedule_attendance_mismatch".to_string(),
                severity: "warning".to_string(),
                message: format!(
                    "{} 원생의 {} 정규 출결이 현행 수업 스케줄 요일과 일치하지 않습니다.",
                    name, event_date
                ),
                target_table: Some("regular_attendances".to_string()),
                target_id: Some(att_id),
            }
        })
        .collect())
}

/// 5. 결석 소멸기한 미설정 — status='absent' AND makeup_deadline IS NULL.
async fn check_absent_without_deadline(
    pool: &SqlitePool,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT ra.id AS att_id, s.name AS name, ra.event_date AS event_date \
         FROM regular_attendances ra \
         JOIN students s ON s.id = ra.student_id \
         WHERE ra.status = 'absent' AND ra.makeup_deadline IS NULL",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let att_id: i64 = r.get("att_id");
            let name: String = r.get("name");
            let event_date: String = r.get("event_date");
            DiagnosisIssue {
                check_id: "absent_without_deadline".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "{} 원생의 {} 결석에 보강 소멸기한이 설정되지 않았습니다.",
                    name, event_date
                ),
                target_table: Some("regular_attendances".to_string()),
                target_id: Some(att_id),
            }
        })
        .collect())
}

/// 6. 고아 보강 데이터 — 어떤 결석에도 배분되지 않은 보강 (ADR-011: makeup_allocations 기준).
async fn check_orphan_makeups(pool: &SqlitePool) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT m.id AS makeup_id, s.name AS name, m.event_date AS event_date \
         FROM makeup_attendances m \
         JOIN students s ON s.id = m.student_id \
         WHERE NOT EXISTS (SELECT 1 FROM makeup_allocations al \
                           WHERE al.makeup_id = m.id)",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let makeup_id: i64 = r.get("makeup_id");
            let name: String = r.get("name");
            let event_date: String = r.get("event_date");
            DiagnosisIssue {
                check_id: "orphan_makeup".to_string(),
                severity: "warning".to_string(),
                message: format!(
                    "{} 원생의 {} 보강 기록이 어떤 결석과도 연결되지 않았습니다 (고아 보강).",
                    name, event_date
                ),
                target_table: Some("makeup_attendances".to_string()),
                target_id: Some(makeup_id),
            }
        })
        .collect())
}

/// 7. 수납 정합성 — is_paid=1 인데 결제수단 누락, 또는 카드결제인데 카드사 누락.
async fn check_payment_integrity(pool: &SqlitePool) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT p.id AS payment_id, s.name AS name, b.bill_year_month AS ym, \
                p.payment_method_id AS method_id, p.card_company_id AS card_id, \
                COALESCE(pm.is_card_type, 0) AS is_card \
         FROM payments p \
         JOIN bills b ON b.id = p.bill_id \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payment_methods pm ON pm.id = p.payment_method_id \
         WHERE p.is_paid = 1 \
           AND (p.payment_method_id IS NULL \
                OR (pm.is_card_type = 1 AND p.card_company_id IS NULL))",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let payment_id: i64 = r.get("payment_id");
            let name: String = r.get("name");
            let ym: String = r.get("ym");
            let method_id: Option<i64> = r.get("method_id");
            let detail = if method_id.is_none() {
                "결제수단 누락"
            } else {
                "카드 결제인데 카드사 누락"
            };
            DiagnosisIssue {
                check_id: "payment_integrity".to_string(),
                severity: "error".to_string(),
                message: format!("{} 원생 {} 수납 정합성 오류 — {}.", name, ym, detail),
                target_table: Some("payments".to_string()),
                target_id: Some(payment_id),
            }
        })
        .collect())
}

/// 7종 검사 일괄 실행 — 발견 항목 전체를 모아 반환.
async fn run_all_checks(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let mut issues = Vec::new();
    issues.extend(check_negative_makeup_minutes(pool).await?);
    issues.extend(check_active_students_missing_attendance(pool, year_month).await?);
    issues.extend(check_active_students_missing_billing(pool, year_month).await?);
    issues.extend(check_schedule_attendance_mismatch(pool, year_month).await?);
    issues.extend(check_absent_without_deadline(pool).await?);
    issues.extend(check_orphan_makeups(pool).await?);
    issues.extend(check_payment_integrity(pool).await?);
    Ok(issues)
}

/// 진단 실행 + 이력 저장 + 12개월 초과 정리 (내부 — pool 주입).
async fn run_and_record(
    pool: &SqlitePool,
    run_type: &str,
    run_date: &str,
    year_month: &str,
) -> Result<DiagnosisResult, AppError> {
    let issues = run_all_checks(pool, year_month).await?;
    let issues_found = issues.len() as i64;
    let details = serde_json::to_string(&issues)
        .map_err(|e| AppError::Config(format!("진단 결과 직렬화 실패: {}", e)))?;

    // 기존 이력 재검증 — 수동/자동 실행 시마다 이전 진단결과 항목이 현재도 검출되는지 확인하여
    // 이미 해결된(현재 미검출) 항목은 각 이력에서 자동 제거하고, 항목이 모두 비게 된 이력은 삭제한다.
    reconcile_resolved_issues(pool, &issues).await?;

    // 자동진단 월 추적 — 이상 0건이면 이력을 남기지 않으므로(완전 0건 정책), 월 1회 자동 실행
    // 여부(AC-6.6-1)는 app_settings 에 별도 보관한다.
    if run_type == "auto" {
        sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(LAST_AUTO_DIAGNOSIS_KEY)
        .bind(year_month)
        .execute(pool)
        .await
        .map_err(AppError::Db)?;
    }

    // 현재 실행 기록 — 이상이 있을 때만(>0) 직전 이력과 다르면 새 이력을 추가한다.
    // 완전 0건 정책: 모든 이상이 해결되면 재검증이 과거 이상-이력을 이미 삭제하고, 새 이력도
    // 남기지 않으므로 "아무 기록도 없음" 상태가 된다(화면은 실행 결과를 직접 표시).
    if issues_found > 0 && !is_same_as_latest(pool, issues_found, &details).await? {
        sqlx::query(
            "INSERT INTO diagnosis_history \
                (run_date, run_type, total_checks, issues_found, details) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(run_date)
        .bind(run_type)
        .bind(TOTAL_CHECKS)
        .bind(issues_found)
        .bind(&details)
        .execute(pool)
        .await
        .map_err(AppError::Db)?;
    }

    // 12개월 초과 이력 자동 정리 (AC-6.6-4).
    sqlx::query("DELETE FROM diagnosis_history WHERE run_date < date('now', '-12 months')")
        .execute(pool)
        .await
        .map_err(AppError::Db)?;

    Ok(DiagnosisResult {
        run_date: run_date.to_string(),
        run_type: run_type.to_string(),
        total_checks: TOTAL_CHECKS,
        issues_found,
        issues,
    })
}

/// 이상 항목의 안정적 식별자 — 메시지 텍스트는 제외하여(카운트 변동에도) 같은 문제로 인식한다.
fn issue_identity(i: &DiagnosisIssue) -> (String, Option<String>, Option<i64>) {
    (i.check_id.clone(), i.target_table.clone(), i.target_id)
}

/// 기존 이력 재검증 (PRD §6.6 — 자동 해결 반영).
///
/// 현재 검출 목록(`current`)에 더 이상 없는(= 해결된) 항목을 각 이력에서 제거한다. 남은 항목은
/// 현재 검출 내용으로 메시지를 갱신하고(카운트 변동 반영 + 중복 적재 방지), 모든 항목이 해결된
/// 이력은 삭제한다.
async fn reconcile_resolved_issues(
    pool: &SqlitePool,
    current: &[DiagnosisIssue],
) -> Result<(), AppError> {
    use std::collections::HashMap;
    // 현재 검출 항목을 식별자로 매핑 — 남은 항목을 최신 내용으로 교체하기 위함.
    let current_by_id: HashMap<(String, Option<String>, Option<i64>), &DiagnosisIssue> =
        current.iter().map(|i| (issue_identity(i), i)).collect();

    let rows = sqlx::query("SELECT id, details FROM diagnosis_history")
        .fetch_all(pool)
        .await
        .map_err(AppError::Db)?;

    for row in rows {
        let id: i64 = row.try_get("id").map_err(AppError::Db)?;
        let details: String = row.try_get("details").map_err(AppError::Db)?;
        let stored: Vec<DiagnosisIssue> = serde_json::from_str(&details)
            .map_err(|e| AppError::Config(format!("진단 이력 details 파싱 실패: {}", e)))?;
        let original_len = stored.len();

        // 여전히 검출되는 항목만, 현재 내용(메시지 최신화)으로 유지 — 저장 순서 보존.
        let kept: Vec<DiagnosisIssue> = stored
            .iter()
            .filter_map(|i| current_by_id.get(&issue_identity(i)).map(|cur| (*cur).clone()))
            .collect();

        if kept.is_empty() {
            // 항목이 모두 비게 됨(전부 해결 또는 빈 이력) → 이력 삭제. 완전 0건 정책상 빈
            // 이력은 남기지 않는다(레거시 '이상 없음' 이력도 다음 실행 때 정리된다).
            sqlx::query("DELETE FROM diagnosis_history WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await
                .map_err(AppError::Db)?;
        } else if kept.len() != original_len
            || kept.iter().zip(&stored).any(|(k, s)| k.message != s.message)
        {
            // 일부 해결되었거나 메시지가 바뀜 → 남은 항목으로 갱신.
            let new_details = serde_json::to_string(&kept)
                .map_err(|e| AppError::Config(format!("진단 결과 직렬화 실패: {}", e)))?;
            sqlx::query("UPDATE diagnosis_history SET details = ?, issues_found = ? WHERE id = ?")
                .bind(&new_details)
                .bind(kept.len() as i64)
                .bind(id)
                .execute(pool)
                .await
                .map_err(AppError::Db)?;
        }
    }
    Ok(())
}

/// 가장 최근 이력의 결과가 주어진 (이상 건수, 상세 JSON) 와 동일한지 판정한다.
///
/// `details` 는 동일 데이터·동일 검사 순서면 serde_json 직렬화가 결정적이라 문자열 비교로
/// 충분하다. 이력이 하나도 없으면 (첫 실행) `false` 를 반환해 반드시 기록되게 한다.
async fn is_same_as_latest(
    pool: &SqlitePool,
    issues_found: i64,
    details: &str,
) -> Result<bool, AppError> {
    let row = sqlx::query(
        "SELECT issues_found, details FROM diagnosis_history ORDER BY id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Db)?;

    match row {
        Some(r) => {
            let prev_found: i64 = r.try_get("issues_found").map_err(AppError::Db)?;
            let prev_details: String = r.try_get("details").map_err(AppError::Db)?;
            Ok(prev_found == issues_found && prev_details == details)
        }
        None => Ok(false),
    }
}

/// 이력 조회 (내부 — pool 주입). 최신순 limit 건.
async fn fetch_history(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<DiagnosisHistoryRow>, AppError> {
    let rows = sqlx::query(
        "SELECT id, run_date, run_type, total_checks, issues_found, details, created_at \
         FROM diagnosis_history ORDER BY run_date DESC, id DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    rows.into_iter().map(row_to_history).collect()
}

/// diagnosis_history 행 → DiagnosisHistoryRow (details JSON 파싱).
fn row_to_history(r: sqlx::sqlite::SqliteRow) -> Result<DiagnosisHistoryRow, AppError> {
    let details: String = r.try_get("details").map_err(AppError::Db)?;
    let issues: Vec<DiagnosisIssue> = serde_json::from_str(&details)
        .map_err(|e| AppError::Config(format!("진단 이력 details 파싱 실패: {}", e)))?;
    Ok(DiagnosisHistoryRow {
        id: r.try_get("id").map_err(AppError::Db)?,
        run_date: r.try_get("run_date").map_err(AppError::Db)?,
        run_type: r.try_get("run_type").map_err(AppError::Db)?,
        total_checks: r.try_get("total_checks").map_err(AppError::Db)?,
        issues_found: r.try_get("issues_found").map_err(AppError::Db)?,
        issues,
        created_at: r.try_get("created_at").map_err(AppError::Db)?,
    })
}

/// 당월 자동 진단 필요 여부 (내부 — pool 주입). 당월 'auto' 기록이 없으면 true.
async fn auto_needed(pool: &SqlitePool, year_month: &str) -> Result<bool, AppError> {
    // 완전 0건 정책: 이상이 없으면 이력이 없으므로, 자동 실행 여부는 app_settings 추적값으로 판정.
    let last: Option<String> = sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?")
        .bind(LAST_AUTO_DIAGNOSIS_KEY)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Db)?;
    Ok(last.as_deref() != Some(year_month))
}

// ----------------------------------------------------------------------------
// Tauri IPC commands (얇은 래퍼)
// ----------------------------------------------------------------------------

/// 자가 진단 실행 (수동/자동). 7종 검사 + 이력 저장 + 12개월 초과 정리.
#[tauri::command]
pub async fn run_diagnosis(run_type: String) -> Result<DiagnosisResult, String> {
    if run_type != "auto" && run_type != "manual" {
        return Err(String::from(AppError::UserFacing(
            "진단 유형이 올바르지 않습니다.".to_string(),
        )));
    }
    let pool = pool().map_err(String::from)?;
    run_and_record(pool, &run_type, &current_date(), &current_year_month())
        .await
        .map_err(String::from)
}

/// 진단 이력 조회 (최신순 limit 건).
#[tauri::command]
pub async fn get_diagnosis_history(limit: i64) -> Result<Vec<DiagnosisHistoryRow>, String> {
    let limit = limit.clamp(1, 120);
    let pool = pool().map_err(String::from)?;
    fetch_history(pool, limit).await.map_err(String::from)
}

/// 대시보드 알림용 최신 진단 결과 1건.
#[tauri::command]
pub async fn get_latest_diagnosis() -> Result<Option<DiagnosisHistoryRow>, String> {
    let pool = pool().map_err(String::from)?;
    Ok(fetch_history(pool, 1).await.map_err(String::from)?.into_iter().next())
}

/// 당월 자동 진단 필요 여부 (매월 1일 첫 실행 판단, AC-6.6-1).
#[tauri::command]
pub async fn check_auto_diagnosis_needed() -> Result<bool, String> {
    let pool = pool().map_err(String::from)?;
    auto_needed(pool, &current_year_month())
        .await
        .map_err(String::from)
}

/// 진단 이력 1건 삭제 (행 단위). 존재하지 않는 id 는 무시한다(멱등).
#[tauri::command]
pub async fn delete_diagnosis_history(id: i64) -> Result<(), String> {
    let pool = pool().map_err(String::from)?;
    delete_history_row(pool, id).await.map_err(String::from)
}

/// 진단 이력 전체 삭제 ("이력 비우기").
#[tauri::command]
pub async fn clear_diagnosis_history() -> Result<(), String> {
    let pool = pool().map_err(String::from)?;
    clear_history(pool).await.map_err(String::from)
}

async fn delete_history_row(pool: &SqlitePool, id: i64) -> Result<(), AppError> {
    sqlx::query("DELETE FROM diagnosis_history WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(AppError::Db)?;
    Ok(())
}

async fn clear_history(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM diagnosis_history")
        .execute(pool)
        .await
        .map_err(AppError::Db)?;
    Ok(())
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    /// 더미 재원생 1건 INSERT → id 반환.
    async fn insert_student(pool: &SqlitePool, serial: &str, name: &str) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES (?, ?, 'male', 'elementary', 3, '2026-01-01') RETURNING id",
        )
        .bind(serial)
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("student insert");
        row.0
    }

    /// 청구 대상 원생(현행 월요일 스케줄 + 2026-06 출결 보유) — P2-10 검사3은 청구 대상만
    /// missing_billing 으로 잡는다. 출결을 함께 넣어 검사2(출결 미생성)·검사4(요일 불일치)는
    /// 미발동시켜, 본 헬퍼로 만든 원생의 이상은 missing_billing 1건으로 한정한다 (2026-06 전용).
    async fn insert_billable_student(pool: &SqlitePool, serial: &str, name: &str) -> i64 {
        let sid = insert_student(pool, serial, name).await;
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(pool).await.unwrap();
        // 2026-06-01 = 월요일 → 스케줄 요일과 일치(검사4 미발동), present(검사5 미발동).
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) VALUES (?, '2026-06-01', '2026-06', 120)")
            .bind(sid).execute(pool).await.unwrap();
        sid
    }

    // ── 검사 1: 보강필요시간 음수 ──
    #[tokio::test]
    async fn negative_makeup_detected_when_overmakeup() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // ADR-011: 60분 결석에 90분 배분(초과) → 이상 감지.
        let aid: (i64,) = sqlx::query_as("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'makeup_done', 60) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        let mid: (i64,) = sqlx::query_as("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 90) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) VALUES (?, ?, 90)")
            .bind(mid.0).bind(aid.0).execute(&pool).await.unwrap();
        let issues = check_negative_makeup_minutes(&pool).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "negative_makeup_minutes");
    }

    #[tokio::test]
    async fn negative_makeup_clean_when_balanced() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'absent', 90)")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 90)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_negative_makeup_minutes(&pool).await.unwrap();
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn negative_makeup_clean_for_matched_pair() {
        // 성춘향 회귀: 결석이 보강완료(makeup_done)로 매칭된 정상 쌍은 오탐 아님.
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "성춘향").await;
        let mid: (i64,) = sqlx::query_as("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-05-26', '2026-05', 'makeup_attended', 60) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_deadline, makeup_attendance_id) VALUES (?, '2026-05-22', '2026-05', 'makeup_done', 60, '2026-06', ?)")
            .bind(sid).bind(mid.0).execute(&pool).await.unwrap();
        let issues = check_negative_makeup_minutes(&pool).await.unwrap();
        assert!(issues.is_empty(), "makeup_done 60 − makeup_attended 60 = 0, 미플래그여야 함");
    }

    #[tokio::test]
    async fn negative_makeup_excludes_expired() {
        // 결석 120(보강완료 60 + 소멸 60) + 보강 60 → 소멸 면제 → net = 60−60 = 0, 미플래그.
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        let mid: (i64,) = sqlx::query_as("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 60) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_attendance_id) VALUES (?, '2026-06-02', '2026-06', 'makeup_done', 60, ?)")
            .bind(sid).bind(mid.0).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_deadline) VALUES (?, '2026-06-03', '2026-06', 'makeup_expired', 60, '2026-07')")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_negative_makeup_minutes(&pool).await.unwrap();
        assert!(issues.is_empty(), "소멸 60 은 보강 대상에서 제외 → 음수 아님");
    }

    // ── 검사 2: 당월 출결 미생성 ──
    #[tokio::test]
    async fn missing_attendance_detected_for_scheduled_active_student() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 현행 스케줄 보유, 당월 출결 없음
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_active_students_missing_attendance(&pool, "2026-06").await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "missing_attendance");
    }

    #[tokio::test]
    async fn missing_attendance_skips_student_without_schedule() {
        let pool = test_pool_in_memory().await.unwrap();
        insert_student(&pool, "S1", "김학생").await; // 스케줄 없음 → 제외
        let issues = check_active_students_missing_attendance(&pool, "2026-06").await.unwrap();
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn missing_attendance_clean_when_present() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) VALUES (?, '2026-06-01', '2026-06', 120)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_active_students_missing_attendance(&pool, "2026-06").await.unwrap();
        assert!(issues.is_empty());
    }

    // ── 검사 3: 당월 청구 미생성 ──
    #[tokio::test]
    async fn missing_billing_detected() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // P2-10: 청구 대상(현행 스케줄 + 주당시간>0)이어야 감지.
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_active_students_missing_billing(&pool, "2026-06").await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "missing_billing");
    }

    /// P2-10: 스케줄 없는 재원생은 청구 skip 대상이므로 만성 오탐을 내지 않는다.
    #[tokio::test]
    async fn missing_billing_skips_student_without_schedule() {
        let pool = test_pool_in_memory().await.unwrap();
        insert_student(&pool, "S1", "김학생").await; // 스케줄 없음 → 청구 대상 아님 → 제외
        let issues = check_active_students_missing_billing(&pool, "2026-06").await.unwrap();
        assert!(issues.is_empty(), "스케줄 없는 재원생은 경고 대상 아님");
    }

    #[tokio::test]
    async fn missing_billing_clean_when_billed() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 200000, 200000)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_active_students_missing_billing(&pool, "2026-06").await.unwrap();
        assert!(issues.is_empty());
    }

    // ── 검사 4: 스케줄/출결 요일 불일치 ──
    #[tokio::test]
    async fn schedule_mismatch_detected_on_wrong_weekday() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 스케줄은 월요일(1)만. 출결은 2026-06-03(수요일) → 불일치.
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) VALUES (?, '2026-06-03', '2026-06', 120)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_schedule_attendance_mismatch(&pool, "2026-06").await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "schedule_attendance_mismatch");
    }

    #[tokio::test]
    async fn schedule_mismatch_clean_on_matching_weekday() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 2026-06-01 은 월요일. 스케줄 월(1) → 일치.
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, class_minutes) VALUES (?, '2026-06-01', '2026-06', 120)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_schedule_attendance_mismatch(&pool, "2026-06").await.unwrap();
        assert!(issues.is_empty());
    }

    // ── 검사 5: 결석 소멸기한 미설정 ──
    #[tokio::test]
    async fn absent_without_deadline_detected() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_deadline) VALUES (?, '2026-06-02', '2026-06', 'absent', 90, NULL)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_absent_without_deadline(&pool).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "absent_without_deadline");
    }

    #[tokio::test]
    async fn absent_with_deadline_clean() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_deadline) VALUES (?, '2026-06-02', '2026-06', 'absent', 90, '2026-07')")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_absent_without_deadline(&pool).await.unwrap();
        assert!(issues.is_empty());
    }

    // ── 검사 6: 고아 보강 ──
    #[tokio::test]
    async fn orphan_makeup_detected() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 어떤 정규출결도 이 보강을 참조하지 않음 → 고아.
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 60)")
            .bind(sid).execute(&pool).await.unwrap();
        let issues = check_orphan_makeups(&pool).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "orphan_makeup");
    }

    #[tokio::test]
    async fn orphan_makeup_clean_when_referenced() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        let mid: (i64,) = sqlx::query_as("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 60) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        let aid: (i64,) = sqlx::query_as("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'makeup_done', 60) RETURNING id")
            .bind(sid).fetch_one(&pool).await.unwrap();
        // ADR-011: 배분 레코드로 참조 표현
        sqlx::query("INSERT INTO makeup_allocations (makeup_id, absence_id, allocated_minutes) VALUES (?, ?, 60)")
            .bind(mid.0).bind(aid.0).execute(&pool).await.unwrap();
        let issues = check_orphan_makeups(&pool).await.unwrap();
        assert!(issues.is_empty());
    }

    // ── 검사 7: 수납 정합성 ──
    async fn insert_bill_with_payment(
        pool: &SqlitePool,
        sid: i64,
        is_paid: i64,
        method_id: Option<i64>,
        card_id: Option<i64>,
    ) {
        let bid: (i64,) = sqlx::query_as("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 200000, 200000) RETURNING id")
            .bind(sid).fetch_one(pool).await.unwrap();
        let paid_date = if is_paid == 1 { Some("2026-06-10") } else { None };
        sqlx::query("INSERT INTO payments (bill_id, is_paid, paid_date, payment_method_id, card_company_id) VALUES (?, ?, ?, ?, ?)")
            .bind(bid.0).bind(is_paid).bind(paid_date).bind(method_id).bind(card_id)
            .execute(pool).await.unwrap();
    }

    #[tokio::test]
    async fn payment_integrity_detects_missing_method() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        insert_bill_with_payment(&pool, sid, 1, None, None).await; // 수납완료인데 결제수단 없음
        let issues = check_payment_integrity(&pool).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "payment_integrity");
    }

    #[tokio::test]
    async fn payment_integrity_detects_card_without_company() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 카드 계열 결제수단(code='card', is_card_type=1) id 조회
        let card_method: (i64,) = sqlx::query_as("SELECT id FROM payment_methods WHERE is_card_type = 1 LIMIT 1")
            .fetch_one(&pool).await.unwrap();
        insert_bill_with_payment(&pool, sid, 1, Some(card_method.0), None).await; // 카드인데 카드사 없음
        let issues = check_payment_integrity(&pool).await.unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[tokio::test]
    async fn payment_integrity_clean_when_unpaid() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        insert_bill_with_payment(&pool, sid, 0, None, None).await; // 미수납 → 검사 대상 아님
        let issues = check_payment_integrity(&pool).await.unwrap();
        assert!(issues.is_empty());
    }

    // ── run_and_record + 이력 + auto_needed ──
    #[tokio::test]
    async fn run_and_record_persists_history_and_counts() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 청구 미생성 1건 보장 (issues_found >= 1)
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        let result = run_and_record(&pool, "manual", "2026-06-01", "2026-06").await.unwrap();
        assert_eq!(result.total_checks, 7);
        assert!(result.issues_found >= 1);

        let history = fetch_history(&pool, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].run_type, "manual");
        assert_eq!(history[0].issues_found, result.issues_found);
        assert_eq!(history[0].issues.len() as i64, result.issues_found);
    }

    #[tokio::test]
    async fn auto_needed_true_when_no_auto_record_this_month() {
        let pool = test_pool_in_memory().await.unwrap();
        assert!(auto_needed(&pool, "2026-06").await.unwrap());
    }

    #[tokio::test]
    async fn auto_needed_false_after_auto_run_this_month() {
        let pool = test_pool_in_memory().await.unwrap();
        run_and_record(&pool, "auto", "2026-06-01", "2026-06").await.unwrap();
        assert!(!auto_needed(&pool, "2026-06").await.unwrap());
        // 다른 달은 여전히 필요
        assert!(auto_needed(&pool, "2026-07").await.unwrap());
    }

    #[tokio::test]
    async fn get_latest_returns_most_recent() {
        let pool = test_pool_in_memory().await.unwrap();
        run_and_record(&pool, "auto", "2026-05-01", "2026-05").await.unwrap();
        // 결과가 달라지도록 데이터 변경 — 동일 결과면 dedup 으로 두 번째 기록이 스킵된다.
        let sid = insert_student(&pool, "S1", "김학생").await;
        sqlx::query("INSERT INTO student_schedules (student_id, day_of_week, start_time, duration_hours, effective_from) VALUES (?, 1, '15:00', 2, '2026-01-01')")
            .bind(sid).execute(&pool).await.unwrap();
        run_and_record(&pool, "manual", "2026-06-01", "2026-06").await.unwrap();
        let latest = fetch_history(&pool, 1).await.unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].run_date, "2026-06-01");
    }

    #[tokio::test]
    async fn run_and_record_skips_duplicate_when_unchanged() {
        let pool = test_pool_in_memory().await.unwrap();
        // 이상 1건(청구 미생성)을 보장 — 완전 0건 정책상 0건이면 기록이 안 남으므로.
        insert_billable_student(&pool, "S1", "김학생").await;
        // 동일 데이터로 자동→수동→수동 3회 실행 (사용자 보고 시나리오 재현).
        run_and_record(&pool, "auto", "2026-06-06", "2026-06").await.unwrap();
        run_and_record(&pool, "manual", "2026-06-06", "2026-06").await.unwrap();
        run_and_record(&pool, "manual", "2026-06-06", "2026-06").await.unwrap();
        // 결과가 변하지 않았으므로 이력은 첫 1건만 남는다.
        let history = fetch_history(&pool, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].run_type, "auto");
    }

    #[tokio::test]
    async fn run_and_record_appends_when_result_changes() {
        let pool = test_pool_in_memory().await.unwrap();
        // 1차: 원생 A 1명 → 청구 미생성 1건.
        insert_billable_student(&pool, "A1", "가").await;
        let first = run_and_record(&pool, "manual", "2026-06-06", "2026-06").await.unwrap();
        assert_eq!(first.issues_found, 1);
        // 원생 B 추가로 결과가 달라짐(이상 2건) → 새 이력 추가.
        insert_billable_student(&pool, "B1", "나").await;
        let second = run_and_record(&pool, "manual", "2026-06-07", "2026-06").await.unwrap();
        assert_eq!(second.issues_found, 2);
        // 결과가 바뀌었으므로 새 이력이 추가되어 2건.
        let history = fetch_history(&pool, 10).await.unwrap();
        assert_eq!(history.len(), 2);
    }

    #[tokio::test]
    async fn run_and_record_prunes_resolved_issue_on_rerun() {
        let pool = test_pool_in_memory().await.unwrap();
        // 청구 대상(스케줄 보유) 원생 2명 → 각각 청구 미생성 1건씩 (missing_billing). 다른 검사는 미해당.
        let a = insert_billable_student(&pool, "A1", "가").await;
        let b = insert_billable_student(&pool, "B1", "나").await;
        let first = run_and_record(&pool, "manual", "2026-06-06", "2026-06").await.unwrap();
        assert_eq!(first.issues_found, 2);

        // A의 청구를 생성 → A의 missing_billing 해결. 재실행 시 재검증으로 A 항목만 제거.
        sqlx::query("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 200000, 200000)")
            .bind(a).execute(&pool).await.unwrap();
        let second = run_and_record(&pool, "manual", "2026-06-07", "2026-06").await.unwrap();
        assert_eq!(second.issues_found, 1);

        // 이력은 1건(갱신), 남은 항목은 B의 청구 미생성뿐.
        let history = fetch_history(&pool, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].issues_found, 1);
        assert_eq!(history[0].issues[0].check_id, "missing_billing");
        assert_eq!(history[0].issues[0].target_id, Some(b));
    }

    #[tokio::test]
    async fn run_and_record_deletes_record_when_all_resolved() {
        let pool = test_pool_in_memory().await.unwrap();
        let a = insert_billable_student(&pool, "A1", "가").await;
        let first = run_and_record(&pool, "manual", "2026-06-06", "2026-06").await.unwrap();
        assert_eq!(first.issues_found, 1);

        // 유일한 이상(A 청구 미생성) 해결 → 재실행 시 그 이력은 삭제되고 이상 보유 이력이 사라진다.
        sqlx::query("INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount) VALUES (?, '2026-06', 4, 200000, 200000)")
            .bind(a).execute(&pool).await.unwrap();
        let second = run_and_record(&pool, "manual", "2026-06-07", "2026-06").await.unwrap();
        assert_eq!(second.issues_found, 0);

        // 완전 0건 정책: 모든 이상이 해결되면 아무 이력도 남지 않는다.
        let history = fetch_history(&pool, 10).await.unwrap();
        assert!(history.is_empty(), "모든 이상이 해결되면 이력이 완전히 비어야 함");
    }

    #[tokio::test]
    async fn auto_run_tracked_even_when_no_issues() {
        // 완전 0건이어도 자동진단은 월 1회로 추적되어야 한다 (AC-6.6-1).
        let pool = test_pool_in_memory().await.unwrap();
        assert!(auto_needed(&pool, "2026-06").await.unwrap());
        // 빈 데이터(이상 0건)로 auto 실행 → 이력은 안 남지만 추적값은 기록.
        run_and_record(&pool, "auto", "2026-06-01", "2026-06").await.unwrap();
        assert!(fetch_history(&pool, 10).await.unwrap().is_empty(), "0건이면 이력 없음");
        assert!(!auto_needed(&pool, "2026-06").await.unwrap(), "이번 달 자동진단 완료로 추적");
        assert!(auto_needed(&pool, "2026-07").await.unwrap(), "다음 달은 여전히 필요");
    }

    // ── 이력 수동 삭제 (T2) ──
    #[tokio::test]
    async fn delete_history_row_removes_only_target() {
        let pool = test_pool_in_memory().await.unwrap();
        let id1: (i64,) = sqlx::query_as("INSERT INTO diagnosis_history (run_date, run_type, total_checks, issues_found, details) VALUES ('2026-06-01', 'manual', 7, 0, '[]') RETURNING id")
            .fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO diagnosis_history (run_date, run_type, total_checks, issues_found, details) VALUES ('2026-06-02', 'manual', 7, 1, '[]')")
            .execute(&pool).await.unwrap();
        delete_history_row(&pool, id1.0).await.unwrap();
        let rows = fetch_history(&pool, 120).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].run_date, "2026-06-02");
    }

    #[tokio::test]
    async fn delete_history_row_nonexistent_is_noop() {
        let pool = test_pool_in_memory().await.unwrap();
        sqlx::query("INSERT INTO diagnosis_history (run_date, run_type, total_checks, issues_found, details) VALUES ('2026-06-01', 'manual', 7, 0, '[]')")
            .execute(&pool).await.unwrap();
        delete_history_row(&pool, 99_999).await.unwrap(); // 존재하지 않는 id — 에러 없이 멱등
        assert_eq!(fetch_history(&pool, 120).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn clear_history_removes_all() {
        let pool = test_pool_in_memory().await.unwrap();
        for d in ["2026-06-01", "2026-06-02", "2026-06-03"] {
            sqlx::query("INSERT INTO diagnosis_history (run_date, run_type, total_checks, issues_found, details) VALUES (?, 'manual', 7, 0, '[]')")
                .bind(d).execute(&pool).await.unwrap();
        }
        clear_history(&pool).await.unwrap();
        assert!(fetch_history(&pool, 120).await.unwrap().is_empty());
    }
}
