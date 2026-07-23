//! CSV 원생 가져오기 (Sprint 16 T2, PRD §4.13.1) — 실데이터 이관.
//!
//! ## 범위 (단순·안전)
//!
//! 대상은 `students` 한 테이블뿐이다. 학교 FK(`school_id`)·수업 스케줄(`student_schedules`)은
//! CSV 로 넣지 않고 앱에서 수동 입력한다(`school_id` 는 NULL 로 INSERT).
//!
//! ## 인터페이스 (2 IPC)
//!
//! - [`preview_students_csv`] — 파싱·검증·중복판정만 수행(드라이런, INSERT 없음). 미리보기용.
//! - [`import_students_csv`] — 가져오기 직전 백업 1회 후 유효·비중복 행을 단일 트랜잭션(`insert_student_tx`)으로 INSERT (중간 실패 시 전체 롤백).
//!
//! ## 컬럼
//!
//! 이름(필수) / 학년(필수, "초3"·"중2") / 입교일(필수, 비면 실행일) / 성별·생년월일·연락처·일련번호(선택).
//! 헤더명은 동의어 사전으로 자동 인식한다. 필수 컬럼을 못 찾으면 헤더 단계에서 거부한다.
//!
//! ## 중복 (skip 만)
//!
//! 일련번호가 이미 존재하거나 (이름+모연락처)가 이미 존재하면 건너뛴다. 덮어쓰기는 사고
//! 위험이 커 제공하지 않는다. 같은 파일 내 중복도 INSERT 진행에 따라 누적 반영해 막는다.
//!
//! ## 인코딩
//!
//! UTF-8 BOM 제거 후 UTF-8 디코딩을 시도하고, 실패 시 EUC-KR(CP949)로 디코딩한다(엑셀 한글).

use crate::commands::audit::{self, AuditEventType};
use crate::commands::backup::{create_backup, BackupLayer};
use crate::commands::db;
use crate::commands::students::{insert_student_tx, Gender, NewStudent, SchoolLevel};
use crate::error::AppError;
use serde::Serialize;
use sqlx::Row;
use std::collections::HashSet;

// ============================================================================
// 응답 타입 (프론트 정합 — snake_case)
// ============================================================================

/// 미리보기 행 — 원본 표시값 + 검증 상태.
#[derive(Debug, Serialize)]
pub struct PreviewRow {
    /// CSV 데이터 행 번호 (헤더 제외, 1부터).
    pub row_number: usize,
    pub name: String,
    /// 원본 학년 텍스트 (예: "초3").
    pub grade_label: String,
    /// 표시용 성별 ("남"/"여"/"").
    pub gender_label: String,
    pub enroll_date: String,
    pub serial_no: Option<String>,
    /// "ok" | "warning" | "duplicate" | "error".
    pub status: String,
    pub messages: Vec<String>,
}

/// 미리보기 결과 — 행 목록 + 집계.
#[derive(Debug, Serialize)]
pub struct PreviewResult {
    pub rows: Vec<PreviewRow>,
    pub total: usize,
    /// 가져올 수 있는 행 (ok + warning).
    pub importable: usize,
    pub duplicate: usize,
    pub error: usize,
}

/// 가져오기 결과 — 집계 + 백업 메모 + 행별 오류.
#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub inserted: usize,
    pub skipped: usize,
    pub errored: usize,
    pub errors: Vec<String>,
    pub backup_note: String,
}

// ============================================================================
// 내부 — 파싱/검증 (순수 함수, DB 비의존)
// ============================================================================

/// 바이트열을 문자열로 디코딩. UTF-8 BOM 제거 → UTF-8 시도 → 실패 시 EUC-KR.
fn decode_bytes(bytes: &[u8]) -> String {
    let body = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(body) {
        Ok(s) => s.to_string(),
        Err(_) => {
            let (cow, _, _) = encoding_rs::EUC_KR.decode(body);
            cow.into_owned()
        }
    }
}

/// 인식 가능한 컬럼 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Name,
    Grade,
    Enroll,
    Gender,
    Birth,
    Serial,
    PhoneMother,
    PhoneFather,
    PhoneStudent,
}

