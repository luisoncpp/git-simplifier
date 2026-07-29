# Close-to-tray `prevent_close` also blocks Quit unless exit is armed

Hiding on `CloseRequested` with `api.prevent_close()` is the right close-to-tray pattern, but `app.exit` still closes windows through that same event. If every close is prevented, Quit appears to hang or do nothing.

Arm an `ExitAllowed` (or equivalent) flag in the Quit handler *before* `app.exit`, and only call `prevent_close` when that flag is false. Do not assume exit bypasses window close handling.

See [desktop-shell.md](../architecture/desktop-shell.md) and [close-to-tray.md](../flows/close-to-tray.md).
