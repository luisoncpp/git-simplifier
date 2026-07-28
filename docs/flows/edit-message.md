# Edit Message Flow

## Trigger

The user opens Edit Message with a configured remote-tracking Base, then submits an Editable-range commit and replacement message bytes.

## Discovery

1. The UI selects the tab and requests `list_editable_commits` with the configured Base.
2. Inspection runs `git log` over `Base..HEAD` in first-parent, oldest-first order with explicit field and record separators.
3. The parser discards formatting-only record fragments and returns typed commit IDs, subjects, full messages, and author signatures.
4. The UI renders the returned commits as selectable values; it never asks the user to type a SHA.

## Sequence

1. Preflight rejects detached HEAD, an active Git operation, an invalid Base, missing merge base, or an empty Editable range.
2. The target commit is resolved and must be present in the first-parent Editable range; Base and teammate-side commits are rejected.
3. The planner reads the range oldest to newest and changes only the target metadata message.
4. Applying the immutable plan rechecks HEAD and Base, records the operation, rebuilds the same trees and all parent links through `commit-tree`, and moves the branch with one expected-old-SHA `update-ref`.
5. The operation log is completed with the new branch SHA and recovery metadata.

## Reads

- Editable-commit discovery reads the first-parent `Base..HEAD` log.
- Planning reads the current symbolic branch, HEAD, Base, merge base, first-parent commit IDs, commit trees, parent IDs, and signatures.
- Git operation markers such as merge, rebase, cherry-pick, and bisect state.

## Writes and side effects

- Writes replacement commit objects and a temporary index only; the user’s index and worktree are untouched.
- Moves the current branch and records the reflog action as `git-helper edit-message`.
- Creates or updates `.git/githelper/oplog.json`.

## Common failure modes

- A target outside the Editable range is rejected before any plan can be applied.
- A changed HEAD or Base makes the plan stale and aborts before ref mutation.
- Git reports raw stderr through `GitError`.
