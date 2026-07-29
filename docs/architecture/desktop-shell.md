# Desktop Shell

The Tauri host in `src-tauri/` owns window lifecycle and the always-on system tray. Git work stays in `git-helper-core` and the command boundary; this doc covers only shell behavior.

## Close to tray

| Action | Result |
|--------|--------|
| Window close (X) | Hide the main window; process keeps running |
| Tray left-click | Show, unminimize, and focus the main window |
| Tray menu **Show** | Same as left-click |
| Tray menu **Quit** | Only path that exits the process |

Implementation: `src-tauri/src/tray.rs`, wired from `lib.rs` via `.setup` and `.on_window_event`. Requires the Tauri `tray-icon` feature.

`ExitAllowed` is managed app state. Close always hides unless Quit has set the flag, so `app.exit` can destroy the window without being blocked by `prevent_close`.

## Constraints to preserve

- The tray icon is visible for the whole process lifetime, not only while hidden.
- There is no in-UI Quit; users must use the tray menu.
- Left-click must not open the menu (`show_menu_on_left_click(false)`); the menu is for right-click.
- Tray icon comes from `default_window_icon()` (bundled `icons/`), not a separate asset.

## Recent repositories

Successful `open_repository` calls append/promote the path in the app data file owned by `commands/recents.rs`. This is machine-local preference data, separate from repository-local `.git/githelper/` state. See [switch-repository.md](../flows/switch-repository.md).
