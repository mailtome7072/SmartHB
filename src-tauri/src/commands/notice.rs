//! 공지문(교습비 안내 이미지) 도메인 IPC — Sprint 12 (PRD §4.10).
//!
//! 청구 데이터를 소스로, 배경서식 위에 텍스트박스를 배치하여 원생별 PNG를 일괄 생성한다.
//! 파일 시스템 기반 — 배경서식/출력 이미지는 클라우드 동기화 폴더 하위에 둔다 (양 PC 공유).
//!
//! ## 구성
//! - 배경서식(T2): `{data_root}/assets/` 에 이미지 파일로 관리 (DB 테이블 없음).
//! - 레이아웃(T3): 텍스트박스 위치/속성을 `app_settings` 의 JSON 값으로 저장 (AC-4.10-3).
//! - 출력(T4): 생성된 PNG를 `{data_root}/output/{YYYYMM}/` 에 저장 (PRD §4.10.2).
//!
//! ## 보안
//! - 이미지 바이너리는 `Vec<u8>` 로 IPC 전달 (base64 Rust 크레이트 미도입 — 의존성 최소화).
//! - 파일명/원생명은 [`sanitize_component`] 로 경로 traversal(`..`, 경로 구분자) 차단 후 사용.

use crate::commands::db;
use crate::commands::paths;
use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::path::{Path, PathBuf};

const KEY_NOTICE_LAYOUT: &str = "notice_layout";
const KEY_NOTICE_LAYOUT_NAMES: &str = "notice_layout_names";
const ALLOWED_IMAGE_EXTS: [&str; 3] = ["png", "jpg", "jpeg"];

// ─────────────────────── 직렬화 타입 ───────────────────────

/// 배경서식 파일 메타데이터.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoticeAsset {
    pub name: String,
    pub size: i64,
    /// 수정 시각 (epoch millis). 프론트에서 표시 포맷.
    pub modified_ms: i64,
}

/// 텍스트박스 1종 설정 — **배경 원본 해상도 대비 비율(0..1)** 로 관리한다.
/// 이렇게 하면 미리보기 표시 배율·생성 원본 해상도와 무관하게 동일 레이아웃이 유지된다.
fn default_enabled() -> bool {
    true
}

/// 데이터 필드 기본 텍스트박스 5종 (청구월/교습기간/보강데이/원생명/청구액).
/// 교습기간·보강데이는 기본 비활성.
fn default_textboxes() -> Vec<TextboxConfig> {
    let mk = |field: &str, y_ratio: f64, enabled: bool| TextboxConfig {
        id: field.to_string(),
        field_type: field.to_string(),
        text: None,
        enabled,
        x_ratio: 0.1,
        y_ratio,
        w_ratio: 0.8,
        h_ratio: 0.12,
        font_ratio: 0.5,
        font_weight: "bold".to_string(),
        font_color: "#1A1A1A".to_string(),
        text_align: "center".to_string(),
    };
    vec![
        mk("bill_month", 0.05, true),
        mk("teaching_period", 0.2, false),
        mk("makeup_day", 0.35, false),
        mk("student_name", 0.55, true),
        mk("bill_amount", 0.75, true),
    ]
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextboxConfig {
    /// 고유 키 (구버전 호환: 누락 시 빈 문자열 → 프론트가 field_type 으로 대체).
    #[serde(default)]
    pub id: String,
    pub field_type: String, // "bill_month" | "student_name" | "bill_amount" | "custom"
    /// 사용자 정의 박스("custom")의 표시 텍스트. 데이터 필드는 None.
    #[serde(default)]
    pub text: Option<String>,
    /// 체크 시에만 배경 위에 표시·생성된다. (구버전 레이아웃 호환: 누락 시 true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub x_ratio: f64,       // 배경 폭 대비 좌측 위치 (0..1)
    pub y_ratio: f64,       // 배경 높이 대비 상단 위치 (0..1)
    pub w_ratio: f64,       // 배경 폭 대비 너비 (0..1)
    pub h_ratio: f64,       // 배경 높이 대비 높이 (0..1)
    pub font_ratio: f64,    // 박스 높이 대비 글자 크기 (0..1) — 박스 리사이즈 시 폰트 자동 연동
    pub font_weight: String, // "normal" | "bold"
    pub font_color: String,  // hex
    pub text_align: String,  // "left" | "center" | "right"
}

/// 공지문 레이아웃 — 배경서식 + 텍스트박스 3종.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NoticeLayout {
    pub background_asset: Option<String>,
    pub textboxes: Vec<TextboxConfig>,
}

