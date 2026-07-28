# Path-limited index reset preserves unrelated staged work

After a history rewrite that must leave the worktree untouched, reset only the selected paths in the real index. A whole-index reset would discard unrelated staged changes that the operation never intended to modify. The temporary index used to rebuild commits must remain separate from this path-limited cleanup.
