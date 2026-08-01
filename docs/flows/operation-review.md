# Operation review flow

## Trigger

The user submits an operation form, or clicks a shortcut that prepares one directly: force push after a rewrite, sync resume, Saved work restore/delete, or "Switch to <branch>" from the Saved work section.

## Sequence

1. `submitState` blocks the submit button while the selection is incomplete and renders the reason. Nothing reaches Rust.
2. `buildRequest` turns `state.draft` into one flat `prepare_operation` payload of typed identifiers taken from discovery. The UI never types a path, ref, or SHA.
3. `commands/prepare/` validates the input, builds the plan, stores exactly one `PendingOperation` in `AppState`, and returns impact, preserved state, warnings, exact commands, and the apply label.
4. **Review mode (default):** the review renders beside the form, and focus moves to its title. The selection stays intact behind it. Apply consumes the plan by id. Cancel — the button or Escape — releases it. Changing operation or section cancels first rather than abandoning the plan.
5. **Skip mode:** when `state.skipReview` is on (repo-bar **Review | Skip** toggle, persisted in app data), the primary action is labeled **Apply …** and `prepare` immediately calls `apply_operation` with the returned `plan_id` in the same busy turn — no review pane, no focus jump.
6. On success the controller reuses the snapshot side effects by reloading once, shows the outcome, and offers force push only when the operation was a rewrite.
7. On failure the review is discarded (or was never shown in Skip mode), state is reloaded before the error is shown, and the error banner keeps the Git message verbatim.

## Reads

- `state.draft` for user intent, `state.snapshot.overview` for Base, branch, and upstream.
- `state.skipReview` for whether apply follows prepare immediately (loaded from `ui-preferences.json` on start).
- Rust discovery results for the identifiers being sent.

## Writes

- Nothing until apply. `prepare_operation` only stores a plan in memory.

## Invalidations

Repository switches, stale HEADs, changed sync fingerprints, cancellation, and an already-consumed plan id all reject the apply with an explicit message.

## Files to inspect

- `ui/app/Private/operations.ts`, `controller.ts`, `preferences.ts`, `review-mode.ts`, `views/review.ts`, `views/shell.ts`
- `src-tauri/src/commands/prepare/`, `apply.rs`, `prefs.rs`, `review_commands/`

## Common failure modes

- A review whose commands were hand-written drifts from the code that applies it; the builders in `review_commands/` derive every line from the plan to prevent that.
- A composite operation that failed mid-way may have written recoverable state, so the snapshot is reloaded before the error is surfaced.
