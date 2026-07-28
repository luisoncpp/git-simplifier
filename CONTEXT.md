# Git Helper

A desktop tool that turns routine-but-awkward git operations into single, safe actions, running alongside the user's existing git clients on the same repository.

## Language

**Base**:
The remote branch that the current work is meant to land on (e.g. `origin/develop`, `origin/master`). Always a remote ref — never the local branch of the same name. Exactly one per repository; branches do not have individual bases.
_Avoid_: base branch, target branch, trunk, mainline

**Upstream**:
The remote-tracking ref of the *current* branch (`@{upstream}`), i.e. where this branch itself is pushed. Distinct from **Base**: a feature branch's upstream is `origin/feature-x`, while its base is `origin/develop`.
_Avoid_: remote, origin

**Editable range**:
The commits this app is allowed to rewrite: those reachable from HEAD by following **first parents only**, minus those already on **Base**. Commits that arrived through a merged side branch are outside it and are never rewritten, even though they are on the branch and not on **Base**.
_Avoid_: unmerged commits, my commits, local commits

**Uncommit**:
Removing one or more files from the current commit so that **Base** is what HEAD records for those paths, while the working-tree version of the file is left untouched. An operation the user requests.
_Avoid_: revert, unstage, rollback

**Rewrite** (default mode of **Uncommit**):
Replacing an existing commit with one that never contained the file. History shows no trace of the accident. Reaches any commit that is on the current branch and not yet on **Base**; commits already on **Base** are out of reach and fall back to a **Removal commit**.
_Avoid_: amend (that word belongs to **Reword**), squash

**Removal commit** (alternative mode of **Uncommit**):
A new commit added on top that takes the file back to its **Base** content. The accident stays visible in history.
_Avoid_: revert commit, cleanup commit

**Edit message**:
Changing the message of any commit that is on the current branch and not yet on **Base**, leaving its tree, parents and authorship exactly as they were.
_Avoid_: amend, reword — `git commit --amend` also sweeps staged changes into the commit, which is precisely what this operation must not do, so the word must not appear in the UI or the code.
_Note_: the text being changed is the commit **message**; "description" means the same thing but only one word should appear in the product.

**Excluded submodule**:
A submodule the user has declared off-limits: its pointer must never appear in local changes and must never be committed from this repository. Exclusion is a standing rule, not an operation — it combines hiding the submodule from status, blocking it at commit time, and cleaning any pointer already committed within the **Editable range**.
_Avoid_: pinned submodule, frozen submodule, ignored submodule

**Saved work**:
A snapshot of a branch's uncommitted *tracked* changes, held under `refs/githelper/wip/<branch>` so it belongs to that branch by name rather than to a stack position. Untracked files are never part of it — they stay in the working tree.
_Avoid_: stash, WIP, shelved changes — the shared stash stack is a different thing and the app does not use it.

**Undo**:
Reversing the effect of an operation the app itself performed. A safety net over any operation, not a git concept.
_Avoid_: rollback, revert

## Flagged ambiguities

- **"uncommit" vs "undo"** — three letters apart, unrelated meanings. **Uncommit** is a git operation the user asks for; **Undo** reverses an app operation. "Undo the uncommit" is legal and means the opposite of "uncommit". UI labels must never abbreviate either into the other.

- **"base" vs "upstream"** — the design doc uses both, plus a bare `develop`, interchangeably. Resolved above: they are different refs and never substitute for each other.
