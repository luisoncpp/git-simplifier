# Discoverable Git identifiers must be selectable

The UI must not make users type repository paths, branch names, commit IDs, or submodule paths that Rust already knows. Discovery belongs in typed core queries; selectors carry those values to a review request. Immutable core plans must cross the Tauri boundary intact so the UI cannot silently change commands or bypass impact review.
