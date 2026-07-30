# Flow Docs

Operational guides organized by user or system action: "when this happens, everything that follows is this."

## Purpose

Use `docs/flows/` when you need to follow behavior end-to-end from a trigger instead of from a subsystem boundary.

This folder is for:

- debugging a concrete user action
- understanding which functions run in sequence
- finding which state is read, written, or only projected for UI
- locating side effects quickly without codebase-wide search

## How this differs from other doc types

- `docs/architecture/`
  - explains what a subsystem is, its design, and its invariants
- `docs/flows/`
  - explains what happens when an action occurs
- `docs/lessons-learned/`
  - explains counter-intuitive facts discovered while working in the area
- `docs/plan/`
  - explains how to change or refactor something

## Recommended format

Each flow doc should try to include:

1. Trigger
2. Entry point
3. Step-by-step sequence
4. Reads
5. Writes
6. Side effects
7. Files to inspect
8. Common failure modes

Keep these docs operational. Prefer short tables, explicit file names, and sequence lists over long essays.

| File | Scope |
|------|-------|
| [close-to-tray.md](./close-to-tray.md) | Window close hides to tray; tray Show/Quit |
| [raw-diff.md](./raw-diff.md) | Generate and copy the current branch's committed `Base...HEAD` patch as text |
| [files-diff.md](./files-diff.md) | Per-file diff viewer: unified/side-by-side, gap expansion, file navigator |
| [edit-message.md](./edit-message.md) | Backend Edit message planning and application |
| [excluded-submodule.md](./excluded-submodule.md) | Backend Excluded submodule configuration and guard installation |
| [force-push.md](./force-push.md) | Backend explicit force-push after a rewrite |
| [recovery-panel.md](./recovery-panel.md) | Backend operation history and ref-only recovery guidance |
| [quick-switch.md](./quick-switch.md) | Backend Quick branch switch and Saved work restoration |
| [switch-repository.md](./switch-repository.md) | Recent repository menu, persistence, and open/prune |
| [split-branch.md](./split-branch.md) | Backend Split branch copy of selected paths onto a new branch |
| [sync.md](./sync.md) | Backend Sync with Base, Saved work reapply, and resumable conflicts |
| [uncommit-rewrite.md](./uncommit-rewrite.md) | Backend Uncommit planning and application |