/// 헤더명(공백 제거)을 컬럼 종류로 매핑. 동의어 사전.
fn match_header(raw: &str) -> Option<Field> {
    let n: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
    match n.as_str() {
        "이름" | "성명" | "학생명" | "학생이름" => Some(Field::Name),
        "학년" | "학년반" | "학교학년" | "학교급학년" => Some(Field::Grade),
        "입교일" | "등록일" | "입학일" | "입교일자" | "등록일자" => Some(Field::Enroll),
        "성별" => Some(Field::Gender),
        "생년월일" | "생일" | "출생일" => Some(Field::Birth),
        "일련번호" | "번호" | "순번" => Some(Field::Serial),
        "모연락처" | "어머니" | "엄마" | "모" | "보호자연락처" | "연락처" | "전화"
        | "전화번호" | "휴대폰" | "핸드폰" => Some(Field::PhoneMother),
        "부연락처" | "아버지" | "아빠" | "부" => Some(Field::PhoneFather),
        "학생연락처" | "본인연락처" | "학생전화" => Some(Field::PhoneStudent),
        _ => None,
    }
}

/// 헤더 행에서 컬럼 인덱스를 해석한 결과.
struct ColumnMap {
    name: usize,
    grade: usize,
    enroll: usize,
    gender: Option<usize>,
    birth: Option<usize>,
    serial: Option<usize>,
    phone_mother: Option<usize>,
    phone_father: Option<usize>,
    phone_student: Option<usize>,
}

impl ColumnMap {
    fn from_headers(headers: &csv::StringRecord) -> Result<Self, AppError> {
        let mut name = None;
        let mut grade = None;
        let mut enroll = None;
        let mut gender = None;
        let mut birth = None;
        let mut serial = None;
        let mut phone_mother = None;
        let mut phone_father = None;
        let mut phone_student = None;
        for (idx, h) in headers.iter().enumerate() {
            match match_header(h) {
                Some(Field::Name) => name = name.or(Some(idx)),
                Some(Field::Grade) => grade = grade.or(Some(idx)),
                Some(Field::Enroll) => enroll = enroll.or(Some(idx)),
                Some(Field::Gender) => gender = gender.or(Some(idx)),
                Some(Field::Birth) => birth = birth.or(Some(idx)),
                Some(Field::Serial) => serial = serial.or(Some(idx)),
                Some(Field::PhoneMother) => phone_mother = phone_mother.or(Some(idx)),
                Some(Field::PhoneFather) => phone_father = phone_father.or(Some(idx)),
                Some(Field::PhoneStudent) => phone_student = phone_student.or(Some(idx)),
                None => {}
            }
        }
        let mut missing = Vec::new();
        if name.is_none() {
            missing.push("이름");
        }
        if grade.is_none() {
            missing.push("학년");
        }
        if enroll.is_none() {
            missing.push("입교일");
        }
        if !missing.is_empty() {
            return Err(AppError::UserFacing(format!(
                "CSV 에서 필수 컬럼을 찾을 수 없습니다: {}. 첫 행에 컬럼 제목(이름·학년·입교일)이 있는지 확인해 주세요.",
                missing.join(", ")
            )));
        }
        Ok(Self {
            name: name.unwrap(),
            grade: grade.unwrap(),
            enroll: enroll.unwrap(),
            gender,
            birth,
            serial,
            phone_mother,
            phone_father,
            phone_student,
        })
    }
}

/// "초3"·"중2" → (학교급, 학년). 초등 1~6, 중등 1~3 범위 검증.
fn parse_grade(raw: &str) -> Result<(SchoolLevel, i64), String> {
    let t = raw.trim();
    let level = if t.contains('초') {
        SchoolLevel::Elementary
    } else if t.contains('중') {
        SchoolLevel::Middle
    } else {
        return Err(format!(
            "학년 형식을 인식할 수 없습니다: '{}'. '초3'·'중2' 형식으로 입력해 주세요.",
            raw
        ));
    };
    let digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    let grade: i64 = digits
        .parse()
        .map_err(|_| format!("학년에서 숫자를 찾을 수 없습니다: '{}'.", raw))?;
    let max = match level {
        SchoolLevel::Elementary => 6,
        SchoolLevel::Middle => 3,
    };
    if !(1..=max).contains(&grade) {
        return Err(format!("학년 범위(1~{})를 벗어났습니다: '{}'.", max, raw));
    }
    Ok((level, grade))
}

/// 성별 텍스트 → (Gender, 경고). 비거나 불명이면 '남' 기본 + 경고(NOT NULL 제약, 사후 수정).
fn parse_gender(raw: &str) -> (Gender, Option<String>) {
    match raw.trim() {
        "남" | "남자" | "male" | "M" | "m" => (Gender::Male, None),
        "여" | "여자" | "female" | "F" | "f" => (Gender::Female, None),
        "" => (
            Gender::Male,
            Some("성별 미입력 — '남'으로 가져옵니다(이후 수정 가능).".to_string()),
        ),
        other => (
            Gender::Male,
            Some(format!("성별 '{}'을(를) 인식할 수 없어 '남'으로 가져옵니다.", other)),
        ),
    }
}

