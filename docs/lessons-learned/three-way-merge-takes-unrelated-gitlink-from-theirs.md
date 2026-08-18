# A three-way merge records their gitlink; pinning ours puts it in the PR

Verified against the Commit merge fixture: Base moves `wiki`, the feature branch only conflicts on `README.md`. Before the merge, `Base...HEAD` does not list `wiki`. After a normal three-way merge it still must not.

`git read-tree -m <base> HEAD MERGE_HEAD` treats a gitlink like any other blob. If ours matches the merge base and theirs does not, the result is theirs. That matches Base, so `Base...HEAD` after the merge still omits `wiki`. GitHub and other MR views follow that range.

Writing HEAD's gitlink back into the merge tree looks like "the branch never touched the submodule", but it is a new diff versus Base. The MR then *gains* `wiki` even though the pre-merge file list did not include it.

`submodule.<path>.ignore = all` hides that pointer from `git diff --name-only` unless `--ignore-submodules=none` is passed, so a `Base…HEAD` subset check can miss the extra path. The excluded-submodule guard compares the index to HEAD, so a legitimate theirs gitlink looks staged. Allow the guard when the staged pointer matches `MERGE_HEAD`; do not rewrite the merge tree to ours.
