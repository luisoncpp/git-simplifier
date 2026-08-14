# Switch repository

## Trigger

The user opens the rail Repository control and picks a recent path, removes one, browses for a new folder, or right-clicks a repository row or the current picker to copy its path, open its folder in the system file manager, open it in the configured IDE, open it in Codechart, open a terminal, or open bash.

## Entry point

UI: `repository-switcher.ts` via `toggle-repo-menu` / `open-recent` / `remove-recent` / `pick-repository` / `copy-repository-path` / `open-in-explorer` / `open-in-ide` / `open-in-codechart` / `open-in-terminal` / `open-in-bash`.
Tauri: `open_repository`, `list_recent_repositories`, `remove_recent_repository`, `open_in_explorer`, `open_in_ide`, `open_in_codechart`, `open_in_terminal`, `open_in_bash`. Clipboard copy stays in the UI.

## Step-by-step sequence

1. `start()` loads persisted recents into `state.recentRepositories`.
2. Toggle opens the filter menu; Esc / outside click closes it.
3. Pressing a recent path immediately previews its selected color. Releasing the press closes the
   menu synchronously, records the path as the in-flight selection, cancels any pending review, then
   calls `open_repository`. While the call is pending, the repository picker shows that target
   instead of continuing to show the previous path.
4. Rust opens the path; on success it remembers the path at the front of the recent list and returns a snapshot.
5. On open failure, Rust removes that path from recents (idempotent) and returns the error; the previous repository session stays open.
6. The controller clears draft / outcome / expanded, reloads from the returned snapshot immediately so the new repository is on screen before any network wait, then fetches remotes for the new repository (same as Refresh — status-bar progress bar and stop button), reloads once more so moved remote-tracking refs are reflected, refreshes the recent list, and clears the in-flight selection. A failed fetch sets `state.warning` and still loads the local snapshot. A failed open restores the previous repository as the visible selection.
7. Right-clicking a recent row or the current picker opens a context menu. **Copy path** dismisses the
   menu synchronously, then writes that row's path to the clipboard. **Open in File Explorer**
   calls `open_in_explorer`, which uses the desktop opener plugin to open that folder in the OS file manager.
   **Open in the IDE** dismisses the menu synchronously, then calls `open_in_ide` with the menu path; Rust
   looks up that path's IDE in `project-settings.json` (default VS Code) and spawns the editor with the
   folder argument. **Open in Codechart** dismisses the menu synchronously, then calls `open_in_codechart`;
   Rust resolves `codechart.exe` from `ui-preferences.json` (saved path, else `%LOCALAPPDATA%\codechart\codechart.exe`)
   and spawns it with the folder argument. **Open in Terminal** dismisses the menu synchronously, then calls
   `open_in_terminal`; Rust resolves `terminal_path` from `ui-preferences.json` (saved command, else Windows Terminal
   with PowerShell if available, otherwise Windows PowerShell) and spawns the terminal rooted in the repository folder.
   **Open in bash** dismisses the menu synchronously, then calls `open_in_bash`; Rust resolves `bash_path` (saved path,
   else autodetected Git Bash) and launches it using the configured terminal (Windows Terminal, custom terminal, or direct).

## Reads

- App data file `recent-repositories.json` (paths only)
- App data file `project-settings.json` (per-path IDE, for open-in-IDE only)
- App data file `ui-preferences.json` (`codechart_path`, `terminal_path`, `bash_path`)
- `GitRepository::open` + `load_state` for a successful switch

## Writes

- `recent-repositories.json` on successful open (promote / dedupe) and on remove / failed open prune
- No Git writes

## Side effects

- Pending review is cancelled before switching
- Outcome banner and draft selections are cleared for the new repository
- A remote fetch runs after a successful open, streaming progress to the status bar with a stop button that kills it; unreachable remotes become a dismissible **Fetch failed** warning, not a blocked open

## Files to inspect

- `src-tauri/src/commands/recents.rs`
- `src-tauri/src/commands/actions.rs` (`open_repository`)
- `src-tauri/src/commands/state.rs` (`open_path`)
- `ui/app/Private/repository-switcher.ts`
- `ui/app/Private/discovery.ts` (`fetchRemotes`)
- `ui/app/Private/views/repo-menu.ts`
- `src-tauri/src/commands/project_settings.rs`
- `src-tauri/src/commands/ide.rs`
- `src-tauri/src/commands/codechart.rs`
- `src-tauri/src/commands/terminal.rs`
- `src-tauri/src/commands/bash.rs`
- `src-tauri/src/commands/prefs.rs`
- `src/inspection/fetch/`
- `ui/app/Private/views/status-bar.ts`

## Common failure modes

- Folder picker cancelled → no-op
- Path is not a Git repository (or has no commits yet) → error banner; path pruned from recents; prior repo kept. Open probes the worktree before swapping the session, so a bare exit-code inspection error is not the first signal.
- Configured Base is missing locally → discovery after open fails with an actionable Invalid Base message (fetch or pick another Base), not a bare exit code
- Remote unreachable (tunnel down, offline, auth) → open still succeeds; **Fetch failed** warning banner with the Git stderr
- Browser / no desktop bridge → recents stay empty; browse reports unavailable
- Unavailable clipboard API → **Copy path** reports an error instead of claiming success
- IDE not on PATH or custom command missing → error banner from `open_in_ide` spawn failure
- Codechart not installed and path left empty → error banner from `open_in_codechart` spawn failure
- Custom terminal executable not found or failed to spawn → error banner from `open_in_terminal` spawn failure
- Git Bash executable not found or failed to spawn → error banner from `open_in_bash` spawn failure
