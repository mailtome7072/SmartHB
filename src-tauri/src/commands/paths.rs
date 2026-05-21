//! 앱 데이터 경로 + SQLCipher PRAGMA 단편 (T11 통합 + R20 hotfix).
//!
//! Sprint 3 sprint-review R20 해소 (2026-05-21):
//! `data_root()` 가 config.json 의 `cloud_folder_path` 를 참조하도록 동적화. backup·integrity·
//! lock·sync·startup 호출부는 동일 API 를 그대로 사용한다.
//!
//! ## 동기화 정책
//!
//! - 앱 시작 시 `lib.rs::setup` hook 이 [`init_data_root_from_config`] 를 호출하여 OS
//!   app_config_dir 의 config.json 을 읽고 내부 상태를 초기화한다.
//! - 마법사가 폴더를 선택하면 `setup::save_cloud_folder` 가 [`update_data_root`] 를 호출하여
//!   런타임 중에도 즉시 갱신된다. SQLite pool 은 아직 미초기화 상태이므로 race 없음.
//! - cloud_folder_path 가 비어 있으면 fallback `./SmartHB-data` (개발 편의).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[cfg(not(feature = "cipher"))]
#[allow(dead_code)]
const _CIPHER_MARKER: () = ();

const FALLBACK_DEV_ROOT: &str = "./SmartHB-data";
const SMARTHB_SUBDIR: &str = "smarthb";
const DB_FILENAME: &str = "app.db";

static DATA_ROOT: OnceLock<Mutex<PathBuf>> = OnceLock::new();

fn root_cell() -> &'static Mutex<PathBuf> {
    DATA_ROOT.get_or_init(|| Mutex::new(PathBuf::from(FALLBACK_DEV_ROOT)))
}

/// 앱 데이터 루트 디렉토리 — backup·integrity·lock·sync·startup 모듈 공유 단일 진입점.
pub(crate) fn data_root() -> PathBuf {
    root_cell()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_else(|_| PathBuf::from(FALLBACK_DEV_ROOT))
}

/// 소스 DB 파일 경로 — startup·integrity·sync 가 검증·복원·mtime 감시에 공유.
pub(crate) fn db_path() -> PathBuf {
    data_root().join(DB_FILENAME)
}

/// 데이터 루트를 런타임 중 갱신한다. 마법사가 폴더를 새로 지정할 때 호출.
///
/// SQLite pool 이 이미 초기화된 후에 호출하면 pool 은 옛 경로를 계속 사용한다 — 마법사
/// 흐름에서만 호출되어 unlock 이전임이 보장된다.
pub(crate) fn update_data_root(new_path: PathBuf) {
    if let Ok(mut guard) = root_cell().lock() {
        *guard = new_path;
    }
}

/// 앱 시작 시 1회 호출 — config.json 의 cloud_folder_path 가 있으면 그 하위 `smarthb/` 로
/// data root 를 설정. 없으면 fallback 유지.
pub(crate) fn init_data_root_from_config(config_path: &std::path::Path) {
    let Ok(json) = std::fs::read_to_string(config_path) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json) else {
        return;
    };
    let Some(path) = parsed
        .get("cloud_folder_path")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    update_data_root(PathBuf::from(path).join(SMARTHB_SUBDIR));
}

/// SQLCipher `PRAGMA key` 적용용 SQL 단편.
///
/// hex 인코딩만 허용되므로 `[0-9a-f]` 만 사용 → SQL injection 안전. db / backup / integrity
/// 3 모듈이 동일 형식으로 PRAGMA key 적용이 필요하여 단일 단편 함수로 통합.
#[cfg(feature = "cipher")]
pub(crate) fn pragma_key_sql(hex_key: &str) -> String {
    format!("PRAGMA key = \"x'{}'\";", hex_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 각 테스트 후 default 로 복원 — OnceLock 은 동일 프로세스 전역이라 테스트 간 격리 필요.
    fn reset() {
        update_data_root(PathBuf::from(FALLBACK_DEV_ROOT));
    }

    #[test]
    fn data_root_default_is_fallback() {
        reset();
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
    }

    #[test]
    fn update_data_root_changes_subsequent_reads() {
        update_data_root(PathBuf::from("/tmp/test-root"));
        assert_eq!(data_root(), PathBuf::from("/tmp/test-root"));
        assert!(db_path().starts_with("/tmp/test-root"));
        assert!(db_path().ends_with(DB_FILENAME));
        reset();
    }

    #[test]
    fn init_from_config_uses_cloud_folder_path() {
        let tmp = std::env::temp_dir().join("smarthb-paths-test-config.json");
        std::fs::write(
            &tmp,
            r#"{"cloud_folder_path":"/tmp/my-cloud","setup_completed":false}"#,
        )
        .unwrap();
        init_data_root_from_config(&tmp);
        assert_eq!(data_root(), PathBuf::from("/tmp/my-cloud").join(SMARTHB_SUBDIR));
        std::fs::remove_file(&tmp).ok();
        reset();
    }

    #[test]
    fn init_from_config_ignores_empty_path() {
        reset();
        let tmp = std::env::temp_dir().join("smarthb-paths-test-empty.json");
        std::fs::write(&tmp, r#"{"cloud_folder_path":"","setup_completed":false}"#).unwrap();
        init_data_root_from_config(&tmp);
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn init_from_config_ignores_missing_file() {
        reset();
        let missing = std::env::temp_dir().join("does-not-exist-smarthb-config.json");
        init_data_root_from_config(&missing);
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn pragma_key_sql_uses_blob_literal() {
        let sql = pragma_key_sql("deadbeef");
        assert_eq!(sql, "PRAGMA key = \"x'deadbeef'\";");
        assert!(!sql.contains('\''), "단일 따옴표 SQL 인젝션 통로 차단 (raw 따옴표 없음)");
    }
}
