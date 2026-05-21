//! 초기 설정 마법사 IPC (Sprint 3 T8, PRD §4.0).
//!
//! ## 설계 — chicken-and-egg 회피
//!
//! 사용자 클라우드 폴더 경로는 **OS app_config_dir 의 `config.json`** 에 저장한다.
//! DB(SQLCipher) 자체가 클라우드 폴더 안에 위치할 예정이라 `paths::data_root()` 동적화를
//! 위해서는 DB 열기 전(unlock 전)에 경로를 알 수 있어야 한다.
//!
//! `app_settings.cloud_folder_path` 는 보조 메타데이터로 유지(V200 시드) — 양 PC 간 같은
//! 클라우드 폴더 공유 사실을 DB 안에서도 확인할 수 있도록.
//!
//! ## R12 salt 이전 / paths::data_root() 동적화
//!
//! 본 모듈은 마법사의 **IPC 인터페이스 + config.json 영속화** 까지만 담당한다.
//! Keychain salt → `{cloud}/smarthb/salt.bin` 이전과 `paths::data_root()` 동적화는 별도
//! sweep(후속 sprint 또는 본 sprint 의 별도 Task)으로 분리한다 — 이 두 작업은 auth/recovery/
//! backup/integrity 모듈에 광범위한 영향을 미치므로 마법사 UI 구현과 동시 진행 시 변경량
//! 폭증 위험이 높다.

use crate::commands::paths;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// 마법사 진행 상태 — config.json 의 직렬화 표현.
///
/// `#[serde(default)]` 로 후방 호환 보장 — 신규 필드 추가 시 기존 config.json 파일이 그대로
/// 로드되며 누락 필드는 `Default` 값으로 채워진다.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SetupStatus {
    /// 클라우드 동기화 폴더 절대 경로. 미선택 상태는 빈 문자열.
    pub cloud_folder_path: String,
    /// 마법사 완료(complete_setup IPC 호출) 여부.
    pub setup_completed: bool,
}

/// app_config_dir 하위 SmartHB 디렉토리의 `config.json` 경로를 반환한다.
///
/// 디렉토리 생성은 write_status 직전에만 수행 — read 만 하는 경로(get_setup_status)에서는
/// 파일이 없을 때 `NotFound` 분기로 자연스럽게 처리된다.
fn config_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| AppError::Config(format!("config dir 조회 실패: {}", e)))?;
    Ok(dir.join("config.json"))
}

/// config.json 을 읽어 SetupStatus 를 반환. 파일이 없으면 기본값(`Default::default()`).
fn read_status(app: &AppHandle) -> Result<SetupStatus, AppError> {
    let path = config_path(app)?;
    match fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s)
            .map_err(|e| AppError::Config(format!("config.json 파싱 실패: {}", e))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(SetupStatus::default()),
        Err(e) => Err(AppError::Config(format!("config.json 읽기 실패: {}", e))),
    }
}

/// config.json 을 atomic 하게 갱신한다 (tmp → rename).
fn write_status(app: &AppHandle, status: &SetupStatus) -> Result<(), AppError> {
    let path = config_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::Config(format!("config dir 생성 실패: {}", e)))?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(status)
        .map_err(|e| AppError::Config(format!("config.json 직렬화 실패: {}", e)))?;
    fs::write(&tmp, json)
        .map_err(|e| AppError::Config(format!("config.json 쓰기 실패: {}", e)))?;
    fs::rename(&tmp, &path)
        .map_err(|e| AppError::Config(format!("config.json rename 실패: {}", e)))?;
    Ok(())
}

// ----------------------------------------------------------------------------
// Tauri IPC commands
// ----------------------------------------------------------------------------

/// 사용자가 선택한 클라우드 동기화 폴더를 저장한다.
///
/// `{path}/smarthb/` 디렉토리를 생성하고 config.json 에 경로를 기록.
/// DB 는 unlock 시점에 별도로 열린다 — 본 IPC 는 DB 와 무관.
#[tauri::command]
pub async fn save_cloud_folder(app: AppHandle, path: String) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(String::from(AppError::UserFacing(
            "폴더 경로가 비어 있습니다.".to_string(),
        )));
    }
    let smarthb_dir = PathBuf::from(trimmed).join("smarthb");
    fs::create_dir_all(&smarthb_dir).map_err(|e| {
        String::from(AppError::UserFacing(format!(
            "선택한 폴더에 smarthb 디렉토리를 만들 수 없습니다: {}",
            e
        )))
    })?;

    let mut status = read_status(&app).map_err(String::from)?;
    status.cloud_folder_path = trimmed.to_string();
    write_status(&app, &status).map_err(String::from)?;
    // R20: paths 모듈에 즉시 반영 — 다음 단계(비밀번호 설정 + DB pool 초기화)부터 새 경로 사용.
    paths::update_data_root(smarthb_dir);
    Ok(())
}

/// 마법사 완료를 표시한다. 모든 단계 완료 후 호출.
#[tauri::command]
pub async fn complete_setup(app: AppHandle) -> Result<(), String> {
    let mut status = read_status(&app).map_err(String::from)?;
    if status.cloud_folder_path.is_empty() {
        return Err(String::from(AppError::UserFacing(
            "클라우드 폴더가 선택되지 않았습니다.".to_string(),
        )));
    }
    status.setup_completed = true;
    write_status(&app, &status).map_err(String::from)?;
    Ok(())
}

/// 마법사 진행 상태를 조회한다. 미진입 시 기본값(빈 경로 + setup_completed=false) 반환.
#[tauri::command]
pub async fn get_setup_status(app: AppHandle) -> Result<SetupStatus, String> {
    read_status(&app).map_err(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_status_default_is_empty_and_not_completed() {
        let s = SetupStatus::default();
        assert_eq!(s.cloud_folder_path, "");
        assert!(!s.setup_completed);
    }

    #[test]
    fn setup_status_serde_round_trip() {
        let s = SetupStatus {
            cloud_folder_path: "/Users/dev/MYBOX".to_string(),
            setup_completed: true,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SetupStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cloud_folder_path, s.cloud_folder_path);
        assert_eq!(back.setup_completed, s.setup_completed);
    }

    #[test]
    fn setup_status_parses_when_field_missing() {
        // 후방 호환: 신규 필드 추가 시 기존 config.json 이 깨지지 않아야 한다.
        let json = r#"{"cloud_folder_path":"/x"}"#;
        let s: SetupStatus = serde_json::from_str(json).unwrap();
        assert_eq!(s.cloud_folder_path, "/x");
        assert!(!s.setup_completed, "기본값 false 적용");
    }
}
