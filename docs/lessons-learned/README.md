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
| [operation-reviews-must-mirror-composite-writes.md](./operation-reviews-must-mirror-composite-writes.md) | Composite operation reviews must show their full ordered Git write sequence | 2026-07-27 |
| [superproject-sync-must-disable-submodule-recursion.md](./superproject-sync-must-disable-submodule-recursion.md) | A superproject snapshot must ignore nested dirt while every worktree mutation explicitly disables submodule recursion | 2026-07-27 |
| [write-lock-owners-must-not-relock.md](./write-lock-owners-must-not-relock.md) | A standalone write deadlocks if an outer transaction lock wraps `GitRunner::run` | 2026-07-27 |
| [returned-snapshots-must-be-reused.md](./returned-snapshots-must-be-reused.md) | Re-fetching after a mutation already returned fresh state repeats expensive Git scans | 2026-07-27 |
| [tauri-sync-commands-block-window-thread.md](./tauri-sync-commands-block-window-thread.md) | Awaiting a synchronous Tauri command does not keep Git work off the desktop window thread | 2026-07-27 |
| [stdin-git-output-must-be-piped.md](./stdin-git-output-must-be-piped.md) | Git commands fed through stdin need explicit stdout/stderr pipes or their successful output escapes the runner | 2026-07-27 |
| [submodule-ignore-all-hides-changes-from-diff-cached.md](./submodule-ignore-all-hides-changes-from-diff-cached.md) | `submodule.<name>.ignore = all` hides the submodule from `git diff --cached` too, so the obvious pre-commit guard silently never fires | 2026-07-27 |
| [path-limited-index-reset-preserves-staged-work.md](./path-limited-index-reset-preserves-staged-work.md) | A path-limited index reset is required after a no-worktree rewrite so unrelated staged changes survive | 2026-07-27 |
| [git-path-hooks-returns-directory.md](./git-path-hooks-returns-directory.md) | `git rev-parse --git-path hooks` returns a directory, so append the hook filename before filesystem access | 2026-07-27 |
| [failed-mutations-require-state-refresh.md](./failed-mutations-require-state-refresh.md) | A failed reviewed mutation may consume its plan and leave recoverable repository state, so the UI must refresh before presenting the error | 2026-07-27 |
| [recorded-phases-need-recovery-transitions.md](./recorded-phases-need-recovery-transitions.md) | Every durable in-flight phase must map to a safe retry, resume, or explicit inspection path | 2026-07-27 |
