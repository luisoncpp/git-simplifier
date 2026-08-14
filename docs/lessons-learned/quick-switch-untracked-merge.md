# Untracked overlap on quick switch

Overlapping untracked files cannot remain in the worktree across a checkout when the target branch already tracks the same path. Git would overwrite them silently or refuse the switch.

## Park on the target, never source Saved work

Untracked overlap content must land on the **target** branch after checkout. It is parked in a stash-shaped ref at `refs/githelper/untracked-merge/<operation-id>`, then reapplied with `stash apply --index` (plain apply only when no unmerged paths exist). It is never folded into `refs/githelper/wip/<source>` Saved work.

## Typed prepare block vs string error

When `merge_untracked` is false, prepare returns an `OperationBlock` (`kind: untracked_overwrite`) instead of a string error. A string error would surface as “That did not run” and, with Skip-review on, would hide the **Switch with merge** offer entirely because prepare would fail before the UI could show alternatives.

## Pull pause keeps the park ref

If `git pull --ff-only` fails after the switch, the untracked park ref is persisted on the oplog snapshot map (like carry) and reapplied after replace / merge-pull / cancel, skipping reapply while `MERGE_HEAD` exists.
