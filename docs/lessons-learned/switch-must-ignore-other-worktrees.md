# Git switch requires --ignore-other-worktrees

`git switch` by default refuses to check out a branch if it is already checked out by any linked worktree (e.g., external AI agent workspaces or secondary checkouts), failing with exit code 128: `fatal: '<branch>' is already used by worktree at '<path>'`.

## Why this happens

Git enforces single-worktree checkouts per branch by default to prevent concurrent edits from clobbering each other. In multi-worktree or agent-assisted development workflows, branches are frequently checked out in isolated worktrees (such as `.claude/worktrees/*`).

## The fix

Pass `--ignore-other-worktrees` to all `git switch` invocations:
- `git switch --no-recurse-submodules --ignore-other-worktrees --no-guess -- <target>`
- `git switch --no-recurse-submodules --ignore-other-worktrees -c <local> <remote-ref>`

This allows Quick switch to check out the branch in the user's active worktree without failing on worktree collision checks.
