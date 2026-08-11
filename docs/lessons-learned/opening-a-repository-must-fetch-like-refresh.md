# Opening a repository must fetch like Refresh

Refresh was taught to run `git fetch --all` so an unreachable remote becomes a dismissible **Fetch failed** warning. Opening another repository only called `reload` with the snapshot, so the same dead tunnel looked healthy until the user pressed Refresh.

Keep one contract: after the live session points at a path, learn whether its remotes are reachable before the user trusts Base / Sync / publish. A failed fetch must not block the local snapshot — warning, not hard error.
