//! 데이터 내보내기 IPC (Sprint 14 T5, PRD §4.13.2).
//!
//! 원생/출결/청구-수납 데이터를 CSV(UTF-8 BOM, Excel 한글 호환)로 내보낸다.
//! 저장 경로는 프론트엔드가 Tauri Dialog `save` 로 사전 취득해 전달한다 (AC-4.13-3).
//!
//! ## 설계
//!
//! - 각 내보내기는 `&SqlitePool` 을 받아 CSV 문자열 + 행 수를 만드는 내부 함수 — 인메모리
//!   테스트로 독립 검증 가능.
//! - IPC 커맨드는 전역 `db::pool()` 조회 → CSV 생성 → 파일 쓰기를 묶는 얇은 래퍼.
//! - BOM(U+FEFF) 접두 → Excel 이 UTF-8 한글을 자동 인식 (R99). 라인 구분은 CRLF(Excel 호환).
//!
//! ## 계획(sprint14.md T5) 대비 정제
//!
//! - **출결**: 단순 "보강여부" 플래그 대신 정규 출결 + 보강 출결을 UNION 해 `구분` 컬럼으로
//!   구분한다 — 보강 세션도 실제 출결 기록이라 누락하면 데이터가 불완전해진다.
//! - **청구**: `청구상태`(미확정/확정) 컬럼을 추가해 확정 여부를 함께 내보낸다.
//! - **기간**: 출결/청구의 `year_month` 는 `Option` — `None` 이면 전체 기간.
//!
//! ## 이연 (Sprint 15)
//!
//! - Excel(.xlsx) 형식 + 비밀번호 보호 옵션 (AC-4.13-4).

use crate::commands::db::pool;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// 내보내기 1회 결과 — 저장 경로 / 데이터 행 수 / 기록된 파일 바이트 크기.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub file_path: String,
    /// 헤더를 제외한 데이터 행 수.
    pub row_count: i64,
    /// 실제 기록된 파일 크기 (BOM 3바이트 포함).
    pub byte_size: i64,
}

// ----------------------------------------------------------------------------
// CSV 유틸 (순수 함수 — FS 미접근)
// ----------------------------------------------------------------------------

/// CSV 필드 1개 escape — 쉼표/따옴표/개행 포함 시 큰따옴표로 감싸고 내부 따옴표는 2배로.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// 필드 슬라이스를 CSV 라인(CRLF 종료)으로 결합.
fn csv_row(fields: &[&str]) -> String {
    let escaped: Vec<String> = fields.iter().map(|f| csv_field(f)).collect();
    let mut line = escaped.join(",");
    line.push_str("\r\n");
    line
}

/// CSV 문자열에 UTF-8 BOM(0xEF 0xBB 0xBF)을 접두해 바이트 벡터로 변환.
fn with_bom(content: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(content.len() + 3);
    bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    bytes.extend_from_slice(content.as_bytes());
    bytes
}

/// CSV 내용을 BOM 접두 후 지정 경로에 기록 → 기록 바이트 수 반환.
fn write_csv(file_path: &str, content: &str) -> Result<i64, AppError> {
    let bytes = with_bom(content);
    std::fs::write(file_path, &bytes)?;
    Ok(bytes.len() as i64)
}

// ----------------------------------------------------------------------------
// 라벨 변환
// ----------------------------------------------------------------------------

fn gender_label(code: &str) -> &str {
    match code {
        "male" => "남",
        "female" => "여",
        other => other,
    }
}

fn school_level_label(code: &str) -> &str {
    match code {
        "elementary" => "초등",
        "middle" => "중등",
        other => other,
    }
}

fn attendance_status_label(status: &str) -> &str {
    match status {
        "present" => "출석",
        "absent" => "결석",
        "makeup_done" => "보강완료",
        "makeup_expired" => "소멸",
        "makeup_attended" => "보강출석",
        other => other,
    }
}

fn bill_status_label(status: &str) -> &str {
    match status {
        "draft" => "미확정",
        "confirmed" => "확정",
        other => other,
    }
}

// ----------------------------------------------------------------------------
// CSV 빌더 (내부 함수 — pool 주입으로 테스트 가능)
// ----------------------------------------------------------------------------

