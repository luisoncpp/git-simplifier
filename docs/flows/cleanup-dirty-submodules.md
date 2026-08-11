# Cleanup Dirty Submodules Flow

## Trigger

The user opens the **Submodules** tab, selects one or more dirty gitlinks in **Cleanup dirty submodules**, chooses **Uncommit from Base…HEAD** and/or **Revert**, and submits **Review cleanup**.

## Sequence

1. `list_dirty_submodules` discovers gitlinks that are locally dirty (`status --porcelain=v2 --ignore-submodules=none`) and/or differ in `Base...HEAD` (`git diff --ignore-submodules=none`), intersected with `HEAD` gitlinks.
2. `GitRepository::plan_submodule_cleanup` validates every selected path is still dirty, builds an optional `RewritePlan` for paths in the Editable range when Uncommit is checked, and records the revert command sequence.
3. The review lists impact per step: Uncommit rewrites history for Editable-range pointers; Revert restores each gitlink to `HEAD` and resets nested checkouts.
4. `GitRepository::apply_submodule_cleanup` verifies `HEAD` is unchanged, then runs **Uncommit first** (when enabled), then **Revert** (when enabled).
5. Revert for a gitlink still in `HEAD`: `git restore --source=HEAD --staged --worktree` with a literal pathspec, `git submodule update --force`, then `git -C <path> checkout --force HEAD` and `git -C <path> clean -fd` to clear tracked and untracked nested dirt.
6. Revert for a gitlink removed from `HEAD` by Uncommit: `git submodule deinit -f` and remove the worktree directory.
7. The operation is recorded as `cleanup_submodules` in `.git/githelper/oplog.json`. A rewrite may offer force push afterwards.

## Reads

- `HEAD` gitlinks (`ls-tree -r HEAD`).
- Porcelain v2 status with `--ignore-submodules=none`.
- `Base...HEAD` name-status diff with `--ignore-submodules=none` when Uncommit is requested or when building `in_editable_range`.

## Writes and side effects

- Branch history when Uncommit runs (new SHAs after Base).
- Superproject index and worktree gitlink entries when Revert runs.
- Nested submodule checkouts aligned to the restored gitlink (or removed when the gitlink is gone).
- `.git/githelper/oplog.json`.

## Files to inspect

- `src/inspection/dirty_submodules.rs`
- `src/submodule_cleanup/plan.rs`, `apply.rs`
- `src/repository.rs`
- `ui/app/Private/views/form-submodules.ts`
- `tests/submodule_cleanup_fixtures.rs`

## Common failure modes

- Neither Uncommit nor Revert is selected.
- Uncommit is checked but Base is not set.
- A path is no longer dirty when apply runs (stale plan).
- Uncommit-only leaves nested checkouts untouched until Revert is run separately.
- `changed_paths` and Uncommit/Split checklists now pass `--ignore-submodules=none` so committed gitlink diffs are visible even when `ignore = all` hides them from status.
