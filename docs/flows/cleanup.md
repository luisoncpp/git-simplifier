# Cleanup Flow

Deletes branches already contained in Base. Remote deletions run before local ones, because the local branch is the backup for the irreversible half.

## Trigger

User picks **Cleanup** in the Actions tab strip, unticks anything to keep, and applies the review. Backend callers submit a Base ref, the chosen ref names, and whether remote counterparts may be deleted.

## Entry point

UI: `list_cleanup_branches` → `prepare_operation` → `prepare::cleanup::cleanup` → `apply_operation` → `apply::cleanup`.

Core: `GitRepository::discover_cleanup` → `GitRepository::plan_cleanup` → `GitRepository::apply_cleanup`.

## Discovery sequence

1. `state::ensure_remote_base` rejects a Base that is not under `refs/remotes/`.
2. Base is resolved once to a SHA, so discovery, planning, and verification cannot disagree because a fetch landed between them.
3. Four `for-each-ref` reads plus `git remote` and `git config --get user.email` gather everything; cost is flat in branch count.
4. `eligibility::classify` — pure — applies every safety rule and annotates each row with `mine`, `kind`, `protected`, and its remote counterpart.
5. The result is the **maximal** offerable set. The three UI toggles filter it client-side and never reach Git.

## Apply sequence

1. `plan::verify_current` re-runs `ensure_no_operation` and recomputes eligibility, then asserts each chosen branch is still present at its recorded SHA.
2. Per remote: begin a `cleanup-remote-branches` record, run one `git push --atomic --force-with-lease=<ref>:<sha> <remote> :<ref>` per remote, finish the record.
3. Begin one `cleanup-local-branches` record, run `git update-ref -d -m 'git-helper cleanup' <ref> <expected-sha>` per branch, finish the record.

## Never offered

- The branch at HEAD, and any branch checked out in another worktree. `update-ref -d` does **not** refuse a checked-out branch the way `git branch -d` does; it would leave that worktree's HEAD dangling.
- The local branch Base tracks. Without this, Cleanup offers to delete the mainline.
- Any branch with a `refs/githelper/wip/<branch>` snapshot, which deleting the branch would orphan.
- `refs/remotes/*/HEAD`, the Base ref itself, and a branch whose upstream remote is `.`.

Shared names (`main`, `master`, `develop`, `dev`, `trunk`) are offered with a badge but never pre-ticked.

## Reads

- Base as a SHA; merged locals with `%(HEAD)`, `%(worktreepath)`, `%(authoremail:trim)`, and the three `%(upstream…)` atoms.
- Every remote-tracking ref, and the subset merged into Base.
- `refs/githelper/wip` for Saved work, `git remote` for remote names, `user.email` for identity.

## Writes and side effects

- Deletes chosen branches on their remotes, then locally.
- Never moves HEAD, never touches the index or working tree.
- Appends two kinds of record to `.git/githelper/oplog.json`.

## Recovery

Local deletions record their old SHA in `refs_before` with `reversible: true`, so the Recovery panel offers a working `git update-ref` restore; the commits survive until `gc`. Remote deletions record `reversible: false` with an empty `refs_before`, so they fail both gates and yield no recovery command — a remote-tracking ref there would restore a local pointer to a server branch that no longer exists. `details["restore"]` carries the real `git push` repair instead.

## Files to inspect

- `ui/app/Private/views/form-cleanup.ts`, `ui/app/Private/draft/cleanup.ts`
- `src-tauri/src/commands/prepare/cleanup.rs`, `src-tauri/src/commands/apply.rs`
- `src/cleanup/`
- `src/repository/cleanup.rs`
- `tests/cleanup_fixtures.rs`

## Common failure modes

- A chosen ref that is not in a freshly computed eligible set: `NotEligible`. Planning never trusts the caller's list, so an exclusion cannot be bypassed through the API.
- A chosen branch moved since the review, or Base was rewound so a branch is no longer merged: `StalePlan`.
- Someone pushed to the branch since the last fetch: undetectable locally, caught by the explicit lease at push time as `RemoteRejected`. `--atomic` means nothing is deleted, and locals are still intact because remotes run first.
- `only_mine` with no `user.email` configured: the filter matches nothing, and the form says so rather than showing an empty list.
- **Authorship is an approximation.** A merged branch is an ancestor of Base, so `Base..branch` is empty and there are no commits unique to it; the tip author is the signal. It is wrong when the last commit on the branch was someone else's merge, or when someone else rebased it. The form shows the author email beside each row for that reason.
- A **squash-merged** branch is not an ancestor of Base and is never listed. Cleanup under-reports rather than offering to delete work Git cannot prove is integrated.