/// 원생 명단 CSV — 이름순. (일련번호/이름/성별/학교급/학년/학교/입교일/퇴교일/주수업시간/교습비)
async fn build_students_csv(pool: &SqlitePool) -> Result<(String, i64), AppError> {
    let mut csv = csv_row(&[
        "일련번호",
        "이름",
        "성별",
        "학교급",
        "학년",
        "학교",
        "입교일",
        "퇴교일",
        "주수업시간",
        "교습비",
    ]);

    // weekly_hours = 현행 스케줄(effective_to IS NULL) duration_hours 합 (billing.rs 와 동일 정의).
    // 교습비 = standard_fees.amount WHERE weekly_hours = ? AND is_active = 1 (매핑 없으면 공란).
    let rows = sqlx::query(
        "SELECT s.serial_no, s.name, s.gender, s.school_level, s.grade, \
                sc.name AS school_name, s.enroll_date, s.withdraw_date, \
                COALESCE(wh.weekly_hours, 0) AS weekly_hours, sf.amount AS fee_amount \
         FROM students s \
         LEFT JOIN schools sc ON sc.id = s.school_id \
         LEFT JOIN (SELECT student_id, COALESCE(SUM(duration_hours), 0) AS weekly_hours \
                    FROM student_schedules WHERE effective_to IS NULL \
                    GROUP BY student_id) wh ON wh.student_id = s.id \
         LEFT JOIN standard_fees sf \
                ON sf.weekly_hours = COALESCE(wh.weekly_hours, 0) AND sf.is_active = 1 \
         ORDER BY s.name, s.serial_no",
    )
    .fetch_all(pool)
    .await?;

    let mut count = 0i64;
    for r in &rows {
        let serial_no: String = r.try_get("serial_no")?;
        let name: String = r.try_get("name")?;
        let gender: String = r.try_get("gender")?;
        let school_level: String = r.try_get("school_level")?;
        let grade: i64 = r.try_get("grade")?;
        let school_name: Option<String> = r.try_get("school_name")?;
        let enroll_date: String = r.try_get("enroll_date")?;
        let withdraw_date: Option<String> = r.try_get("withdraw_date")?;
        let weekly_hours: i64 = r.try_get("weekly_hours")?;
        let fee_amount: Option<i64> = r.try_get("fee_amount")?;

        csv.push_str(&csv_row(&[
            &serial_no,
            &name,
            gender_label(&gender),
            school_level_label(&school_level),
            &grade.to_string(),
            school_name.as_deref().unwrap_or(""),
            &enroll_date,
            withdraw_date.as_deref().unwrap_or(""),
            &weekly_hours.to_string(),
            &fee_amount.map(|a| a.to_string()).unwrap_or_default(),
        ]));
        count += 1;
    }
    Ok((csv, count))
}

/// 출결 CSV — 정규 + 보강 출결 UNION, 원생명·일자순. (원생명/일자/구분/상태/수업시간(분))
async fn build_attendances_csv(
    pool: &SqlitePool,
    year_month: Option<&str>,
) -> Result<(String, i64), AppError> {
    let mut csv = csv_row(&["원생명", "일자", "구분", "상태", "수업시간(분)"]);

    let rows = sqlx::query(
        "SELECT student_name, event_date, kind, status, class_minutes FROM ( \
            SELECT s.name AS student_name, ra.event_date AS event_date, '정규수업' AS kind, \
                   ra.status AS status, ra.class_minutes AS class_minutes \
            FROM regular_attendances ra JOIN students s ON s.id = ra.student_id \
            WHERE (? IS NULL OR ra.year_month = ?) \
            UNION ALL \
            SELECT s.name, ma.event_date, '보강', ma.status, ma.class_minutes \
            FROM makeup_attendances ma JOIN students s ON s.id = ma.student_id \
            WHERE (? IS NULL OR ma.year_month = ?) \
         ) ORDER BY student_name, event_date, kind",
    )
    .bind(year_month)
    .bind(year_month)
    .bind(year_month)
    .bind(year_month)
    .fetch_all(pool)
    .await?;

    let mut count = 0i64;
    for r in &rows {
        let student_name: String = r.try_get("student_name")?;
        let event_date: String = r.try_get("event_date")?;
        let kind: String = r.try_get("kind")?;
        let status: String = r.try_get("status")?;
        let class_minutes: i64 = r.try_get("class_minutes")?;

        csv.push_str(&csv_row(&[
            &student_name,
            &event_date,
            &kind,
            attendance_status_label(&status),
            &class_minutes.to_string(),
        ]));
        count += 1;
    }
    Ok((csv, count))
}

