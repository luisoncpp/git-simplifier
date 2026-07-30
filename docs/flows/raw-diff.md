# Raw diff

The text half of Inspection. For the per-file viewer over the same patch, see [files-diff.md](./files-diff.md).

## Trigger

The user opens **Inspection → Raw diff**, refreshes while it is open, or changes Base.

## Sequence

1. `AppController.setView` selects `raw-diff` and cancels any pending write review. `isInspectionView` is what admits it, because the group holds two views.
2. `loadViewData` dispatches on the view id and calls `loadBranchDiff`, so opening Raw diff never fetches the structured diff. It reads Base from the repository snapshot; without Base it renders setup guidance and sends no Git request.
3. `generate_branch_diff` validates the Base ref and delegates through the open `GitRepository`.
4. `inspection::diff` runs a read-only `git diff Base...HEAD` built by the shared `diff_args`. Stable options disable color, external diff/textconv, rename collapsing, relative paths, submodule hiding, and configurable prefixes.
5. The UI renders the returned patch. **Copy diff** sends that exact state value to the clipboard and announces success.

## Reads and writes

- Reads: configured Base, HEAD, their merge base, and committed trees.
- Git writes: none. Index and working-tree changes are intentionally excluded.
- Other side effect: the explicit copy action replaces clipboard text.

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
