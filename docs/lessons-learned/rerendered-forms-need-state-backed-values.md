# A re-rendered form needs state-backed values, and the textarea needs an exception

Rendering the whole shell with `innerHTML` on every state change is fine for a workbench this size, but only if **no user intent lives in the DOM**. When it does, the bugs are silent rather than loud:

- A commit-message textarea seeded with `commits[0].message` never updated when the user picked a different commit. Selecting commit B and submitting sent A's text as B's new message — a wrong rewrite, not an error.
- Selections, filter queries, and checkbox state vanished on every refresh, and cancelling a review discarded the selection the user was reviewing.

Keeping every control's value in a draft object fixes all of it at once, and makes reconciliation explicit: after discovery runs again, selections that no longer exist in the repository are dropped instead of being sent back to Rust.

Two things a full re-render still breaks, both worth handling once in the render helper:

- **Caret and scroll.** Capture the caret of the element carrying a stable `data-focus` key and the offset of every `[data-scroll]` container, then reapply after the swap. Compare keys as dataset values — interpolating a repository path into `[data-focus="…"]` throws on a filename containing a quote.
- **Native undo.** Replacing a textarea's node destroys its undo stack, so Ctrl+Z stops working mid-message. Free-text fields where undo matters must not be re-rendered while being typed into; record the draft and patch only the controls that depend on it.

Related: a per-commit draft map means switching commits and back restores the edit instead of silently discarding it.