/// 청구-수납 CSV — 청구월·원생명순.
/// (원생명/청구월/청구상태/청구액/할인액/최종액/수납여부/입금일/결제수단)
async fn build_billing_csv(
    pool: &SqlitePool,
    year_month: Option<&str>,
) -> Result<(String, i64), AppError> {
    let mut csv = csv_row(&[
        "원생명",
        "청구월",
        "청구상태",
        "청구액",
        "할인액",
        "최종액",
        "수납여부",
        "입금일",
        "결제수단",
    ]);

    let rows = sqlx::query(
        "SELECT s.name AS student_name, b.bill_year_month, b.status, \
                b.bill_amount, b.adjusted_amount, \
                p.is_paid, p.paid_date, pm.label AS method_label, cc.label AS card_label \
         FROM bills b \
         JOIN students s ON s.id = b.student_id \
         LEFT JOIN payments p ON p.bill_id = b.id \
         LEFT JOIN payment_methods pm ON pm.id = p.payment_method_id \
         LEFT JOIN card_companies cc ON cc.id = p.card_company_id \
         WHERE (? IS NULL OR b.bill_year_month = ?) \
         ORDER BY b.bill_year_month, s.name",
    )
    .bind(year_month)
    .bind(year_month)
    .fetch_all(pool)
    .await?;

    let mut count = 0i64;
    for r in &rows {
        let student_name: String = r.try_get("student_name")?;
        let bill_year_month: String = r.try_get("bill_year_month")?;
        let status: String = r.try_get("status")?;
        let bill_amount: i64 = r.try_get("bill_amount")?;
        let adjusted_amount: i64 = r.try_get("adjusted_amount")?;
        let is_paid: Option<i64> = r.try_get("is_paid")?;
        let paid_date: Option<String> = r.try_get("paid_date")?;
        let method_label: Option<String> = r.try_get("method_label")?;
        let card_label: Option<String> = r.try_get("card_label")?;

        let discount = (bill_amount - adjusted_amount).max(0);
        let paid_label = if is_paid == Some(1) { "수납완료" } else { "미납" };
        // 결제수단: 카드 결제면 "카드(카드사)", 그 외엔 결제수단 라벨, 미수납이면 공란.
        let method = match (method_label.as_deref(), card_label.as_deref()) {
            (Some(m), Some(c)) => format!("{}({})", m, c),
            (Some(m), None) => m.to_string(),
            _ => String::new(),
        };

        csv.push_str(&csv_row(&[
            &student_name,
            &bill_year_month,
            bill_status_label(&status),
            &bill_amount.to_string(),
            &discount.to_string(),
            &adjusted_amount.to_string(),
            paid_label,
            paid_date.as_deref().unwrap_or(""),
            &method,
        ]));
        count += 1;
    }
    Ok((csv, count))
}

// ----------------------------------------------------------------------------
// IPC 커맨드 (전역 pool + 파일 쓰기 래퍼)
// ----------------------------------------------------------------------------

/// 원생 명단을 CSV 로 내보낸다.
#[tauri::command]
pub async fn export_students(file_path: String) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let (csv, row_count) = build_students_csv(pool).await.map_err(String::from)?;
    let byte_size = write_csv(&file_path, &csv).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count, byte_size })
}

/// 출결 데이터를 CSV 로 내보낸다. `year_month` 가 `None` 이면 전체 기간.
#[tauri::command]
pub async fn export_attendances(
    year_month: Option<String>,
    file_path: String,
) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let (csv, row_count) = build_attendances_csv(pool, year_month.as_deref())
        .await
        .map_err(String::from)?;
    let byte_size = write_csv(&file_path, &csv).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count, byte_size })
}

