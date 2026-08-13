# Commit merge flow

## Trigger

The user finishes resolving an in-progress Git merge (`MERGE_HEAD` is set, every conflict is resolved in the index) and submits **Commit merge** from the rail, or clicks the Sync conflict banner while phase is `base-merge-conflict`.

## Sequence

1. Preflight refuses detached HEAD, rebase/cherry-pick/bisect, missing `MERGE_HEAD`, or any unmerged index entry.
2. A temporary index runs `read-tree --empty` then `read-tree -m <merge-base> HEAD MERGE_HEAD`.
3. For each path still conflicted in that index, stage 0 from the real index is copied (or the path is removed if the resolution deleted it).
4. `write-tree` produces the merge tree; unrelated staged paths (diff `--cached` vs `MERGE_HEAD` but absent from that tree) are listed as `excluded_paths` in the review.
5. Apply installs the tree with `read-tree <tree>` on the real index, then `git -c submodule.recurse=false commit --no-edit` while `MERGE_HEAD` is still present.
6. When Base is configured and `MERGE_HEAD` equals Base, apply verifies every path in `Base…HEAD` after the commit was already in that set before HEAD moved.
7. If Sync is paused at `base-merge-conflict`, the outcome offers **Resume sync** (Saved work is not reapplied until then).

## Reads

- `MERGE_HEAD`, `HEAD`, merge base, porcelain-v2 unmerged state, configured `githelper.base`.
- Staged resolutions on conflicted paths only.

## Writes

- One merge commit on the current branch; `MERGE_HEAD` cleared by Git.
- Real index replaced by the planned tree; unrelated staged/untracked files remain in the worktree unstaged.
- Oplog entry `commit-merge` with `refs_before` / `refs_after` for HEAD.

## Files to inspect

- `src/merge_commit/` (plan, tree, apply)
- `src-tauri/src/commands/prepare/worktree.rs` (`commit_merge`)
- `ui/app/Private/views/form-worktree.ts`, `banners.ts`

## Common failure modes

- Unmerged paths still present → prepare refused.
- No `MERGE_HEAD` → empty form / submit blocked.
- HEAD or `MERGE_HEAD` moved since prepare → stale plan on apply.
- `commit --no-edit` fails after `read-tree` (hook, empty message) → error with Git stderr; index may already match the planned tree.