/// 표시용 성별 라벨.
fn gender_label(g: Gender) -> &'static str {
    match g {
        Gender::Male => "남",
        Gender::Female => "여",
    }
}

/// "2026.3.1"·"2026/03/01" 등 → "2026-03-01" 정규화 + 유효성 검증.
fn normalize_date(raw: &str) -> Result<String, String> {
    let t = raw.trim().replace(['.', '/', ' '], "-");
    let parts: Vec<&str> = t.split('-').filter(|p| !p.is_empty()).collect();
    if parts.len() != 3 {
        return Err(format!("날짜 형식이 올바르지 않습니다: '{}'. 예: 2026-03-01", raw));
    }
    let y: i32 = parts[0].parse().map_err(|_| format!("날짜 연도 인식 실패: '{}'.", raw))?;
    let m: u32 = parts[1].parse().map_err(|_| format!("날짜 월 인식 실패: '{}'.", raw))?;
    let d: u32 = parts[2].parse().map_err(|_| format!("날짜 일 인식 실패: '{}'.", raw))?;
    chrono::NaiveDate::from_ymd_opt(y, m, d)
        .ok_or_else(|| format!("존재하지 않는 날짜입니다: '{}'.", raw))?;
    Ok(format!("{:04}-{:02}-{:02}", y, m, d))
}

