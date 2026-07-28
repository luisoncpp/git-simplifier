# No one-click Undo; keep the recording, show a recovery panel

§2.2 of the design doc made "Undo last operation" a load-bearing, non-deferrable feature. We split that claim in two: **recording** (capturing every ref an operation will move, writing backup refs for content that would otherwise be unreachable, appending to the oplog) stays and remains non-deferrable; the single **Undo button** is dropped. In its place the app shows a recovery panel — past operations, the refs each one moved, and a copy-pasteable command to return to the previous state.

## Why

A one-click Undo has to answer questions with no good general answer: what it means to undo an operation after the user has edited files, or worked in their terminal, in the intervening ten minutes. `update-ref` restores refs without touching the worktree, so a button labelled "Undo" would routinely leave a working tree that does not match what the user had — an undo that lies. Recording carries almost all of the safety value and none of that ambiguity.

## Consequences

- Recording is still mandatory from the first write operation. §2.2's argument holds for this half: retrofitting it means auditing every operation a second time.
- The `refs/githelper/backup/*` namespace (§2.3) stays, and is **required** for any operation that calls `git stash create` (§4.4, §4.5). A stash snapshot commit is referenced by nothing; without a backup ref it is unreachable the instant it is created and `gc` may delete it. There is no reflog fallback for it.
- Edit message and Uncommit need no backup ref of their own: they only move a branch pointer, and `update-ref` writes the prior position to the branch reflog. For these two, the recovery panel shows `git reset --hard <prior sha>`.
- The oplog's `commands` list continues to serve §1's teaching goal, so it earns its keep independently of recovery.
