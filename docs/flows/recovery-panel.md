# Recovery panel

## Trigger

The UI opens or refreshes the operation history for a repository.

## Entry point

`GitRepository::list_operations`

The static web entry point is `ui/index.html`; its recovery view is toggled by `ui/app.js` and is ready for a Tauri command adapter.

## Sequence

1. Resolve the repository's Git directory.
2. Read `.git/githelper/oplog.json` if it exists; a repository with no history returns an empty list.
3. Convert each recorded operation into `RecoveryEntry`, preserving refs, snapshots, commands, details, and an in-flight phase.
4. For reversible operations, expose a copy-pasteable `git update-ref` command that restores recorded refs.
5. `src-tauri/src/commands.rs::list_operations` exposes the same entries through Tauri; the UI renders the operation trail and recovery details as read-only data.
6. Browser-only mode uses sample records until a Tauri host supplies the repository response.

## Reads

- `.git/githelper/oplog.json`
- the repository Git directory via `git rev-parse --git-dir`

## Writes and side effects

None. The read is serialized with operation writes so the UI does not observe a partially-written log.

## Common failure modes

- malformed JSON is surfaced as a recovery-history error;
- a recovery command restores refs only and may leave the worktree unchanged, so this is guidance rather than a one-click Undo.