/// 선택 텍스트 필드 추출 — trim 후 빈 값이면 None.
fn optional_text(rec: &csv::StringRecord, idx: Option<usize>) -> Option<String> {
    idx.and_then(|i| rec.get(i))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// 한 행을 검증해 INSERT payload 와 경고 목록으로 변환.
fn build_payload(
    rec: &csv::StringRecord,
    cm: &ColumnMap,
    today: &str,
) -> Result<(NewStudent, Vec<String>), String> {
    let mut warnings = Vec::new();

    let name = rec.get(cm.name).unwrap_or("").trim();
    if name.is_empty() {
        return Err("이름이 비어 있습니다.".to_string());
    }

    let (school_level, grade) = parse_grade(rec.get(cm.grade).unwrap_or(""))?;

    let enroll_raw = rec.get(cm.enroll).unwrap_or("").trim();
    let enroll_date = if enroll_raw.is_empty() {
        warnings.push(format!("입교일 미입력 — 오늘({})로 설정합니다.", today));
        today.to_string()
    } else {
        normalize_date(enroll_raw)?
    };

    let (gender, gender_warn) = match cm.gender {
        Some(i) => parse_gender(rec.get(i).unwrap_or("")),
        None => (
            Gender::Male,
            Some("성별 컬럼이 없어 모두 '남'으로 가져옵니다(이후 수정 가능).".to_string()),
        ),
    };
    if let Some(w) = gender_warn {
        warnings.push(w);
    }

    // 생년월일은 선택 — 형식 오류 시 행 전체를 막지 않고 비워두고 경고만.
    let birth_date = match optional_text(rec, cm.birth) {
        Some(raw) => match normalize_date(&raw) {
            Ok(d) => Some(d),
            Err(msg) => {
                warnings.push(format!("생년월일 무시 — {}", msg));
                None
            }
        },
        None => None,
    };

    Ok((
        NewStudent {
            serial_no: optional_text(rec, cm.serial),
            name: name.to_string(),
            gender,
            school_level,
            grade,
            school_id: None,
            phone_student: optional_text(rec, cm.phone_student),
            phone_mother: optional_text(rec, cm.phone_mother),
            phone_father: optional_text(rec, cm.phone_father),
            birth_date,
            enroll_date,
        },
        warnings,
    ))
}

/// 파싱된 한 행 — 미리보기 표시 + INSERT payload(에러면 None).
struct ParsedRow {
    preview: PreviewRow,
    payload: Option<NewStudent>,
}

/// CSV 바이트열을 파싱. 헤더 단계 오류(필수 컬럼 누락 등)는 Err 로 전체 거부.
fn parse_csv(bytes: &[u8]) -> Result<Vec<ParsedRow>, AppError> {
    let text = decode_bytes(bytes);
    let today = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(text.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| AppError::UserFacing(format!("CSV 헤더를 읽을 수 없습니다: {}", e)))?
        .clone();
    let cm = ColumnMap::from_headers(&headers)?;

    let mut out = Vec::new();
    for (idx, result) in rdr.records().enumerate() {
        let row_number = idx + 1;
        let rec = match result {
            Ok(r) => r,
            Err(e) => {
                out.push(error_row(row_number, String::new(), String::new(), None, format!("행을 읽을 수 없습니다: {}", e)));
                continue;
            }
        };
        if rec.iter().all(|f| f.trim().is_empty()) {
            continue; // 빈 행 무시
        }
        let name = rec.get(cm.name).unwrap_or("").trim().to_string();
        let grade_label = rec.get(cm.grade).unwrap_or("").trim().to_string();
        let serial = optional_text(&rec, cm.serial);
        match build_payload(&rec, &cm, &today) {
            Ok((payload, warnings)) => {
                let status = if warnings.is_empty() { "ok" } else { "warning" };
                out.push(ParsedRow {
                    preview: PreviewRow {
                        row_number,
                        name,
                        grade_label,
                        gender_label: gender_label(payload.gender).to_string(),
                        enroll_date: payload.enroll_date.clone(),
                        serial_no: payload.serial_no.clone(),
                        status: status.to_string(),
                        messages: warnings,
                    },
                    payload: Some(payload),
                });
            }
            Err(msg) => out.push(error_row(row_number, name, grade_label, serial, msg)),
        }
    }
    Ok(out)
}

/// 에러 미리보기 행 생성.
fn error_row(
    row_number: usize,
    name: String,
    grade_label: String,
    serial_no: Option<String>,
    msg: String,
) -> ParsedRow {
    ParsedRow {
        preview: PreviewRow {
            row_number,
            name,
            grade_label,
            gender_label: String::new(),
            enroll_date: String::new(),
            serial_no,
            status: "error".to_string(),
            messages: vec![msg],
        },
        payload: None,
    }
}

// ============================================================================
// 내부 — 중복 판정 (순수 함수)
// ============================================================================

/// 기존 원생 인덱스 — 중복 판정용.
struct ExistingStudents {
    serials: HashSet<String>,
    name_phone: HashSet<(String, String)>,
}

/// 일련번호 존재 OR (이름+모연락처) 존재 시 중복.
fn is_duplicate(existing: &ExistingStudents, p: &NewStudent) -> bool {
    if let Some(s) = &p.serial_no {
        if existing.serials.contains(s) {
            return true;
        }
    }
    if let Some(m) = &p.phone_mother {
        if existing.name_phone.contains(&(p.name.clone(), m.clone())) {
            return true;
        }
    }
    false
}

/// 성공 INSERT 를 인덱스에 누적 — 같은 파일 내 중복도 막는다.
fn remember(existing: &mut ExistingStudents, serial: &str, name: &str, phone_mother: Option<&str>) {
    existing.serials.insert(serial.to_string());
    if let Some(m) = phone_mother {
        existing.name_phone.insert((name.to_string(), m.to_string()));
    }
}

async fn load_existing(pool: &sqlx::SqlitePool) -> Result<ExistingStudents, AppError> {
    let rows = sqlx::query("SELECT serial_no, name, phone_mother FROM students")
        .fetch_all(pool)
        .await
        .map_err(AppError::Db)?;
    let mut serials = HashSet::new();
    let mut name_phone = HashSet::new();
    for r in rows {
        let serial: String = r.try_get("serial_no").map_err(AppError::Db)?;
        serials.insert(serial);
        let name: String = r.try_get("name").map_err(AppError::Db)?;
        let phone: Option<String> = r.try_get("phone_mother").map_err(AppError::Db)?;
        if let Some(p) = phone {
            name_phone.insert((name, p));
        }
    }
    Ok(ExistingStudents { serials, name_phone })
}

// ============================================================================
// Tauri IPC commands
// ============================================================================

/// CSV 미리보기 — 파싱·검증·중복판정만(INSERT 없음).
#[tauri::command]
pub async fn preview_students_csv(file_path: String) -> Result<PreviewResult, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다: {}", e))?;
    let parsed = parse_csv(&bytes).map_err(String::from)?;
    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
    let existing = load_existing(pool).await.map_err(String::from)?;

    let mut rows = Vec::with_capacity(parsed.len());
    let (mut importable, mut duplicate, mut error) = (0usize, 0usize, 0usize);
    for ParsedRow { mut preview, payload } in parsed {
        match payload {
            Some(p) if is_duplicate(&existing, &p) => {
                preview.status = "duplicate".to_string();
                preview.messages.push("이미 등록된 원생입니다 — 가져오기에서 제외됩니다.".to_string());
                duplicate += 1;
            }
            Some(_) => importable += 1,
            None => error += 1,
        }
        rows.push(preview);
    }
    let total = rows.len();
    Ok(PreviewResult {
        rows,
        total,
        importable,
        duplicate,
        error,
    })
}

