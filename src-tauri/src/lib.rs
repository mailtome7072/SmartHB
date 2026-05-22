mod commands;
mod error;
mod startup;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Sprint 5 T1 — 동일 PC 다중 인스턴스 차단. 두 번째 실행 시 기존 창 포커스 + 새 프로세스 즉시 종료.
        // PRD §5.3 의 app.lock 본래 의도(양 PC 간 시점 분리)와 분리하여 같은 머신 내 충돌을 원천 차단.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // R20 (Sprint 3 sprint-review): 앱 시작 시 config.json 의 cloud_folder_path 를 읽어
            // paths::data_root() 가 동적 경로를 반환하도록 초기화. 마법사가 폴더를 다시 지정하면
            // setup::save_cloud_folder 가 paths::update_data_root 를 호출해 즉시 갱신한다.
            if let Ok(dir) = app.path().app_config_dir() {
                commands::paths::init_data_root_from_config(&dir.join("config.json"));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::diagnose_sqlcipher,
            commands::auth::check_auth_status,
            commands::auth::set_password,
            commands::auth::unlock_db,
            commands::recovery::generate_recovery_code,
            commands::recovery::verify_recovery_code,
            commands::recovery::reset_password_with_code,
            commands::lock::check_lock_status,
            commands::lock::acquire_lock,
            commands::lock::release_lock,
            commands::backup::create_backup,
            commands::backup::list_backups,
            commands::backup::restore_backup,
            commands::integrity::check_integrity,
            commands::integrity::auto_restore,
            commands::sync::check_sync_status,
            commands::audit::get_audit_logs,
            commands::students::next_serial_number,
            commands::students::create_student,
            commands::students::update_student,
            commands::students::get_student,
            commands::students::withdraw_student,
            commands::students::reinstate_student,
            commands::students::list_students,
            commands::students::count_students,
            commands::schedules::set_schedule,
            commands::schedules::delete_schedule,
            commands::schedules::get_schedules,
            commands::schedules::get_schedule_history,
            commands::schedules::get_weekly_hours,
            commands::fees::list_fees,
            commands::fees::create_fee,
            commands::fees::update_fee,
            commands::fees::match_fee_by_hours,
            commands::codes::list_codes,
            commands::codes::count_codes,
            commands::codes::create_code,
            commands::codes::update_code,
            commands::codes::reorder_codes,
            commands::setup::save_cloud_folder,
            commands::setup::complete_setup,
            commands::setup::get_setup_status,
            commands::settings::get_operating_hours,
            commands::settings::save_operating_hours,
            commands::academic::create_study_period,
            commands::academic::update_study_period,
            commands::academic::list_study_periods,
            commands::academic::get_study_period,
            commands::academic::confirm_study_period,
            commands::academic::delete_study_period,
            commands::academic::list_schedule_codes,
            commands::academic::create_schedule_code,
            commands::academic::update_schedule_code,
            commands::academic::toggle_schedule_code_active,
            startup::app_startup_sequence,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_handle, event| {
            // PRD §5.3 — 종료 시 exit 백업 + 락 해제. async runtime 이 살아있는 동안 block_on.
            if let tauri::RunEvent::ExitRequested { .. } = event {
                tauri::async_runtime::block_on(startup::exit_hook());
            }
        });
}
