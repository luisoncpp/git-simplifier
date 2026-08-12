# Stash apply can refuse dirty overlap without leaving markers

`git stash apply` three-way merges when it can. When the working tree already has unstaged edits on the same paths, Git often aborts with `local changes would be overwritten by merge` and leaves the tree unchanged — no conflict markers, nothing for a mergetool to open.

Parking that dirt (`stash create` + durable ref), resetting hard, applying Saved work onto the clean tree, staging the result, then reapplying the park turns the refuse into a real merge (clean or conflicted). Do not treat overwrite-refuse like a conflicted apply: there are no unmerged paths to detect, and a plain retry will fail the same way.

After the first apply, stage before the park apply. A plain stash apply leaves the worktree dirty versus the index; the second apply would hard-refuse for the same reason.
