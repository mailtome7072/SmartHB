pub mod academic;
pub mod attendance;
pub mod audit;
pub mod auth;
pub mod billing;
pub mod backup;
pub mod calendar;
pub mod codes;
pub mod db;
pub mod diagnosis;
pub mod expiration;
pub mod fees;
pub mod integrity;
pub mod lock;
pub mod makeup;
pub mod notice;
pub mod pagination;
pub mod paths;
pub mod runtime;
pub mod schedules;
pub mod settings;
pub mod setup;
pub mod students;
pub mod sync;

/// 예시 Tauri 커맨드
/// 실제 커맨드는 기능별로 서브모듈로 분리 (예: mod students; mod classes;)
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("안녕하세요, {}! SmartHB에 오신 것을 환영합니다.", name)
}

/// 앱 종료 IPC (Sprint 6 후속 hotfix 보강 + Sprint 7 발견).
///
/// 사이드바 "종료" 버튼이 호출. `AppHandle::exit(0)` → `RunEvent::ExitRequested` 트리거 →
/// `startup::exit_hook` 의 release_lock + exit 백업 정상 수행.
///
/// `@tauri-apps/api/window::close()` 는 capabilities 권한(`core:window:allow-close`) 필요 +
/// macOS 에서 마지막 창 닫혀도 앱이 메뉴바에 남는 케이스가 있어, 백엔드 IPC 로 처리.
#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
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
