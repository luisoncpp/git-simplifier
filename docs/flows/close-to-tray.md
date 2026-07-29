# Close to Tray

## Trigger

User clicks the main window close button (X), or interacts with the system tray.

## Entry point

- Close: `src-tauri/src/tray.rs::on_window_event` via `lib.rs` `.on_window_event`
- Tray: `src-tauri/src/tray.rs::install` handlers (`on_tray_icon_event`, `on_menu_event`)

## Sequence

### Close (X)

1. Tauri emits `WindowEvent::CloseRequested`.
2. If `ExitAllowed` is false: `window.hide()`, then `api.prevent_close()`.
3. Process stays alive; tray icon remains.

### Restore

1. Left-click tray, or tray menu **Show**.
2. `show_main` finds window `"main"`, then `unminimize` → `show` → `set_focus`.

### Quit

1. Tray menu **Quit** calls `quit_app`.
2. `ExitAllowed` is set true.
3. `app.exit(0)` closes the window; `CloseRequested` does not prevent close.
4. Process exits.

## Reads / Writes

| Kind | What |
|------|------|
| Read | `ExitAllowed`, webview window `"main"` |
| Write | `ExitAllowed` on Quit; window visibility |

## Side effects

- System tray icon created at startup (tooltip `Git Helper`, menu Show/Quit).
- No Git or repository I/O.

## Files to inspect

- `src-tauri/src/tray.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml` (`tray-icon` feature)
- `test/close-to-tray.test.mjs`

## Common failure modes

| Symptom | Likely cause |
|---------|----------------|
| X quits the app | `prevent_close` / `on_window_event` not wired |
| Quit does nothing / hangs | `ExitAllowed` not set before `app.exit` |
| Left-click opens menu only | `show_menu_on_left_click` left at default true |
| No tray icon | Missing `tray-icon` feature or `default_window_icon` |
