# Git Simplifier

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
_Avoid_: unstage, rollback — and never label this **Revert**; that word is the separate worktree operation

**Revert**:
Overwrite selected *tracked* paths in the index and working tree from **HEAD** or from **Base**, without rewriting commits. The checklist is the union of tracked local dirt and `Base...HEAD` diffs; untracked paths are out of scope. Reverting to Base can leave the paths differing from HEAD as intentional local changes.
_Avoid_: uncommit, discard, checkout, restore — Git verbs and neighboring product terms must not replace this label

**Rewrite** (default mode of **Uncommit**):
Replacing an existing commit with one that never contained the file. History shows no trace of the accident. Reaches any commit that is on the current branch and not yet on **Base**; commits already on **Base** are out of reach and fall back to a **Removal commit**.
_Avoid_: amend (that word belongs to **Reword**), squash

**Removal commit** (alternative mode of **Uncommit**):
A new commit added on top that takes the file back to its **Base** content. The accident stays visible in history.
_Avoid_: cleanup commit — and never call this a “revert commit”; **Revert** is the worktree operation above

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

**Split branch**:
Creating a new branch that contains only some of the current branch's changes, selected by file path. Rooted at the merge base of **Base** and HEAD, so it carries the selected work and nothing else.
_Avoid_: extract, carve out, cherry-pick — cherry-pick moves whole commits, and Split works by path.

**Copy** (the only implemented mode of **Split branch**):
The original branch is left exactly as it was. The same change now exists on two branches, and removing it from the original is a separate decision the user makes later.
_Avoid_: duplicate, branch off

**Move** (not implemented):
The same selection, but the original branch must also lose the change — by **Rewrite** or by a revert commit. Deliberately out of scope: it is a different, destructive operation, and the word "split" hides the difference. UI must never offer "split" without saying which of the two it does.

**Undo**:
Reversing the effect of an operation the app itself performed. A safety net over any operation, not a git concept.
_Avoid_: rollback — and never label Undo as **Revert**

## Flagged ambiguities

- **"uncommit" vs "undo" vs "revert"** — neighboring spellings, unrelated meanings. **Uncommit** rewrites history and leaves the worktree alone; **Revert** overwrites the worktree/index and leaves history alone; **Undo** reverses an app operation. UI labels must never collapse any of these into another.

- **"split" alone is ambiguous** — it names two operations with opposite risk profiles (**Copy** and **Move**). Only **Copy** exists. Any label, message, or doc that says "split" without qualifying it is a bug.

- **"base" vs "upstream"** — the design doc uses both, plus a bare `develop`, interchangeably. Resolved above: they are different refs and never substitute for each other.
