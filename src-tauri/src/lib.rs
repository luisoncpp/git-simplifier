#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod file_diff_window;
mod saved_work_diff_window;
mod tray;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppState::new())
        .manage(file_diff_window::FileDiffSessions::new())
        .manage(saved_work_diff_window::SavedWorkDiffSessions::new())
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
            commands::prefs::get_ui_preferences,
            commands::actions::set_skip_review,
            commands::codechart::set_codechart_path,
            commands::terminal::set_terminal_path,
            commands::bash::set_bash_path,
            commands::actions::open_in_explorer,
            commands::ide::get_project_settings,
            commands::ide::set_project_ide,
            commands::ide::open_in_ide,
            commands::ide::open_file_in_ide,
            commands::codechart::open_in_codechart,
            commands::terminal::open_in_terminal,
            commands::bash::open_in_bash,
            commands::actions::load_snapshot,
            commands::actions::fetch_remotes,
            commands::actions::cancel_fetch,
            commands::actions::list_operations,
            commands::actions::list_saved_work,
            commands::actions::list_base_choices,
            commands::actions::set_base,
            commands::actions::list_changed_paths,
            commands::actions::list_revert_paths,
            commands::diffs::generate_branch_diff,
            commands::diffs::generate_files_diff,
            commands::diffs::generate_full_file_diff,
            commands::diffs::generate_saved_work_files_diff,
            commands::diffs::generate_saved_work_full_file_diff,
            file_diff_window::open_file_diff_window,
            file_diff_window::file_diff_session,
            saved_work_diff_window::open_saved_work_diff_window,
            saved_work_diff_window::saved_work_diff_session,
            commands::actions::list_editable_commits,
            commands::actions::list_local_branches,
            commands::actions::list_cleanup_branches,
            commands::actions::list_submodules,
            commands::actions::list_dirty_submodules,
            commands::actions::prepare_operation,
            commands::actions::apply_operation,
            commands::actions::cancel_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running Git Simplifier");
}
