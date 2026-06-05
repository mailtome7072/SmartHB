//! 데이터 내보내기 IPC (Sprint 14 T5/T6, PRD §4.13.2).
//!
//! 원생/출결/청구-수납 데이터를 **엑셀(.xlsx)** 로 내보낸다. 저장 경로는 프론트엔드가
//! Tauri Dialog `save` 로 사전 취득해 전달한다 (AC-4.13-3).
//!
//! ## 서식 규칙 (사용자 요청 2026-06-05)
//!
//! - **정렬**: 일련번호가 있는 자료(원생)는 일련번호 오름차순.
//! - **금전 컬럼**(교습비/청구액/할인액/최종액): 숫자 + 천단위 콤마(`#,##0`) + **우측정렬**.
//! - **그 외 컬럼**: **좌측정렬**.
//! - **컬럼 너비**: 데이터 최장 길이에 맞춰 자동(autofit).
//! - **수업시간 단위**: '시간'으로 통일(분 → 시간 환산).
//!
//! ## 설계
//!
//! - 각 내보내기는 `&SqlitePool` 을 받아 [`SheetData`](셀 값 + 정렬 의미)를 만드는 내부 함수
//!   → 인메모리 테스트로 셀 내용/정렬/정렬순서를 독립 검증.
//! - [`write_xlsx`] 가 SheetData 를 서식과 함께 .xlsx 로 기록 (Cell 변형이 정렬/표시형식 결정).
//! - IPC 커맨드는 전역 `db::pool()` 조회 → build → write 를 묶는 얇은 래퍼.
//!
//! ## 이연 (Sprint 15)
//!
//! - 비밀번호 보호 옵션 (AC-4.13-4).

use crate::commands::db::pool;
use crate::error::AppError;
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// 내보내기 1회 결과 — 저장 경로 / 데이터 행 수 / 파일 바이트 크기.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExportResult {
    pub file_path: String,
    /// 헤더를 제외한 데이터 행 수.
    pub row_count: i64,
    pub byte_size: i64,
}

// ----------------------------------------------------------------------------
// 시트 데이터 (순수 — DB/FS 미접근, 테스트 대상)
// ----------------------------------------------------------------------------

/// 엑셀 셀 1개 — 변형이 정렬/표시형식을 결정한다.
#[derive(Debug, Clone, PartialEq)]
enum Cell {
    /// 좌측정렬 텍스트.
    Text(String),
    /// 좌측정렬 정수 숫자 (일련번호/학년 — 숫자 형식 저장, 천단위 없음).
    Int(i64),
    /// 금전 — 우측정렬 + 천단위 콤마(`#,##0`).
    Money(i64),
    /// 수업시간 — 좌측정렬 숫자, 단위 '시간'(헤더에 명시).
    Hours(f64),
}

