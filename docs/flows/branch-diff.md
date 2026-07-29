# Branch diff

## Trigger

The user opens **Inspection → Branch diff**, refreshes while it is open, or changes Base.

## Sequence

1. `AppController.setView` selects `inspection` and cancels any pending write review.
2. `loadViewData` reads Base from the repository snapshot. Without Base, it renders setup guidance and sends no Git request.
3. `generate_branch_diff` validates the Base ref and delegates through the open `GitRepository`.
4. `inspection::diff` runs a read-only `git diff Base...HEAD`. Stable options disable color, external diff/textconv, rename collapsing, relative paths, submodule hiding, and configurable prefixes.
5. The UI renders the returned patch. **Copy diff** sends that exact state value to the clipboard and announces success.

## Reads and writes

- Reads: configured Base, HEAD, their merge base, and committed trees.
- Git writes: none. Index and working-tree changes are intentionally excluded.
- Other side effect: the explicit copy action replaces clipboard text.

## Files to inspect

- `src/inspection/diff.rs`
- `src/repository/read.rs`
- `src-tauri/src/commands/actions.rs`
- `ui/app/Private/discovery.js`
- `ui/app/Private/views/inspection.js`

## Failure modes

- Missing Base: setup guidance replaces the diff.
- Invalid or unavailable Base, Git failure, or non-UTF-8 patch text: the standard error banner reports the failure.
- Unavailable clipboard API: the copy action reports an error instead of claiming success.
