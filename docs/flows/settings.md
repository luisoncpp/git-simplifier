# Settings

## Trigger

The user clicks the gear at the bottom of the left rail.

## Entry point

UI: `set-view` with value `settings` from the rail gear in `views/shell.ts`.
Tauri: `get_ui_preferences`, `set_codechart_path`, `get_project_settings`, `set_project_ide`.

## Step-by-step sequence

1. `setView("settings")` cancels any pending review and renders the Settings pane (no Git discovery).
2. **User settings** (always visible): a Codechart path text field. Empty means auto-guess (`%LOCALAPPDATA%\codechart\codechart.exe`, or `%USERPROFILE%\AppData\Local\…` when `LOCALAPPDATA` is missing). The placeholder shows the guessed path. Used by **Open in Codechart** from the repository menu.
3. **Project settings** (repository required): when a repository is open, the pane shows its path and an IDE `<select>` (VS Code, Cursor, Visual Studio, Rider, Custom). Choosing **Custom** reveals a command text field. When no repository is open, this section explains that a repo is needed and offers **Choose a repository**; user settings remain editable.
4. `state.codechartPath` and `state.guessedCodechartPath` load from `get_ui_preferences` on start; changing the Codechart field calls `set_codechart_path` immediately and writes `ui-preferences.json` (merge preserves `skip_review`).
5. `state.projectIde` loads on every snapshot reload via `get_project_settings` for the open path; unset paths default to VS Code.
6. Changing the IDE preset or custom command calls `set_project_ide` immediately and updates `project-settings.json`. A failed save keeps the session choice; restart restores the file.

## Reads

- App data file `ui-preferences.json` (`codechart_path`, `skip_review`)
- `project-settings.json` entry for the open repository path

## Writes

- `ui-preferences.json` on Codechart path change (merge preserves other fields)
- `project-settings.json` on IDE change (merge by normalized path; other projects untouched)

## Side effects

- None beyond preference persistence

## Files to inspect

- `ui/app/Private/views/settings.ts`
- `ui/app/Private/preferences.ts`
- `ui/app/Private/project-settings.ts`
- `src-tauri/src/commands/prefs.rs`
- `src-tauri/src/commands/codechart.rs`
- `src-tauri/src/commands/project_settings.rs`
- `src-tauri/src/commands/ide.rs`

## Common failure modes

- No repository open → project IDE controls hidden; Codechart path still editable
- Custom IDE command left empty → `open_in_ide` fails when that path is opened from the context menu
- Codechart not installed and path left empty → `open_in_codechart` spawn failure shows an error banner
