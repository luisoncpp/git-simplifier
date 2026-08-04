# Local untracked list must not read ignore-tree bodies

When Local Files diff includes gitignored paths (Respect gitignore off), it unions ordinary and gitignored `ls-files -o` results so those paths can appear without embedding their contents.

Reading every ignored body into that list fails on real repos: `target/`, `.cargo-target/`, and similar trees hold tens of thousands of artifacts. The IPC payload OOMs or times out, `fileDiffs` stays null, and the UI shows "No local changes" even when tracked dirt is present.

List gitignored and `node_modules` entries as incomplete stubs (annotations only). Load bodies lazily through `full_file_diff` when a filter reveal hydrates them. Default filters must also skip the ignored walk — see `untracked-filters-must-constrain-ls-files.md`.
