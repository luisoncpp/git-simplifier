# Failed repository opens must not look successful

`AppState::open_path` used to set `init_error` on failure while leaving the previous `GitRepository` in place. `open_repository` then called `snapshot`, which prefers the live repository over `init_error`, so the UI received a successful snapshot of the *old* path after a bad picker choice.

Any open entry point must return `Err` when `GitRepository::open` fails. Keeping the previous session is correct; reporting success for the failed path is not. Recent-list recording belongs after a successful open only.
