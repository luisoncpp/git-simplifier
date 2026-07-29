#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod tray;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::new())
        .manage(tray::ExitAllowed::new())
        .setup(|app| {
            tray::install(app)?;
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::actions::app_ready,
            commands::actions::open_repository,
            commands::actions::list_recent_repositories,
            commands::actions::remove_recent_repository,
            commands::actions::load_snapshot,
            commands::actions::list_operations,
            commands::actions::list_saved_work,
            commands::actions::list_base_choices,
            commands::actions::set_base,
            commands::actions::list_changed_paths,
            commands::actions::generate_branch_diff,
            commands::actions::list_editable_commits,
            commands::actions::list_local_branches,
            commands::actions::list_submodules,
            commands::actions::prepare_operation,
            commands::actions::apply_operation,
            commands::actions::cancel_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running Git Helper");
}
