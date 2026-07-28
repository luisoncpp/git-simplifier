# Superproject operations must separate nested dirt from recursive mutation

Porcelain v2 with `--ignore-submodules=none` reports a dirty submodule as a tracked superproject record. That does not mean `git stash create` can preserve the submodule's checked-out commit, tracked modifications, or untracked files. Treating that record as Saved work can produce an empty snapshot or an unnecessary safety rejection.

For an operation that owns only the superproject, such as Sync or Quick switch:

- use `--ignore-submodules=all` when deciding whether a superproject snapshot is needed;
- explicitly disable recursion for every applicable fetch, stash, reset, merge, switch, and stash-reapply command instead of relying on repository or global configuration;
- leave the nested repository untouched, which preserves its actual worktree state.

This separation still allows staged superproject changes to be snapshotted while preventing `submodule.recurse=true` from broadening the operation's mutation boundary.
