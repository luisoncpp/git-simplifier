# Revert Flow

## Trigger

Backend caller submits a remote-tracking Base ref, one or more tracked paths, and a target of HEAD or Base.

## Sequence

1. Preflight refuses an empty selection or an active Git operation (merge, rebase, cherry-pick, bisect).
2. Discovery eligibility is the union of tracked local dirt (`status --porcelain=v2` with untracked omitted) and `Base...HEAD` name-status; untracked paths never appear.
3. Every selected path must still be in that union; otherwise planning fails.
4. The plan derives one `git -c submodule.recurse=false restore --source=<HEAD|Base> --staged --worktree -- :(top,literal)…` command.
5. Applying rechecks HEAD and eligibility, records the operation, runs that restore, and finishes the oplog with no ref moves (`reversible: false`).

## Reads

- HEAD, Base, porcelain tracked dirt, and `Base...HEAD` changed paths.

## Writes and side effects

- Overwrites the index and working tree for the selected paths only.
- Leaves branch refs and commit history untouched.
- Creates or updates `.git/githelper/oplog.json`.

## Files to inspect

- `src/revert/`
- `src-tauri/src/commands/prepare/revert.rs`
- `tests/revert_fixtures.rs`

## Common failure modes

- A moved HEAD or a path that left the eligible set makes the plan stale.
- Opening the repository below the Git root without `:(top,literal)` would silently match nothing; the plan always pins pathspecs that way.
- Untracked files are never listed and are never deleted by this operation.
