# Workbench UI

The UI is a single deep module written in strict TypeScript (`tsc --noEmit` is the lint). `ui/app/index.ts` is the only public interface (`AppController`, `QuickFileDiffApp`, `renderShell`, the bridges); everything else lives under `ui/app/Private/` and must not be imported from outside it. Imports carry explicit `.ts` extensions and type-only imports use `import type`, so Node's type stripping can run the sources directly in tests without a build step.

| File | Responsibility |
|------|----------------|
| `Private/types.ts` | Wire shapes mirroring the Rust contracts, `AppState`, `Draft`, the `Bridge` interface, and the `__TAURI__` global |
| `Private/state.ts` | `createState`: the initial `AppState` (mirrors `createDraft`) |
| `Private/controller.ts` | `AppController`: owns state, busy/error handling, and the prepare → apply → cancel boundary |
| `Private/repository-switcher.ts` | Recent repository menu: filter, open, remove, and persistence refresh |
| `Private/path-diff-menu.ts` | Path-list context menu and `open_file_diff_window` invoke |
| `Private/quick-file-diff/` | Nested deep module: secondary-window single-file diff app. Only its `index.ts` may be imported |
| `Private/draft/` | Nested deep module: draft shape and derived reads (visible paths, selected commit, message drafts). Only its `index.ts` may be imported |
| `Private/operations/` | Nested deep module: operation catalog, request builders, and `submitState`. Only its `index.ts` may be imported |
| `Private/event-tables.ts` | Delegated event dispatch tables keyed by `data-event` |
| `Private/events.ts` | Delegated `click`/`change`/`input`/`keydown` binding and handler routing |
| `Private/selection.ts` | Draft mutations (path selection, message drafts, flags) plus the targeted patches they need |
| `Private/discovery.ts` | Snapshot reload and per-operation discovery; drops selections that no longer exist |
| `Private/snapshot.ts` | Typed reads over the Rust snapshot, including human sync-phase labels |
| `Private/dom.ts` | HTML escaping and `renderInto`, which preserves caret and scroll across a re-render |
| `Private/views/repo-menu.ts` | Rail repository picker, filterable recent list, and the reveal-in-explorer context menu |
| `Private/views/branch-picker.ts` | Searchable Quick switch branch menu (local + remote-only) |
| `Private/branch-switcher.ts` | Branch menu open/filter/pick keyboard handlers |
| `Private/views/inspection.ts` | Shared Inspection chrome, Raw diff presentation, and the clipboard action |
| `Private/files-diff/` | Nested deep module: the structured per-file diff surface (unified/side-by-side, gap expansion, file navigator, Prism adapter, `singleFileDiff`). Only its `index.ts` may be imported |
| `Private/views/path-list.ts` | The changed-path checklist, shared by Uncommit, Revert, and Split branch |
| `Private/views/*` | Pure functions from state to markup |

## State rules