impl NoticeLayout {
    /// 저장된 레이아웃이 없을 때의 기본값 — 데이터 필드 비율 기본 배치(배경 대비).
    /// 교습기간/보강데이는 기본 비활성(체크 시 표시).
    fn default_layout() -> Self {
        NoticeLayout {
            background_asset: None,
            textboxes: default_textboxes(),
        }
    }
}

/// 이미지 일괄 저장 입력 1건.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoticeImageItem {
    pub student_name: String,
    pub image: Vec<u8>,
}

// ─────────────────────── 헬퍼 ───────────────────────

/// 파일명/원생명 구성요소를 안전하게 정규화 — 경로 traversal + 파일시스템 위험 문자 제거.
/// 공백 → 언더스코어. 결과가 비면 "unnamed" 반환.
fn sanitize_component(raw: &str) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_whitespace() => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    // ".." 및 선행/후행 점 제거 (경로 traversal · 숨김파일 방지)
    let cleaned = cleaned.replace("..", "_").trim_matches('.').to_string();
    if cleaned.is_empty() {
        "unnamed".to_string()
    } else {
        cleaned
    }
}

fn has_allowed_ext(name: &str) -> bool {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_IMAGE_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn ensure_dir(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("디렉토리 생성 실패 ({}): {}", dir.display(), e))
}

// ─────────────────────── T2: 배경서식 관리 ───────────────────────

/// `assets/` 디렉토리의 배경서식(PNG/JPG) 목록.
#[tauri::command]
pub async fn list_notice_assets() -> Result<Vec<NoticeAsset>, String> {
    let dir = paths::assets_dir();
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("배경서식 목록 조회 실패: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if has_allowed_ext(n) => n.to_string(),
            _ => continue,
        };
        let meta = entry.metadata().map_err(|e| format!("파일 메타 조회 실패: {}", e))?;
        let modified_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        out.push(NoticeAsset {
            name,
            size: meta.len() as i64,
            modified_ms,
        });
    }
    out.sort_by_key(|a| std::cmp::Reverse(a.modified_ms));
    Ok(out)
}

/// 배경서식 바이트 읽기 — 미리보기/생성용. `assets/{filename}` 의 내용을 Vec<u8> 로 반환.
#[tauri::command]
pub async fn read_notice_asset(filename: String) -> Result<Vec<u8>, String> {
    let safe = sanitize_component(&filename);
    let target = paths::assets_dir().join(&safe);
    std::fs::read(&target).map_err(|e| format!("배경서식 읽기 실패: {}", e))
}

/// 배경서식 저장 — `assets/{filename}`. 동일 파일명은 덮어쓰기(프론트에서 확인).
#[tauri::command]
pub async fn save_notice_asset(filename: String, data: Vec<u8>) -> Result<String, String> {
    if !has_allowed_ext(&filename) {
        return Err("PNG 또는 JPG 이미지 파일만 업로드할 수 있습니다.".to_string());
    }
    let safe = sanitize_component(&filename);
    let dir = paths::assets_dir();
    ensure_dir(&dir)?;
    let target = dir.join(&safe);
    std::fs::write(&target, &data).map_err(|e| format!("배경서식 저장 실패: {}", e))?;
    Ok(safe)
}

/// 배경서식 삭제 — `assets/{filename}`.
#[tauri::command]
pub async fn delete_notice_asset(filename: String) -> Result<(), String> {
    let safe = sanitize_component(&filename);
    let target = paths::assets_dir().join(&safe);
    match std::fs::remove_file(&target) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("배경서식 삭제 실패: {}", e)),
    }
}

// ─────────────────────── T3: 레이아웃 저장/로드 ───────────────────────

/// 공지문 레이아웃 저장 — `app_settings.notice_layout` JSON (AC-4.10-3).
#[tauri::command]
pub async fn save_notice_layout(layout: NoticeLayout) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    let json = serde_json::to_string(&layout)
        .map_err(|e| format!("레이아웃 직렬화 실패: {}", e))?;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(KEY_NOTICE_LAYOUT)
    .bind(json)
    .execute(pool)
    .await
    .map_err(|e| format!("레이아웃 저장 실패: {}", e))?;
    Ok(())
}

