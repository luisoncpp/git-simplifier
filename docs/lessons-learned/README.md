# Lessons Learned

Knowledge that helps future development: effective strategies, counter-intuitive facts, and patterns worth remembering across the codebase.

## When to Add

- When a strategy that seemed right turned out to be wrong or suboptimal.
- When something counter-intuitive was discovered through experimentation.
- When a workaround for external dependency behavior was needed and the reason isn't obvious from code.
- When a pattern proved effective and worth formalizing.

## How to Add

Create a new file in this directory named after the topic (e.g., `quill-bounds-always-relative-to-container.md`, `optimistic-ui-pattern-for-toggle-sync.md`). Then add it to the index below.

The entry should answer: **what is counter-intuitive or effective that I should know before starting similar work?**

Avoid: "bug description + fix". Prefer: "what I learned that applies to future work."

## Index

| File | Topic | Date |
|------|-------|------|
| [git-log-format-separators-keep-line-terminators.md](./git-log-format-separators-keep-line-terminators.md) | Custom `git log` separators do not suppress Git's surrounding line terminators | 2026-07-27 |
| [delegated-async-ui-events-need-error-boundaries.md](./delegated-async-ui-events-need-error-boundaries.md) | Delegated async UI events need local error handling or rejected discovery makes controls look inert | 2026-07-27 |
| [operation-reviews-must-mirror-composite-writes.md](./operation-reviews-must-mirror-composite-writes.md) | Composite operation reviews must show their full ordered Git write sequence | 2026-07-27 |
| [superproject-sync-must-disable-submodule-recursion.md](./superproject-sync-must-disable-submodule-recursion.md) | A superproject snapshot must ignore nested dirt while every worktree mutation explicitly disables submodule recursion | 2026-07-27 |
| [write-lock-owners-must-not-relock.md](./write-lock-owners-must-not-relock.md) | A standalone write deadlocks if an outer transaction lock wraps `GitRunner::run` | 2026-07-27 |
| [returned-snapshots-must-be-reused.md](./returned-snapshots-must-be-reused.md) | Re-fetching after a mutation already returned fresh state repeats expensive Git scans | 2026-07-27 |
| [tauri-sync-commands-block-window-thread.md](./tauri-sync-commands-block-window-thread.md) | Awaiting a synchronous Tauri command does not keep Git work off the desktop window thread | 2026-07-27 |
| [webview-window-build-needs-async-command.md](./webview-window-build-needs-async-command.md) | Building a `WebviewWindow` from a sync command deadlocks WebView2 (blank, unclosable) | 2026-07-31 |
| [stdin-git-output-must-be-piped.md](./stdin-git-output-must-be-piped.md) | Git commands fed through stdin need explicit stdout/stderr pipes or their successful output escapes the runner | 2026-07-27 |
| [submodule-ignore-all-hides-changes-from-diff-cached.md](./submodule-ignore-all-hides-changes-from-diff-cached.md) | `submodule.<name>.ignore = all` hides the submodule from `git diff --cached` too, so the obvious pre-commit guard silently never fires | 2026-07-27 |
| [path-limited-index-reset-preserves-staged-work.md](./path-limited-index-reset-preserves-staged-work.md) | A path-limited index reset is required after a no-worktree rewrite so unrelated staged changes survive | 2026-07-27 |
| [git-path-hooks-returns-directory.md](./git-path-hooks-returns-directory.md) | `git rev-parse --git-path hooks` returns a directory, so append the hook filename before filesystem access | 2026-07-27 |
| [failed-mutations-require-state-refresh.md](./failed-mutations-require-state-refresh.md) | A failed reviewed mutation may consume its plan and leave recoverable repository state, so the UI must refresh before presenting the error | 2026-07-27 |
| [failed-repository-open-must-return-error.md](./failed-repository-open-must-return-error.md) | A failed `open_path` that keeps the old repo must still return `Err`, or snapshot reports the wrong path as success | 2026-07-28 |
| [recorded-phases-need-recovery-transitions.md](./recorded-phases-need-recovery-transitions.md) | Every durable in-flight phase must map to a safe retry, resume, or explicit inspection path | 2026-07-27 |
| [review-commands-must-be-derived-from-the-plan.md](./review-commands-must-be-derived-from-the-plan.md) | A hand-written review command silently lies; derive every line from the plan being applied | 2026-07-28 |
| [rerendered-forms-need-state-backed-values.md](./rerendered-forms-need-state-backed-values.md) | Full re-render is safe only when no user intent lives in the DOM — and the textarea still needs an exception | 2026-07-28 |
| [oplog-timestamps-are-nanosecond-strings.md](./oplog-timestamps-are-nanosecond-strings.md) | Oplog times are nanosecond epoch strings and ids embed them; both need formatting before display | 2026-07-28 |
| [git-paths-are-prefix-relative-unless-pinned.md](./git-paths-are-prefix-relative-unless-pinned.md) | Listed names and `:(literal)` pathspecs disagree below the Git root; pin both with `--no-relative` and `:(top,literal)` | 2026-07-28 |
| [created-refs-need-an-absent-marker-for-recovery.md](./created-refs-need-an-absent-marker-for-recovery.md) | A create-only operation looks irreversible unless the new ref is recorded with an empty previous value | 2026-07-28 |
| [windowless-builds-need-both-app-and-child-fixes.md](./windowless-builds-need-both-app-and-child-fixes.md) | Hiding the console needs a fix in the entry point *and* in every Git spawn; the second is invisible until the first lands | 2026-07-28 |
| [close-to-tray-prevent-close-blocks-quit.md](./close-to-tray-prevent-close-blocks-quit.md) | `prevent_close` on hide-to-tray also blocks Quit unless exit is armed first | 2026-07-28 |
| [async-menu-actions-should-dismiss-before-await.md](./async-menu-actions-should-dismiss-before-await.md) | Async menu choices should close before awaiting and project their target onto the stable parent control | 2026-07-28 |
| [failed-stash-apply-can-mutate-before-error.md](./failed-stash-apply-can-mutate-before-error.md) | A failed indexed stash apply may already have created conflicts, so a fallback must not run blindly | 2026-07-28 |
| [absolutely-positioned-live-regions-need-an-inset.md](./absolutely-positioned-live-regions-need-an-inset.md) | A clipped live region can still extend the document from its static position unless it has an explicit inset | 2026-07-28 |
| [explicit-diff-prefixes-do-not-disable-noprefix.md](./explicit-diff-prefixes-do-not-disable-noprefix.md) | Explicit patch prefixes still need `diff.noprefix=false` to defeat repository config | 2026-07-28 |
| [node-type-stripping-needs-erasable-imports.md](./node-type-stripping-needs-erasable-imports.md) | Node runs `.ts` test imports by stripping, so type-only imports need `import type` and explicit `.ts` extensions — enforced by `verbatimModuleSyntax` + `erasableSyntaxOnly` | 2026-07-29 |
| [unified-patch-parsing-has-two-load-bearing-rules.md](./unified-patch-parsing-has-two-load-bearing-rules.md) | `str::lines()` eats CRLF, and a hunk must end at its declared counts because content lines can look structural | 2026-07-29 |
| [whole-shell-rerender-fights-scroll-into-view.md](./whole-shell-rerender-fights-scroll-into-view.md) | `renderInto` restores scroll synchronously after the swap, so a jump must scroll after `render()` — and a stale node makes working code look broken | 2026-07-29 |
| [browser-only-deps-need-a-lazy-global.md](./browser-only-deps-need-a-lazy-global.md) | A browser-only library needs a document-gated dynamic import, a global published before its plugins, and a test that keeps it out of the bundler-free test graph | 2026-07-29 |
| [base-chooser-must-refresh-when-editing.md](./base-chooser-must-refresh-when-editing.md) | A configured Base skips initial choice discovery, so editing Base must reload remote-tracking choices before rendering the selector | 2026-07-30 |
| [merged-branches-have-no-unique-commits.md](./merged-branches-have-no-unique-commits.md) | `Base..branch` is empty for exactly the branches a cleanup targets, so "who authored its commits" is vacuous | 2026-08-01 |
| [strip-n-counts-the-whole-refname.md](./strip-n-counts-the-whole-refname.md) | `%(refname:strip=N)` counts the branch name as a component, and the usual empty-line filter then hides the damage | 2026-08-01 |
| [untracked-file-birth-time-fallback.md](./untracked-file-birth-time-fallback.md) | Untracked Local diff uses birth-or-mtime vs HEAD `%ct`; maximal untracked via ls-files union | 2026-08-04 |
| [local-untracked-list-must-not-read-ignore-tree-bodies.md](./local-untracked-list-must-not-read-ignore-tree-bodies.md) | Local untracked list stubs gitignored/`node_modules` bodies or ignore trees empty the view | 2026-08-04 |
| [untracked-filters-must-constrain-ls-files.md](./untracked-filters-must-constrain-ls-files.md) | Untracked filters must constrain `ls-files`, not post-filter a maximal ignored walk | 2026-08-04 |
| [work-parked-outside-app-owned-refs-is-invisible-work.md](./work-parked-outside-app-owned-refs-is-invisible-work.md) | Work left on the shared stash stack reads as "No Saved work"; park it on a ref the listing already reads | 2026-08-07 |
| [opening-a-repository-must-fetch-like-refresh.md](./opening-a-repository-must-fetch-like-refresh.md) | Opening another repository must fetch remotes the same way Refresh does | 2026-08-11 |
| [git-errors-must-surface-stderr-and-open-must-probe.md](./git-errors-must-surface-stderr-and-open-must-probe.md) | Command errors must include stderr; open must probe the worktree before swapping sessions | 2026-08-11 |
