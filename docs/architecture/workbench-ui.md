# Workbench UI

The UI is a single deep module. `ui/app/index.js` is the only public interface (`AppController`, `renderShell`, the bridges); everything else lives under `ui/app/Private/` and must not be imported from outside it.

| File | Responsibility |
|------|----------------|
| `Private/controller.js` | `AppController`: owns state, busy/error handling, and the prepare → apply → cancel boundary |
| `Private/repository-switcher.js` | Recent repository menu: filter, open, remove, and persistence refresh |
| `Private/events.js` | Delegated `click`/`change`/`input`/`keydown` dispatch tables keyed by `data-event` |
| `Private/selection.js` | Draft mutations (path selection, message drafts, flags) plus the targeted patches they need |
| `Private/discovery.js` | Snapshot reload and per-operation discovery; drops selections that no longer exist |
| `Private/draft.js` | Draft shape and the derived reads over it (visible paths, selected commit, message drafts) |
| `Private/operations.js` | Operation catalog, request builders, and `submitState` |
| `Private/snapshot.js` | Typed reads over the Rust snapshot, including human sync-phase labels |
| `Private/dom.js` | HTML escaping and `renderInto`, which preserves caret and scroll across a re-render |
| `Private/views/repo-menu.js` | Rail repository picker and filterable recent list |
| `Private/views/path-list.js` | The changed-path checklist, shared by Uncommit and Split branch |
| `Private/views/*` | Pure functions from state to markup |

## State rules

- **Discovery data is never mixed with user intent.** `state.paths`, `state.commits`, `state.branches`, and `state.submodules` come from Rust; `state.draft` holds what the user picked. A refresh replaces the former and reconciles the latter, so a selection that disappeared from the repository cannot be sent back.
- **Recent repositories are app preference, not Git state.** `state.recentRepositories` is a list of paths loaded from the desktop app data file; it is never written into `.git`. Opening a repository promotes its path; remove only drops the preference entry.
- **Every form control is state-backed.** Re-rendering therefore cannot lose a typed message, a filter query, or a path selection, and cancelling a review returns the user to the exact selection they had.
- **A message draft is per commit** (`draft.messages` keyed by commit id). Changing the selected commit shows that commit's message; it never carries text from another commit into a rewrite.
- **The Editable range is presented newest first.** Rust returns it oldest first for planning; the commit a user rewords is almost always the newest one.
- **A path selection belongs to one operation.** Uncommit and Split branch read the same changed-path list but mean opposite things by a tick — remove this from history versus copy this elsewhere — so each keeps its own set and `pathSetFor` picks by operation. Sharing one set would let a selection made for one operation arrive pre-ticked in the other.
- `state.outcome` is cleared when the operation changes, when a new review is prepared, and when another repository is opened, so a result banner can never describe stale work.

## Rendering

`renderInto` replaces the shell markup, then restores the caret of the element carrying `data-focus` and the scroll offset of every `[data-scroll]` container. Keys are compared as dataset values, not interpolated into selectors, because they can contain repository paths.

Two things are patched instead of re-rendered: the commit-message textarea keeps its native undo stack, so typing only refreshes `#message-tools` and `#submit-row`. The optional Split branch message is simpler — nothing depends on it, so typing records the value and skips the render entirely.

## Disabled controls always say why

`submitState` returns a reason, and the submit row renders it. The submit label comes from a per-operation word list whose fallback is the operation id, so a new operation that forgets its entry shows `Review split_branch` instead of `Review split`; a test asserts no label contains an underscore. A base-dependent operation with no Base, an unchanged message, an empty path selection, and a branch without an upstream each explain themselves instead of showing an inert primary button.

## Follow-up offers

A result banner may offer exactly one follow-up. `offer_force_push` is a flag because it always means the current branch; `offer_publish_branch` carries the branch name, because a newly created branch is not checked out and a boolean could not say which one to push. The two are different operations with different risk, so the banner never shows force-push wording for a first push.

## Review surface

A pending review renders as a second column beside the form (stacked below 860 CSS pixels) rather than replacing it. Focus moves to the review title when it opens; Escape cancels it, which also releases the plan held in `AppState`. Switching operation or section cancels a pending review instead of abandoning it.
