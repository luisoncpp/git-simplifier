# Workbench UI

The UI is a single deep module written in strict TypeScript (`tsc --noEmit` is the lint). `ui/app/index.ts` is the only public interface (`AppController`, `renderShell`, the bridges); everything else lives under `ui/app/Private/` and must not be imported from outside it. Imports carry explicit `.ts` extensions and type-only imports use `import type`, so Node's type stripping can run the sources directly in tests without a build step.

| File | Responsibility |
|------|----------------|
| `Private/types.ts` | Wire shapes mirroring the Rust contracts, `AppState`, `Draft`, the `Bridge` interface, and the `__TAURI__` global |
| `Private/state.ts` | `createState`: the initial `AppState` (mirrors `createDraft`) |
| `Private/controller.ts` | `AppController`: owns state, busy/error handling, and the prepare → apply → cancel boundary |
| `Private/repository-switcher.ts` | Recent repository menu: filter, open, remove, and persistence refresh |
| `Private/events.ts` | Delegated `click`/`change`/`input`/`keydown` dispatch tables keyed by `data-event` |
| `Private/selection.ts` | Draft mutations (path selection, message drafts, flags) plus the targeted patches they need |
| `Private/discovery.ts` | Snapshot reload and per-operation discovery; drops selections that no longer exist |
| `Private/draft.ts` | Draft shape and the derived reads over it (visible paths, selected commit, message drafts) |
| `Private/operations.ts` | Operation catalog, request builders, and `submitState` |
| `Private/snapshot.ts` | Typed reads over the Rust snapshot, including human sync-phase labels |
| `Private/dom.ts` | HTML escaping and `renderInto`, which preserves caret and scroll across a re-render |
| `Private/views/repo-menu.ts` | Rail repository picker and filterable recent list |
| `Private/views/branch-picker.ts` | Searchable Quick switch branch menu (local + remote-only) |
| `Private/branch-switcher.ts` | Branch menu open/filter/pick keyboard handlers |
| `Private/views/inspection.ts` | Shared Inspection chrome, Raw diff presentation, and the clipboard action |
| `Private/files-diff/` | Nested deep module: the structured per-file diff surface (unified/side-by-side, gap expansion, file navigator, Prism adapter). Only its `index.ts` may be imported |
| `Private/views/path-list.ts` | The changed-path checklist, shared by Uncommit and Split branch |
| `Private/views/*` | Pure functions from state to markup |

## State rules

- **Discovery data is never mixed with user intent.** `state.paths`, `state.commits`, `state.branches`, and `state.submodules` come from Rust; `state.draft` holds what the user picked. A refresh replaces the former and reconciles the latter, so a selection that disappeared from the repository cannot be sent back. Quick switch draft fields include `pullAfterSwitch` (default on) and `createFromRemote` when the chosen row is remote-only.
- **Recent repositories are app preference, not Git state.** `state.recentRepositories` is a list of paths loaded from the desktop app data file; it is never written into `.git`. Opening a repository promotes its path; remove only drops the preference entry.
- **An in-flight repository choice is visible intent.** `state.repoOpeningPath` temporarily supplies the repository picker's name and path after the menu closes and before the new snapshot arrives. It is cleared after success or failure, so a successful snapshot takes over and a failed open visibly returns to the previous repository.
- **Every form control is state-backed.** Re-rendering therefore cannot lose a typed message, a filter query, or a path selection, and cancelling a review returns the user to the exact selection they had.
- **A message draft is per commit** (`draft.messages` keyed by commit id). Changing the selected commit shows that commit's message; it never carries text from another commit into a rewrite.
- **The Editable range is presented newest first.** Rust returns it oldest first for planning; the commit a user rewords is almost always the newest one.
- **A path selection belongs to one operation.** Uncommit and Split branch read the same changed-path list but mean opposite things by a tick — remove this from history versus copy this elsewhere — so each keeps its own set and `pathSetFor` picks by operation. Sharing one set would let a selection made for one operation arrive pre-ticked in the other.
- `state.outcome` is cleared when the operation changes, when a new review is prepared, and when another repository is opened, so a result banner can never describe stale work.
- `state.branchDiff` is read-only discovery data. Entering Inspection, refreshing, changing Base, or opening another repository regenerates it from Rust; the UI never reconstructs a Git command.
- **The Inspection group is two views, not one.** `ViewId` has no `"inspection"` member: `"files-diff"` and `"raw-diff"` are siblings and every gate asks `isInspectionView`. Leaving one of the two named after the group would guarantee a future bug, and `loadViewData` gates *per view* so entering one section never pays for the other's Git work.
- **The structured diff is two discovery collections and one intent object.** `state.fileDiffs` (the context-3 diff) is never replaced by an expanded version, which is what keeps every gap index and anchor id stable; `state.fileDiffsFull` is the per-path full-context cache, and its *presence* is what stops a second Rust call rather than a separate flag; `state.diffView` holds the layout, the collapsed set, the per-gap reveals, and the navigator.
- **Layout and navigator state are session preference.** `resetFileDiffs` clears the diffs, the cache, the collapsed set, and the reveals, but never the layout or the navigator — a refresh or a Base change must not silently undo a choice the user made, the same reasoning that keeps `draft.pullAfterSwitch` sticky.
- **Files diff keeps its own `collapsed` set** instead of sharing `state.expanded` with Recovery. Every file starts open, so default-open maps naturally onto an empty set, whereas `state.expanded` is keyed by oplog id and defaults closed. Consolidating the two would need prefixed keys and inverted defaults for no gain.
- A wholly added or wholly deleted file has no gaps: its patch already contains every line, so it must not offer an expander.