/// 공지문 레이아웃 조회 — 없으면 기본값(3종 텍스트박스) 반환.
#[tauri::command]
pub async fn get_notice_layout() -> Result<NoticeLayout, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(KEY_NOTICE_LAYOUT)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("레이아웃 조회 실패: {}", e))?;
    match row {
        Some(r) => {
            let json: String = r.try_get("value").map_err(|e| e.to_string())?;
            // 형식 불일치(구버전 레이아웃 등)는 에러 대신 기본값으로 안전 복귀.
            Ok(serde_json::from_str(&json).unwrap_or_else(|_| NoticeLayout::default_layout()))
        }
        None => Ok(NoticeLayout::default_layout()),
    }
}

// ─── 이름 있는 공지문 레이아웃(템플릿) 저장/조회 ───

fn named_layout_key(name: &str) -> String {
    format!("notice_layout::{}", name)
}

async fn read_layout_names(pool: &SqlitePool) -> Result<Vec<String>, String> {
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(KEY_NOTICE_LAYOUT_NAMES)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("레이아웃 목록 조회 실패: {}", e))?;
    match row {
        Some(r) => {
            let json: String = r.try_get("value").map_err(|e| e.to_string())?;
            Ok(serde_json::from_str(&json).unwrap_or_default())
        }
        None => Ok(vec![]),
    }
}

async fn upsert_setting(pool: &SqlitePool, key: &str, value: &str) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| format!("설정 저장 실패: {}", e))?;
    Ok(())
}

/// 저장된 공지문 템플릿 이름 목록 (가나다순).
#[tauri::command]
pub async fn list_notice_layouts() -> Result<Vec<String>, String> {
    let pool = db::pool().map_err(String::from)?;
    read_layout_names(pool).await
}

/// 현재 레이아웃을 이름을 붙여 템플릿으로 저장 (다른 이름으로 저장).
#[tauri::command]
pub async fn save_notice_layout_named(name: String, layout: NoticeLayout) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("템플릿 이름을 입력해 주세요.".to_string());
    }
    if name.contains("::") {
        return Err("템플릿 이름에 '::' 는 사용할 수 없습니다.".to_string());
    }
    let pool = db::pool().map_err(String::from)?;
    let layout_json =
        serde_json::to_string(&layout).map_err(|e| format!("레이아웃 직렬화 실패: {}", e))?;
    upsert_setting(pool, &named_layout_key(&name), &layout_json).await?;

    let mut names = read_layout_names(pool).await?;
    if !names.contains(&name) {
        names.push(name);
        names.sort();
        let names_json =
            serde_json::to_string(&names).map_err(|e| format!("목록 직렬화 실패: {}", e))?;
        upsert_setting(pool, KEY_NOTICE_LAYOUT_NAMES, &names_json).await?;
    }
    Ok(())
}

/// 이름으로 저장된 템플릿 조회 — 없으면 기본값.
#[tauri::command]
pub async fn get_notice_layout_named(name: String) -> Result<NoticeLayout, String> {
    let pool = db::pool().map_err(String::from)?;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(named_layout_key(&name))
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("템플릿 조회 실패: {}", e))?;
    match row {
        Some(r) => {
            let json: String = r.try_get("value").map_err(|e| e.to_string())?;
            Ok(serde_json::from_str(&json).unwrap_or_else(|_| NoticeLayout::default_layout()))
        }
        None => Ok(NoticeLayout::default_layout()),
    }
}

/// 이름 템플릿 삭제.
#[tauri::command]
pub async fn delete_notice_layout_named(name: String) -> Result<(), String> {
    let pool = db::pool().map_err(String::from)?;
    sqlx::query("DELETE FROM app_settings WHERE key = ?")
        .bind(named_layout_key(&name))
        .execute(pool)
        .await
        .map_err(|e| format!("템플릿 삭제 실패: {}", e))?;
    let names: Vec<String> = read_layout_names(pool)
        .await?
        .into_iter()
        .filter(|n| n != &name)
        .collect();
    let names_json = serde_json::to_string(&names).map_err(|e| format!("목록 직렬화 실패: {}", e))?;
    upsert_setting(pool, KEY_NOTICE_LAYOUT_NAMES, &names_json).await?;
    Ok(())
}

