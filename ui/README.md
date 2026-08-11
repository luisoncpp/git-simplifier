# Git Simplifier UI

A compact dark workbench, not a preview dashboard. Vanilla TypeScript (strict), one deep module: `ui/app/index.ts` is the public interface and everything under `ui/app/Private/` is implementation. Wire shapes mirror the Rust contracts in `ui/app/Private/types.ts`. See [docs/architecture/workbench-ui.md](../docs/architecture/workbench-ui.md) for the state and rendering rules.

Run `npm.cmd run tauri dev` for desktop repository access. Rust build artifacts land in `.cargo-target/` (shared by the workspace); run `npm run clean:rust` to reclaim disk space. Browser mode deliberately reports that it cannot reach a repository and shows the reason of the last attempt; it contains no authoritative fixtures. Test fixtures may be supplied to `FixtureBridge` only.

`prismjs` is the only runtime dependency. It is imported lazily and only when a `document` exists, so syntax highlighting degrades to escaped plain text everywhere else — which is why `npm test`, which loads these sources with no bundler, never resolves it.

All writes go through `prepare_operation` → `OperationReview` → `apply_operation` or `cancel_operation`. The UI sends typed identifiers selected from Rust discovery results and never reconstructs Git commands. A failed repository open leaves the last valid repository active.

## Layout

Uncommit and Split branch share the changed-path checklist but keep separate selections, because a tick means the opposite thing in each. A repository rail, a single-row repository bar (branch, Base with a Change affordance, upstream, working-tree chips), the operation tab strip, and the form. The rail's Inspection group holds two read-only views: **Files diff** (per-file, unified by default with a side-by-side toggle, a **HEAD / Local** compare toggle, gaps expandable up/down/all, and a collapsible file navigator that starts closed) and **Raw diff**, the same patch as copyable text with the same compare toggle. A pending review opens as a second column beside the form and stacks below 860 CSS pixels. There is no decorative page heading in operation screens; the tab strip is the title.

## Keyboard

- `Tab` reaches every control; the operation strip is a single tab stop with `Left`/`Right` moving between operations.
- `Escape` cancels a pending review.
- Disabled primary actions always render the reason next to them.

## Checks

```bash
npm run lint && npm test
```

`npm run lint` type-checks the UI with `tsc --noEmit`; `npm test` runs the workbench tests in `test/`, which import the `.ts` sources directly (Node strips the types).
