# Current Status

## Implemented

- Rust `git-helper-core` crate with argv-only Git execution, stable non-interactive environment, raw stderr propagation, Git version validation, and per-repository write locking.
- Immutable first-parent Uncommit planning and materialization through a temporary index, `commit-tree`, and one expected-old-SHA `update-ref`, without worktree checkout.
- Editable-range Edit message planning and materialization with preserved trees, parents, authorship, index, and worktree.
- Excluded submodule plans and application for repo-local ignore settings, optional recursion disabling, and non-destructive pre-commit guard installation.
- Quick branch switch with branch-scoped Saved work refs, untracked-file clobber preflight, explicit restoration, listing, and deletion.
- Sync with Base core operation: remote-tracking fetch, tracked Saved work backup, merge/reapply conflict classification, oplog phases, and explicit resume.
- Split branch in Copy mode: selected paths copied onto a new branch rooted at the merge base, built in a temporary detached worktree, with Unity `.meta` companions resolved by the planner and the source branch left untouched.
- Explicit force-push handoff for rewritten branches, using the observed remote SHA in `--force-with-lease` and stale-plan checks.
- Operation recording under `.git/githelper/oplog.json`.
- Read-only recovery history API with ref-only recovery commands for reversible operations.
- Integration fixtures covering straight histories, merge parents, teammate merges, dropped commits, repeated paths, gitlinks, stale plans, literal special-character paths, worktree preservation, staged-change preservation, and resulting tree contents.
- Vite + Tauri desktop workbench in `src-tauri/` loading a modular vanilla UI, with native dialog permissions and an honest browser-unavailable state.
- Typed inspection APIs for overview, remote Base choices, changed paths, editable commits, local branches, and gitlinks; Base is persisted as `githelper.base` with no guessed fallback.
- Transactional repository opening and a review-safe Tauri adapter covering uncommit, edit message, exclusion, split branch, quick switch, sync/resume, Saved work restore/delete, and explicit force push.

## Not implemented

- Submodule cleanup chaining, backup-ref garbage collection, and desktop smoke/accessibility automation remain follow-up hardening.
- Split branch Move mode and hunk-level splitting are deliberately out of scope.
