# Review commands must be derived from the plan, not written next to it

A hand-written review string looks correct forever. The Uncommit and Edit message reviews advertised `git rebase --rebase-merges <base>`, and Edit message shipped the literal `<base>` placeholder, while the engine actually rebuilds commits through a temporary index (`read-tree`, `update-index`, `write-tree`, `commit-tree`, one expected-old-SHA `update-ref`, and a path-limited `reset --mixed`). Nothing failed, no test caught it, and the product's central promise — see the exact effect before it happens — was false. Delete saved work advertised `refs/githelper/saved/<branch>`; the real namespace is `refs/githelper/wip/<branch>`, so a user who copied the command would have run a silent no-op.

The fix is structural, not editorial: a review builder takes the plan or the recorded state and reads the same fields the apply path uses, so the mode, object id, ref, and snapshot in the review are literally the ones that will be written. Where a value only exists at apply time, use a labelled placeholder and say so.

Two habits keep this honest:

- **Assert the absence of the plausible-but-wrong command.** `assert!(!commands.contains("rebase"))` catches the regression that an equality assertion on a hand-written list would happily lock in.
- **Reviews must include consequences, not just the headline write.** Restoring Saved work deletes its ref afterwards; the original review only mentioned the `stash apply`, so the destructive half was invisible.
