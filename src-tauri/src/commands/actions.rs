use std::sync::atomic::{AtomicU64, Ordering};

use git_helper_core::{FetchControl, FetchProgress, InspectionError, RefName};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

use super::apply;
use super::data::{
    BaseRequest, DirtySubmodulesRequest, OpenRepositoryInput, OperationOutcome, OperationReview,
    PrepareOperationRequest, RepositorySnapshot,
};
use super::prepare;
use super::prefs::PrefsStore;
use super::recents::{RecentRepository, RecentStore};
use super::repository::{self, with_repository};
use super::state::AppState;

static NEXT_PLAN: AtomicU64 = AtomicU64::new(1);

fn plan_id() -> String {
    format!("op-{}", NEXT_PLAN.fetch_add(1, Ordering::Relaxed))
}

#[tauri::command]
pub fn app_ready() -> &'static str {
    "git-helper-ui-ready"
}

#[tauri::command(async)]
pub fn open_repository(
    app: AppHandle,
    state: State<'_, AppState>,
    request: OpenRepositoryInput,
) -> Result<RepositorySnapshot, String> {
    let path = request.path;
    if let Err(error) = state.open_path(path.clone().into()) {
        let _ = RecentStore::from_app(&app).and_then(|store| store.remove(&path));
        return Err(error);
    }
    let snapshot = repository::snapshot(state.inner())?;
    let _ = RecentStore::from_app(&app).and_then(|store| store.remember(&path));
    Ok(snapshot)
}

#[tauri::command(async)]
pub fn list_recent_repositories(app: AppHandle) -> Result<Vec<RecentRepository>, String> {
    RecentStore::from_app(&app)?.list()
}

#[tauri::command(async)]
pub fn remove_recent_repository(
    app: AppHandle,
    path: String,
) -> Result<Vec<RecentRepository>, String> {
    RecentStore::from_app(&app)?.remove(&path)
}

#[tauri::command(async)]
pub fn set_skip_review(
    app: AppHandle,
    skip_review: bool,
) -> Result<super::prefs::UiPreferences, String> {
    PrefsStore::from_app(&app)?.set_skip_review(skip_review)
}

#[tauri::command(async)]
pub fn reveal_in_explorer(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command(async)]
pub fn load_snapshot(state: State<'_, AppState>) -> Result<RepositorySnapshot, String> {
    repository::snapshot(state.inner())
}

#[tauri::command(async)]
pub fn fetch_remotes(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let control = FetchControl::new();
    state.register_fetch(control.clone());
    let progress = move |event: FetchProgress| {
        let _ = app.emit("fetch-progress", event);
    };
    let result = with_repository(state.inner(), |repository| {
        repository
            .fetch_remotes_with_progress(&control, progress)
            .map_err(fetch_error_message)
    });
    state.clear_fetch();
    result.map(|_| ())
}

/// The cancel path touches only the fetch slot, never the repository mutex, so
/// it runs while a fetch holds that mutex.
#[tauri::command(async)]
pub fn cancel_fetch(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_fetch();
    Ok(())
}

fn fetch_error_message(error: InspectionError) -> String {
    let text = error.to_string();
    let detail = text.strip_prefix("Git inspection failed: ").unwrap_or(&text);
    format!("Could not fetch remotes: {detail}")
}

#[tauri::command(async)]
pub fn list_operations(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::RecoveryEntry>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_operations()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_saved_work(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::SavedWork>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_saved_work()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_base_choices(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::RemoteBaseChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_base_choices()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn set_base(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<RepositorySnapshot, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository.set_base(base).map_err(|error| error.to_string())
    })?;
    repository::snapshot(state.inner())
}

#[tauri::command(async)]
pub fn list_changed_paths(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<Vec<git_helper_core::ChangedPath>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_changed_paths(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_revert_paths(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<Vec<git_helper_core::ChangedPath>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_revert_paths(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_editable_commits(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<Vec<git_helper_core::EditableCommit>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_editable_commits(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_local_branches(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::LocalBranchChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_local_branches()
            .map_err(|error| error.to_string())
    })
}

/// Returns the maximal offerable set once. The three Cleanup toggles filter this
/// result in the UI, so flipping one never costs another repository scan.
#[tauri::command(async)]
pub fn list_cleanup_branches(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<git_helper_core::CleanupDiscovery, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .discover_cleanup(&base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_submodules(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::SubmoduleChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_submodules()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_dirty_submodules(
    state: State<'_, AppState>,
    request: DirtySubmodulesRequest,
) -> Result<Vec<git_helper_core::DirtySubmodule>, String> {
    let base = request
        .base
        .map(RefName::new)
        .transpose()
        .map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_dirty_submodules(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn prepare_operation(
    state: State<'_, AppState>,
    request: PrepareOperationRequest,
) -> Result<OperationReview, String> {
    let prepared = prepare::prepare(state.inner(), plan_id(), request)?;
    state.set_pending(prepared.pending)?;
    Ok(prepared.review)
}

#[tauri::command(async)]
pub fn cancel_operation(state: State<'_, AppState>, plan_id: String) -> Result<(), String> {
    state.cancel_pending(&plan_id)
}

#[tauri::command(async)]
pub fn apply_operation(
    state: State<'_, AppState>,
    plan_id: String,
) -> Result<OperationOutcome, String> {
    let operation = state.take_pending(&plan_id)?;
    apply::apply(state.inner(), operation)
}
