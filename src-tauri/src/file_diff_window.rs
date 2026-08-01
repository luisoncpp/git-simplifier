//! Secondary window that shows one file's diff. Hide-to-tray stays main-only.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

use git_helper_core::DiffCompare;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const LABEL_PREFIX: &str = "file-diff-";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenFileDiffRequest {
    pub path: String,
    pub base: String,
    #[serde(default)]
    pub compare: DiffCompare,
    #[serde(default)]
    pub compare_toggle: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileDiffSession {
    pub path: String,
    pub base: String,
    pub compare: DiffCompare,
    pub compare_toggle: bool,
}

pub struct FileDiffSessions(Mutex<HashMap<String, FileDiffSession>>);

impl FileDiffSessions {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Opens or focuses a window for `request.path`. Same path reuses its label.
/// Must be async on Windows: a sync command that builds a WebviewWindow deadlocks
/// WebView2, leaving a blank window whose close button does nothing.
#[tauri::command(async)]
pub fn open_file_diff_window(
    app: AppHandle,
    sessions: State<'_, FileDiffSessions>,
    request: OpenFileDiffRequest,
) -> Result<(), String> {
    let label = window_label(&request.path);
    let session = FileDiffSession {
        path: request.path.clone(),
        base: request.base,
        compare: request.compare,
        compare_toggle: request.compare_toggle,
    };
    store_session(&sessions, &label, session)?;
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.unminimize();
        let _ = existing.show();
        let _ = existing.set_focus();
        let _ = existing.emit("file-diff-reload", ());
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("file-diff.html".into()))
        .title(request.path)
        .inner_size(920.0, 720.0)
        .min_inner_size(480.0, 360.0)
        .resizable(true)
        .build()
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// The calling window's label selects the session — no label round-trip needed.
#[tauri::command]
pub fn file_diff_session(
    window: WebviewWindow,
    sessions: State<'_, FileDiffSessions>,
) -> Result<FileDiffSession, String> {
    sessions
        .0
        .lock()
        .map_err(|_| "file diff session lock was poisoned".to_string())?
        .get(window.label())
        .cloned()
        .ok_or_else(|| "no file diff session for this window".to_string())
}

pub fn forget(app: &AppHandle, label: &str) {
    if !label.starts_with(LABEL_PREFIX) {
        return;
    }
    let Some(sessions) = app.try_state::<FileDiffSessions>() else {
        return;
    };
    let Ok(mut map) = sessions.0.lock() else {
        return;
    };
    map.remove(label);
}

fn store_session(
    sessions: &FileDiffSessions,
    label: &str,
    session: FileDiffSession,
) -> Result<(), String> {
    sessions
        .0
        .lock()
        .map_err(|_| "file diff session lock was poisoned".to_string())?
        .insert(label.to_string(), session);
    Ok(())
}

fn window_label(path: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{LABEL_PREFIX}{:x}", hasher.finish())
}
