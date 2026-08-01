# A merged branch has no commits unique to it

"Branches created by me" sounds like it should read the authors of the commits the branch added: `git log --format=%ae Base..branch`.

For the branches a cleanup feature targets, that range is **always empty**. Eligibility *is* "the tip is an ancestor of Base", so `Base..branch` contains nothing by construction. Every merge-base-derived range collapses the same way — `merge-base(Base, branch)` is the branch tip itself. Any "not reachable from X" definition where X includes Base yields the empty set, so the filter silently matches nothing (or everything, depending on which way the predicate is written).

The usable signals, in order of what they actually prove:

- **Tip commit author** (`%(authoremail:trim)` on the branch ref). For a merged branch the tip *is* the last commit made on it, because the merge commit lands on the mainline rather than on the branch. One `for-each-ref` field, no extra process, and it works identically for remote-tracking branches. Wrong when the last commit was someone else's merge of Base into the branch, or when someone else rebased it.
- **Reflog fork point** (`git merge-base --fork-point`). Most faithful, but expires with the reflog and never exists for a branch that was only ever fetched.

Cleanup uses the tip author and shows the email next to each row, so the approximation is visible rather than hidden behind a checkbox.

The general lesson: before writing a filter over a set, check whether the set's *defining property* makes the filter's input empty. Here the eligibility rule and the authorship rule were designed independently and were individually reasonable; their composition was vacuous.