## Rendering

`renderInto` replaces the shell markup, then restores the caret of the element carrying `data-focus` and the scroll offset of every `[data-scroll]` container. Keys are compared as dataset values, not interpolated into selectors, because they can contain repository paths.

Two things are patched instead of re-rendered: the commit-message textarea keeps its native undo stack, so typing only refreshes `#message-tools` and `#submit-row`. The optional Split branch message is simpler — nothing depends on it, so typing records the value and skips the render entirely.

The rail reserves an **Inspection** group for read-only tools, in the order it lists them: **Files diff** renders the same `Base...HEAD` patch per file with line numbers, syntax highlighting, and expandable context, and **Raw diff** renders it as text and copies it through the platform clipboard in one action.

Two rendering rules the Files diff depends on:

- **A jump must scroll after `render()`.** `renderInto` restores every `[data-scroll]` offset *synchronously* after the markup swap, so a scroll performed before or during a render loses the race — and an `href="#file-3"` fragment both fights that restore and pushes a history entry. `jumpToFile` therefore renders first, then calls `dom.ts`'s `revealByDataset`, which finds the card by dataset value for the same reason focus and scroll restoration do: keys carry repository paths.
- **Anchor ids come from the array index, never the path**, which may contain slashes, spaces, quotes, and non-ASCII. The path travels alongside as `data-file`.

`prismjs` is the project's only runtime dependency and is confined to `files-diff/highlight.ts`. The core and every grammar load through `import()` behind a `globalThis.document` check, so the bundler-free test runner never resolves the package and highlighting degrades to escaped plain text; the core is published to `globalThis.Prism` before any grammar runs, because the component files bind a free `Prism` through the global. Prism's output is **already escaped HTML** — every branch of `highlightCode` returns safe markup, and callers must concatenate it raw rather than passing it through `esc` again. Tokenizing is per line, so a construct spanning several lines (a block comment, a template literal) is coloured wrongly; whole-file tokenizing is incompatible with a windowed view. The tint of a changed row is carried entirely by its background and its `+`/`−` glyph, leaving every foreground colour to Prism — a `color` on the row would either lose to the tokens or invite an `!important` that smothers highlighting exactly where it matters most.

## Disabled controls always say why

`submitState` returns a reason, and the submit row renders it. The submit label comes from a per-operation word list whose fallback is the operation id, so a new operation that forgets its entry shows `Review split_branch` instead of `Review split`; a test asserts no label contains an underscore. A base-dependent operation with no Base, an unchanged message, an empty path selection, and a branch without an upstream each explain themselves instead of showing an inert primary button.

## Follow-up offers

A result banner may offer exactly one follow-up. `offer_force_push` is a flag because it always means the current branch; `offer_publish_branch` carries the branch name, because a newly created branch is not checked out and a boolean could not say which one to push. The two are different operations with different risk, so the banner never shows force-push wording for a first push.

## Review surface

A pending review renders as a second column beside the form (stacked below 860 CSS pixels) rather than replacing it. Focus moves to the review title when it opens; Escape cancels it, which also releases the plan held in `AppState`. Switching operation or section cancels a pending review instead of abandoning it.
