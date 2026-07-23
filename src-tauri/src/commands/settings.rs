//! 영구 설정 IPC — 교습소 운영 시간 등 (PRD §4.0/§4.12, Sprint 4 T2).
//!
//! 마법사는 `setup.rs` 가 담당 (config.json + DB seed). 본 모듈은 unlock 이후 사용자가
//! 영구 설정 화면에서 변경하는 항목 — `app_settings` key/value (schema-less) 활용.
//! DB 마이그레이션 불필요.
//!
//! ## 운영 시간 데이터 모델
//!
//! - 요일 표현: ISO 8601 weekday (1=월, 2=화, ..., 7=일) — `schedules.rs` 와 일관.
//! - 시간 표현: "HH:MM" 문자열. 미운영 요일은 open/close 모두 None.
//! - 저장 형식: `app_settings.value` 에 JSON 직렬화 `Vec<DayHours>` (7개 요일).

use crate::commands::db::pool;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::Row;

const KEY_OPERATING_HOURS: &str = "operating_hours";

/// 요일별 운영 시간. open/close 가 모두 None 이면 미운영.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DayHours {
    /// ISO 8601 weekday: 1=월, 2=화, 3=수, 4=목, 5=금, 6=토, 7=일
    pub day_of_week: u8,
    /// "HH:MM" 형식 (None = 미운영)
    pub open_time: Option<String>,
    pub close_time: Option<String>,
}

/// PRD §4.0 마법사 디폴트: 월~금 13:00~20:00, 토/일 미운영. (T11 사용자 요청)
fn default_operating_hours() -> Vec<DayHours> {
    (1u8..=7)
        .map(|d| {
            if d <= 5 {
                DayHours {
                    day_of_week: d,
                    open_time: Some("13:00".to_string()),
                    close_time: Some("20:00".to_string()),
                }
            } else {
                DayHours {
                    day_of_week: d,
                    open_time: None,
                    close_time: None,
                }
            }
        })
        .collect()
}

/// 현재 운영 시간 조회. 저장된 값 없으면 디폴트 반환.
#[tauri::command]
pub async fn get_operating_hours() -> Result<Vec<DayHours>, String> {
    let pool = pool().await.map_err(String::from)?;
    let pool = &pool;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(KEY_OPERATING_HOURS)
        .fetch_optional(pool)
        .await
        .map_err(|e| String::from(AppError::Db(e)))?;

    match row {
        Some(r) => {
            let json: String = r
                .try_get("value")
                .map_err(|e| String::from(AppError::Db(e)))?;
            serde_json::from_str(&json).map_err(|e| {
                String::from(AppError::Config(format!(
                    "operating_hours 파싱 실패: {}",
                    e
                )))
            })
        }
        None => Ok(default_operating_hours()),
    }
}

/// 운영 시간 저장. 7개 요일 모두 포함, day_of_week 1-7 유효, open/close 짝 검증.
#[tauri::command]
pub async fn save_operating_hours(hours: Vec<DayHours>) -> Result<(), String> {
    if hours.len() != 7 {
        return Err(String::from(AppError::UserFacing(
            "7개 요일을 모두 입력해주세요.".to_string(),
        )));
    }
    for h in &hours {
        if !(1..=7).contains(&h.day_of_week) {
            return Err(String::from(AppError::UserFacing(format!(
                "잘못된 요일 코드: {} (1=월~7=일)",
                h.day_of_week
            ))));
        }
        if h.open_time.is_some() != h.close_time.is_some() {
            return Err(String::from(AppError::UserFacing(
                "시작/종료 시간은 함께 입력하거나 함께 비워주세요.".to_string(),
            )));
        }
        // HH:MM 형식 간단 검증 (자릿수 + ':')
        for t in [&h.open_time, &h.close_time].into_iter().flatten() {
            if t.len() != 5 || !t.contains(':') {
                return Err(String::from(AppError::UserFacing(format!(
                    "시간 형식이 잘못됐습니다: {} (HH:MM)",
                    t
                ))));
            }
        }
    }

    let pool = pool().await.map_err(String::from)?;
    let pool = &pool;
    let json = serde_json::to_string(&hours).map_err(|e| {
        String::from(AppError::Config(format!(
            "operating_hours 직렬화 실패: {}",
            e
        )))
    })?;

    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(KEY_OPERATING_HOURS)
    .bind(json)
    .execute(pool)
    .await
    .map_err(|e| String::from(AppError::Db(e)))?;

    Ok(())
}

// ─────────────────────── 교습소 정보 (Sprint 15 T1) ───────────────────────
//
// PRD §4.12 교습소 기본 정보. `app_settings.academy_info` 에 JSON 저장(텍스트 필드 +
// 로고/2D바코드 이미지 파일명). 이미지 본체는 `assets/` 에 파일로 저장하고(notice.rs
// `save_notice_asset` 재사용) 여기에는 파일명만 보관 — DB 비대화 방지 + 양 PC 동기화.
// 공지문(§4.10)에는 연동하지 않는 교습소 정보 화면 전용 데이터. DB 마이그레이션 불필요.

const KEY_ACADEMY_INFO: &str = "academy_info";

