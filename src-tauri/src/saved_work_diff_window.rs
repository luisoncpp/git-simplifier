//! Secondary window that previews applying Saved work onto a branch tree.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use git_helper_core::{ObjectId, SavedWorkApplyPreview};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const LABEL_PREFIX: &str = "saved-work-diff-";

#[derive(Clone, Debug, Deserialize)]
pub struct OpenSavedWorkDiffRequest {
    pub branch: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SavedWorkDiffSession {
    pub branch: String,
    pub on_current_branch: bool,
    pub before_tree: ObjectId,
    pub after_tree: ObjectId,
    pub worktree_conflicts: bool,
    pub index_conflicts: bool,
}

pub struct SavedWorkDiffSessions(Mutex<HashMap<String, SavedWorkDiffSession>>);

impl SavedWorkDiffSessions {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Opens or focuses a window for `request.branch`. Must be async on Windows.
#[tauri::command(async)]
pub fn open_saved_work_diff_window(
    app: AppHandle,
    state: State<'_, crate::commands::AppState>,
    sessions: State<'_, SavedWorkDiffSessions>,
    request: OpenSavedWorkDiffRequest,
) -> Result<(), String> {
    let preview = crate::commands::repository::with_repository(state.inner(), |repository| {
        repository
            .preview_saved_work_apply(request.branch.clone())
            .map_err(|error| error.to_string())
    })?;
    let label = window_label(&request.branch);
    let session = session_from_preview(preview);
    store_session(&sessions, &label, session)?;
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.unminimize();
        let _ = existing.show();
        let _ = existing.set_focus();
        let _ = existing.emit("saved-work-diff-reload", ());
        return Ok(());
    }
    WebviewWindowBuilder::new(
        &app,
        &label,
        WebviewUrl::App("saved-work-diff.html".into()),
    )
    .title(format!("Saved work · {}", request.branch))
    .inner_size(960.0, 760.0)
    .min_inner_size(480.0, 360.0)
    .resizable(true)
    .build()
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn saved_work_diff_session(
    window: WebviewWindow,
    sessions: State<'_, SavedWorkDiffSessions>,
) -> Result<SavedWorkDiffSession, String> {
    session_for(&window, &sessions)
}

pub fn session_for(
    window: &WebviewWindow,
    sessions: &SavedWorkDiffSessions,
) -> Result<SavedWorkDiffSession, String> {
    sessions
        .0
        .lock()
        .map_err(|_| "saved work diff session lock was poisoned".to_string())?
        .get(window.label())
        .cloned()
        .ok_or_else(|| "no saved work diff session for this window".to_string())
}

pub fn forget(app: &AppHandle, label: &str) {
    if !label.starts_with(LABEL_PREFIX) {
        return;
    }
    let Some(sessions) = app.try_state::<SavedWorkDiffSessions>() else {
        return;
    };
    let Ok(mut map) = sessions.0.lock() else {
        return;
    };
    map.remove(label);
}

fn session_from_preview(preview: SavedWorkApplyPreview) -> SavedWorkDiffSession {
    SavedWorkDiffSession {
        branch: preview.branch,
        on_current_branch: preview.on_current_branch,
        before_tree: preview.before_tree,
        after_tree: preview.after_tree,
        worktree_conflicts: preview.worktree_conflicts,
        index_conflicts: preview.index_conflicts,
    }
}

fn store_session(
    sessions: &SavedWorkDiffSessions,
    label: &str,
    session: SavedWorkDiffSession,
) -> Result<(), String> {
    sessions
        .0
        .lock()
        .map_err(|_| "saved work diff session lock was poisoned".to_string())?
        .insert(label.to_string(), session);
    Ok(())
}

fn window_label(branch: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    branch.hash(&mut hasher);
    format!("{LABEL_PREFIX}{:x}", hasher.finish())
}
