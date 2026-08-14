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

## Saved work apply-diff windows

`src-tauri/src/saved_work_diff_window.rs` mirrors the quick file-diff shell for multi-file previews (`saved-work-diff-<hash>` → `saved-work-diff.html`). `open_saved_work_diff_window` computes the merge-tree preview up front, stores tree OIDs and conflict flags in `SavedWorkDiffSessions`, and reuses the same destroy-on-close / reload-on-focus rules. Capabilities include `saved-work-diff-*`.

## Constraints to preserve

- The tray icon is visible for the whole process lifetime, not only while hidden.
- There is no in-UI Quit; users must use the tray menu.
- Left-click must not open the menu (`show_menu_on_left_click(false)`); the menu is for right-click.
- Tray icon comes from `default_window_icon()` (bundled `icons/`), not a separate asset.
- Only the main window hides on close; secondary windows must destroy and drop their session (`file-diff-*`, `saved-work-diff-*`).

## Recent repositories

Successful `open_repository` calls append/promote the path in the app data file owned by `commands/recents.rs`. This is machine-local preference data, separate from repository-local `.git/githelper/` state. See [switch-repository.md](../flows/switch-repository.md).

## Project settings, open in IDE, Codechart, Terminal, and Bash

Per-repository preferences live in `project-settings.json` under app data (`commands/project_settings.rs`). The first field is `ide`: a tagged choice (`vscode`, `cursor`, `visual-studio`, `rider`, or `custom` with a command path). Missing entries default to VS Code.

Global user preferences live in `ui-preferences.json` (`commands/prefs.rs`). `codechart_path` overrides the default Codechart install location; empty means auto-guess under `%LOCALAPPDATA%\codechart\codechart.exe`. `terminal_path` overrides the default terminal; empty uses Windows Terminal (`wt.exe`) running PowerShell if available, otherwise Windows PowerShell (`powershell.exe`). `bash_path` overrides Git's default bash executable; empty autodetects Git Bash from the Git install (e.g. `C:\Program Files\Git\bin\bash.exe`, LocalAppData, or PATH).

`open_in_ide` (`commands/ide.rs`) loads the path's IDE choice, maps presets to CLI programs (`code`, `cursor`, `devenv`, `rider`), and spawns the folder argument without waiting. `open_file_in_ide` uses the same IDE choice and opens a file inside the repository: VS Code and Cursor pass `--reuse-window`, Visual Studio passes `/edit`, Rider and custom commands receive the absolute file path. Windows uses `CREATE_NO_WINDOW` on the spawn so `.cmd` shims do not flash a console.

`open_in_codechart` (`commands/codechart.rs`) resolves `codechart.exe` from saved or guessed paths and spawns it directly with the folder argument (no `cmd /C`; `CREATE_NO_WINDOW` on Windows). Settings are edited from the rail gear; see [settings.md](../flows/settings.md).

`open_in_terminal` (`commands/terminal.rs`) spawns a terminal rooted in the repository folder: if `terminal_path` is configured it spawns that command/program; otherwise it prefers Windows Terminal (`wt.exe -d <path> powershell.exe`) or falls back to Windows PowerShell (`powershell.exe` with working directory and `CREATE_NEW_CONSOLE`). Unlike IDE/Codechart spawns, console windows are not suppressed (`CREATE_NO_WINDOW` is omitted) so the interactive terminal appears.

`open_in_bash` (`commands/bash.rs`) launches Git Bash in the repository folder using the terminal configured for **Open in Terminal** (Windows Terminal `wt -d <path> <bash>`, custom terminal `<term> -e <bash>`, or direct bash console spawn when no custom terminal or wt is available). The bash executable defaults to Git's `bash.exe` or the user's `bash_path` override.
