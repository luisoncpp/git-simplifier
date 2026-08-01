use git_helper_core::{RefName, RepoPath};
use tauri::State;

use super::data::{DiffRequest, FilePathRequest};
use super::repository::with_repository;
use super::state::AppState;

/// The two Inspection surfaces. Raw diff takes the patch as text; Files diff
/// takes the same patch parsed per file, plus one file at a time at full context
/// so the viewer can widen a gap without another round trip.
#[tauri::command(async)]
pub fn generate_branch_diff(
    state: State<'_, AppState>,
    request: DiffRequest,
) -> Result<String, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .branch_diff(base, request.compare)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn generate_files_diff(
    state: State<'_, AppState>,
    request: DiffRequest,
) -> Result<Vec<git_helper_core::FileDiff>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .files_diff(base, request.compare)
            .map_err(|error| error.to_string())
    })
}

/// `None` means the path no longer differs from Base — HEAD may have moved since
/// the diff was loaded — which the viewer treats as a refresh, not an error.
#[tauri::command(async)]
pub fn generate_full_file_diff(
    state: State<'_, AppState>,
    request: FilePathRequest,
) -> Result<Option<git_helper_core::FileDiff>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    let path = RepoPath::new(request.path).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .full_file_diff(base, path, request.compare)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn generate_saved_work_files_diff(
    state: State<'_, AppState>,
    window: tauri::WebviewWindow,
    sessions: State<'_, crate::saved_work_diff_window::SavedWorkDiffSessions>,
) -> Result<Vec<git_helper_core::FileDiff>, String> {
    let session = crate::saved_work_diff_window::session_for(&window, &sessions)?;
    with_repository(state.inner(), |repository| {
        repository
            .saved_work_apply_files_diff(session.before_tree, session.after_tree)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn generate_saved_work_full_file_diff(
    state: State<'_, AppState>,
    window: tauri::WebviewWindow,
    sessions: State<'_, crate::saved_work_diff_window::SavedWorkDiffSessions>,
    request: super::data::SavedWorkFilePathInput,
) -> Result<Option<git_helper_core::FileDiff>, String> {
    let session = crate::saved_work_diff_window::session_for(&window, &sessions)?;
    let path = RepoPath::new(request.path).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .saved_work_apply_full_file_diff(session.before_tree, session.after_tree, path)
            .map_err(|error| error.to_string())
    })
}
