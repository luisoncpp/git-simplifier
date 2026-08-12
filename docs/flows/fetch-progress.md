# Fetch progress and cancellation

## Trigger

App start, the repo-bar **Refresh** button, or opening another repository — anything that calls the `fetch_remotes` command.

## Entry point

UI: `discovery.ts` `fetchRemotes` (spans `state.fetch.active`), `controller.ts` `onFetchProgress` / `cancelFetch`, `views/status-bar.ts`.
Tauri: `commands/actions.rs` `fetch_remotes` / `cancel_fetch`; core `src/inspection/fetch/`.

## Step-by-step sequence

1. The UI marks `state.fetch.active` and invokes `fetch_remotes`.
2. Rust creates a `FetchControl`, stores it in `AppState`, and spawns `git fetch --all --no-tags --no-recurse-submodules --progress` with stderr piped and stdout nulled.
3. The fetch loop parses `\r`/`\n`-separated stderr fragments; each recognized `phase: N% (done/total)` line becomes a `FetchProgress` emitted as the `fetch-progress` event. Non-progress lines accumulate into a 20-line error tail.
4. The UI listener updates `state.fetch` and re-renders the status footer; events arriving after the command settled are dropped.
5. The stop button arms cancel on `pointerdown` (not `click`, so a progress re-render cannot swallow the press). That invokes `cancel_fetch`, which kills the Git process tree through the shared `FetchControl`. The killed process closes stderr; the loop sees EOF and reports `FetchStatus::Cancelled`, which the command maps to success — no warning banner.
6. When the command settles, the UI clears `state.fetch.active` and reloads the snapshot so moved remote-tracking refs show up.

## Reads

- Nothing beyond the fetch itself and the usual snapshot reload.

## Writes

- Remote-tracking refs, via Git only. No app files.

## Side effects

- One `fetch-progress` event per parsed progress fragment; `cancel_fetch` kills the Git process.

## Files to inspect

- `src/inspection/fetch/mod.rs`, `src/inspection/fetch/progress.rs`
- `src-tauri/src/commands/actions.rs`, `src-tauri/src/commands/state.rs`
- `ui/app/Private/discovery.ts`, `ui/app/Private/controller.ts`, `ui/app/Private/views/status-bar.ts`

## Common failure modes

- Remote unreachable → `fetch_remotes` returns the Git stderr tail as the error; the UI shows the dismissible **Fetch failed** warning and still reloads local state.
- Cancel pressed before spawn or after completion → immediate `Cancelled` or a no-op; never an error.
- A killed Git can leave a transport child (e.g. `ssh`) briefly alive; it exits when its transport pipes break, which is what finally closes stderr.
