# Git Helper UI

The UI is a compact dark workbench, not a preview dashboard. It is vanilla JavaScript split into a thin entry point, `AppController`, a host bridge, and small view functions. The controller owns repository data, navigation, pending review, busy/error state, and refresh.

Run `npm.cmd run tauri dev` for desktop repository access. Browser mode deliberately shows “Desktop repository access unavailable”; it contains no authoritative fixtures. Test fixtures may be supplied to `FixtureBridge` only.

All writes use `prepare_operation` → `OperationReview` → `apply_operation` or `cancel_operation`. JavaScript sends typed identifiers selected from Rust discovery results and never reconstructs Git commands. A failed repository open leaves the last valid repository active.

The workbench supports Base selection, uncommit, edit message, submodule exclusion, quick switch, sync/resume, saved-work restore/delete, force-push continuation, and recovery inspection. Controls are keyboard reachable, have visible focus, and collapse below 760 CSS pixels.
