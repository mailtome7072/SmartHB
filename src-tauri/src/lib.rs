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
        // V18 (Sprint 7 post-review): 윈도우 크기/위치 자동 저장·복원. 첫 시작은 tauri.conf.json
        // `center: true` 로 모니터 중앙. 이후 사용자 종료 시 마지막 크기/위치를 OS 로컬에 저장.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // R20 (Sprint 3 sprint-review): 앱 시작 시 config.json 의 cloud_folder_path 를 읽어
            // paths::data_root() 가 동적 경로를 반환하도록 초기화. 마법사가 폴더를 다시 지정하면
            // setup::save_cloud_folder 가 paths::update_data_root 를 호출해 즉시 갱신한다.
            //
            // Sprint 7 T3 (R37): device.id 는 양 PC 구분용이므로 클라우드 동기화 폴더가 아닌 OS
            // 로컬 `app_config_dir/device.id` 에 영속화. 양 PC 가 각자 다른 UUID 보유하여
            // stale lock 자동 점유가 "본 디바이스" 락을 올바르게 식별.
            if let Ok(dir) = app.path().app_config_dir() {
                commands::paths::init_data_root_from_config(&dir.join("config.json"));
                commands::lock::init_device_id_path(dir.join("device.id"));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::quit_app,
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
            commands::academic::get_cascade_delete_preview,
            commands::academic::delete_study_period_cascade,
            commands::academic::list_schedule_codes,
            commands::academic::create_schedule_code,
            commands::academic::update_schedule_code,
            commands::academic::toggle_schedule_code_active,
            commands::academic::create_schedule_event,
            commands::academic::update_schedule_event,
            commands::academic::delete_schedule_event,
            commands::academic::list_schedule_events,
            commands::academic::auto_place_assessment_dates,
            commands::attendance::check_attendance_exists,
            commands::attendance::generate_attendances,
            commands::attendance::get_attendance_grid,
            commands::attendance::toggle_attendance,
            commands::attendance::update_absence_memo,
            commands::attendance::get_attendance_summary,
            commands::makeup::get_pending_absences,
            commands::makeup::get_makeup_eligible_dates,
            commands::makeup::create_makeup_with_absences,
            commands::makeup::cancel_makeup,
            commands::makeup::get_absence_history,
            startup::app_startup_sequence,
        ])
        .on_window_event(|_window, event| {
            // V24 (Sprint 7 post-review): macOS 빨간 X / Windows 윈도우 닫기 시점에도 exit_hook
            // 보장 호출 — RunEvent::ExitRequested 가 dock-resident 환경에서 즉시 발생 안 할 수
            // 있어 이중 핸들. idempotent (AtomicBool RAN) 가드로 중복 실행 방지.
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tauri::async_runtime::block_on(startup::exit_hook());
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_handle, event| {
            // PRD §5.3 — 종료 시 exit 백업 + 락 해제. async runtime 이 살아있는 동안 block_on.
            if let tauri::RunEvent::ExitRequested { .. } = event {
                tauri::async_runtime::block_on(startup::exit_hook());
            }
        });
}
