# Desktop Shell

The Tauri host in `src-tauri/` owns window lifecycle and the always-on system tray. Git work stays in `git-helper-core` and the command boundary; this doc covers only shell behavior.

## Close to tray

| Action | Result |
|--------|--------|
| **Main** window close (X) | Hide the main window; process keeps running |
| **Secondary** window close (X) | Destroy that window (quick file diff) |
| Tray left-click | Show, unminimize, and focus the main window |
| Tray menu **Show** | Same as left-click |
| Tray menu **Quit** | Only path that exits the process |

Implementation: `src-tauri/src/tray.rs`, wired from `lib.rs` via `.setup` and `.on_window_event`. Requires the Tauri `tray-icon` feature.

`ExitAllowed` is managed app state. Main-window close always hides unless Quit has set the flag, so `app.exit` can destroy the window without being blocked by `prevent_close`. Secondary windows (`file-diff-*`) are excluded from hide-to-tray.

## Quick file-diff windows

`src-tauri/src/file_diff_window.rs` creates labeled `WebviewWindow`s (`file-diff-<hash>`) loading `file-diff.html`. Session args live in managed `FileDiffSessions`; the window reads them via `file_diff_session` (Tauri injects the calling window). Re-opening the same path focuses the existing window and emits `file-diff-reload`. Capabilities include `file-diff-*` alongside `main`. `open_file_diff_window` is `#[tauri::command(async)]` — a sync build deadlocks WebView2 on Windows (blank, unclosable window).

## Constraints to preserve

- The tray icon is visible for the whole process lifetime, not only while hidden.
- There is no in-UI Quit; users must use the tray menu.
- Left-click must not open the menu (`show_menu_on_left_click(false)`); the menu is for right-click.
- Tray icon comes from `default_window_icon()` (bundled `icons/`), not a separate asset.
- Only the main window hides on close; secondary windows must destroy and drop their session.

## Recent repositories

Successful `open_repository` calls append/promote the path in the app data file owned by `commands/recents.rs`. This is machine-local preference data, separate from repository-local `.git/githelper/` state. See [switch-repository.md](../flows/switch-repository.md).
