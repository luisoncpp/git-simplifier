# Sync with Base Flow

## Trigger

The backend caller submits the configured remote-tracking Base ref, such as `refs/remotes/origin/develop`.

## Sequence

1. Preflight rejects a detached HEAD, an active Git operation, or another in-flight Sync. Base is parsed into its remote and branch components.
2. The operation log records the current branch, HEAD, Base position, and `fetch` phase.
3. A non-recursive `git fetch --no-tags <remote> +<branch>:<base-ref>` refreshes only the remote-tracking Base ref. A local branch named after Base is never moved. If fetch fails, Retry Sync verifies the recorded branch and HEAD, repeats fetch, and continues the same operation.
4. The app compares untracked paths with paths Base would write. Overlapping paths are rejected before tracked work is changed.
5. Superproject tracked changes are captured with a non-recursive `git stash create`, then anchored under `refs/githelper/backup/*`. Untracked files and nested submodule worktree dirt are not included.
6. Non-recursive reset and merge commands clean superproject tracked changes and merge Base without checking out, cleaning, or fetching inside submodules.
7. A Base merge failure leaves Git's merge state and the Saved work ref in place, with the oplog phase set to `base-merge-conflict`.
8. After a successful merge, the snapshot is reapplied with `git stash apply --index`, falling back to plain apply if the staged split cannot be restored. A failure is recorded as `wip-reapply-conflict`.
9. The caller resolves conflicts in an existing Git client, then **Commit merge** in this app (or an external `git commit`) so unrelated staged files cannot enter the merge commit. While `MERGE_HEAD` still exists during `base-merge-conflict`, the Sync banner offers Commit merge instead of Resume sync.
9b. After the merge commit, **Resume sync** reapplies Saved work. The operation completes only after Git has no unmerged entries.
9a. Resuming does **not** reapply the snapshot — it assumes the resolution left the carried work in the tree. When the tree instead has no tracked changes, that assumption is false, so the result carries `saved_work_warning`, the backup ref is kept, and the UI shows a warn-tone banner instead of a clean success. Resolving a reapply conflict by discarding is the case this covers.
10. If the desktop apply call returns an error, the consumed review is discarded and the repository snapshot is reloaded. A recorded conflict is immediately surfaced with its resume review action instead of leaving the stale Start Sync review visible.

## Reads

- Current symbolic branch, HEAD, Base ref, remote configuration used by `fetch`, and Git operation markers.
- Porcelain-v2 status for tracked and untracked paths.
- The Base-vs-HEAD path diff for untracked-file clobber detection.
- The Saved work backup ref and the in-flight Sync record when resuming or displaying status.

## Writes and side effects

- Updates the requested remote-tracking Base ref through `fetch`.
- May create `refs/githelper/backup/<sync-id>-wip` and leaves it durable after success for recovery.
- Resets tracked work, may create a merge commit, and reapplies tracked work to the working tree and index.
- Preserves each submodule's checked-out commit, tracked modifications, and untracked files, even when repository configuration otherwise enables recursive submodule commands.
- Appends and advances the in-flight operation under `.git/githelper/oplog.json`.

## Files to inspect

- `src/sync/start.rs`, `work.rs`, `resume.rs`, and `model.rs`
- `src/repository.rs`
- `src/recording/oplog.rs`
- `tests/sync_fixtures.rs` and `tests/sync_recovery_fixtures.rs`

## Common failure modes

- An untracked path that Base would write is rejected before `stash create`; the local file remains untouched.
- A Base merge conflict must be resolved and committed before `resume_sync` can reapply Saved work.
- A Saved work reapply conflict remains visible in the working tree and the backup ref remains available until resolution.
- Fetch failures leave an in-flight record at the fetch phase so the recovery surface can explain that no merge was attempted and offer Retry Sync. Retry is refused if the branch or HEAD changed after the interruption.
