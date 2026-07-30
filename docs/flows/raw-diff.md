# Raw diff

The text half of Inspection. For the per-file viewer over the same patch, see [files-diff.md](./files-diff.md).

## Trigger

The user opens **Inspection → Raw diff**, refreshes while it is open, changes Base, or switches the **HEAD / Local** compare toggle.

## Sequence

1. `AppController.setView` selects `raw-diff` and cancels any pending write review. `isInspectionView` is what admits it, because the group holds two views.
2. `loadViewData` dispatches on the view id and calls `loadBranchDiff`, so opening Raw diff never fetches the structured diff. It reads Base from the repository snapshot; without Base it renders setup guidance and sends no Git request.
3. `generate_branch_diff` validates the Base ref and delegates through the open `GitRepository`, passing `compare` from `state.diffView`.
4. `inspection::diff` runs a read-only `git diff` built by the shared `diff_args`. **HEAD** (default) uses `Base...HEAD` — merge base to HEAD, committed only. **Local** resolves `git merge-base Base HEAD` and diffs that commit to the working tree (tracked changes only). Stable options disable color, external diff/textconv, rename collapsing, relative paths, submodule hiding, and configurable prefixes.
5. The UI renders the returned patch. **Copy diff** sends that exact state value to the clipboard and announces success.

## Reads and writes

- Reads: configured Base, HEAD, their merge base, and — in **HEAD** mode — committed trees; in **Local** mode, the working tree as well.
- Git writes: none.
- Other side effect: the explicit copy action replaces clipboard text. The compare mode is session preference and survives refresh.

## Files to inspect

- `src/inspection/diff.rs`
- `src/repository/read.rs`
- `src-tauri/src/commands/diffs.rs`
- `ui/app/Private/discovery.ts`
- `ui/app/Private/views/inspection.ts`

## Failure modes

- Missing Base: setup guidance replaces the diff.
- Invalid or unavailable Base, Git failure, or non-UTF-8 patch text: the standard error banner reports the failure. Files diff parses this same string, so a non-UTF-8 patch disables both sections identically.
- Unavailable clipboard API: the copy action reports an error instead of claiming success.
