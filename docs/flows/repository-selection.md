# Repository selection flow

1. Select **Open a repository**. Tauri opens the official single-directory native picker.
2. Rust validates and snapshots the candidate before replacing the active `GitRepository`.
3. On success, the overview and all selectors refresh from live Git data.
4. On failure, the previous repository remains active and an assertive error is shown.

When `githelper.base` is absent, the workbench displays discovered remote refs. The user selects and confirms one; Rust persists the exact ref locally with one write command and returns the refreshed repository snapshot. The write relies on `GitRunner::run` for locking and must not wrap that call in another repository write lock. The controller reuses the returned snapshot and only loads the newly enabled operation data; it must not request a second full snapshot. Base-dependent actions stay disabled until Base is saved.
