mod commands;
mod error;
mod startup;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            commands::students::list_students,
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
