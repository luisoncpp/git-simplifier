# Auto-defaults must not stick like user picks

Discovery reconcile (`adoptBranch`, and the same pattern elsewhere) keeps a draft value when it is still present in the fresh list. That is correct for a choice the user made; it is wrong for an automatic fallback (first row, alphabetical branch, etc.).

If the fallback ran while Base was missing—or while matching failed—the draft stays on that row forever after Base appears, because “still valid” looks like intent.

Mark auto-filled draft fields separately from user picks (`branchPicked`, or clear the field before re-adopting). Only user picks survive refresh; auto-defaults may be recomputed whenever discovery runs.
