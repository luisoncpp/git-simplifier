use tauri::{AppHandle, State};

use super::ide_spawn::{ide_file_spawn_spec, ide_folder_spawn_spec, resolve_repo_file, spawn_ide};
use super::project_settings::{IdeChoice, ProjectSettingsStore};
use super::repository::with_repository;
use super::state::AppState;

#[tauri::command(async)]
pub fn get_project_settings(
    app: AppHandle,
    path: String,
) -> Result<super::project_settings::ProjectSettings, String> {
    ProjectSettingsStore::from_app(&app)?.get(&path)
}

#[tauri::command(async)]
pub fn set_project_ide(
    app: AppHandle,
    path: String,
    ide: IdeChoice,
) -> Result<super::project_settings::ProjectSettings, String> {
    ProjectSettingsStore::from_app(&app)?.set_ide(&path, ide)
}

#[tauri::command(async)]
pub fn open_in_ide(app: AppHandle, path: String) -> Result<(), String> {
    let settings = ProjectSettingsStore::from_app(&app)?.get(&path)?;
    let spec = ide_folder_spawn_spec(&settings.ide)?;
    spawn_ide(&spec, Some(&path))
}

#[tauri::command(async)]
pub fn open_file_in_ide(
    app: AppHandle,
    state: State<'_, AppState>,
    repo_path: String,
    file_path: String,
) -> Result<(), String> {
    let settings = ProjectSettingsStore::from_app(&app)?.get(&repo_path)?;
    let absolute = with_repository(state.inner(), |repo| {
        let root = repo.worktree_root().map_err(|error| error.to_string())?;
        let path = resolve_repo_file(&root, &file_path)?;
        Ok(path.to_string_lossy().into_owned())
    })?;
    let spec = ide_file_spawn_spec(&settings.ide, &absolute)?;
    spawn_ide(&spec, None)
}
