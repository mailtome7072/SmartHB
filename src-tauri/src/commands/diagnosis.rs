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

/// 1. 보강필요시간 음수(이상) — 보강 출석분이 "보강 대상 결석분"보다 많은 원생.
///
/// 앱 정의(attendance.rs)와 정합: 결석 중 **보강 대상**은 `absent` + `makeup_done`(소멸은 면제로
/// 제외)이며, 여기서 보강완료(`makeup_attended`)를 차감한다. 정상 매칭이면 0 이상(= 잔여 미보강
/// 결석분), 음수면 과보강/고아 보강 등 데이터 이상이다.
///   net = SUM(class_minutes WHERE status IN ('absent','makeup_done'))
///       − SUM(makeup_attendances.class_minutes WHERE status='makeup_attended')
async fn check_negative_makeup_minutes(
    pool: &SqlitePool,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id AS student_id, s.name AS name, \
            COALESCE((SELECT SUM(class_minutes) FROM regular_attendances \
                      WHERE student_id = s.id AND status IN ('absent', 'makeup_done')), 0) \
          - COALESCE((SELECT SUM(class_minutes) FROM makeup_attendances \
                      WHERE student_id = s.id AND status = 'makeup_attended'), 0) AS net \
         FROM students s",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    let mut issues = Vec::new();
    for r in rows {
        let net: i64 = r.try_get("net").map_err(AppError::Db)?;
        if net < 0 {
            let student_id: i64 = r.try_get("student_id").map_err(AppError::Db)?;
            let name: String = r.try_get("name").map_err(AppError::Db)?;
            issues.push(DiagnosisIssue {
                check_id: "negative_makeup_minutes".to_string(),
                severity: "error".to_string(),
                message: format!(
                    "{} 원생의 보강필요시간이 음수입니다 ({}분 초과 보강). 출결/보강 기록을 확인해주세요.",
                    name, -net
                ),
                target_table: Some("students".to_string()),
                target_id: Some(student_id),
            });
        }
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

/// 3. 재원중 원생 당월 청구 미생성 — 재원생 중 당월 bills 0건.
async fn check_active_students_missing_billing(
    pool: &SqlitePool,
    year_month: &str,
) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT s.id AS student_id, s.name AS name FROM students s \
         WHERE s.withdraw_date IS NULL \
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

/// 6. 고아 보강 데이터 — 어떤 정규출결에서도 참조(makeup_attendance_id)하지 않는 보강.
async fn check_orphan_makeups(pool: &SqlitePool) -> Result<Vec<DiagnosisIssue>, AppError> {
    let rows = sqlx::query(
        "SELECT m.id AS makeup_id, s.name AS name, m.event_date AS event_date \
         FROM makeup_attendances m \
         JOIN students s ON s.id = m.student_id \
         WHERE NOT EXISTS (SELECT 1 FROM regular_attendances ra \
                           WHERE ra.makeup_attendance_id = m.id)",
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
    let row = sqlx::query(
        "SELECT EXISTS(SELECT 1 FROM diagnosis_history \
                       WHERE run_type = 'auto' AND substr(run_date, 1, 7) = ?) AS done",
    )
    .bind(year_month)
    .fetch_one(pool)
    .await
    .map_err(AppError::Db)?;
    let done: i64 = row.try_get("done").map_err(AppError::Db)?;
    Ok(done == 0)
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

    // ── 검사 1: 보강필요시간 음수 ──
    #[tokio::test]
    async fn negative_makeup_detected_when_overmakeup() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 결석 60분 1건 + 보강 출석 90분 1건 → net = -30
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'absent', 60)")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-05', '2026-06', 'makeup_attended', 90)")
            .bind(sid).execute(&pool).await.unwrap();
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
        insert_student(&pool, "S1", "김학생").await;
        let issues = check_active_students_missing_billing(&pool, "2026-06").await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].check_id, "missing_billing");
    }

    #[tokio::test]
    async fn missing_billing_clean_when_billed() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
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
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes, makeup_attendance_id) VALUES (?, '2026-06-02', '2026-06', 'makeup_done', 60, ?)")
            .bind(sid).bind(mid.0).execute(&pool).await.unwrap();
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
        run_and_record(&pool, "manual", "2026-06-01", "2026-06").await.unwrap();
        let latest = fetch_history(&pool, 1).await.unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].run_date, "2026-06-01");
    }
}
