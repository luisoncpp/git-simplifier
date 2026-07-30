# Files diff

The per-file viewer over the same patch [raw-diff.md](./raw-diff.md) shows as text.

## Trigger

The user opens **Inspection → Files diff**, refreshes while it is open, changes Base, or switches the **HEAD / Local** compare toggle. Inside the view: the layout toggle, a file's collapse control, a gap expander, and the file navigator.

## Load sequence

1. `AppController.setView` selects `files-diff`, cancels any pending write review, and calls `resetFileDiffs`.
2. `loadViewData` dispatches on the view id and calls `loadFileDiffs`. Without Base it returns before any Git request and the view renders `missingBaseGuidance`.
3. `generate_files_diff` → `GitRepository::files_diff` → `inspection::diff::files_diff`, which parses the string `branch_diff` already returns for the same `compare` mode. One `git diff` process, both surfaces.
4. Any file whose rendered rows exceed `MAX_ROWS_PER_FILE` is added to `collapsed`, so it opens closed rather than making every later re-render slow.
5. `ensureGrammars` loads the Prism core and the grammars the changed paths need — a no-op without a document.
6. `filesDiffView` renders every file expanded, showing only the hunks: the unchanged runs between them stay hidden behind expanders.

## Expansion sequence

1. `expand-gap` carries the path in `data-value` and the gap index and direction in `data-gap`/`data-dir`.
2. `expandGap` awaits `ensureFullDiff` **before** recording the reveal, so no frame ever claims lines it cannot render. A path already in `state.fileDiffsFull` skips the call entirely — presence is the cache.
3. `generate_full_file_diff` returns that one file at full context with `complete: true`, or `null` if it no longer differs from Base.
4. `widenReveal` grows `down`, `up`, or sets `all`. `gapWindow` renders the two revealed blocks and drops the expander once they meet, so an overshooting reveal needs no clamp.

## Navigator sequence

`jump-to-file` deletes the path from `collapsed`, renders, and only then scrolls via `revealByDataset` — `renderInto` restores scroll synchronously after the swap, so an earlier scroll would be undone.

## Reads and writes

- Reads: configured Base, HEAD, their merge base, and — in **HEAD** mode — committed trees; in **Local** mode, the working tree as well.
- Git writes: none.
- Other side effects: none. No clipboard, no persistence — the layout, compare mode, and navigator choices live in `AppState` for the session only.

## Files to inspect

- `ui/app/Private/files-diff/` (`reads.ts` for the gap arithmetic, `rows.ts` for what both layouts share, `highlight.ts` for Prism)
- `ui/app/Private/discovery.ts`, `ui/app/Private/events.ts`, `ui/app/Private/dom.ts`
- `ui/styles/files-diff.css`
- `src/inspection/patch/`, `src/inspection/diff.rs`
- `src-tauri/src/commands/diffs.rs`

## Failure modes

- Missing Base: setup guidance, no Git request.
- Non-UTF-8 patch text: the standard error banner, and Raw diff fails identically — they parse the same string.
- Binary file, or a mode-only change: the card says so instead of rendering rows.
- A grammar that fails to load: the lines render as escaped plain text and no error is surfaced, because highlighting is decoration.
- A very large diff: files past the row cap open collapsed with the reason in view; Raw diff remains the way to read one whole.
- A modified file whose last hunk already reaches EOF still offers a trailing expander, because the file's length is unknown until it is fetched. One click resolves it and the control disappears. Added and deleted files are exempt — their patches already hold every line.
