# Edit Message Flow

## Trigger

Backend caller submits a remote-tracking Base ref, an Editable-range commit, and replacement message bytes.

## Sequence

1. Preflight rejects detached HEAD, an active Git operation, an invalid Base, missing merge base, or an empty Editable range.
2. The target commit is resolved and must be present in the first-parent Editable range; Base and teammate-side commits are rejected.
3. The planner reads the range oldest to newest and changes only the target metadata message.
4. Applying the immutable plan rechecks HEAD and Base, records the operation, rebuilds the same trees and all parent links through `commit-tree`, and moves the branch with one expected-old-SHA `update-ref`.
5. The operation log is completed with the new branch SHA and recovery metadata.

## Reads

- Current symbolic branch, HEAD, Base, merge base, first-parent commit IDs, commit trees, parent IDs, and signatures.
- Git operation markers such as merge, rebase, cherry-pick, and bisect state.

## Writes and side effects

- Writes replacement commit objects and a temporary index only; the user’s index and worktree are untouched.
- Moves the current branch and records the reflog action as `git-helper edit-message`.
- Creates or updates `.git/githelper/oplog.json`.

## Common failure modes

- A target outside the Editable range is rejected before any plan can be applied.
- A changed HEAD or Base makes the plan stale and aborts before ref mutation.
- Git reports raw stderr through `GitError`.