- **Discovery data is never mixed with user intent.** `state.paths`, `state.commits`, `state.branches`, `state.submodules`, and `state.dirtySubmodules` come from Rust; `state.draft` holds what the user picked. A refresh replaces the former and reconciles the latter, so a selection that disappeared from the repository cannot be sent back. Quick switch draft fields include `pullAfterSwitch` (default on), `carryChanges` (default on), and `createFromRemote` when the chosen row is remote-only. The initial target prefers Base (local same-named branch, else the remote-only row); `draft.branchPicked` is set only when the user picks a row, so an alphabetical fallback does not stick across Base changes or refresh.
- **Refresh fetches before it reloads.** The repo-bar **Refresh** button runs `git fetch --all --no-tags --no-recurse-submodules` with no review, then reloads the snapshot and discovery data. A failed fetch sets `state.warning` and shows a dismissible banner; the local reload still runs. Opening another repository runs the same fetch-then-reload sequence for that path, so an unreachable remote is visible immediately after a switch rather than only after an explicit Refresh.
- **Base choices are loaded when the chooser opens.** Initial discovery loads `state.baseChoices` only when the repository has no configured Base; changing an existing Base therefore explicitly refreshes the remote-tracking choices before rendering the selector. The chooser never treats an empty, not-yet-loaded list as proof that no remote exists.
- **Recent repositories are app preference, not Git state.** `state.recentRepositories` is a list of paths loaded from the desktop app data file; it is never written into `.git`. Opening a repository promotes its path; remove only drops the preference entry.
- **Skip review is app preference, not Git state.** `state.skipReview` is loaded from `ui-preferences.json` on start and persisted when the repo-bar **Review | Skip** toggle changes. Default is **Review**. In **Skip**, primary actions read **Apply …** and `prepare` chains straight into `apply_operation` without rendering the review pane; prepare still runs so validation and the pending plan boundary stay intact.
- **An in-flight repository choice is visible intent.** `state.repoOpeningPath` temporarily supplies the repository picker's name and path after the menu closes and before the new snapshot arrives. It is cleared after success or failure, so a successful snapshot takes over and a failed open visibly returns to the previous repository.
- **Every form control is state-backed.** Re-rendering therefore cannot lose a typed message, a filter query, or a path selection, and cancelling a review returns the user to the exact selection they had.
- **A message draft is per commit** (`draft.messages` keyed by commit id). Changing the selected commit shows that commit's message; it never carries text from another commit into a rewrite.
- **The Editable range is presented newest first.** Rust returns it oldest first for planning; the commit a user rewords is almost always the newest one.
- **A path selection belongs to one operation.** Uncommit, Revert, and Split branch can all show a path checklist, but each keeps its own set (`selectedPaths`, `revertPaths`, `splitPaths`) and `pathSetFor` picks by operation. Sharing one set would let a selection made for one operation arrive pre-ticked in another. Revert’s discovery list is wider than Uncommit’s: tracked local dirt unioned with `Base...HEAD`.
- `state.outcome` is cleared when the operation changes, when a new review is prepared, and when another repository is opened, so a result banner can never describe stale work.
- `state.branchDiff` is read-only discovery data. Entering Inspection, refreshing, changing Base, or opening another repository regenerates it from Rust; the UI never reconstructs a Git command.
- **The Inspection group is two views, not one.** `ViewId` has no `"inspection"` member: `"files-diff"` and `"raw-diff"` are siblings and every gate asks `isInspectionView`. Leaving one of the two named after the group would guarantee a future bug, and `loadViewData` gates *per view* so entering one section never pays for the other's Git work.
- **The structured diff is two discovery collections and one intent object.** `state.fileDiffs` (the context-3 diff) is never replaced by an expanded version, which is what keeps every gap index and anchor id stable; `state.fileDiffsFull` is the per-path full-context cache, and its *presence* is what stops a second Rust call rather than a separate flag; `state.diffView` holds the compare mode, layout, the collapsed set, the per-gap reveals, and the navigator.
- **Layout, compare, untracked filters, and navigator state are session preference.** `resetFileDiffs` clears the diffs, the cache, the collapsed set, and the reveals, but never the layout, compare mode, untracked filter toggles, or navigator — a refresh or a Base change must not silently undo a choice the user made, the same reasoning that keeps `draft.pullAfterSwitch` sticky.
- **Local untracked filters are discovery queries.** `generate_files_diff` in Local mode receives the five toggles and constrains `ls-files` / body synthesis before the search — ignored trees are not walked when Respect gitignore is on. `visibleFileDiffs` remains a client-side guard. Tracked entries never carry `untracked` and always render. The toggles default on (checked = filter active); flipping one reloads the Local list. Gitignored and `node_modules` entries arrive as incomplete stubs when a wider query includes them, until `ensureFullDiff` hydrates.
- **Files diff keeps its own `collapsed` set** instead of sharing `state.expanded` with Recovery. Every file starts open, so default-open maps naturally onto an empty set, whereas `state.expanded` is keyed by oplog id and defaults closed. Consolidating the two would need prefixed keys and inverted defaults for no gain.
- A wholly added or wholly deleted file has no gaps: its patch already contains every line, so it must not offer an expander.
- **Quick file diff does not share Inspection diff state.** The secondary window owns its own `DiffViewState` and loads via `generate_full_file_diff`; `state.fileDiffs` / `fileDiffsFull` / `diffView.reveals` stay Inspection-only so a quick view cannot disturb the multi-file surface. Rendering is shared through `singleFileDiff` / `fileContent`.
- **Saved work apply diff reuses the multi-file pane, not Inspection state.** `files-diff/pane.ts` renders the file list for both Inspection and the `saved-work-diff` secondary window. That window loads tree-to-tree diffs from session-held `before_tree` / `after_tree` OIDs; gap expand calls `generate_saved_work_full_file_diff`.
- **Cleanup's three toggles are display filters, not queries.** `list_cleanup_branches` returns the maximal eligible set once, annotated with `mine`, `kind`, `protected`, and the remote counterpart; `cleanupChoices` narrows it in the browser. That keeps the `discoveryFor` contract — which loads once on operation select and only knows `(bridge, base)` — and makes flipping a toggle free. A remote-only row needs *both* "check all remote branches" and "also delete on its remote", because deleting one is itself a remote deletion.
- **Cleanup selection is `draft.cleanupOverrides`, a `Map` of explicit choices only.** A row is ticked when `overrides.get(ref) ?? !choice.protected`. One field expresses both "everything pre-ticked" and "shared names start unticked", and the filters can change the visible set with no reseeding and no "has the user touched this yet" flag — a positive `Set` would need reseeding on every filter change, and an inverted one could not express the protected exception. It is deliberately outside `pathSetFor`, which is for positive path sets.
- **Path context menus are operation-scoped.** Right-click on a path row only offers **View diff** for Uncommit, Revert, and Split branch (`pathDiffRequest`); other operations keep the browser default.
- **Submodules is a two-column tab with per-column submit.** The left column is **Exclude submodule** (standing rule); the right is **Cleanup dirty submodules** (dirty gitlink checklist with **Uncommit from Base…HEAD** and **Revert**, both on by default). The shared `#submit-row` is hidden; `excludeSubmitState` and `cleanupSubmitState` drive `submit-exclude-submodule` and `submit-cleanup-submodules`. Discovery loads `list_submodules` and `list_dirty_submodules` together; adopting dirty paths pre-ticks every entry in `draft.cleanupSubmodulePaths`.

