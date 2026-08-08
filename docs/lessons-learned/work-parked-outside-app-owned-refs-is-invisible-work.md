# Work parked outside app-owned refs is work the app will tell you it does not have

The app lists Saved work from `refs/githelper/wip/*` only. It never runs `git stash list`. So anything the app leaves on the shared stash stack is, from the user's point of view, gone: the Saved work panel says **"No Saved work"** while the only copy of their changes sits in `refs/stash`, visible only in a third-party client like TortoiseGit.

Quick switch used to do exactly that. When carry's `git stash pop --index` and its plain-pop fallback both failed, the code returned a warning telling the user to run `git stash drop` themselves and moved on. The entry stayed on the stack, unreferenced by any app surface.

## The rule

Every code path that parks user work must park it on a ref the listing surface already reads. A warning string is not a recovery path — the user cannot act on advice about a snapshot the app will not show them.

The fast-forward-failure path had this right all along (`anchor_carry` writes `refs/githelper/carry/<id>`); the pop-failure path simply never got the same treatment. When adding a new failure branch, check whether a sibling branch already anchors its snapshot, and match it.

## The guard that comes with it

Rescuing onto `refs/githelper/wip/<branch>` must refuse when that ref already exists. Overwriting trades one lost snapshot for another, which is the same bug wearing a different hat.

Related: [returned-snapshots-must-be-reused.md](./returned-snapshots-must-be-reused.md), [created-refs-need-an-absent-marker-for-recovery.md](./created-refs-need-an-absent-marker-for-recovery.md).