/// 시트 1장 — 헤더 + 데이터 행. build_* 가 생성, [`write_xlsx`] 가 기록.
struct SheetData {
    headers: Vec<&'static str>,
    rows: Vec<Vec<Cell>>,
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

/// 분 → 시간(소수). 60분 = 1.0, 90분 = 1.5.
fn minutes_to_hours(minutes: i64) -> f64 {
    minutes as f64 / 60.0
}

// ----------------------------------------------------------------------------
// .xlsx 기록 (서식 적용)
// ----------------------------------------------------------------------------

fn xlsx_err(e: rust_xlsxwriter::XlsxError) -> AppError {
    AppError::Io(std::io::Error::other(e.to_string()))
}

/// SheetData 를 .xlsx 로 기록 → 기록 파일 바이트 수 반환.
/// 헤더 굵게·좌측 / 금전 우측+천단위 / 그 외 좌측. autofit 으로 컬럼 너비 자동.
fn write_xlsx(sheet: &SheetData, file_path: &str) -> Result<i64, AppError> {
    let mut workbook = Workbook::new();
    let ws = workbook.add_worksheet();

    let header_fmt = Format::new().set_bold().set_align(FormatAlign::Left);
    let left_fmt = Format::new().set_align(FormatAlign::Left);
    let money_fmt = Format::new()
        .set_num_format("#,##0")
        .set_align(FormatAlign::Right);

    for (c, h) in sheet.headers.iter().enumerate() {
        ws.write_with_format(0, c as u16, *h, &header_fmt)
            .map_err(xlsx_err)?;
    }
    for (r, row) in sheet.rows.iter().enumerate() {
        let row_idx = (r + 1) as u32;
        for (c, cell) in row.iter().enumerate() {
            let col = c as u16;
            match cell {
                Cell::Text(s) => ws.write_with_format(row_idx, col, s.as_str(), &left_fmt),
                Cell::Int(n) => ws.write_with_format(row_idx, col, *n as f64, &left_fmt),
                Cell::Money(n) => ws.write_with_format(row_idx, col, *n as f64, &money_fmt),
                Cell::Hours(h) => ws.write_with_format(row_idx, col, *h, &left_fmt),
            }
            .map_err(xlsx_err)?;
        }
    }

    // 컬럼 너비 자동 맞춤 — 가장 긴 데이터가 다 보이도록.
    ws.autofit();
    workbook.save(file_path).map_err(xlsx_err)?;

    let size = std::fs::metadata(file_path)?.len() as i64;
    Ok(size)
}

// ----------------------------------------------------------------------------
// 시트 빌더 (내부 함수 — pool 주입으로 테스트 가능)
// ----------------------------------------------------------------------------

/// 원생 명단 — 일련번호 오름차순.
async fn build_students_sheet(pool: &SqlitePool) -> Result<SheetData, AppError> {
    let headers = vec![
        "일련번호",
        "이름",
        "성별",
        "학교급",
        "학년",
        "학교",
        "입교일",
        "퇴교일",
        "주수업시간(시간)",
        "교습비",
    ];

    // weekly_hours = 현행 스케줄(effective_to IS NULL) duration_hours 합 (billing.rs 와 동일 정의).
    // 교습비 = standard_fees.amount WHERE weekly_hours = ? AND is_active = 1 (매핑 없으면 공란).
    // 정렬: 일련번호 오름차순(숫자 기준).
    let db_rows = sqlx::query(
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
         ORDER BY CAST(s.serial_no AS INTEGER), s.serial_no",
    )
    .fetch_all(pool)
    .await?;

    let mut rows = Vec::with_capacity(db_rows.len());
    for r in &db_rows {
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

        // 일련번호: 숫자 형식 저장(숫자 파싱 실패 시 텍스트 폴백). 학년: 숫자 형식.
        let serial_cell = match serial_no.parse::<i64>() {
            Ok(n) => Cell::Int(n),
            Err(_) => Cell::Text(serial_no),
        };
        rows.push(vec![
            serial_cell,
            Cell::Text(name),
            Cell::Text(gender_label(&gender).to_string()),
            Cell::Text(school_level_label(&school_level).to_string()),
            Cell::Int(grade),
            Cell::Text(school_name.unwrap_or_default()),
            Cell::Text(enroll_date),
            Cell::Text(withdraw_date.unwrap_or_default()),
            Cell::Hours(weekly_hours as f64),
            match fee_amount {
                Some(a) => Cell::Money(a),
                None => Cell::Text(String::new()),
            },
        ]);
    }
    Ok(SheetData { headers, rows })
}

/// 출결 — 정규 + 보강 출결 UNION, 원생명·일자순. `year_month` None 이면 전체.
async fn build_attendances_sheet(
    pool: &SqlitePool,
    year_month: Option<&str>,
) -> Result<SheetData, AppError> {
    let headers = vec!["원생명", "일자", "구분", "상태", "수업시간(시간)"];

    let db_rows = sqlx::query(
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

    let mut rows = Vec::with_capacity(db_rows.len());
    for r in &db_rows {
        let student_name: String = r.try_get("student_name")?;
        let event_date: String = r.try_get("event_date")?;
        let kind: String = r.try_get("kind")?;
        let status: String = r.try_get("status")?;
        let class_minutes: i64 = r.try_get("class_minutes")?;

        rows.push(vec![
            Cell::Text(student_name),
            Cell::Text(event_date),
            Cell::Text(kind),
            Cell::Text(attendance_status_label(&status).to_string()),
            Cell::Hours(minutes_to_hours(class_minutes)),
        ]);
    }
    Ok(SheetData { headers, rows })
}

/// 청구-수납 — 청구월·원생명순. `year_month` None 이면 전체.
async fn build_billing_sheet(
    pool: &SqlitePool,
    year_month: Option<&str>,
) -> Result<SheetData, AppError> {
    let headers = vec![
        "원생명",
        "청구월",
        "청구상태",
        "청구액",
        "할인액",
        "최종액",
        "수납여부",
        "입금일",
        "결제수단",
    ];

    let db_rows = sqlx::query(
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

    let mut rows = Vec::with_capacity(db_rows.len());
    for r in &db_rows {
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

        rows.push(vec![
            Cell::Text(student_name),
            Cell::Text(bill_year_month),
            Cell::Text(bill_status_label(&status).to_string()),
            Cell::Money(bill_amount),
            Cell::Money(discount),
            Cell::Money(adjusted_amount),
            Cell::Text(paid_label.to_string()),
            Cell::Text(paid_date.unwrap_or_default()),
            Cell::Text(method),
        ]);
    }
    Ok(SheetData { headers, rows })
}

// ----------------------------------------------------------------------------
// IPC 커맨드 (전역 pool + .xlsx 기록 래퍼)
// ----------------------------------------------------------------------------

/// 원생 명단을 .xlsx 로 내보낸다.
#[tauri::command]
pub async fn export_students(file_path: String) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let sheet = build_students_sheet(pool).await.map_err(String::from)?;
    let byte_size = write_xlsx(&sheet, &file_path).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count: sheet.rows.len() as i64, byte_size })
}

