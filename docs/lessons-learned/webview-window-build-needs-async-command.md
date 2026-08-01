# Creating a WebviewWindow from a sync command freezes it on Windows

On Windows, `WebviewWindowBuilder::build` must not run inside a synchronous Tauri command (or a sync event handler). WebView2 deadlocks: the new window appears blank and its close button does nothing.

Mark the opener with `#[tauri::command(async)]` so Tauri dispatches it off the window thread — the same attribute Git-backed commands already use. Hide-to-tray being wrong is a separate failure mode; a frozen blank window with an inert X is this deadlock.

See also: [tauri-sync-commands-block-window-thread.md](./tauri-sync-commands-block-window-thread.md).
