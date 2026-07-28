# Operation review flow

Every mutation follows **Select → Review → Apply → Result**. Discovery returns typed branches, commits, paths, and gitlinks; the UI never asks for a path or SHA. `prepare_operation` stores one typed plan in `AppState` and returns impact, preserved state, warnings, exact commands, and the final action label.

Apply consumes the matching plan ID once. Cancel removes it. Repository switches, stale HEADs, changed sync fingerprints, and cancellation invalidate the review. After a successful rewrite, Force push is a separate explicit review using `--force-with-lease`.