/// CSV 가져오기 — 백업 1회 후 유효·비중복 행을 INSERT.
#[tauri::command]
pub async fn import_students_csv(file_path: String) -> Result<ImportResult, String> {
    let bytes = std::fs::read(&file_path)
        .map_err(|e| format!("파일을 읽을 수 없습니다: {}", e))?;
    let parsed = parse_csv(&bytes).map_err(String::from)?;

    // 가져오기 직전 백업 1회 — 실패해도 진행하되 결과에 명시(개발 모드는 백업 stub).
    let backup_note = match create_backup(BackupLayer::Exit).await {
        Ok(_) => "가져오기 전 백업을 생성했습니다.".to_string(),
        Err(e) => format!("백업을 생성하지 못했습니다(가져오기는 계속 진행): {}", e),
    };

    let pool = db::pool().await.map_err(String::from)?;
    let pool = &pool;
    let mut existing = load_existing(pool).await.map_err(String::from)?;

    // 전체 가져오기를 단일 트랜잭션으로 묶는다 — 중간 DB 오류 시 부분 삽입 없이 전부 롤백(코드리뷰 C2).
    let mut tx = pool
        .begin()
        .await
        .map_err(AppError::Db)
        .map_err(String::from)?;

    let (mut inserted, mut skipped, mut errored) = (0usize, 0usize, 0usize);
    let mut errors = Vec::new();
    // 커밋 성공 후 기록할 audit 대상 — 트랜잭션 외부에서 일괄 기록.
    let mut inserted_serials = Vec::new();
    for ParsedRow { preview, payload } in parsed {
        let row_number = preview.row_number;
        let payload = match payload {
            Some(p) => p,
            None => {
                // 파싱·검증 실패 행은 DB 에 닿지 않으므로 롤백 대상이 아니다(미리보기에서 이미 노출).
                errored += 1;
                if let Some(m) = preview.messages.first() {
                    errors.push(format!("{}행: {}", row_number, m));
                }
                continue;
            }
        };
        if is_duplicate(&existing, &payload) {
            skipped += 1;
            continue;
        }
        match insert_student_tx(&mut tx, &payload).await {
            Ok(s) => {
                inserted += 1;
                remember(&mut existing, &s.serial_no, &s.name, s.phone_mother.as_deref());
                inserted_serials.push(s.serial_no);
            }
            Err(e) => {
                // 원자성: DB 오류가 한 건이라도 발생하면 전체 롤백 후 중단(부분 삽입 방지).
                tx.rollback().await.ok();
                return Err(format!(
                    "{}행에서 오류가 발생하여 가져오기를 취소했습니다(삽입된 행 없음): {}",
                    row_number,
                    String::from(e)
                ));
            }
        }
    }

    tx.commit().await.map_err(AppError::Db).map_err(String::from)?;

    // R13 PII 마스킹: serial_no 만 기록. 트랜잭션 커밋 후 일괄 기록.
    for serial in &inserted_serials {
        audit::try_record(AuditEventType::StudentCreated, Some(serial), None).await;
    }

    Ok(ImportResult {
        inserted,
        skipped,
        errored,
        errors,
        backup_note,
    })
}

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8_plain_and_bom() {
        assert_eq!(decode_bytes("이름,학년".as_bytes()), "이름,학년");
        let mut with_bom = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice("이름".as_bytes());
        assert_eq!(decode_bytes(&with_bom), "이름");
    }

    #[test]
    fn decode_euc_kr_fallback() {
        // "이름" in EUC-KR (CP949)
        let (euc, _, _) = encoding_rs::EUC_KR.encode("이름");
        let decoded = decode_bytes(&euc);
        assert_eq!(decoded, "이름");
    }

    #[test]
    fn parse_grade_elementary_and_middle() {
        assert_eq!(parse_grade("초3").unwrap(), (SchoolLevel::Elementary, 3));
        assert_eq!(parse_grade("중2").unwrap(), (SchoolLevel::Middle, 2));
        assert_eq!(parse_grade(" 초등 6 ").unwrap(), (SchoolLevel::Elementary, 6));
    }

    #[test]
    fn parse_grade_rejects_out_of_range_and_unknown() {
        assert!(parse_grade("초7").is_err()); // 초등 최대 6
        assert!(parse_grade("중4").is_err()); // 중등 최대 3
        assert!(parse_grade("3").is_err()); // 학교급 표기 없음
        assert!(parse_grade("고1").is_err()); // 미지원 학교급
    }

    #[test]
    fn parse_gender_variants() {
        assert_eq!(parse_gender("남").0, Gender::Male);
        assert_eq!(parse_gender("여자").0, Gender::Female);
        assert_eq!(parse_gender("female").0, Gender::Female);
        // 빈 값/불명 → 남 기본 + 경고
        let (g, w) = parse_gender("");
        assert_eq!(g, Gender::Male);
        assert!(w.is_some());
        let (g2, w2) = parse_gender("?");
        assert_eq!(g2, Gender::Male);
        assert!(w2.is_some());
    }

    #[test]
    fn normalize_date_formats() {
        assert_eq!(normalize_date("2026-03-01").unwrap(), "2026-03-01");
        assert_eq!(normalize_date("2026.3.1").unwrap(), "2026-03-01");
        assert_eq!(normalize_date("2026/12/25").unwrap(), "2026-12-25");
        assert!(normalize_date("2026-13-01").is_err()); // 존재하지 않는 월
        assert!(normalize_date("abc").is_err());
    }

    fn parse_text(csv: &str) -> Vec<ParsedRow> {
        parse_csv(csv.as_bytes()).unwrap()
    }

    #[test]
    fn parse_csv_maps_headers_and_rows() {
        let csv = "이름,학년,입교일,성별,연락처\n홍길동,초3,2026-03-01,남,010-1111-2222\n";
        let rows = parse_text(csv);
        assert_eq!(rows.len(), 1);
        let p = rows[0].payload.as_ref().unwrap();
        assert_eq!(p.name, "홍길동");
        assert_eq!(p.school_level, SchoolLevel::Elementary);
        assert_eq!(p.grade, 3);
        assert_eq!(p.gender, Gender::Male);
        assert_eq!(p.phone_mother.as_deref(), Some("010-1111-2222"));
        assert_eq!(p.school_id, None);
        assert_eq!(rows[0].preview.status, "ok");
    }

    #[test]
    fn parse_csv_missing_required_column_is_rejected() {
        // 입교일 컬럼 없음
        let err = parse_csv("이름,학년\n홍길동,초3\n".as_bytes());
        assert!(err.is_err());
    }

    #[test]
    fn parse_csv_row_errors_do_not_abort_others() {
        let csv = "이름,학년,입교일\n,초3,2026-03-01\n김철수,초5,2026-03-02\n";
        let rows = parse_text(csv);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].preview.status, "error"); // 이름 누락
        assert!(rows[0].payload.is_none());
        // 둘째 행은 정상 파싱됨 — 에러 행이 후속 행을 막지 않는다.
        // (성별 컬럼이 없는 CSV라 status 는 warning, payload 는 Some)
        assert!(rows[1].payload.is_some());
    }

    #[test]
    fn parse_csv_empty_enroll_defaults_today_with_warning() {
        let csv = "이름,학년,입교일\n홍길동,초3,\n";
        let rows = parse_text(csv);
        assert_eq!(rows[0].preview.status, "warning");
        assert!(rows[0].payload.as_ref().unwrap().enroll_date.len() == 10);
    }

    #[test]
    fn is_duplicate_by_serial_and_name_phone() {
        let mut existing = ExistingStudents {
            serials: HashSet::new(),
            name_phone: HashSet::new(),
        };
        existing.serials.insert("100".to_string());
        existing
            .name_phone
            .insert(("홍길동".to_string(), "010-1".to_string()));

        let mk = |serial: Option<&str>, name: &str, phone: Option<&str>| NewStudent {
            serial_no: serial.map(String::from),
            name: name.to_string(),
            gender: Gender::Male,
            school_level: SchoolLevel::Elementary,
            grade: 3,
            school_id: None,
            phone_student: None,
            phone_mother: phone.map(String::from),
            phone_father: None,
            birth_date: None,
            enroll_date: "2026-03-01".to_string(),
        };

        assert!(is_duplicate(&existing, &mk(Some("100"), "다른이름", None)));
        assert!(is_duplicate(&existing, &mk(None, "홍길동", Some("010-1"))));
        assert!(!is_duplicate(&existing, &mk(Some("999"), "신규", Some("010-9"))));
    }
}
