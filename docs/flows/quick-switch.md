# Quick Branch Switch Flow

## Trigger

Backend caller submits the name of another local branch, or explicitly asks to restore or delete Saved work.

## Switch sequence

1. Preflight rejects detached HEAD, an active merge/rebase/cherry-pick/bisect, an invalid or missing local target branch, and an existing Saved work ref for the source branch when changes are not being carried. Nested submodule dirt is excluded from the superproject tracked-change check.
2. Preflight lists untracked paths and rejects any path that overlaps a tracked path on the target branch. Non-conflicting untracked files remain in the checkout.
3. Applying the plan records an in-flight `quick-switch` operation.
4. If tracked superproject changes exist, a non-recursive `git stash create` creates a snapshot without touching the shared stash stack. The snapshot is first anchored at `refs/githelper/wip/<source-branch>`.
5. Only after the WIP ref is written, tracked changes are removed with a non-recursive `git reset --hard HEAD`, then `git switch --no-recurse-submodules --no-guess -- <target-branch>` moves the checkout.
6. When **carry changes** is enabled, tracked changes are stored with `git stash push`, the checkout switches, and the stash is restored with `git stash pop --index` (falling back to a plain pop). Pop conflicts do not block the switch; the result reports a warning and leaves conflict markers for the user to resolve. No Saved work ref is written for the source branch.
7. The result reports the newly created Saved work and any Saved work already waiting for the target branch.

## Restore sequence

1. The caller explicitly requests restoration after returning to a branch; opening the app does not restore automatically.
2. The current branch's WIP ref is applied non-recursively with `git stash apply --index`.
3. If index restoration fails, a plain `git stash apply` is attempted and the result reports that the staged split was not restored.
4. The WIP ref is deleted with an expected snapshot SHA only after apply succeeds. Failed application leaves the ref available for retry or inspection.

## Reads

- Current symbolic branch and HEAD.
- Local target branch commit.
- Git operation markers and porcelain-v2 status.
- Target tree paths for untracked-file clobber detection.
- `refs/githelper/wip/*` for Saved work listing and restoration.

## Writes and side effects

- Creates/deletes `refs/githelper/wip/<branch>`.
- May write a temporary stash snapshot object, reset tracked files, and switch the current checkout.
- Leaves nested submodule checkout SHAs, tracked modifications, and untracked files physically in place.
- Leaves untracked files in place unless Git itself refuses the switch because of a conflict detected after preflight.
- Appends an operation record under `.git/githelper/oplog.json`.

## Files to inspect

- `src/switch/plan.rs`, `apply.rs`, and `model.rs`
- `src/repository.rs`
- `src/recording/oplog.rs`
- `tests/switch_fixtures.rs`

## Common failure modes

- A second switch away from a branch with existing Saved work is rejected instead of overwriting the only snapshot.
- A plan becomes stale if HEAD, the target branch, tracked status, or untracked conflict set changes before application.
- Nested submodule state is preserved in place rather than snapshotted per branch. Changes made inside a submodule while away from the source branch replace that in-place state.
- If restoration conflicts, the WIP ref remains until the user resolves or explicitly deletes it.
- Carry pop conflicts leave the checkout on the target branch with conflict markers and may keep the stash entry until the user resolves and drops it.