// ─── 월 정보(교습기간·보강데이) 텍스트 ───

/// 청구년월의 교습기간·보강데이 표기 텍스트 (공지문 데이터 필드용).
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NoticeMonthInfo {
    /// 'M월D일(요일)~ M월D일(요일)'. 교습기간 없으면 None.
    pub teaching_period_text: Option<String>,
    /// 'D(요일) 10시~13시'(다건은 ', ' 연결). 보강데이 없으면 None.
    pub makeup_day_text: Option<String>,
}

fn weekday_ko(d: NaiveDate) -> &'static str {
    match d.weekday() {
        Weekday::Mon => "월",
        Weekday::Tue => "화",
        Weekday::Wed => "수",
        Weekday::Thu => "목",
        Weekday::Fri => "금",
        Weekday::Sat => "토",
        Weekday::Sun => "일",
    }
}

/// 'YYYY-MM-DD' → 'M월D일(요일)'.
fn fmt_month_day(date_str: &str) -> Option<String> {
    let d = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
    Some(format!("{}월{}일({})", d.month(), d.day(), weekday_ko(d)))
}

/// 'YYYY-MM' 형식 가벼운 검증.
fn validate_year_month_loose(ym: &str) -> Result<(), String> {
    let ok = ym.len() == 7
        && &ym[4..5] == "-"
        && ym[0..4].bytes().all(|b| b.is_ascii_digit())
        && ym[5..7].bytes().all(|b| b.is_ascii_digit());
    if ok {
        Ok(())
    } else {
        Err("청구년월 형식이 올바르지 않습니다 (YYYY-MM).".to_string())
    }
}

#[tauri::command]
pub async fn get_notice_month_info(year_month: String) -> Result<NoticeMonthInfo, String> {
    validate_year_month_loose(&year_month)?;
    let pool = db::pool().map_err(String::from)?;

    // 교습기간
    let sp = sqlx::query("SELECT start_date, end_date FROM study_periods WHERE year_month = ?")
        .bind(&year_month)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("교습기간 조회 실패: {}", e))?;
    let teaching_period_text = sp.and_then(|r| {
        let start: String = r.try_get("start_date").ok()?;
        let end: String = r.try_get("end_date").ok()?;
        Some(format!("{}~ {}", fmt_month_day(&start)?, fmt_month_day(&end)?))
    });

    // 보강데이 (해당 월의 '보강데이' schedule_events)
    let rows = sqlx::query(
        "SELECT e.event_date FROM schedule_events e \
         JOIN schedule_codes c ON c.id = e.code_id \
         WHERE c.code_name = '보강데이' AND e.event_date LIKE ? \
         ORDER BY e.event_date",
    )
    .bind(format!("{}-%", year_month))
    .fetch_all(pool)
    .await
    .map_err(|e| format!("보강데이 조회 실패: {}", e))?;
    let parts: Vec<String> = rows
        .iter()
        .filter_map(|r| {
            let date: String = r.try_get("event_date").ok()?;
            let d = NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok()?;
            Some(format!("{}({}) 10시~13시", d.day(), weekday_ko(d)))
        })
        .collect();
    let makeup_day_text = if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    };

    Ok(NoticeMonthInfo {
        teaching_period_text,
        makeup_day_text,
    })
}

// ─────────────────────── T4: 공지문 이미지 저장 ───────────────────────

fn notice_image_path(year_month: &str, student_name: &str) -> PathBuf {
    let compact: String = year_month.chars().filter(|c| *c != '-').collect();
    let safe_name = sanitize_component(student_name);
    paths::notice_output_dir(year_month).join(format!("{}_{}.png", compact, safe_name))
}

/// 단건 공지문 PNG 저장 — `output/{YYYYMM}/{YYYYMM}_{원생명}.png`. 저장 경로 반환.
#[tauri::command]
pub async fn save_notice_image(
    year_month: String,
    student_name: String,
    image: Vec<u8>,
) -> Result<String, String> {
    let dir = paths::notice_output_dir(&year_month);
    ensure_dir(&dir)?;
    let target = notice_image_path(&year_month, &student_name);
    std::fs::write(&target, &image).map_err(|e| format!("공지문 이미지 저장 실패: {}", e))?;
    Ok(target.to_string_lossy().to_string())
}