/// 출결 데이터를 .xlsx 로 내보낸다. `year_month` 가 `None` 이면 전체 기간.
#[tauri::command]
pub async fn export_attendances(
    year_month: Option<String>,
    file_path: String,
) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let sheet = build_attendances_sheet(pool, year_month.as_deref())
        .await
        .map_err(String::from)?;
    let byte_size = write_xlsx(&sheet, &file_path).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count: sheet.rows.len() as i64, byte_size })
}

/// 청구-수납 데이터를 .xlsx 로 내보낸다. `year_month` 가 `None` 이면 전체 기간.
#[tauri::command]
pub async fn export_billing(
    year_month: Option<String>,
    file_path: String,
) -> Result<ExportResult, String> {
    let pool = pool().map_err(String::from)?;
    let sheet = build_billing_sheet(pool, year_month.as_deref())
        .await
        .map_err(String::from)?;
    let byte_size = write_xlsx(&sheet, &file_path).map_err(String::from)?;
    Ok(ExportResult { file_path, row_count: sheet.rows.len() as i64, byte_size })
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

    // ── 원생 시트: 일련번호 오름차순 + 금전/시간 셀 ──
    #[tokio::test]
    async fn students_sheet_sorted_by_serial_asc() {
        let pool = test_pool_in_memory().await.unwrap();
        // 일부러 역순/두 자리 섞어 삽입 → 숫자 오름차순(1,2,10) 기대.
        insert_student(&pool, "10", "김십").await;
        insert_student(&pool, "2", "이둘").await;
        let sid1 = insert_student(&pool, "1", "박일").await;
        // 박일에 주 4시간 스케줄 → 교습비 200000(V201 보정값) 매핑.
        sqlx::query(
            "INSERT INTO student_schedules \
             (student_id, day_of_week, start_time, duration_hours, effective_from) \
             VALUES (?, 1, '15:00', 4, '2026-01-01')",
        )
        .bind(sid1)
        .execute(&pool)
        .await
        .unwrap();

        let sheet = build_students_sheet(&pool).await.unwrap();
        assert_eq!(sheet.rows.len(), 3);
        // 정렬: 1 → 2 → 10 (일련번호는 숫자 형식).
        assert_eq!(sheet.rows[0][0], Cell::Int(1));
        assert_eq!(sheet.rows[1][0], Cell::Int(2));
        assert_eq!(sheet.rows[2][0], Cell::Int(10));
        // 학년도 숫자 형식.
        assert_eq!(sheet.rows[0][4], Cell::Int(1));
        // 박일(serial 1): 주수업시간 Hours(4), 교습비 Money(200000).
        assert_eq!(sheet.rows[0][8], Cell::Hours(4.0));
        assert_eq!(sheet.rows[0][9], Cell::Money(200_000));
        // 성별/학교급 라벨.
        assert_eq!(sheet.rows[0][2], Cell::Text("여".into()));
        assert_eq!(sheet.rows[0][3], Cell::Text("중등".into()));
    }

    #[tokio::test]
    async fn students_sheet_empty() {
        let pool = test_pool_in_memory().await.unwrap();
        let sheet = build_students_sheet(&pool).await.unwrap();
        assert_eq!(sheet.rows.len(), 0);
        assert_eq!(sheet.headers[0], "일련번호");
    }

    // ── 출결 시트: 정규+보강 UNION + 분→시간 환산 ──
    #[tokio::test]
    async fn attendances_sheet_unions_and_converts_hours() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "1", "이결석").await;
        // 결석 90분 → 1.5시간.
        sqlx::query("INSERT INTO regular_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-02', '2026-06', 'absent', 90)")
            .bind(sid).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO makeup_attendances (student_id, event_date, year_month, status, class_minutes) VALUES (?, '2026-06-09', '2026-06', 'makeup_attended', 60)")
            .bind(sid).execute(&pool).await.unwrap();

        let sheet = build_attendances_sheet(&pool, Some("2026-06")).await.unwrap();
        assert_eq!(sheet.rows.len(), 2, "정규 1 + 보강 1");
        // 분 → 시간 환산 확인 (90분=1.5, 60분=1.0). 마지막 컬럼.
        let hours: Vec<&Cell> = sheet.rows.iter().map(|r| &r[4]).collect();
        assert!(hours.contains(&&Cell::Hours(1.5)));
        assert!(hours.contains(&&Cell::Hours(1.0)));

        // 전체(None) 도 2건.
        let all = build_attendances_sheet(&pool, None).await.unwrap();
        assert_eq!(all.rows.len(), 2);
        // 다른 월 → 0.
        let other = build_attendances_sheet(&pool, Some("2026-05")).await.unwrap();
        assert_eq!(other.rows.len(), 0);
    }

    // ── 청구 시트: 금전 셀 ──
    #[tokio::test]
    async fn billing_sheet_money_cells() {
        let pool = test_pool_in_memory().await.unwrap();
        let sid = insert_student(&pool, "1", "박청구").await;
        let bid: (i64,) = sqlx::query_as(
            "INSERT INTO bills (student_id, bill_year_month, weekly_hours, bill_amount, adjusted_amount, status) \
             VALUES (?, '2026-06', 4, 250000, 230000, 'confirmed') RETURNING id",
        )
        .bind(sid)
        .fetch_one(&pool)
        .await
        .unwrap();
        let pm_card: (i64,) = sqlx::query_as("SELECT id FROM payment_methods WHERE code='card'")
            .fetch_one(&pool).await.unwrap();
        let cc: (i64,) = sqlx::query_as("SELECT id FROM card_companies WHERE code='shinhan'")
            .fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO payments (bill_id, is_paid, paid_date, payment_method_id, card_company_id) VALUES (?, 1, '2026-06-05', ?, ?)")
            .bind(bid.0).bind(pm_card.0).bind(cc.0).execute(&pool).await.unwrap();

        let sheet = build_billing_sheet(&pool, Some("2026-06")).await.unwrap();
        assert_eq!(sheet.rows.len(), 1);
        let row = &sheet.rows[0];
        assert_eq!(row[3], Cell::Money(250_000)); // 청구액
        assert_eq!(row[4], Cell::Money(20_000)); // 할인액
        assert_eq!(row[5], Cell::Money(230_000)); // 최종액
        assert_eq!(row[6], Cell::Text("수납완료".into()));
        assert_eq!(row[8], Cell::Text("카드(신한카드)".into())); // 결제수단
    }

    // ── .xlsx 기록 ──
    #[tokio::test]
    async fn write_xlsx_creates_nonempty_file() {
        let sheet = SheetData {
            headers: vec!["이름", "교습비"],
            rows: vec![vec![Cell::Text("홍길동".into()), Cell::Money(200_000)]],
        };
        let path = std::env::temp_dir().join("smarthb_export_test.xlsx");
        let path_str = path.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&path);
        let size = write_xlsx(&sheet, &path_str).unwrap();
        assert!(size > 0, "xlsx 파일이 생성되어야 함");
        assert!(path.exists());
        let _ = std::fs::remove_file(&path);
    }
}
