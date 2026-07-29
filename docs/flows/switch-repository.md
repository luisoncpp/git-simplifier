# Switch repository

## Trigger

The user opens the rail Repository control and picks a recent path, removes one, or browses for a new folder.

## Entry point

UI: `repository-switcher.js` via `toggle-repo-menu` / `open-recent` / `remove-recent` / `pick-repository`.
Tauri: `open_repository`, `list_recent_repositories`, `remove_recent_repository`.

## Step-by-step sequence

1. `start()` loads persisted recents into `state.recentRepositories`.
2. Toggle opens the filter menu; Esc / outside click closes it.
3. Choosing a recent path (or browsing) cancels any pending review, records the path as the in-flight
   selection, then calls `open_repository`. While the call is pending, an open menu marks that target
   as selected instead of continuing to mark the previous repository.
4. Rust opens the path; on success it remembers the path at the front of the recent list and returns a snapshot.
5. On open failure, Rust removes that path from recents (idempotent) and returns the error; the previous repository session stays open.
6. The controller reloads from the returned snapshot (or refreshes the recent list after a failure),
   clears draft / outcome / expanded, and clears the in-flight selection. A failure therefore restores
   the previous repository as the visible selection.

## Reads

- App data file `recent-repositories.json` (paths only)
- `GitRepository::open` + `load_state` for a successful switch

## Writes

- `recent-repositories.json` on successful open (promote / dedupe) and on remove / failed open prune
- No Git writes

## Side effects

- Pending review is cancelled before switching
- Outcome banner and draft selections are cleared for the new repository

## Files to inspect

- `src-tauri/src/commands/recents.rs`
- `src-tauri/src/commands/actions.rs` (`open_repository`)
- `src-tauri/src/commands/state.rs` (`open_path`)
- `ui/app/Private/repository-switcher.js`
- `ui/app/Private/views/repo-menu.js`

## Common failure modes

- Folder picker cancelled → no-op
- Path is not a Git repository → error banner; path pruned from recents; prior repo kept
- Browser / no desktop bridge → recents stay empty; browse reports unavailable
