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
//! sweep(후속 sprint 또는 본 sprint 의 별도 Task)으로 분리한다 — 이 두 작업은 auth/
//! backup/integrity 모듈에 광범위한 영향을 미치므로 마법사 UI 구현과 동시 진행 시 변경량
//! 폭증 위험이 높다.

use crate::commands::paths;
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
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
    /// 실행 시 PIN 인증 스킵 여부 (ADR-008). true 면 키체인의 기존 유도키로 무입력 잠금해제.
    /// **PC별 로컬 설정** — 클라우드 동기화 대상 아님(app_config_dir 의 config.json 에만 저장).
    /// 기본 false (PIN 인증 ON 유지). 후방 호환: 기존 config.json 에 필드 없으면 false.
    pub skip_pin_on_launch: bool,
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

/// config.json 을 읽어 SetupStatus 를 반환. 파일이 없거나 손상돼 있으면 기본값.
///
/// **손상 복구 정책 (2026-05-21 사고 대응)**: PC 강제 종료 시 NTFS power-loss 패턴으로
/// `fs::write` + `fs::rename` 도중 메타데이터만 커밋되고 데이터 페이지가 NULL 로 남는 경우가
/// 있다 (실측: 90 바이트 전체 0x00). 이 상태를 그대로 두면 사용자는 "설정 정보를 불러오는 중
/// 오류" 만 보고 앱을 사용할 수 없게 된다. 본 함수는 손상을 감지하면 손상본을
/// `config.json.corrupted-{unix_ts}` 로 백업한 뒤 기본값으로 fallback 한다 — 마법사가 처음부터
/// 다시 진행되어 자동 복구된다. 백업 실패는 fatal 이 아니다 (best-effort).
fn read_status(app: &AppHandle) -> Result<SetupStatus, AppError> {
    let path = config_path(app)?;
    Ok(read_status_from_path(&path))
}

/// 테스트 가능한 핵심 로직. AppHandle 없이 경로만으로 동작한다.
fn read_status_from_path(path: &Path) -> SetupStatus {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return SetupStatus::default(),
        Err(e) => {
            eprintln!("[setup] config.json 읽기 실패 (default 로 fallback): {}", e);
            return SetupStatus::default();
        }
    };
    if is_corrupted(&bytes) {
        eprintln!(
            "[setup] config.json 손상 감지 ({} 바이트, all-zero 또는 빈 파일). 백업 후 default 로 fallback.",
            bytes.len()
        );
        backup_corrupted(path);
        return SetupStatus::default();
    }
    match serde_json::from_slice::<SetupStatus>(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[setup] config.json 파싱 실패 (손상 의심, 백업 후 default 로 fallback): {}",
                e
            );
            backup_corrupted(path);
            SetupStatus::default()
        }
    }
}

/// 빈 파일 또는 NULL 바이트만 있는 파일을 손상으로 간주한다. JSON 파싱 단계 전 빠른 컷.
fn is_corrupted(bytes: &[u8]) -> bool {
    bytes.is_empty() || bytes.iter().all(|&b| b == 0)
}

