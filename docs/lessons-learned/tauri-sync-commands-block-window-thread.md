# Tauri sync commands block the window thread

Tauri commands use blocking dispatch by default, even when JavaScript awaits them. A synchronous handler that runs Git can therefore stop the desktop window from processing input and paint events until every child process finishes.

Mark Git-backed handlers with `#[tauri::command(async)]`. This keeps the Rust function synchronous while Tauri dispatches it through the worker thread pool. Keep trivial in-memory commands synchronous only when they cannot touch repository or filesystem state.
