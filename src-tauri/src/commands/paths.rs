//! 앱 데이터 경로 + SQLCipher PRAGMA 단편 (T11 통합).
//!
//! T10 까지 `backup` 모듈이 임시로 보유하던 공유 경로 헬퍼를 분리하여 lifecycle-friendly
//! 의존성 그래프를 만든다 (`backup` 이 `paths` 에 의존, 다른 모듈이 `backup` 을 경유하지 않음).
//!
//! ## T9 마법사 통합 예정
//!
//! 현재 `DATA_ROOT_DEV` 는 `./SmartHB-data` 임시 위치. 초기 설정 마법사 (후속 sprint) 가
//! 사용자가 지정한 클라우드 동기화 폴더 경로를 `app_settings` 에 저장하면, 본 모듈이
//! 유일한 단일 변경 지점이 된다.

#[cfg_attr(not(feature = "cipher"), allow(dead_code))]
use std::path::PathBuf;

/// 앱 데이터 루트 디렉토리 (T7 임시 — T9 마법사 통합 시 클라우드 폴더로 이전).
const DATA_ROOT_DEV: &str = "./SmartHB-data";

/// 소스 DB 파일명 — `DATA_ROOT_DEV` 하위.
const DB_FILENAME: &str = "app.db";

/// 앱 데이터 루트 — backup·integrity·lock·sync·startup 모듈 공유 단일 진입점.
pub(crate) fn data_root() -> PathBuf {
    PathBuf::from(DATA_ROOT_DEV)
}

/// 소스 DB 파일 경로 — startup·integrity·sync 가 검증·복원·mtime 감시에 공유.
pub(crate) fn db_path() -> PathBuf {
    data_root().join(DB_FILENAME)
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

    #[test]
    fn data_root_returns_dev_path() {
        assert_eq!(data_root(), PathBuf::from(DATA_ROOT_DEV));
    }

    #[test]
    fn db_path_is_under_data_root() {
        assert!(db_path().starts_with(DATA_ROOT_DEV));
        assert!(db_path().ends_with(DB_FILENAME));
    }

    #[cfg(feature = "cipher")]
    #[test]
    fn pragma_key_sql_uses_blob_literal() {
        let sql = pragma_key_sql("deadbeef");
        assert_eq!(sql, "PRAGMA key = \"x'deadbeef'\";");
        assert!(!sql.contains('\''), "단일 따옴표 SQL 인젝션 통로 차단 (raw 따옴표 없음)");
    }
}
