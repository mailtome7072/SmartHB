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

#[cfg(not(feature = "cipher"))]
#[allow(dead_code)]
const _CIPHER_MARKER: () = ();

const FALLBACK_DEV_ROOT: &str = "./SmartHB-data";
const SMARTHB_SUBDIR: &str = "smarthb";
const DB_FILENAME: &str = "app.db";
const SALT_FILENAME: &str = "salt.bin";
const ASSETS_SUBDIR: &str = "assets";
const OUTPUT_SUBDIR: &str = "output";

// 프로덕션: 프로세스 전역 OnceLock<Mutex<PathBuf>>. setup::save_cloud_folder 와
// lib.rs::setup 두 곳에서만 update_data_root 를 호출하며, 모두 unlock 이전 단일 thread.
//
// 테스트: thread_local RefCell. cargo test 가 테스트별로 별도 thread 를 띄우므로 각 테스트가
// 독립된 DATA_ROOT 를 보유 → reset 헬퍼 없이 병렬 실행 안전 (A21 해소).
#[cfg(not(test))]
mod storage {
    use super::{FALLBACK_DEV_ROOT, PathBuf};
    use std::sync::{Mutex, OnceLock};

    static DATA_ROOT: OnceLock<Mutex<PathBuf>> = OnceLock::new();

    fn cell() -> &'static Mutex<PathBuf> {
        DATA_ROOT.get_or_init(|| Mutex::new(PathBuf::from(FALLBACK_DEV_ROOT)))
    }

    pub(super) fn read() -> PathBuf {
        cell()
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| PathBuf::from(FALLBACK_DEV_ROOT))
    }

    pub(super) fn write(new_path: PathBuf) {
        if let Ok(mut guard) = cell().lock() {
            *guard = new_path;
        }
    }
}

#[cfg(test)]
mod storage {
    use super::{FALLBACK_DEV_ROOT, PathBuf};
    use std::cell::RefCell;

    thread_local! {
        static DATA_ROOT: RefCell<PathBuf> = RefCell::new(PathBuf::from(FALLBACK_DEV_ROOT));
    }

    pub(super) fn read() -> PathBuf {
        DATA_ROOT.with(|c| c.borrow().clone())
    }

    pub(super) fn write(new_path: PathBuf) {
        DATA_ROOT.with(|c| *c.borrow_mut() = new_path);
    }
}

/// 앱 데이터 루트 디렉토리 — backup·integrity·lock·sync·startup 모듈 공유 단일 진입점.
pub(crate) fn data_root() -> PathBuf {
    storage::read()
}

/// 소스 DB 파일 경로 — startup·integrity·sync 가 검증·복원·mtime 감시에 공유.
pub(crate) fn db_path() -> PathBuf {
    data_root().join(DB_FILENAME)
}

/// PBKDF2 salt 파일 경로 — 클라우드 동기화 폴더에 평문 32바이트 저장 (Sprint 7 T2, A17/A27).
///
/// salt 는 비밀이 아니므로 Keychain 대신 cloud sync 폴더에 두어 양 PC 자동 동기화.
/// 첫 설치 시 `auth::set_password` 가 생성, 기존 Keychain salt 는 1회 마이그레이션.
pub(crate) fn salt_path() -> PathBuf {
    data_root().join(SALT_FILENAME)
}

/// 공지문 배경서식 디렉토리 — `{data_root}/assets/` (Sprint 12, PRD §4.10).
/// 양 PC 공유를 위해 클라우드 동기화 폴더 하위에 둔다. 실제 생성은 호출부(notice IPC)에서 `create_dir_all`.
pub(crate) fn assets_dir() -> PathBuf {
    data_root().join(ASSETS_SUBDIR)
}

/// 공지문 PNG 출력 루트 — `{data_root}/output/` (Sprint 12, PRD §4.10.2).
/// 실제 저장 경로(`output/{공지문이름}/{청구년월}/`)는 notice.rs 에서 구성한다.
pub(crate) fn output_root() -> PathBuf {
    data_root().join(OUTPUT_SUBDIR)
}

/// 클라우드 폴더 경로로부터 데이터 루트(`{cloud}/smarthb`)를 구성한다.
/// `init_data_root_from_config` 와 동일한 규칙을 공유 — DB 폴더 변경(T3)이 대상 루트를 계산할 때 사용.
pub(crate) fn data_root_for(cloud_folder: &std::path::Path) -> PathBuf {
    cloud_folder.join(SMARTHB_SUBDIR)
}

/// 데이터 루트를 런타임 중 갱신한다. 마법사가 폴더를 새로 지정할 때 호출.
///
/// SQLite pool 이 이미 초기화된 후에 호출하면 pool 은 옛 경로를 계속 사용한다 — 마법사
/// 흐름에서만 호출되어 unlock 이전임이 보장된다.
pub(crate) fn update_data_root(new_path: PathBuf) {
    storage::write(new_path);
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

    // DATA_ROOT 가 thread_local 이라 각 #[test] 는 독립된 fallback 값으로 시작.
    // 따라서 reset 헬퍼·reset 호출 불필요 (A21 — 병렬 실행 안전성 확보).

    #[test]
    fn data_root_default_is_fallback() {
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
    }

    #[test]
    fn update_data_root_changes_subsequent_reads() {
        update_data_root(PathBuf::from("/tmp/test-root"));
        assert_eq!(data_root(), PathBuf::from("/tmp/test-root"));
        assert!(db_path().starts_with("/tmp/test-root"));
        assert!(db_path().ends_with(DB_FILENAME));
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
    }

    #[test]
    fn init_from_config_ignores_empty_path() {
        let tmp = std::env::temp_dir().join("smarthb-paths-test-empty.json");
        std::fs::write(&tmp, r#"{"cloud_folder_path":"","setup_completed":false}"#).unwrap();
        init_data_root_from_config(&tmp);
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
        std::fs::remove_file(&tmp).ok();
    }

    #[test]
    fn init_from_config_ignores_missing_file() {
        let missing = std::env::temp_dir().join("does-not-exist-smarthb-config.json");
        init_data_root_from_config(&missing);
        assert_eq!(data_root(), PathBuf::from(FALLBACK_DEV_ROOT));
    }

    #[test]
    fn assets_dir_composes_under_data_root() {
        update_data_root(PathBuf::from("/tmp/notice-root"));
        assert_eq!(assets_dir(), PathBuf::from("/tmp/notice-root/assets"));
    }

    #[test]
    fn output_root_composes_under_data_root() {
        update_data_root(PathBuf::from("/tmp/notice-root"));
        assert_eq!(output_root(), PathBuf::from("/tmp/notice-root/output"));
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn pragma_key_sql_uses_blob_literal() {
        let sql = pragma_key_sql("deadbeef");
        assert_eq!(sql, "PRAGMA key = \"x'deadbeef'\";");
        // blob literal `x'...'` 의 마커 작은따옴표 2개 외에는 따옴표가 없어야 한다 —
        // 사용자 입력은 hex (`[0-9a-f]`) 만 허용되므로 따옴표 삽입 통로가 없음을 보장.
        assert_eq!(
            sql.matches('\'').count(),
            2,
            "blob literal 마커 외 따옴표가 추가로 삽입되면 SQL injection 위험 — hex 검증 위반",
        );
    }
}
