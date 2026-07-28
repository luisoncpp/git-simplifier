# Uncommit Rewrite Flow

## Trigger

Backend caller submits a remote-tracking Base ref and one or more repository paths.

## Sequence

1. Preflight rejects detached HEAD, an active Git operation, an invalid Base, missing merge base, or an empty Editable range.
2. The planner reads the first-parent range oldest to newest and loads the relevant commit/tree objects.
3. Each planned tree restores selected paths to Base; unchanged results are marked dropped.
4. The caller may inspect the immutable plan without changing refs, the index, or the worktree.
5. Applying the plan rechecks HEAD and Base, records the operation, builds replacement trees in a temporary index, and creates replacement commits with `commit-tree`.
6. One `update-ref` moves the branch using the observed old SHA.
7. The affected paths are reset in the real index; the working-tree files remain untouched.
8. The operation log is completed with the new branch SHA and recovery metadata.

## Reads

- Current symbolic branch, HEAD, Base, merge-base, first-parent commit IDs, commit metadata, and tree entries.
- Git operation markers such as merge, rebase, cherry-pick, and bisect state.

## Writes and side effects

- Writes replacement objects and a temporary index only; these do not affect the user’s checkout.
- Moves the current branch with an expected-old-SHA check.
- Resets only selected paths in the real index and leaves unrelated staged changes and worktree files untouched.
- Creates or updates `.git/githelper/oplog.json`.

## Files to inspect

- `src/repository.rs`
- `src/rewrite/preflight.rs`, `planner.rs`, `materialize.rs`, and `materialize_steps.rs`
- `src/recording/oplog.rs`
- `tests/rewrite_fixtures.rs`

## Common failure modes

- A changed HEAD or Base makes the plan stale and aborts before ref mutation.
- Git reports raw stderr through `GitError`.
- A failure after recording leaves an unfinished operation entry for later recovery inspection.
