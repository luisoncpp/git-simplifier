# Quick file diff

One-file diff in a secondary Tauri window, opened from Uncommit / Revert / Split path lists. Reuses the Files Diff tables; does not share Inspection state.

## Trigger

Right-click a path row → context menu **View diff**.

## Sequence

1. `pathDiffRequest` picks compare: Uncommit/Split → `head` (no toggle); Revert → `local` with HEAD/Local toggle.
2. Main UI invokes `open_file_diff_window` with path, base, compare, and toggle flag.
3. Rust stores a `FileDiffSession`, creates or focuses `file-diff-<path-hash>`, title = path.
4. `file-diff.html` boots `QuickFileDiffApp`, calls `file_diff_session`, then `generate_full_file_diff`.
5. Renders via `singleFileDiff` (same tables/highlighting as Inspection). Layout toggle always; compare toggle only when requested.
6. Closing the window destroys it (not hide-to-tray) and drops the session. Re-open focuses and emits `file-diff-reload`.

## Reads / writes

- Reads: Base, merge base, and the chosen tip (`HEAD` or working tree) for one path.
- Git writes: none.
- Shell: secondary `WebviewWindow` only.

## Files to inspect

- `ui/app/Private/path-diff-menu.ts`, `quick-file-diff/`, `files-diff/single.ts`
- `ui/file-diff.html`, `ui/file-diff-app.ts`
- `src-tauri/src/file_diff_window.rs`, `tray.rs` (main-only hide)
- `src-tauri/capabilities/default.json` (`file-diff-*`)

## Failure modes

| Symptom | Likely cause |
|---------|----------------|
| Menu does nothing | Operation is not Uncommit/Revert/Split, or Base is unset |
| Window opens empty / errors | No open repository, or path no longer differs (`null` full diff) |
| Window blank and X does nothing | `open_file_diff_window` ran as a sync command (WebView2 deadlock on Windows) |
| Close hides instead of destroying | Hide-to-tray applied to a non-`main` label |
| Capabilities deny invoke | Window label not under `file-diff-*` in `default.json` |