/// 청구-수납 데이터를 CSV 로 내보낸다. `year_month` 가 `None` 이면 전체 기간.
#[tauri::command]
pub async fn export_billing(
    year_month: Option<String>,
    file_path: String,
) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let (csv, row_count) = build_billing_csv(pool, year_month.as_deref())
        .await
        .map_err(String::from)?;
    let byte_size = write_csv(&file_path, &csv).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count, byte_size })
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;
    use crate::commands::db::test_pool_in_memory;

    /// 더미 재원생 1건 INSERT → id 반환.
    async fn insert_student(pool: &SqlitePool, serial: &str, name: &str) -> i64 {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO students (serial_no, name, gender, school_level, grade, enroll_date) \
             VALUES (?, ?, 'female', 'middle', 1, '2026-01-01') RETURNING id",
        )
        .bind(serial)
        .bind(name)
        .fetch_one(pool)
        .await
        .expect("student insert");
        row.0
    }

    // ── csv_field escape ──
    #[test]
    fn csv_field_quotes_special_chars() {
        assert_eq!(csv_field("홍길동"), "홍길동");
        assert_eq!(csv_field("김,철수"), "\"김,철수\"");
        assert_eq!(csv_field("그는 \"천재\""), "\"그는 \"\"천재\"\"\"");
        assert_eq!(csv_field("줄\n바꿈"), "\"줄\n바꿈\"");
    }

    // ── BOM 접두 ──
    #[test]
    fn with_bom_prefixes_utf8_bom() {
        let bytes = with_bom("이름\r\n");
        assert_eq!(&bytes[0..3], &[0xEF, 0xBB, 0xBF]);
        assert_eq!(&bytes[3..], "이름\r\n".as_bytes());
    }

    // ── 원생 CSV ──
    #[tokio::test]
    async fn students_csv_with_data() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "김학생").await;
        // 주 4시간 스케줄 → standard_fees(4 → 200000, V201 시드 보정값) 매핑.
        sqlx::query(
            "INSERT INTO student_schedules \
             (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 1, '15:00', 4, '2026-01-01')",
        )
        .bind(sid)
        .execute(&pool)
        .await
        .unwrap();

        let (csv, count) = build_students_csv(&pool).await.unwrap();
        assert_eq!(count, 1);
        assert!(csv.starts_with("일련번호,이름,"), "헤더 누락");
        assert!(csv.contains("김학생"));
        assert!(csv.contains("여")); // gender female
        assert!(csv.contains("중등")); // school_level middle
        assert!(csv.contains("200000")); // 주 4시간 교습비 (V201 시드 보정값)
    }

    #[tokio::test]
    async fn students_csv_empty_has_header_only() {
        let pool = test_pool_in_memory().await.unwrap();
        let (csv, count) = build_students_csv(&pool).await.unwrap();
        assert_eq!(count, 0);
        // 헤더 1줄(CRLF 종료)만 존재.
        assert_eq!(csv.matches("\r\n").count(), 1);
    }

    // ── 출결 CSV ──
    #[tokio::test]
    async fn attendances_csv_includes_regular_and_makeup() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "이결석").await;
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'absent', 60)")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-09', '2026-06', 'makeup_attended', 60)")
            .bind(sid).execute(&pool).await.unwrap();

        // 당월 필터.
        let (csv, count) = build_attendances_csv(&pool, Some("2026-06")).await.unwrap();
        assert_eq!(count, 2, "정규 1 + 보강 1");
        assert!(csv.contains("정규수업"));
        assert!(csv.contains("보강"));
        assert!(csv.contains("결석"));
        assert!(csv.contains("보강출석"));

        // 다른 월 필터 → 0건.
        let (_csv2, count2) = build_attendances_csv(&pool, Some("2026-05")).await.unwrap();
        assert_eq!(count2, 0);

        // 전체(None) → 2건.
        let (_csv3, count3) = build_attendances_csv(&pool, None).await.unwrap();
        assert_eq!(count3, 2);
    }

    #[tokio::test]
    async fn attendances_csv_empty_has_header_only() {
        let pool = test_pool_in_memory().await.unwrap();
        let (csv, count) = build_attendances_csv(&pool, None).await.unwrap();
        assert_eq!(count, 0);
        assert_eq!(csv.matches("\r\n").count(), 1);
    }

    // ── 청구 CSV ──
    #[tokio::test]
    async fn billing_csv_with_payment() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "S1", "박청구").await;
        // 청구 1건: 청구 250000, 최종 230000(할인 20000), 확정.
        let bid: (i64,) = sqlx::query_as(
            "INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount, status) \
             VALUES (?, '2026-06', 4, 250000, 230000, 'confirmed') RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        // 카드 수납 (payment_methods code='card' id, card_companies 신한 id).
        let pm_card: (i64,) = sqlx::query_as("SELECT id FROM payment_methods WHERE code = 'card'")
            .fetch_one(&pool).await.unwrap();
        let cc_shinhan: (i64,) = sqlx::query_as("SELECT id FROM card_companies WHERE code = 'shinhan'")
            .fetch_one(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO payments (bill_id, is_paid, paid_date, payment_method_id, card_company_id) \
             VALUES (?, 1, '2026-06-05', ?, ?)",
        )
        .bind(bid.0)
        .bind(pm_card.0)
        .bind(cc_shinhan.0)
        .execute(&pool)
        .await
        .unwrap();

        let (csv, count) = build_billing_csv(&pool, Some("2026-06")).await.unwrap();
        assert_eq!(count, 1);
        assert!(csv.contains("박청구"));
        assert!(csv.contains("확정"));
        assert!(csv.contains("250000")); // 청구액
        assert!(csv.contains("20000")); // 할인액
        assert!(csv.contains("230000")); // 최종액
        assert!(csv.contains("수납완료"));
        assert!(csv.contains("2026-06-05")); // 입금일
        assert!(csv.contains("카드(신한카드)")); // 결제수단
    }

    #[tokio::test]
    async fn billing_csv_empty_has_header_only() {
        let pool = test_pool_in_memory().await.unwrap();
        let (csv, count) = build_billing_csv(&pool, None).await.unwrap();
        assert_eq!(count, 0);
        assert_eq!(csv.matches("\r\n").count(), 1);
    }
}
