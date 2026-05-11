/// 예시 Tauri 커맨드
/// 실제 커맨드는 기능별로 서브모듈로 분리 (예: mod students; mod classes;)
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("안녕하세요, {}! SmartHB에 오신 것을 환영합니다.", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        let result = greet("정쌤");
        assert!(result.contains("정쌤"));
    }
}
