# Git failures must carry stderr, and open must probe the worktree

`GitError::Command` used to render only `exit code Some(128)`. Every real failure mode — not a repository, missing Base, fetch auth, no merge base — collapses to that string, so the UI cannot tell the user what to fix and agents cannot tell which argv died.

## What to preserve

1. **Surface stderr (and a short argv summary) in `Display`.** The bytes are already captured; omitting them turns a diagnosable Git fatal into a mystery exit code.
2. **`GitRepository::open` only proves the Git binary runs.** A non-repo folder still opens. The desktop `open_path` boundary must probe `rev-parse --is-inside-work-tree` (and that `HEAD` exists) *before* swapping the live session, or a bad picker choice replaces a working repo and the next snapshot fails with the same opaque inspection error.
3. **Resolve Base before `A...HEAD`.** A configured `githelper.base` that was never fetched (or was pruned) makes every post-open discovery call fail with exit 128. Prefer `InspectionError::InvalidBase` with an actionable sentence over letting `git diff` be the first probe.

## Why tests alone miss it

Fixture repos always have a real worktree and a resolvable Base. The failure is at the desktop open / first-discovery boundary with user paths and stale config — not inside the rewrite engine.
