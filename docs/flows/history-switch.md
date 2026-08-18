# History Switch Flow

## Trigger

The user opens the **History** tab on a symbolic branch, picks a first-parent commit or a local date-time, and submits. Carry tracked changes defaults **off**. Returning to **present** is a normal Quick switch onto the branch that was left behind.

## Discovery

1. The UI selects History and requests `list_history_commits` (no Base).
2. Inspection runs `git log --first-parent --reverse --max-count=301 HEAD` and drops HEAD, so the list is older first-parent commits only (cap 300). Oldest-first matches Editable; the form reverses for display.
3. Date-time is not listed: apply resolves it with `git rev-list -1 --first-parent --until=<local-iso> HEAD` (committer date).

## Sequence

1. Preflight rejects detached HEAD, an active merge/rebase/cherry-pick/bisect, an empty target, target == HEAD, a SHA that is not an ancestor of the current branch, a date-time with no matching commit, and existing Saved work on the source when carry is off.
2. Untracked same-path overlaps without `merge_untracked` return a typed prepare block (`untracked_overwrite`) with **Switch with merge**. Directory-vs-file prefix overlaps hard-refuse. There is no pull.
3. Applying records an in-flight `history-switch` operation (HEAD before/after, present ref, target commit).
4. Tracked dirt: carry off → stash create, Saved work on `refs/githelper/wip/<source>`, then `reset --hard`. Carry on → `stash push` to reapply after detach.
5. Untracked overlaps park at `refs/githelper/untracked-merge/<operation-id>` the same way Quick switch does.
6. Write `git symbolic-ref refs/githelper/present refs/heads/<branch>`, then `git switch --no-recurse-submodules --detach <commit>`. If detach fails, the present marker is deleted.
7. Carry pop and untracked reapply run on the detached commit. The branch pointer stays at the old tip.
8. The result stays on the History tab and offers **Switch to {branch}** (`switch-to`). A persistent banner repeats that offer while `present_branch` is set and HEAD is detached.

## Return to present

Quick switch onto the present branch (default target while detached) reattaches at the tip and deletes `refs/githelper/present`. Carry and pull use Quick switch's own defaults (both on). Saved work is not auto-restored.

## Reads

- Symbolic HEAD, first-parent ancestry, committer dates, porcelain tracked/untracked status.
- Target tree paths for untracked clobber detection.
- `refs/githelper/wip/<source>` when carry is off.

## Writes and side effects

- Writes/deletes `refs/githelper/present`.
- May write `refs/githelper/wip/<source>` and `refs/githelper/untracked-merge/<operation-id>`.
- Detaches HEAD; does not move `refs/heads/<branch>`.
- Appends `.git/githelper/oplog.json`.

## Files to inspect

- `src/switch/history_plan.rs`, `history_apply.rs`, `present.rs`, `checkout.rs`
- `src/inspection/queries.rs` (`history_commits`)
- `src-tauri/src/commands/prepare/history_switch.rs`, `review_commands/history.rs`
- `ui/app/Private/views/form-switch-history.ts`, `banners.ts`, `draft/branches.ts`
- `tests/history_switch_fixtures.rs`

## Common failure modes

- Already detached: History is blocked until Quick switch returns to a branch.
- A SHA not on the current first-parent ancestry is refused; date-time before the first commit is refused.
- Existing Saved work on the source blocks when carry is off (same as Quick switch).
- Untracked same-path overlap needs **Switch with merge**; prefix overlaps cannot.
- Returning does not auto-restore Saved work; the existing banner is the only offer.
