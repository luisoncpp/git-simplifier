# Git Helper UI

A compact dark workbench, not a preview dashboard. Vanilla JavaScript, one deep module: `ui/app/index.js` is the public interface and everything under `ui/app/Private/` is implementation. See [docs/architecture/workbench-ui.md](../docs/architecture/workbench-ui.md) for the state and rendering rules.

Run `npm.cmd run tauri dev` for desktop repository access. Browser mode deliberately reports that it cannot reach a repository and shows the reason of the last attempt; it contains no authoritative fixtures. Test fixtures may be supplied to `FixtureBridge` only.

All writes go through `prepare_operation` → `OperationReview` → `apply_operation` or `cancel_operation`. JavaScript sends typed identifiers selected from Rust discovery results and never reconstructs Git commands. A failed repository open leaves the last valid repository active.

## Layout

Uncommit and Split branch share the changed-path checklist but keep separate selections, because a tick means the opposite thing in each. A repository rail, a single-row repository bar (branch, Base with a Change affordance, upstream, working-tree chips), the operation tab strip, and the form. The rail's Inspection group starts with a read-only Branch diff that can be copied in one click. A pending review opens as a second column beside the form and stacks below 860 CSS pixels. There is no decorative page heading in operation screens; the tab strip is the title.

## Keyboard

- `Tab` reaches every control; the operation strip is a single tab stop with `Left`/`Right` moving between operations.
- `Escape` cancels a pending review.
- Disabled primary actions always render the reason next to them.

## Checks

```bash
npm run lint && npm test
```

`npm run lint` syntax-checks every script under `ui/`; `npm test` runs the workbench tests in `test/`.
