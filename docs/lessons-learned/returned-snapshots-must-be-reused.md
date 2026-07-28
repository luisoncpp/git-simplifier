# Returned snapshots must be reused

A mutation that returns fresh repository state has already paid for the relevant Git scans. Discarding that response and calling the general refresh path repeats worktree status, ref discovery, and state aggregation.

Let one backend aggregation boundary produce the complete snapshot, derive counters from the collected values, and pass a returned snapshot into the refresh renderer. Load only action-specific data that is absent from the snapshot.

Repository identity such as the Git directory and Git version is stable for an open repository and should be cached by the runner rather than rediscovered during every snapshot. Resolve the Git directory lazily because tests and embedding callers may construct the runner before `git init`.
