# Write-lock owners must not re-lock

The repository write mutex is not reentrant. `GitRunner::run` already locks `GitCommand::write`, so wrapping a single write in `GitRunner::with_write_lock` deadlocks when the command tries to acquire the same mutex.

Use one lock owner:

- A standalone write calls `GitRunner::run`.
- A multi-command transaction holds `with_write_lock` at the `GitRepository` boundary and calls `run_unlocked` internally.

Cover standalone mutations with a timeout regression test so a re-lock fails quickly instead of hanging the suite or UI.