/// 교습소 기본 정보. 텍스트 필드 + 이미지 파일명(`assets/` 하위).
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct AcademyInfo {
    /// 교습소명 (필수)
    pub academy_name: String,
    /// 대표자(원장명)
    pub representative: String,
    /// 연락처
    pub phone: String,
    /// 주소
    pub address: String,
    /// 사업자등록번호 (선택)
    pub business_number: Option<String>,
    /// 교습 최대인원 수 (선택)
    pub max_capacity: Option<i64>,
    /// 교습소 면적 — 제곱미터(㎡) (선택)
    pub area_sqm: Option<f64>,
    /// 로고 이미지 파일명 — `assets/` 하위 (선택)
    pub logo_filename: Option<String>,
    /// 2D바코드 이미지 파일명 — `assets/` 하위 (선택)
    pub barcode_filename: Option<String>,
}

/// 교습소 정보 조회. 저장된 값 없으면 빈 정보(Default) 반환.
#[tauri::command]
pub async fn get_academy_info() -> Result<AcademyInfo, String> {
    let pool = pool().await.map_err(String::from)?;
    let pool = &pool;
    let row = sqlx::query("SELECT value FROM app_settings WHERE key = ?")
        .bind(KEY_ACADEMY_INFO)
        .fetch_optional(pool)
        .await
        .map_err(|e| String::from(AppError::Db(e)))?;

    match row {
        Some(r) => {
            let json: String = r
                .try_get("value")
                .map_err(|e| String::from(AppError::Db(e)))?;
            serde_json::from_str(&json).map_err(|e| {
                String::from(AppError::Config(format!("academy_info 파싱 실패: {}", e)))
            })
        }
        None => Ok(AcademyInfo::default()),
    }
}

/// 교습소 정보 저장. 교습소명 필수, 최대인원/면적은 음수 불가.
#[tauri::command]
pub async fn save_academy_info(info: AcademyInfo) -> Result<(), String> {
    if info.academy_name.trim().is_empty() {
        return Err(String::from(AppError::UserFacing(
            "교습소명을 입력해주세요.".to_string(),
        )));
    }
    if matches!(info.max_capacity, Some(c) if c < 0) {
        return Err(String::from(AppError::UserFacing(
            "교습 최대인원 수는 0 이상이어야 합니다.".to_string(),
        )));
    }
    if matches!(info.area_sqm, Some(a) if a < 0.0) {
        return Err(String::from(AppError::UserFacing(
            "교습소 면적은 0 이상이어야 합니다.".to_string(),
        )));
    }

    let pool = pool().await.map_err(String::from)?;
    let pool = &pool;
    let json = serde_json::to_string(&info).map_err(|e| {
        String::from(AppError::Config(format!("academy_info 직렬화 실패: {}", e)))
    })?;

    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, \
         updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')",
    )
    .bind(KEY_ACADEMY_INFO)
    .bind(json)
    .execute(pool)
    .await
    .map_err(|e| String::from(AppError::Db(e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_seven_days() {
        let h = default_operating_hours();
        assert_eq!(h.len(), 7);
        for (idx, d) in h.iter().enumerate() {
            assert_eq!(d.day_of_week as usize, idx + 1);
        }
    }

    #[test]
    fn default_weekdays_open_13_to_20() {
        let h = default_operating_hours();
        for d in &h[..5] {
            assert_eq!(d.open_time.as_deref(), Some("13:00"));
            assert_eq!(d.close_time.as_deref(), Some("20:00"));
        }
    }

    #[test]
    fn default_weekend_is_closed() {
        let h = default_operating_hours();
        for d in &h[5..] {
            assert!(d.open_time.is_none());
            assert!(d.close_time.is_none());
        }
    }

    #[test]
    fn day_hours_serde_roundtrip_open() {
        let h = DayHours {
            day_of_week: 3,
            open_time: Some("14:00".to_string()),
            close_time: Some("20:00".to_string()),
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: DayHours = serde_json::from_str(&json).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn day_hours_serde_roundtrip_closed() {
        let h = DayHours {
            day_of_week: 7,
            open_time: None,
            close_time: None,
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: DayHours = serde_json::from_str(&json).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn vec_of_seven_serializes_compactly() {
        // 저장 형식 검증 — JSON 배열 7원소.
        let default = default_operating_hours();
        let json = serde_json::to_string(&default).unwrap();
        let back: Vec<DayHours> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.len(), 7);
        assert_eq!(back, default);
    }

    #[test]
    fn academy_info_default_is_empty() {
        let info = AcademyInfo::default();
        assert!(info.academy_name.is_empty());
        assert!(info.business_number.is_none());
        assert!(info.logo_filename.is_none());
    }

    #[test]
    fn academy_info_serde_roundtrip_full_and_optional() {
        let full = AcademyInfo {
            academy_name: "스마트해법수학 서현효자점".to_string(),
            representative: "홍길동".to_string(),
            phone: "031-000-0000".to_string(),
            address: "성남시 분당구".to_string(),
            business_number: Some("123-45-67890".to_string()),
            max_capacity: Some(30),
            area_sqm: Some(82.5),
            logo_filename: Some("academy_logo.png".to_string()),
            barcode_filename: Some("academy_barcode.png".to_string()),
        };
        let back: AcademyInfo =
            serde_json::from_str(&serde_json::to_string(&full).unwrap()).unwrap();
        assert_eq!(back, full);

        // 선택 필드 None 도 라운드트립 보존.
        let minimal = AcademyInfo {
            academy_name: "교습소".to_string(),
            ..Default::default()
        };
        let back2: AcademyInfo =
            serde_json::from_str(&serde_json::to_string(&minimal).unwrap()).unwrap();
        assert_eq!(back2, minimal);
    }
}
