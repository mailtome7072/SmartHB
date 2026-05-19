pub mod auth;

/// 예시 Tauri 커맨드
/// 실제 커맨드는 기능별로 서브모듈로 분리 (예: mod students; mod classes;)
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("안녕하세요, {}! SmartHB에 오신 것을 환영합니다.", name)
}

/// SQLCipher 통합 진단 — ADR-001 PoC 검증용.
///
/// `cipher` feature 가 켜진 빌드에서는 `libsqlite3-sys` + `bundled-sqlcipher-vendored-openssl`
/// 로 SQLCipher 가 통합되어 `PRAGMA cipher_version` 응답이 `"4.x.x community"` 형식으로
/// 반환된다. cipher off 빌드에서는 일반 SQLite 만 link 되어 동일 PRAGMA 가 NULL 을 반환한다.
///
/// 후속 T3 (키 관리) 구현 시 본 진단 IPC 는 제거되고 정식 `unlock_db` 흐름으로 대체된다.
#[tauri::command]
pub async fn diagnose_sqlcipher() -> Result<String, String> {
    use crate::error::AppError;
    use sqlx::Row;
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .map_err(AppError::Db)?;

    let row = sqlx::query("PRAGMA cipher_version")
        .fetch_optional(&pool)
        .await
        .map_err(AppError::Db)?;

    let cipher_version: Option<String> = row.and_then(|r| r.try_get(0).ok());

    match cipher_version {
        Some(v) if !v.is_empty() => {
            Ok(format!("SQLCipher 통합 확인 — cipher_version: {}", v))
        }
        _ => Ok("평문 SQLite (cipher feature off 빌드)".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("정쌤");
        assert!(result.contains("정쌤"));
    }

    #[tokio::test]
    async fn test_diagnose_sqlcipher_returns_status() {
        let result = diagnose_sqlcipher().await;
        assert!(result.is_ok(), "diagnose_sqlcipher 가 OK 를 반환해야 함");
        let msg = result.unwrap();
        // cipher feature on/off 어느 빌드든 둘 중 하나의 메시지가 반환되어야 한다.
        assert!(
            msg.contains("SQLCipher 통합 확인") || msg.contains("평문 SQLite"),
            "예상치 못한 응답: {}",
            msg
        );
    }
}
