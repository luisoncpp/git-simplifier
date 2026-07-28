# Split Branch Flow

Copy mode only. The source branch is never rewritten and never receives a revert commit.

## Trigger

The user picks the Split branch operation, names a branch, ticks changed paths, and optionally types a message. A backend caller can submit the same four values directly.

## Entry point

UI: `prepare_operation` with `{"kind": "split_branch", …}` → `prepare::branch::split_branch` → `apply_operation` → `apply::split_branch`.

Core: `GitRepository::plan_split_branch` → `GitRepository::apply_split_branch`.

## Sequence

1. `split::plan::create` reads the current branch, HEAD, and the Base commit, then resolves `merge_base = git merge-base <base> HEAD`. The merge base is stored in the plan; apply never re-derives it from three-dot syntax.
2. The branch name is validated with `check-ref-format --branch` and rejected if `refs/heads/<name>` already exists.
3. `paths::changed_between` lists the changed files over the merge base once. The selection matches a changed file exactly or as a directory prefix; an empty match is `NoChanges`, an empty selection is `EmptySelection`.
4. `paths::companions` adds the changed partner of any matched path: `<asset>.meta` for a selected asset, and the asset for a selected `.meta`.
5. The message is the caller's bytes, or a derived `Split <n> file(s) from <branch>` marked with `message_is_derived`.
6. `review::commands` derives the exact ordered write sequence from the plan; both the plan and the oplog record use that same list.
7. `apply` takes the repository write lock, re-verifies branch, HEAD, Base, and branch absence, then records the operation as in-flight.
8. The patch is read with `git diff --binary --no-renames` limited to `:(literal)` pathspecs. `--no-renames` keeps each patch self-contained, so a rename with only one side selected cannot fail to apply.
9. A detached worktree is added at the merge base, the patch is applied with `apply --index --binary`, and `write-tree` produces the tree. The worktree is removed on both the success and failure paths, followed by `worktree prune`.
10. `commit-tree` parents the new commit on the merge base with the message on stdin; `update-ref <ref> <commit> ''` creates the branch only if it is still absent.
11. The oplog entry is finished with the new ref.

## Reads

- `HEAD`, the current branch, the Base commit, and their merge base.
- Changed file names between merge base and HEAD.
- Existence of `refs/heads/<new branch>`.

## Writes and side effects

- A temporary worktree under `.git/githelper/worktrees/`, removed before the operation returns.
- New commit objects and `refs/heads/<new branch>`.
- `.git/githelper/oplog.json`.

## Recovery

The record is reversible. `refs_before` stores the new branch with an empty value, meaning it did not exist, so the recovery panel offers `git update-ref -d refs/heads/<new branch>`.

## Files to inspect

- `ui/app/Private/views/form-branch.js`, `views/path-list.js`, `operations.js`
- `src-tauri/src/commands/prepare/branch.rs`, `commands/apply.rs`
- `src/split/plan.rs`, `paths.rs`, `apply.rs`, `worktree.rs`, `review.rs`, `record.rs`
- `src/repository.rs`
- `tests/split_fixtures.rs`

## Common failure modes

- The source branch moves between planning and applying, or Base moves: `StalePlan`.
- The branch name is taken, including by a branch created after planning: `ExistingBranch`.
- The selection names a path that did not change over Base: `NoChanges`.
- HEAD is detached: the plan cannot name a source branch.
- The review is prepared but the user leaves the operation: the pending plan is cancelled and the selection survives, because the selection lives in the draft rather than the plan.
- A gitlink change in the selection may not apply cleanly, because `git apply --index` handles `Subproject commit` hunks poorly; the temporary worktree is still removed.
