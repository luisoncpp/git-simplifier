# A failed stash apply may already have mutated the repository

`git stash apply --index` can return exit code 1 after writing conflict markers and unmerged index entries. Treating every failure as permission to retry with a plain `stash apply` runs a second mutation over an already-conflicted repository and hides the useful conflict outcome.

Before using the non-index fallback, check whether the indexed attempt created unmerged paths. If it did, stop, preserve the stash or private Saved work ref, and direct the user to resolve the conflicts.
