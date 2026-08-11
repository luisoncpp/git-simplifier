# Excluded Submodule Flow

## Trigger

Backend caller submits a repository path and chooses whether to install the local `pre-commit` guard and disable recursive submodule commands.

## Sequence

1. `GitRepository::plan_exclude_submodule` reads `HEAD` and rejects the path unless it is an exact `160000` gitlink.
2. The planner reads the current local ignore/recurse values and resolves Git's hooks directory to its `pre-commit` file.
3. The immutable plan exposes the exact config commands, hook guard preview, and an opt-in staging pathspec. Existing hook bytes are retained in the plan so application can append safely.
4. `GitRepository::apply_exclude_submodule` takes the repository write lock and rechecks `HEAD`, the gitlink, config values, and any hook that will be changed.
5. Application writes `submodule.<path>.ignore = all`, optionally writes `submodule.recurse = false`, and appends the guard. An existing hook is never replaced.
6. The operation is recorded as `exclude-submodule` in `.git/githelper/oplog.json`. No branch or other Git ref moves.
7. If the gitlink was already committed in the Editable range, cleanup is offered on the **Submodules** tab as **Cleanup dirty submodules** (Uncommit and/or Revert); see [cleanup-dirty-submodules.md](./cleanup-dirty-submodules.md).

## Reads

- `HEAD` and its exact tree entry for the selected path.
- Repo-local Git config values.
- `git rev-parse --git-path hooks` and the existing `pre-commit` file.

## Writes and side effects

- Repo-local `.git/config` values only.
- The repo-local `pre-commit` hook, either created or appended to.
- `.git/githelper/oplog.json`.
- The hook blocks staged changes to the selected gitlink using `--ignore-submodules=none`, even when status display uses `ignore = all`.

## Files to inspect

- `src/exclusion/plan.rs`, `apply.rs`, and `hook.rs`
- `src/repository.rs`
- `src/recording/oplog.rs`
- `tests/exclusion_fixtures.rs`

## Common failure modes

- A normal file or directory is rejected because exclusion requires an exact gitlink.
- A changed config, hook, or `HEAD` makes the plan stale before any write.
- Existing hook content is preserved; callers must explicitly choose whether the guard may be appended.
- The optional staging command is guidance only and is never run implicitly.
