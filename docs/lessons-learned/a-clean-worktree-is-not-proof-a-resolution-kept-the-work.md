# A clean worktree is not proof the user's resolution kept the work

Resuming after a conflicted Saved work reapply deliberately does not re-run the apply — the user has been editing the conflict markers in place, so reapplying would clobber the resolution. The resume therefore only checks `git status` for unmerged entries and, finding none, finishes the operation and reports the Saved work restored.

That check cannot distinguish the two ways a conflict stops being a conflict:

| resolution | unmerged entries | tracked changes | work is in the tree |
|---|---|---|---|
| user edited the markers and staged | none | yes | yes |
| user ran `reset --hard` / committed a revert | none | **none** | **no** |

Both look identical to an unmerged-entries probe. The second case finished silently and told the user their work was applied while the tree held none of it — and because the operation was recorded as complete, the snapshot ref stopped being offered.

## The rule

A snapshot is only ever written when there *were* tracked changes to set aside. So if a snapshot is anchored and the worktree has no tracked changes at completion time, nothing came back. That is a cheap, sound signal — use it before declaring an operation successful.

More generally: when a step is skipped because the user was supposed to do it manually, verify the user's side of the bargain instead of assuming it. "No error" is not "it worked".

## What the warning must do

Warn, do not block. An error would trap a user who discarded the work deliberately, and the app has no delete path for a sync backup ref. The completion carries a warning naming the ref, the ref is kept, and the banner switches to warn tone.

Related: [failed-mutations-require-state-refresh.md](./failed-mutations-require-state-refresh.md), [work-parked-outside-app-owned-refs-is-invisible-work.md](./work-parked-outside-app-owned-refs-is-invisible-work.md).
