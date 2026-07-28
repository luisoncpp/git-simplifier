# Failed mutations require a state refresh

A mutation error does not imply that nothing changed. Composite Git operations can durably record a recovery phase before stopping, and the Tauri boundary consumes an operation review before executing it.

After any failed reviewed mutation, clear the consumed review and reload the repository snapshot. Preserve the original mutation error when the refresh succeeds. This keeps recovery actions aligned with the real repository and prevents retrying a stale review.
