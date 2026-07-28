# `submodule.<name>.ignore = all` also blinds `git diff --cached`

Verified against git 2.52.0.windows.1 with a real superproject + submodule fixture.

Setting `submodule.<path>.ignore = all` is the knob that stops a submodule from permanently showing as modified. It does that job, but it does **not** do the job people assume comes with it, and it quietly breaks the obvious guard against the real problem.

## What was measured

With a submodule whose checked-out commit differs from the recorded gitlink:

| Command | `ignore` unset | `ignore = all` |
|---|---|---|
| `git status --porcelain` (unstaged) | ` M Submodules/Engine` | *(empty)* |
| `git add -A` | stages the pointer | **still stages the pointer** |
| `git status --porcelain` (after staging) | `M  Submodules/Engine` | `M  Submodules/Engine` |
| `git diff --cached --name-only -- Submodules/` | `Submodules/Engine` | **_(empty)_** |
| `git diff --cached --name-only --ignore-submodules=none -- Submodules/` | `Submodules/Engine` | `Submodules/Engine` |

## Why it matters

1. **`ignore = all` is not a commit guard.** It is a display setting. `git add -A` sweeps the gitlink in regardless, which is the exact accident it gets configured to prevent.
2. **The natural pre-commit hook silently does nothing.** A hook whose check is `git diff --cached --name-only -- Submodules/` returns empty under `ignore = all`, so the commit sails through and the hook looks like it works because it never fires. The hook must pass `--ignore-submodules=none` explicitly. This was reproduced: the naive hook allowed the commit (exit 0); adding the flag made it refuse (exit 1).
3. **`git status` is not a reliable oracle for what the app should show.** Any diff or status the app runs while inspecting submodules needs `--ignore-submodules=none`, or it will report a clean state that is not clean.

## What does work

- Hiding the noise: `submodule.<path>.ignore = all` (repo-local config).
- Blocking the accident: a `pre-commit` hook checking `git diff --cached --name-only --ignore-submodules=none -- <paths>`, or staging through a pathspec exclude — `git add -A -- ':!Submodules/'` was confirmed to leave the pointer unstaged.

Neither alone is sufficient. The display setting and the guard are independent and both are needed.