## Rendering

`renderInto` replaces the shell markup, then restores the caret of the element carrying `data-focus` and the scroll offset of every `[data-scroll]` container. Keys are compared as dataset values, not interpolated into selectors, because they can contain repository paths.

Two things are patched instead of re-rendered: the commit-message textarea keeps its native undo stack, so typing only refreshes `#message-tools` and `#submit-row`. The optional Split branch message is simpler — nothing depends on it, so typing records the value and skips the render entirely.

The rail reserves an **Inspection** group for read-only tools, in the order it lists them: **Files diff** renders the same patch per file with line numbers, syntax highlighting, and expandable context — **HEAD** (merge base → HEAD) or **Local** (merge base → working tree) — and **Raw diff** renders it as text and copies it through the platform clipboard in one action. Both views share a **HEAD / Local** compare toggle beside the Files diff layout controls.

Two rendering rules the Files diff depends on:

- **A jump must scroll after `render()`.** `renderInto` restores every `[data-scroll]` offset *synchronously* after the markup swap, so a scroll performed before or during a render loses the race — and an `href="#file-3"` fragment both fights that restore and pushes a history entry. `jumpToFile` therefore renders first, then calls `dom.ts`'s `revealByDataset`, which finds the card by dataset value for the same reason focus and scroll restoration do: keys carry repository paths.
- **Anchor ids come from the array index, never the path**, which may contain slashes, spaces, quotes, and non-ASCII. The path travels alongside as `data-file`.

`prismjs` is the project's only runtime dependency and is confined to `files-diff/highlight.ts`. The core and every grammar load through `import()` behind a `globalThis.document` check, so the bundler-free test runner never resolves the package and highlighting degrades to escaped plain text; the core is published to `globalThis.Prism` before any grammar runs, because the component files bind a free `Prism` through the global. Prism's output is **already escaped HTML** — every branch of `highlightCode` returns safe markup, and callers must concatenate it raw rather than passing it through `esc` again. Tokenizing is per line, so a construct spanning several lines (a block comment, a template literal) is coloured wrongly; whole-file tokenizing is incompatible with a windowed view. The tint of a changed row is carried entirely by its background and its `+`/`−` glyph, leaving every foreground colour to Prism — a `color` on the row would either lose to the tokens or invite an `!important` that smothers highlighting exactly where it matters most.

## Disabled controls always say why

`submitState` returns a reason, and the submit row renders it. The submit label uses **Review** or **Apply** from `review-mode.ts` according to `skipReview`, then a per-operation word list whose fallback is the operation id, so a new operation that forgets its entry shows `Review split_branch` instead of `Review split`; a test asserts no label contains an underscore. A base-dependent operation with no Base, an unchanged message, an empty path selection, and a branch without an upstream each explain themselves instead of showing an inert primary button.

## Follow-up offers

A result banner may offer exactly one follow-up. `offer_force_push` is a flag because it always means the current branch; `offer_publish_branch` carries the branch name, because a newly created branch is not checked out and a boolean could not say which one to push. The two are different operations with different risk, so the banner never shows force-push wording for a first push. `offer_restore_saved_work` is also a flag for the current branch: Quick switch (and a finished pull resolution) sets it when that branch already has Saved work, and the button opens the existing restore review — never auto-applies. `has_warning` forces a warn-tone result banner when carry, pull resolution, or restore left conflict details, so the headline and tone do not read as success.

## Saved work notice

When the current branch has Saved work and no result banner already offers restore (and no pull decision is pending), a persistent banner offers **Review restore**. That covers repo open, refresh, and dismissing the switch result. Restoration is never automatic.

Each Saved work row also offers **Diff**, which opens a secondary multi-file window previewing the net worktree change restore would apply (merge-tree simulation, conflict-aware). Other-branch rows preview apply onto that branch's tip tree, not onto the current checkout.

## Review surface

A pending review renders as a second column beside the form (stacked below 860 CSS pixels) rather than replacing it. Focus moves to the review title when it opens; Escape cancels it, which also releases the plan held in `AppState`. Switching operation or section cancels a pending review instead of abandoning it. **Skip** mode never opens this pane: the same prepare boundary runs, then apply consumes the plan immediately.
