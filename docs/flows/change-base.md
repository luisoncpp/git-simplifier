# Change Base

## Trigger

The user clicks **Set Base** or **Change** in the repository bar.

## Entry point

`events.ts` delegates `edit-base` to `AppController.editBase`.

## Sequence

1. The controller ignores the action while another operation is busy.
2. It asks Rust for the current `refs/remotes/*` choices through `list_base_choices`.
3. After the read succeeds, it stores the choices and opens the selector.
4. The user selects a remote-tracking ref and clicks **Save**; `set_base` persists it and returns a fresh snapshot.
5. The controller reloads operation and Inspection data from that snapshot.

## Reads

- Configured Base from the repository snapshot.
- Remote-tracking refs from `list_base_choices`.

## Writes

- `githelper.base` through `set_base` only after the user saves.

## Failure modes

- A failed choice read keeps the existing Base visible and reports the error through the normal workbench error state.
- An empty choice result renders the explicit fetch guidance; it is different from the list not having been loaded.

## Files to inspect

- `ui/app/Private/controller.ts`
- `ui/app/Private/discovery.ts`
- `ui/app/Private/views/shell.ts`
- `src-tauri/src/commands/actions.rs`