/// 다건 공지문 PNG 일괄 저장 — 저장 완료 건수 반환.
#[tauri::command]
pub async fn save_notice_images_batch(
    year_month: String,
    items: Vec<NoticeImageItem>,
) -> Result<i64, String> {
    let dir = paths::notice_output_dir(&year_month);
    ensure_dir(&dir)?;
    let mut saved = 0i64;
    for item in &items {
        let target = notice_image_path(&year_month, &item.student_name);
        std::fs::write(&target, &item.image)
            .map_err(|e| format!("공지문 이미지 저장 실패 ({}): {}", item.student_name, e))?;
        saved += 1;
    }
    Ok(saved)
}

/// 해당 월 출력 폴더에 PNG가 이미 존재하는지 — 덮어쓰기 확인용 (AC-4.10-2).
#[tauri::command]
pub async fn check_notice_output_exists(year_month: String) -> Result<bool, String> {
    let dir = paths::notice_output_dir(&year_month);
    if !dir.exists() {
        return Ok(false);
    }
    let mut entries = std::fs::read_dir(&dir).map_err(|e| format!("출력 폴더 조회 실패: {}", e))?;
    Ok(entries.any(|e| {
        e.ok()
            .map(|e| e.path().extension().and_then(|x| x.to_str()) == Some("png"))
            .unwrap_or(false)
    }))
}

// ─────────────────────── 테스트 ───────────────────────

#[cfg(all(test, not(feature = "cipher")))]
mod tests {
    use super::*;

    /// 테스트별 격리된 data_root 설정 (paths storage 가 thread_local).
    fn use_temp_root(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("smarthb-notice-test-{}", tag));
        let _ = std::fs::remove_dir_all(&root);
        paths::update_data_root(root.clone());
        root
    }

    #[test]
    fn sanitize_blocks_traversal_and_separators() {
        // 경로 traversal·구분자가 결과에 남지 않아야 한다 (정확한 치환 문자열보다 안전성 속성 검증).
        let s = sanitize_component("../../etc/passwd");
        assert!(!s.contains(".."), "traversal 잔존: {}", s);
        assert!(!s.contains('/'), "구분자 잔존: {}", s);
        assert_eq!(sanitize_component("홍 길동"), "홍_길동");
        assert_eq!(sanitize_component("a/b\\c"), "a_b_c");
        assert_eq!(sanitize_component("   "), "unnamed");
    }

    #[tokio::test]
    async fn save_list_delete_asset_roundtrip() {
        let _root = use_temp_root("asset");
        // 저장
        let saved = save_notice_asset("배경.png".to_string(), vec![1, 2, 3]).await.expect("save");
        assert_eq!(saved, "배경.png");
        // 목록
        let list = list_notice_assets().await.expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "배경.png");
        assert_eq!(list[0].size, 3);
        // 확장자 거부
        assert!(save_notice_asset("evil.exe".to_string(), vec![0]).await.is_err());
        // 삭제
        delete_notice_asset("배경.png".to_string()).await.expect("delete");
        assert_eq!(list_notice_assets().await.expect("list2").len(), 0);
        // 없는 파일 삭제는 성공(멱등)
        delete_notice_asset("배경.png".to_string()).await.expect("idempotent");
    }

    #[tokio::test]
    async fn save_notice_image_writes_expected_path() {
        let root = use_temp_root("image");
        let path = save_notice_image("2026-05".to_string(), "홍 길동".to_string(), vec![9, 9])
            .await
            .expect("save img");
        assert!(path.ends_with("202605_홍_길동.png"), "경로: {}", path);
        assert!(root.join("output/202605/202605_홍_길동.png").exists());
        // 기존 파일 확인
        assert!(check_notice_output_exists("2026-05".to_string()).await.expect("exists"));
        assert!(!check_notice_output_exists("2026-06".to_string()).await.expect("none"));
    }

    #[tokio::test]
    async fn batch_save_returns_count() {
        let _root = use_temp_root("batch");
        let items = vec![
            NoticeImageItem { student_name: "원생A".to_string(), image: vec![1] },
            NoticeImageItem { student_name: "원생B".to_string(), image: vec![2] },
        ];
        let n = save_notice_images_batch("2026-05".to_string(), items).await.expect("batch");
        assert_eq!(n, 2);
    }
}
