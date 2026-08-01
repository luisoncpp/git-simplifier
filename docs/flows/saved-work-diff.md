# Saved work apply diff

Multi-file preview of what restoring Saved work would change on a branch's working tree. Read-only; never applies the snapshot.

## Trigger

**Diff** on a Saved work row in the rail panel.

## Sequence

1. Main UI invokes `open_saved_work_diff_window` with the row's branch name.
2. Rust runs `preview_saved_work_apply`: resolves the stash commit at `refs/githelper/wip/<branch>`, builds the current (or target-branch tip) worktree tree, and simulates apply with `git merge-tree --write-tree --merge-base=<stash^1>`.
3. Session stores `before_tree`, `after_tree`, conflict flags, and branch metadata; a secondary window loads `saved-work-diff.html`.
4. `SavedWorkDiffApp` calls `saved_work_diff_session`, then `generate_saved_work_files_diff` (context-3) and `generate_saved_work_full_file_diff` on gap expand.
5. Renders through the shared multi-file pane (`files-diff/pane.ts`). Re-open focuses the window and emits `saved-work-diff-reload`.

## Preview semantics

| Row | Ours tree | Meaning |
|-----|-----------|---------|
| Current branch | Today's tracked worktree | What restore would do now |
| Other branch | That branch's tip tree | What restore would do after switching there |

Worktree delta only. Index-half merge runs to set `index_conflicts` when `apply --index` would fail; no second file list.

## Reads / writes

- Reads: WIP ref, stash parents, HEAD / branch tip, porcelain-free worktree capture via temp `GIT_INDEX_FILE`.
- Git writes: dangling tree objects only (merge-tree / write-tree). No ref, index, or worktree mutation.
- Shell: secondary `WebviewWindow` (`saved-work-diff-*`).

## Files to inspect

- `ui/app/Private/views/panels.ts`, `saved-work-diff/`
- `ui/saved-work-diff.html`, `ui/saved-work-diff-app.ts`
- `src/switch/preview.rs`, `src/inspection/diff.rs` (`tree_files_diff`)
- `src-tauri/src/saved_work_diff_window.rs`, `commands/diffs.rs`

## Failure modes

| Symptom | Likely cause |
|---------|----------------|
| Button does nothing | No open repository, or Saved work ref was deleted |
| Window errors on open | Active merge/rebase (same guard as restore) |
| Blank window | `open_saved_work_diff_window` ran as sync command (WebView2 deadlock) |
| Empty file list | Apply preview is a no-op on the chosen tree |
| Warn banner | `merge-tree` reported conflicts on worktree or index half |
