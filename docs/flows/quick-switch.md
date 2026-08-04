# Quick Branch Switch Flow

## Trigger

Backend caller submits another branch name (local, or a remote-tracking ref that should become a new local branch with upstream), optional carry/pull flags, or explicitly asks to restore or delete Saved work. When a previous pull could not fast-forward, the caller submits a pull resolution.

## Switch sequence

0. When selecting the initial target branch in the UI, if the current branch is not Base, the target defaults to Base (preferring the local branch of the same name, or the remote-tracking ref if the local branch does not exist). Auto-defaults stay unmarked (`branchPicked` false) so a later Base change or refresh can replace an alphabetical fallback; a row the user picked is kept.
1. Preflight rejects detached HEAD, an active merge/rebase/cherry-pick/bisect, an invalid target, a missing local target (unless creating from remote), an already-existing local when creating from remote, and an existing Saved work ref for the source branch when changes are not being carried. Nested submodule dirt is excluded from the superproject tracked-change check.
2. Preflight lists untracked paths and rejects any path that overlaps a tracked path on the target tree (local branch or remote-tracking start point). Non-conflicting untracked files remain in the checkout.
3. When **pull after switch** is enabled (default), planning records a same-named remote-tracking ref (`origin/<branch>` preferred) as the pull target when one exists.
4. Applying the plan records an in-flight `quick-switch` operation.
5. If tracked superproject changes exist, a non-recursive `git stash create` creates a snapshot without touching the shared stash stack. The snapshot is first anchored at `refs/githelper/wip/<source-branch>`.
6. Only after the WIP ref is written, tracked changes are removed with a non-recursive `git reset --hard HEAD`, then checkout moves:
   - existing local: `git switch --no-recurse-submodules --no-guess -- <target>`
   - remote-only: `git switch -c <local> <remote-ref>` then `branch.<local>.remote` / `.merge` are set so the remote is upstream
7. When **carry changes** is enabled, tracked changes are stored with `git stash push`, the checkout switches, optional pull runs, then the stash is restored with `git stash pop --index` (falling back to a plain pop). Pop conflicts do not block the switch; the result reports a warning and leaves conflict markers for the user to resolve. No Saved work ref is written for the source branch. The UI surfaces carry and merge-pull warnings as a warn-tone result banner (`has_warning`) with a conflict headline, not a green success banner.
8. When a pull remote is planned, `git pull --ff-only` runs after the switch and before any carry pop.
9. If the fast-forward fails, carry (when present) is moved to `refs/githelper/carry/<operation-id>`, the oplog phase becomes `pull-ff-failed`, and the apply returns with `pull_decision_needed` instead of finishing. The UI offers replace / merge-pull / cancel.

## Pull resolution sequence

1. **Replace with remote**: `git reset --hard --no-recurse-submodules <remote-ref>`, then reapply carry when safe.
2. **Merge pull**: `git pull --no-rebase` (may leave conflicts). Carry stays anchored while `MERGE_HEAD` exists.
3. **Cancel**: skip the update, reapply carry when safe, finish the oplog.

## Restore sequence

1. After a successful Quick switch onto a branch that already has Saved work (and no pull decision is pending), the outcome sets `offer_restore_saved_work` so the result banner offers **Review restore**. Opening the app or refreshing while the current branch has Saved work shows the same offer on a persistent banner. Restoration is never automatic.
2. The caller confirms via the restore review; the current branch's WIP ref is applied non-recursively with `git stash apply --index`.
3. If index restoration fails without creating unmerged paths, a plain `git stash apply` is attempted and the result reports that the staged split was not restored.
4. If the indexed apply creates conflicts, restoration stops without retrying over the unmerged index. The result directs the user to resolve the conflict markers and delete Saved work after checking the result.
5. The WIP ref is deleted with an expected snapshot SHA only after apply succeeds. Failed or conflicted application leaves the ref available for retry or inspection. **Diff** on a Saved work row opens a read-only merge-tree preview of the worktree delta (see [saved-work-diff.md](./saved-work-diff.md)).
6. When a pull decision was pending, the restore offer waits until replace / merge-pull / cancel finishes; the resolve outcome then offers restore if Saved work is still present.

## Reads

- Current symbolic branch and HEAD.
- Local target branch commit, or remote-tracking start point when creating a local branch.
- Same-named remote-tracking refs for optional pull.
- Git operation markers and porcelain-v2 status.
- Target tree paths for untracked-file clobber detection.
- `refs/githelper/wip/*` for Saved work listing and restoration.
- Active `quick-switch` oplog phase for pull decisions.

## Writes and side effects

- Creates/deletes `refs/githelper/wip/<branch>`.
- May create a local branch and write `branch.<name>.remote` / `.merge`.
- May write `refs/githelper/carry/<operation-id>` while a pull decision is pending.
- May write a temporary stash snapshot object, reset tracked files, switch the current checkout, and pull/reset.
- Leaves nested submodule checkout SHAs, tracked modifications, and untracked files physically in place.
- Appends an operation record under `.git/githelper/oplog.json`.

## Files to inspect

- `src/switch/plan.rs`, `apply.rs`, `checkout.rs`, `pull.rs`, `resolve.rs`, and `model.rs`
- `src/inspection/queries.rs` (local + remote-only branch listing)
- `src-tauri/src/commands/prepare/worktree.rs`, `apply.rs`, `review_commands/composite.rs`
- `ui/app/Private/views/branch-picker.ts`, `form-worktree.ts`, `banners.ts`
- `tests/switch_fixtures.rs`

## Common failure modes

- A second switch away from a branch with existing Saved work is rejected instead of overwriting the only snapshot.
- A plan becomes stale if HEAD, the target tip, tracked status, or untracked conflict set changes before application.
- Nested submodule state is preserved in place rather than snapshotted per branch.
- If restoration conflicts, the WIP ref remains until the user resolves or explicitly deletes it.
- Carry pop conflicts leave the checkout on the target branch with conflict markers and may keep the stash entry until the user resolves and drops it.
- A failed fast-forward leaves the checkout on the target branch until the user chooses replace, merge-pull, or cancel.
