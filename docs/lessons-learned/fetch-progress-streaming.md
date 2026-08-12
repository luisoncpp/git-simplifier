# Fetch progress streaming and cancellation

Git only emits progress on stderr, and only when it thinks a terminal is watching — a piped fetch is silent unless `--progress` is passed. Updates to the same meter are separated by `\r`, not `\n`, so a reader must split on both.

Cancellation cannot hold the child lock across a blocking `wait`: the cancel command needs that same lock to kill. Take the stderr handle out of the child, register the child in a shared slot, read to EOF (killing the process is what unblocks the read), then remove the child from the slot before waiting.

A backend event can arrive after the invoking command already settled (queued IPC). The UI must drop progress events received while no fetch is active, or a finished bar lights back up.