/// 손상본을 `config.json.corrupted-{unix_ts}` 로 rename. 실패는 무시 (best-effort).
fn backup_corrupted(path: &Path) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = path.with_extension(format!("json.corrupted-{}", ts));
    if let Err(e) = fs::rename(path, &backup) {
        eprintln!("[setup] 손상본 백업 실패 ({}): {}", backup.display(), e);
    } else {
        eprintln!("[setup] 손상본 백업 완료: {}", backup.display());
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
    fs::write(&tmp, json).map_err(|e| AppError::Config(format!("config.json 쓰기 실패: {}", e)))?;
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

/// 실행 시 PIN 인증 스킵 설정 조회 (ADR-008). DB 접근 없이 config.json 만 읽으므로 unlock 전 호출 가능.
#[tauri::command]
pub async fn get_pin_skip_setting(app: AppHandle) -> Result<bool, String> {
    Ok(read_status(&app).map_err(String::from)?.skip_pin_on_launch)
}

/// 실행 시 PIN 인증 스킵 설정 저장 (ADR-008). PC별 로컬(config.json). unlock 전 호출 가능.
#[tauri::command]
pub async fn set_pin_skip_setting(app: AppHandle, skip: bool) -> Result<(), String> {
    let mut status = read_status(&app).map_err(String::from)?;
    status.skip_pin_on_launch = skip;
    write_status(&app, &status).map_err(String::from)
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
            skip_pin_on_launch: false,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: SetupStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cloud_folder_path, s.cloud_folder_path);
        assert_eq!(back.setup_completed, s.setup_completed);
        assert_eq!(back.skip_pin_on_launch, s.skip_pin_on_launch);
    }

    #[test]
    fn setup_status_parses_when_field_missing() {
        // 후방 호환: 신규 필드 추가 시 기존 config.json 이 깨지지 않아야 한다.
        let json = r#"{"cloud_folder_path":"/x"}"#;
        let s: SetupStatus = serde_json::from_str(json).unwrap();
        assert_eq!(s.cloud_folder_path, "/x");
        assert!(!s.setup_completed, "기본값 false 적용");
    }

    #[test]
    fn skip_pin_default_false_and_backward_compatible() {
        // 기본값 false (PIN 인증 ON).
        assert!(!SetupStatus::default().skip_pin_on_launch);
        // 기존 config.json(필드 없음) → false 로 후방 호환.
        let legacy = r#"{"cloud_folder_path":"/c","setup_completed":true}"#;
        let s: SetupStatus = serde_json::from_str(legacy).unwrap();
        assert!(!s.skip_pin_on_launch, "필드 누락 시 false");
    }

    #[test]
    fn skip_pin_persists_via_read_status_from_path() {
        // set 후 get 일치(파일 라운드트립). write_status 는 AppHandle 필요하므로 직접 직렬화 기록.
        let dir = unique_tmp_dir("skip-pin");
        let path = dir.join("config.json");
        let s = SetupStatus {
            cloud_folder_path: "/c".to_string(),
            setup_completed: true,
            skip_pin_on_launch: true,
        };
        fs::write(&path, serde_json::to_string_pretty(&s).unwrap()).unwrap();
        let back = read_status_from_path(&path);
        assert!(back.skip_pin_on_launch, "저장된 true 가 그대로 로드");
        let _ = fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------------------------------
    // 손상 복구 (2026-05-21 사고 대응) — read_status_from_path fallback 검증.
    // ------------------------------------------------------------------------

    /// 테스트 간 격리를 위한 고유 임시 디렉토리. tempfile crate 도입 회피 — std 만 사용.
    fn unique_tmp_dir(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("smarthb-setup-test-{}-{}", label, ts));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn read_status_returns_default_when_file_missing() {
        let dir = unique_tmp_dir("missing");
        let path = dir.join("config.json");
        let s = read_status_from_path(&path);
        assert_eq!(s.cloud_folder_path, "");
        assert!(!s.setup_completed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_status_returns_default_when_file_is_all_null_bytes() {
        // 실제 사고 재현: 90 바이트 전체 0x00 (PC 다운으로 rename 직후 데이터 페이지 손실).
        let dir = unique_tmp_dir("null");
        let path = dir.join("config.json");
        fs::write(&path, [0u8; 90]).unwrap();
        let s = read_status_from_path(&path);
        assert_eq!(s.cloud_folder_path, "", "손상 fallback default 반환");
        assert!(!s.setup_completed);
        assert!(
            !path.exists(),
            "손상본은 백업으로 rename 되어 원본 경로엔 없어야 함"
        );
        let backed_up = fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("config.json.corrupted-")
        });
        assert!(
            backed_up,
            "config.json.corrupted-* 백업 파일이 생성되어야 함"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_status_returns_default_when_file_is_empty() {
        let dir = unique_tmp_dir("empty");
        let path = dir.join("config.json");
        fs::write(&path, b"").unwrap();
        let s = read_status_from_path(&path);
        assert_eq!(s.cloud_folder_path, "");
        assert!(!s.setup_completed);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_status_returns_default_when_json_is_malformed() {
        let dir = unique_tmp_dir("malformed");
        let path = dir.join("config.json");
        fs::write(&path, b"{not valid json").unwrap();
        let s = read_status_from_path(&path);
        assert_eq!(s.cloud_folder_path, "");
        assert!(!s.setup_completed);
        assert!(!path.exists(), "파싱 실패도 백업 후 fallback");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_status_returns_parsed_when_valid_json() {
        let dir = unique_tmp_dir("valid");
        let path = dir.join("config.json");
        fs::write(
            &path,
            br#"{"cloud_folder_path":"/cloud/smarthb","setup_completed":true}"#,
        )
        .unwrap();
        let s = read_status_from_path(&path);
        assert_eq!(s.cloud_folder_path, "/cloud/smarthb");
        assert!(s.setup_completed);
        assert!(path.exists(), "정상 파일은 그대로 유지");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn is_corrupted_detects_all_zero_and_empty() {
        assert!(is_corrupted(&[]));
        assert!(is_corrupted(&[0u8; 1]));
        assert!(is_corrupted(&[0u8; 90]));
        assert!(!is_corrupted(b"{}"));
        assert!(
            !is_corrupted(b"\0valid"),
            "선두 NULL 만 있으면 손상 아님 (파싱 단계에 위임)"
        );
    }
}
