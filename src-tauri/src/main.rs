// Tauri 2 앱 진입점
// Windows 릴리즈 빌드에서 콘솔 창 숨김
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    smarthb_lib::run();
}
