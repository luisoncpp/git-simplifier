# Fetch Progress, Cancellation, and Paint-Before-Fetch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Show a live progress bar with a stop button in the status bar during fetches, and paint the target repository's local state before the fetch completes (app start, Refresh button, and repository switch).

**Architecture:** The Rust core gains a streaming fetch: it spawns `git fetch --progress` with stderr piped, parses `\r`/`\n`-separated progress fragments into `FetchProgress` values, and reports them through a callback. A shared `FetchControl` handle (cancel flag + child handle) lets a new `cancel_fetch` Tauri command kill the process; a killed fetch reports `FetchStatus::Cancelled` instead of an error. The Tauri `fetch_remotes` command re-emits progress as a `fetch-progress` event; the UI listens, keeps `state.fetch` current, and renders a progress bar + red ✕ stop button in the status footer. The open-repository flow reloads from the returned snapshot *before* fetching, then reloads again after the fetch.

**Tech Stack:** Rust (git-helper-core + Tauri v2 shell, std only — no new dependencies), strict TypeScript workbench UI (no framework), Node's built-in test runner (`test/*.test.mjs`), Cargo integration fixtures (`tests/*_fixtures.rs`).

## Global Constraints

- Functions take at most 3 parameters; functions stay ≤ 30 lines; source files stay ≤ 200 lines (docs/GUIDELINES.md).
- Hardcoded callbacks and hardcoded boolean/extra arguments get `/*name=*/` comments simulating named parameters.
- Deep modules: `ui/app/index.ts` is the only public UI interface; `ui/app/Private/*` internals are never imported from outside (tests may import `Private` files directly). UI imports carry explicit `.ts` extensions; type-only imports use `import type` (`verbatimModuleSyntax` + `erasableSyntaxOnly`, enforced by `npm run lint` = `tsc --noEmit`).
- No new runtime dependencies (Rust or npm). Progress parsing is hand-rolled — no regex crate.
- Every new Tauri command must be declared `#[tauri::command(async)]`; `test/ui-smoke.test.mjs` ("Git-backed Tauri commands run off the window thread") fails otherwise.
- The fetch argv stays `fetch --all --no-tags --no-recurse-submodules`, plus the new `--progress`.
- `src-tauri` has `rust-version = "1.77.2"` — no standard-library APIs newer than 1.77 anywhere in the workspace.
- Verification commands: `npm run lint`, `npm test`, `cargo test`, `npx fallow audit`.
- Commit after every task. Conventional-commit style messages (`feat:`, `fix:`, …).

## "Can the update itself be sped up?" — investigated answer

The user asked whether we could download only branch names. **No**, and the plan deliberately does not try:

- **Refs-only (`git ls-remote` + `update-ref`) is not viable.** Remote-tracking refs would point at objects we never downloaded. Base diffs (`Base...HEAD` merge-base), Sync's merge, Cleanup's merged-branch discovery, and Quick switch's remote-only checkouts all read those objects; they would fail or silently misbehave.
- **`--filter=blob:none` (blobless fetch) moves the wait instead of removing it.** Blobs would be fetched on demand at first read — this app reads blobs constantly (Files diff, Raw diff), so the first diff after opening would stall, and offline diffs would break. Not a fit.
- **`git fetch --all --jobs=N`** only parallelizes across remotes (most repos have one) and interleaves the progress output we parse. Skipped.
- **What actually helps ships in this plan:** the UI stops waiting on the network (local state paints first), the user can cancel a huge fetch, and Git's existing negotiation already downloads only new objects (`--no-tags` already skips tag churn).

## File Structure

**Create:**
- `src/inspection/fetch/progress.rs` — pure progress-line parsing (`FetchProgress`, `parse_progress`) and the `ProgressFeed` chunk splitter that turns a stderr byte stream into events + an error tail.
- `tests/fetch_fixtures.rs` — integration tests: progress events + ref advance, cancel-before-spawn, mid-flight cancel against a hung HTTP remote, failure stderr reporting.
- `test/ui-fetch.test.mjs` — UI tests: fetch state lifecycle, progress-event guard, status bar markup, stop-button dispatch, paint-before-fetch ordering for Refresh.
- `ui/app/Private/views/status-bar.ts` — status footer markup (spinner / fetch progress bar + stop button / review hint / ready). Extracted from `shell.ts` so `shell.ts` stays under the 200-line limit.
- `docs/flows/fetch-progress.md` — flow doc (trigger → sequence → files → failure modes).
- `docs/lessons-learned/fetch-progress-streaming.md` — `--progress` + `\r` splitting; never hold the child lock across a blocking wait; drop late events.

**Modify (Rust core):**
- `src/inspection/fetch.rs` → moved to `src/inspection/fetch/mod.rs`: `FetchControl`, `FetchStatus`, `fetch_remotes_with_progress`, stderr drain loop, finish/status mapping. The old blocking `fetch_remotes` is removed.
- `src/inspection/mod.rs` — export `FetchControl`, `FetchProgress`, `FetchStatus`; replace the wrapper.
- `src/git/process.rs` — add `spawn_piped` (piped stderr, null stdout).
- `src/git/mod.rs` — `GitRunner::spawn_piped` wrapper.
- `src/git/error.rs` — add `GitError::Io` (mid-stream read/wait failures are not spawn failures).
- `src/repository/read.rs` — `GitRepository::fetch_remotes_with_progress` replaces `fetch_remotes`.
- `src/lib.rs` — re-export the three new types.
- `tests/inspection_fixtures.rs` — two existing fetch tests updated to the new API.

**Modify (Tauri shell):**
- `src-tauri/src/commands/state.rs` — `AppState.fetch` slot + `register_fetch` / `cancel_fetch` / `clear_fetch`.
- `src-tauri/src/commands/actions.rs` — `fetch_remotes` registers the control, emits `fetch-progress` events, maps errors; new `cancel_fetch` command.
- `src-tauri/src/lib.rs` — register `cancel_fetch` in the invoke handler.

**Modify (UI):**
- `ui/app/Private/types.ts` — `FetchProgressEvent`, `FetchState`, `AppState.fetch`, `Bridge.listen`.
- `ui/app/Private/state.ts` — initial `fetch` value in `createState`.
- `ui/app/Private/bridge.ts` — `TauriBridge.listen` via `__TAURI__.event.listen`; `FixtureBridge.listen` + `emitEvent` test hook.
- `ui/app/Private/controller.ts` — `start()` subscribes to `fetch-progress`; `onFetchProgress`; `cancelFetch`; `refresh()` paints local state first when nothing is on screen.
- `ui/app/Private/discovery.ts` — `fetchRemotes` manages `state.fetch.active` around the invoke.
- `ui/app/Private/views/shell.ts` — use `statusBar`, delete the old `status()`.
- `ui/app/Private/event-tables.ts` — `cancel-fetch` click entry.
- `ui/app/Private/repository-switcher.ts` — `openRepositoryPath` reloads from the returned snapshot before fetching.
- `ui/styles/workbench.css` — `.fetch-progress`, `.fetch-fill`, `.fetch-stop` styles.

**Modify (existing tests):**
- `test/ui-repo-switcher.test.mjs` — `withRecents` stub tracks the open repository; two new ordering tests.

**Modify (docs):**
- `docs/architecture/workbench-ui.md` — the "Refresh fetches before it reloads" state rule changes; `views/status-bar.ts` row added.
- `docs/architecture/git-core.md` — streaming fetch + `FetchControl` bullet.
- `docs/flows/switch-repository.md` — step 6 sequence, side effects, files to inspect.
- `docs/flows/README.md`, `docs/lessons-learned/README.md` — index rows.

---

### Task 1: Streaming, cancellable fetch in the Rust core

**Files:**
- Move: `src/inspection/fetch.rs` → `src/inspection/fetch/mod.rs`
- Create: `src/inspection/fetch/progress.rs`
- Modify: `src/git/error.rs`, `src/git/process.rs`, `src/git/mod.rs`, `src/inspection/mod.rs`, `src/repository/read.rs`, `src/lib.rs`, `src-tauri/src/commands/actions.rs` (Step 7's keep-it-compiling adaptation only)
- Test: `tests/fetch_fixtures.rs` (new), `src/inspection/fetch/progress.rs` (inline unit tests), `tests/inspection_fixtures.rs` (2 call-site updates)

**Interfaces:**
- Consumes: existing `GitRunner` internals (`self.git`, `self.repo` in `src/git/mod.rs`), the `hide_console`/`set_environment` helpers in `src/git/process.rs`, `GitError`, `InspectionError` (`From<GitError>` already exists), `tests/support/fixture_repo.rs` (`add_bare_origin`, `switch_to_base`, `switch_to_feature`, `commit_file`, `head`).
- Produces (used by Task 2 and by tests):
  - `git_helper_core::FetchProgress { phase: String, done: u64, total: u64 }` — derives `Clone, Debug, Serialize, Eq, PartialEq`.
  - `git_helper_core::FetchStatus` — `Completed | Cancelled`; derives `Clone, Copy, Debug, Eq, PartialEq`.
  - `git_helper_core::FetchControl` — `Clone + Default`; methods `new()`, `cancel()`, `is_running()`.
  - `GitRepository::fetch_remotes_with_progress(&self, control: &FetchControl, on_progress: impl FnMut(FetchProgress)) -> Result<FetchStatus, InspectionError>` — holds the repo write lock for the whole fetch. This replaces `GitRepository::fetch_remotes`, which is deleted (its only callers are the Tauri command and the two fixture tests updated here).

- [ ] **Step 1: Move the fetch module and write the failing parser/feed tests**

```powershell
git mv src/inspection/fetch.rs src/inspection/fetch/mod.rs
```

In `src/inspection/fetch/mod.rs`, add `mod progress;` at the top (keep the existing `fetch_remotes` function unchanged for now — it is replaced in Step 5).

Create `src/inspection/fetch/progress.rs` containing only these tests for now:

```rust
#[cfg(test)]
mod tests {
    use super::{parse_progress, ProgressFeed};

    #[test]
    fn parses_a_local_progress_line() {
        let progress =
            parse_progress("Receiving objects:  45% (123/273), 1.10 MiB | 2.00 MiB/s").unwrap();
        assert_eq!(progress.phase, "Receiving objects");
        assert_eq!(progress.done, 123);
        assert_eq!(progress.total, 273);
    }

    #[test]
    fn parses_a_remote_side_progress_line() {
        let progress = parse_progress("remote: Compressing objects: 100% (8/8), done.").unwrap();
        assert_eq!(progress.phase, "Compressing objects");
        assert_eq!(progress.done, 8);
        assert_eq!(progress.total, 8);
    }

    #[test]
    fn ignores_lines_without_a_progress_counter() {
        assert!(parse_progress("remote: Enumerating objects: 5, done.").is_none());
        assert!(parse_progress("From https://example.test/repo").is_none());
        assert!(parse_progress("fatal: unable to access 'https://x': gone").is_none());
        assert!(parse_progress("").is_none());
    }

    #[test]
    fn feed_reports_progress_across_carriage_return_updates() {
        let mut feed = ProgressFeed::new();
        let mut events = Vec::new();
        feed.push(
            b"Receiving objects:  10% (10/100)\rReceiving obj",
            &mut |event| events.push(event),
        );
        feed.push(
            b"ects:  20% (20/100), done.\nFrom https://example.test/repo\n",
            &mut |event| events.push(event),
        );
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].done, 10);
        assert_eq!(events[1].done, 20);
        assert_eq!(feed.tail_text(), "From https://example.test/repo");
    }

    #[test]
    fn feed_keeps_only_the_last_tail_lines() {
        let mut feed = ProgressFeed::new();
        for index in 0..30 {
            feed.push(format!("line {index}\n").as_bytes(), &mut |_| {});
        }
        let tail = feed.tail_text();
        assert!(tail.starts_with("line 10\n"));
        assert_eq!(tail.lines().count(), 20);
    }
}
```

Run: `cargo test --lib fetch`
Expected: FAIL to compile — `parse_progress` and `ProgressFeed` do not exist.

- [ ] **Step 2: Implement the parser and the feed**

Put the following above the `#[cfg(test)]` module in `src/inspection/fetch/progress.rs` (the tests from Step 1 stay unchanged at the bottom):

```rust
use serde::Serialize;

/// One parsed `phase: N% (done/total)` fragment from `git fetch --progress`.
/// Serialized to the UI as the `fetch-progress` event payload.
#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub struct FetchProgress {
    pub phase: String,
    pub done: u64,
    pub total: u64,
}

const PHASES: [&str; 5] = [
    "Enumerating objects",
    "Counting objects",
    "Compressing objects",
    "Receiving objects",
    "Resolving deltas",
];

const TAIL_LINES: usize = 20;

/// Git rewrites a meter in place with `\r` and ends it with `\n`, so fragments
/// split on both. Fragments that are not progress keep their text for the
/// error tail; progress fragments must not pollute a failure message.
pub(super) struct ProgressFeed {
    pending: Vec<u8>,
    tail: Vec<String>,
}

impl ProgressFeed {
    pub(super) fn new() -> Self {
        Self {
            pending: Vec::new(),
            tail: Vec::new(),
        }
    }

    pub(super) fn push(&mut self, chunk: &[u8], on_progress: &mut dyn FnMut(FetchProgress)) {
        self.pending.extend_from_slice(chunk);
        while let Some(at) = self
            .pending
            .iter()
            .position(|byte| matches!(byte, b'\r' | b'\n'))
        {
            let fragment: Vec<u8> = self.pending.drain(..at).collect();
            self.pending.remove(0);
            self.digest(&fragment, on_progress);
        }
    }

    pub(super) fn tail_text(&self) -> String {
        self.tail.join("\n")
    }

    fn digest(&mut self, fragment: &[u8], on_progress: &mut dyn FnMut(FetchProgress)) {
        let text = String::from_utf8_lossy(fragment);
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        if let Some(progress) = parse_progress(text) {
            on_progress(progress);
            return;
        }
        self.tail.push(text.to_string());
        if self.tail.len() > TAIL_LINES {
            self.tail.remove(0);
        }
    }
}

/// `From https://…` survives as `None`: its "phase" would be `From https`,
/// which is not a progress phase.
fn parse_progress(fragment: &str) -> Option<FetchProgress> {
    let line = fragment.strip_prefix("remote: ").unwrap_or(fragment);
    let (phase, rest) = line.split_once(':')?;
    let phase = phase.trim();
    if !PHASES.contains(&phase) {
        return None;
    }
    let open = rest.find('(')?;
    let close = rest[open..].find(')')? + open;
    let (done, total) = rest[open + 1..close].split_once('/')?;
    Some(FetchProgress {
        phase: phase.to_string(),
        done: done.trim().parse().ok()?,
        total: total.trim().parse().ok()?,
    })
}
```

Run: `cargo test --lib fetch`
Expected: 5 tests PASS.

- [ ] **Step 3: Write the failing integration tests**

Create `tests/fetch_fixtures.rs`:

```rust
mod support;

use std::ffi::OsString;
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use git_helper_core::{FetchControl, FetchStatus, GitCommand};
use support::fixture_repo::FixtureRepo;

#[test]
fn fetch_reports_progress_and_advances_remote_tracking_refs() {
    let fixture = FixtureRepo::new();
    let _remote = fixture.add_bare_origin();
    fixture.switch_to_base();
    fixture.commit_file("base.txt", "base update\n", "base change");
    let base_head = fixture.head();
    run_git(&fixture, &["push", "origin", "base"]);
    fixture.switch_to_feature();

    let mut events = Vec::new();
    let status = fixture
        .repo
        .fetch_remotes_with_progress(&FetchControl::new(), |event| events.push(event))
        .unwrap();

    assert_eq!(status, FetchStatus::Completed);
    assert!(!events.is_empty());
    assert!(events.iter().all(|event| event.done <= event.total));
    assert!(events.iter().all(|event| !event.phase.is_empty()));
    assert_eq!(remote_base(&fixture), base_head);
}

#[test]
fn fetch_cancelled_before_it_starts_spawns_nothing() {
    let fixture = FixtureRepo::new();
    let control = FetchControl::new();
    control.cancel();

    let status = fixture
        .repo
        .fetch_remotes_with_progress(&control, |_| {})
        .unwrap();

    assert_eq!(status, FetchStatus::Cancelled);
    assert!(!control.is_running());
}

#[test]
fn fetch_can_be_cancelled_mid_flight() {
    let fixture = FixtureRepo::new();
    let port = silent_http_port();
    run_git(
        &fixture,
        &["remote", "add", "origin", &format!("http://127.0.0.1:{port}/repo.git")],
    );
    let control = FetchControl::new();
    let (tx, rx) = mpsc::channel();
    {
        let control = control.clone();
        std::thread::spawn(move || {
            let result = fixture
                .repo
                .fetch_remotes_with_progress(&control, |_| {});
            let _ = tx.send(result);
        });
    }
    wait_until_running(&control);

    control.cancel();

    let status = rx
        .recv_timeout(Duration::from_secs(30))
        .expect("a cancelled fetch must not hang")
        .unwrap();
    assert_eq!(status, FetchStatus::Cancelled);
}

#[test]
fn fetch_failure_reports_git_stderr_without_progress_noise() {
    let fixture = FixtureRepo::new();
    let port = unused_port();
    run_git(
        &fixture,
        &["remote", "add", "origin", &format!("http://127.0.0.1:{port}/repo.git")],
    );

    let error = fixture
        .repo
        .fetch_remotes_with_progress(&FetchControl::new(), |_| {})
        .unwrap_err();

    let text = error.to_string();
    assert!(text.contains("unable to access"), "unexpected error: {text}");
}

/// A listener that accepts connections and never answers keeps Git's HTTP
/// transport waiting forever — until the cancel kills it. Leaking the accepted
/// socket keeps the connection open for the rest of the test process.
fn silent_http_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            std::mem::forget(stream);
        }
    });
    port
}

/// Binding then dropping leaves the port closed, so the connection is refused
/// immediately instead of hanging.
fn unused_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_until_running(control: &FetchControl) {
    for _ in 0..500 {
        if control.is_running() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("fetch child was never registered");
}

fn remote_base(fixture: &FixtureRepo) -> String {
    let output = fixture
        .repo
        .run(GitCommand::read(args(&["rev-parse", "refs/remotes/origin/base"])))
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn run_git(fixture: &FixtureRepo, values: &[&str]) {
    fixture.repo.run(GitCommand::write(args(values))).unwrap();
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
```

Run: `cargo test --test fetch_fixtures`
Expected: FAIL to compile — `fetch_remotes_with_progress`, `FetchControl`, `FetchStatus` do not exist.

- [ ] **Step 4: Add `GitError::Io` and the spawned-process plumbing**

In `src/git/error.rs`, add a variant to `GitError` (mid-stream read/wait failures are not spawn failures, and the existing `Spawn` Display text would mislead):

```rust
    Io {
        source: io::Error,
    },
```

Add its Display arm next to the others:

```rust
            Self::Io { source } => write!(formatter, "git I/O failed: {source}"),
```

And in `source()`, change `Self::Spawn { source } => Some(source),` into:

```rust
            Self::Spawn { source } | Self::Io { source } => Some(source),
```

In `src/git/process.rs`, extend the process import to `use std::process::{Child, Command, Output, Stdio};` and add:

```rust
/// Streaming callers need the live child and its stderr; fetch writes nothing
/// useful to stdout, so it is a dead end instead of a pipe that could fill.
pub(crate) fn spawn_piped(git: &Path, repo: &Path, command: &GitCommand) -> Result<Child, GitError> {
    let mut process = Command::new(git);
    process.current_dir(repo).args(&command.args);
    hide_console(&mut process);
    set_environment(&mut process, command);
    process.stdout(Stdio::null()).stderr(Stdio::piped());
    process.spawn().map_err(|source| GitError::Spawn { source })
}
```

In `src/git/mod.rs`, add this method right after `run_unlocked_allowing_exit`:

```rust
    pub(crate) fn spawn_piped(&self, command: &GitCommand) -> Result<std::process::Child, GitError> {
        process::spawn_piped(&self.git, &self.repo, command)
    }
```

Run: `cargo build`
Expected: PASS (a transient "never used" warning for `spawn_piped` disappears in the next step).

- [ ] **Step 5: Implement the streaming fetch and wire the public API**

Replace the whole content of `src/inspection/fetch/mod.rs` with:

```rust
mod progress;

use std::ffi::OsString;
use std::io::Read;
use std::process::{Child, ChildStderr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::git::{GitCommand, GitError, GitRunner};

use super::errors::InspectionError;

pub use progress::FetchProgress;
use progress::ProgressFeed;

const FETCH_ARGS: &[&str] = &[
    "fetch",
    "--all",
    "--no-tags",
    "--no-recurse-submodules",
    "--progress",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FetchStatus {
    Completed,
    Cancelled,
}

/// Shared handle to one in-flight fetch. The desktop layer keeps a clone so
/// its cancel command can kill the Git process; the fetch loop keeps another
/// to learn that happened.
#[derive(Clone, Default)]
pub struct FetchControl {
    cancelled: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
}

impl FetchControl {
    pub fn new() -> Self {
        Self::default()
    }

    /// Killing the process is what unblocks a stuck transfer; the flag is what
    /// lets the fetch loop report cancellation instead of a Git failure.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Ok(mut slot) = self.child.lock() {
            if let Some(child) = slot.as_mut() {
                let _ = child.kill();
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().map(|slot| slot.is_some()).unwrap_or(false)
    }

    fn was_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    fn register(&self, child: Child) {
        if let Ok(mut slot) = self.child.lock() {
            *slot = Some(child);
        }
    }

    /// The child leaves the slot before `wait`, so a cancel never blocks on a
    /// lock held across a blocking wait.
    fn take_child(&self) -> Option<Child> {
        self.child.lock().ok()?.take()
    }
}

pub(super) fn fetch_remotes_with_progress(
    runner: &GitRunner,
    control: &FetchControl,
    on_progress: &mut dyn FnMut(FetchProgress),
) -> Result<FetchStatus, InspectionError> {
    if control.was_cancelled() {
        return Ok(FetchStatus::Cancelled);
    }
    let mut child = runner.spawn_piped(&GitCommand::write(args(FETCH_ARGS)))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| InspectionError::Parse("fetch stderr was not piped".to_string()))?;
    control.register(child);
    match drain_progress(stderr, on_progress) {
        Ok(tail) => finish(control, tail),
        Err(error) => {
            control.cancel();
            Err(error.into())
        }
    }
}

/// Blocking read loop: EOF means every process holding the write end of the
/// pipe has exited — including after a cancel killed Git.
fn drain_progress(
    mut stderr: ChildStderr,
    on_progress: &mut dyn FnMut(FetchProgress),
) -> Result<String, GitError> {
    let mut feed = ProgressFeed::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let read = stderr.read(&mut chunk).map_err(|source| GitError::Io { source })?;
        if read == 0 {
            break;
        }
        feed.push(&chunk[..read], on_progress);
    }
    Ok(feed.tail_text())
}

fn finish(control: &FetchControl, tail: String) -> Result<FetchStatus, InspectionError> {
    let status = control
        .take_child()
        .map(|mut child| child.wait())
        .transpose()
        .map_err(|source| GitError::Io { source })?;
    if control.was_cancelled() {
        return Ok(FetchStatus::Cancelled);
    }
    if status.is_some_and(|exit| exit.success()) {
        return Ok(FetchStatus::Completed);
    }
    Err(GitError::Command {
        args: args(FETCH_ARGS),
        exit_code: status.and_then(|exit| exit.code()),
        stderr: tail.into_bytes(),
    }
    .into())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
```

In `src/inspection/mod.rs`, add after `pub use errors::InspectionError;`:

```rust
pub use fetch::{FetchControl, FetchProgress, FetchStatus};
```

and replace the old `fetch_remotes` wrapper at the bottom of the file with:

```rust
pub(crate) fn fetch_remotes_with_progress(
    runner: &GitRunner,
    control: &FetchControl,
    on_progress: &mut dyn FnMut(FetchProgress),
) -> Result<FetchStatus, InspectionError> {
    fetch::fetch_remotes_with_progress(runner, control, on_progress)
}
```

In `src/repository/read.rs`, extend the `use crate::inspection::{…}` import with `FetchControl, FetchProgress, FetchStatus` and replace the `fetch_remotes` method with:

```rust
    pub fn fetch_remotes_with_progress(
        &self,
        control: &FetchControl,
        mut on_progress: impl FnMut(FetchProgress),
    ) -> Result<FetchStatus, InspectionError> {
        self.runner.with_write_lock(|| {
            inspection::fetch_remotes_with_progress(&self.runner, control, &mut on_progress)
        })
    }
```

In `src/lib.rs`, add `FetchControl`, `FetchProgress`, and `FetchStatus` to the `pub use inspection::{…}` list.

- [ ] **Step 6: Update the two existing fetch tests**

In `tests/inspection_fixtures.rs`, add `FetchControl` to the `use git_helper_core::{…}` import, and replace both occurrences of:

```rust
    fixture.repo.fetch_remotes().unwrap();
```

with:

```rust
    fixture
        .repo
        .fetch_remotes_with_progress(&FetchControl::new(), |_| {})
        .unwrap();
```

(There are exactly two: `fetch_remotes_succeeds_when_no_remotes_are_configured` around line 91 and `fetch_remotes_picks_up_new_refs_on_a_configured_remote` around line 116.)

- [ ] **Step 7: Keep the Tauri command compiling (minimal adaptation, no behavior change yet)**

Without this step the workspace does not compile between Task 1 and Task 2, because `src-tauri/src/commands/actions.rs` still calls the deleted `repository.fetch_remotes()`. In that file, change `use git_helper_core::RefName;` to `use git_helper_core::{FetchControl, RefName};` and replace the command body with (same blocking behavior, no events — Task 2 adds those):

```rust
#[tauri::command(async)]
pub fn fetch_remotes(state: State<'_, AppState>) -> Result<(), String> {
    with_repository(state.inner(), |repository| {
        repository
            .fetch_remotes_with_progress(&FetchControl::new(), |_| {})
            .map_err(|error| {
                let text = error.to_string();
                let detail = text
                    .strip_prefix("Git inspection failed: ")
                    .unwrap_or(&text);
                format!("Could not fetch remotes: {detail}")
            })
    })
    .map(|_| ())
}
```

- [ ] **Step 8: Run the full Rust suite**

Run: `cargo test`
Expected: PASS — the 5 parser/feed unit tests, the 4 new integration tests, and all pre-existing tests, with the whole workspace compiling.

- [ ] **Step 9: Commit**

```powershell
git add src tests src-tauri
git commit -m "feat: stream fetch progress and support cancellation in the core"
```

---

### Task 2: Tauri commands — progress events and `cancel_fetch`

**Files:**
- Modify: `src-tauri/src/commands/state.rs` (fetch slot + 3 methods)
- Modify: `src-tauri/src/commands/actions.rs` (rewrite `fetch_remotes`, add `cancel_fetch`, add `fetch_error_message`)
- Modify: `src-tauri/src/lib.rs` (register `cancel_fetch`)

**Interfaces:**
- Consumes: `FetchControl`, `FetchProgress` from Task 1; `tauri::Emitter` (`app.emit`); existing `with_repository` helper (`src-tauri/src/commands/repository.rs`); existing `AppState` mutex pattern.
- Produces (used by Tasks 3–5): Tauri command `cancel_fetch` (no arguments, returns `Result<(), String>`, no-op when idle); `fetch_remotes` keeps its `Result<(), String>` contract — cancelled maps to `Ok(())`, so the UI sees no warning; event `fetch-progress` with payload `{ phase: string, done: number, total: number }` (serde serializes the `FetchProgress` field names unchanged).

- [ ] **Step 1: Add the fetch slot to `AppState`**

In `src-tauri/src/commands/state.rs`, extend the import to `use git_helper_core::{FetchControl, GitCommand, GitRepository, RepositoryConfig};`, add a field to the struct:

```rust
    pub(super) fetch: Mutex<Option<FetchControl>>,
```

initialize it in `AppState::new` next to the others (`fetch: Mutex::new(None),`), and add these methods to `impl AppState`:

```rust
    pub fn register_fetch(&self, control: FetchControl) {
        if let Ok(mut slot) = self.fetch.lock() {
            *slot = Some(control);
        }
    }

    /// Only the handle is cloned out of the slot; `FetchControl::cancel` takes
    /// its own child lock, so this never blocks on the running fetch.
    pub fn cancel_fetch(&self) {
        let control = self.fetch.lock().ok().and_then(|slot| slot.clone());
        if let Some(control) = control {
            control.cancel();
        }
    }

    pub fn clear_fetch(&self) {
        if let Ok(mut slot) = self.fetch.lock() {
            *slot = None;
        }
    }
```

- [ ] **Step 2: Add a smoke test for the slot plumbing**

Append to `src-tauri/src/commands/state.rs`:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn cancel_fetch_without_a_running_fetch_is_a_no_op() {
        let state = super::AppState::new();
        state.cancel_fetch();
        state.clear_fetch();
    }
}
```

Run: `cargo test -p git-helper`
Expected: PASS. (`AppState::new` probes the current directory; whether or not it finds a repo, the cancel/clear path must not panic.)

- [ ] **Step 3: Rewrite the `fetch_remotes` command and add `cancel_fetch`**

In `src-tauri/src/commands/actions.rs`:
- Change `use git_helper_core::RefName;` to `use git_helper_core::{FetchControl, FetchProgress, InspectionError, RefName};`
- Change `use tauri::{AppHandle, State};` to `use tauri::{AppHandle, Emitter, State};`

Replace the whole `fetch_remotes` command from Task 1 Step 7 (including its error-mapping closure) with:

```rust
#[tauri::command(async)]
pub fn fetch_remotes(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let control = FetchControl::new();
    state.register_fetch(control.clone());
    let mut progress = move |event: FetchProgress| {
        let _ = app.emit("fetch-progress", event);
    };
    let result = with_repository(state.inner(), |repository| {
        repository
            .fetch_remotes_with_progress(&control, &mut progress)
            .map_err(fetch_error_message)
    });
    state.clear_fetch();
    result.map(|_| ())
}

/// The cancel path touches only the fetch slot, never the repository mutex, so
/// it runs while a fetch holds that mutex.
#[tauri::command(async)]
pub fn cancel_fetch(state: State<'_, AppState>) -> Result<(), String> {
    state.cancel_fetch();
    Ok(())
}

fn fetch_error_message(error: InspectionError) -> String {
    let text = error.to_string();
    let detail = text.strip_prefix("Git inspection failed: ").unwrap_or(&text);
    format!("Could not fetch remotes: {detail}")
}
```

- [ ] **Step 4: Register the command**

In `src-tauri/src/lib.rs`, add `commands::actions::cancel_fetch,` to the `tauri::generate_handler!` list, right after `commands::actions::fetch_remotes,`.

- [ ] **Step 5: Verify**

Run: `cargo test` and `npm test`
Expected: PASS. `test/ui-smoke.test.mjs` "Git-backed Tauri commands run off the window thread" now also scans the new `cancel_fetch` — it must carry `#[tauri::command(async)]` or that test fails.

- [ ] **Step 6: Commit**

```powershell
git add src-tauri
git commit -m "feat: emit fetch progress events and add a cancel_fetch command"
```

---

### Task 3: UI fetch state, bridge listener, and controller wiring

**Files:**
- Modify: `ui/app/Private/types.ts`, `ui/app/Private/state.ts`, `ui/app/Private/bridge.ts`, `ui/app/Private/controller.ts`, `ui/app/Private/discovery.ts`
- Test: `test/ui-fetch.test.mjs` (new)

**Interfaces:**
- Consumes: Tauri event `fetch-progress` and command `cancel_fetch` from Task 2; the existing `TauriGlobal.event.listen` typing already present in `types.ts`; the existing `FixtureBridge` test bridge pattern (`ui/app/Private/bridge.ts`); `controllerWith`/`snapshotWith` from `test/support/controller.mjs`.
- Produces (used by Tasks 4–5):
  - `FetchProgressEvent = { phase: string; done: number; total: number }` and `FetchState = { active: boolean; phase: string; done: number; total: number }` in `types.ts`.
  - `AppState.fetch: FetchState` — `active` is true from fetch start until the command settles.
  - `Bridge.listen(event: string, handler: (payload: FetchProgressEvent) => void): void`.
  - `FixtureBridge.emitEvent(event: string, payload: FetchProgressEvent): void` — test hook that delivers to a registered listener.
  - `AppController.onFetchProgress(payload: FetchProgressEvent): void` and `AppController.cancelFetch(): Promise<void>`.

- [ ] **Step 1: Write the failing tests**

Create `test/ui-fetch.test.mjs`:

```js
import assert from "node:assert/strict";
import test from "node:test";
import { AppController, FixtureBridge } from "../ui/app/index.ts";
import { fetchRemotes } from "../ui/app/Private/discovery.ts";
import { controllerWith, snapshotWith } from "./support/controller.mjs";

function stubBridge(responses = {}) {
  return {
    async invoke(command) {
      if (command in responses) return responses[command];
      return [];
    },
    listen() {},
    pickRepository: async () => null,
  };
}

test("fetchRemotes marks the fetch active for the duration of the command", async () => {
  const renders = [];
  let finishFetch;
  const bridge = stubBridge();
  bridge.invoke = (command) => {
    if (command === "fetch_remotes") {
      return new Promise((resolve) => { finishFetch = () => resolve(null); });
    }
    return Promise.resolve([]);
  };
  const controller = controllerWith(bridge);
  controller.render = () => renders.push({ ...controller.state.fetch });

  const fetching = fetchRemotes(controller);
  await Promise.resolve();
  assert.equal(controller.state.fetch.active, true);

  finishFetch();
  const warning = await fetching;
  assert.equal(warning, "");
  assert.equal(controller.state.fetch.active, false);
  assert.deepEqual(renders.at(-1), { active: false, phase: "", done: 0, total: 0 });
});

test("a failed fetch still clears the active flag and returns the warning", async () => {
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = (command) => {
    if (command === "fetch_remotes") return Promise.reject(new Error("network down"));
    return Promise.resolve([]);
  };

  const warning = await fetchRemotes(controller);

  assert.equal(warning, "network down");
  assert.equal(controller.state.fetch.active, false);
});

test("progress events update the fetch state only while a fetch is active", () => {
  const controller = controllerWith(stubBridge());
  const payload = { phase: "Receiving objects", done: 45, total: 100 };

  controller.onFetchProgress(payload);
  assert.equal(controller.state.fetch.phase, "");

  controller.state.fetch.active = true;
  controller.onFetchProgress(payload);
  assert.equal(controller.state.fetch.phase, "Receiving objects");
  assert.equal(controller.state.fetch.done, 45);
});

test("start listens for fetch progress and a live event updates state", async () => {
  const bridge = new FixtureBridge({
    list_recent_repositories: [],
    get_ui_preferences: { skip_review: false },
    fetch_remotes: null,
    load_snapshot: snapshotWith({}),
    list_changed_paths: [],
  });
  const controller = new AppController(bridge);
  controller.render = () => {};
  controller.announce = () => {};

  await controller.start();

  controller.state.fetch.active = true;
  bridge.emitEvent("fetch-progress", { phase: "Resolving deltas", done: 3, total: 4 });
  assert.equal(controller.state.fetch.phase, "Resolving deltas");
  assert.equal(controller.state.fetch.total, 4);
});

test("cancelFetch only invokes the desktop command while a fetch is active", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => { commands.push(command); return []; };

  await controller.cancelFetch();
  assert.deepEqual(commands, []);

  controller.state.fetch.active = true;
  await controller.cancelFetch();
  assert.deepEqual(commands, ["cancel_fetch"]);
});
```

Run: `node --test test/ui-fetch.test.mjs`
Expected: FAIL — `state.fetch` is undefined, `onFetchProgress`/`cancelFetch`/`listen`/`emitEvent` do not exist.

- [ ] **Step 2: Add the fetch state types**

In `ui/app/Private/types.ts`, add above `export interface AppState`:

```ts
/// Payload of the desktop `fetch-progress` event, mirroring the Rust
/// `FetchProgress` struct in src/inspection/fetch/progress.rs.
export interface FetchProgressEvent {
  phase: string;
  done: number;
  total: number;
}

export interface FetchState {
  active: boolean;
  phase: string;
  done: number;
  total: number;
}
```

In the same file, inside `interface AppState`, add right after `busy: boolean;`:

```ts
  /// Live fetch progress, fed by the desktop `fetch-progress` event while the
  /// `fetch_remotes` command runs; `active` spans exactly the invoke.
  fetch: FetchState;
```

And inside `interface Bridge`, add a third member:

```ts
  listen(event: string, handler: (payload: FetchProgressEvent) => void): void;
```

In `ui/app/Private/state.ts`, add to `createState()` right after `busy: false,`:

```ts
    fetch: { active: false, phase: "", done: 0, total: 0 },
```

- [ ] **Step 3: Implement `listen` in both bridges**

In `ui/app/Private/bridge.ts`, change the import to `import type { Bridge, FetchProgressEvent } from "./types.ts";` and add to `TauriBridge`:

```ts
  listen(event: string, handler: (payload: FetchProgressEvent) => void): void {
    const listen = globalThis.__TAURI__?.event?.listen;
    if (typeof listen !== "function") return;
    const subscribing = listen(event, /*forwardPayload=*/ (raw: unknown) => {
      handler((raw as { payload: FetchProgressEvent }).payload);
    });
    Promise.resolve(subscribing).catch(/*listenFailureIsNonFatal=*/ () => {});
  }
```

Add to `FixtureBridge`:

```ts
  private readonly listeners = new Map<string, (payload: FetchProgressEvent) => void>();

  listen(event: string, handler: (payload: FetchProgressEvent) => void): void {
    this.listeners.set(event, handler);
  }

  /** @public Test hook: deliver a backend event to a registered listener. */
  emitEvent(event: string, payload: FetchProgressEvent): void {
    this.listeners.get(event)?.(payload);
  }
```

- [ ] **Step 4: Wire the controller**

In `ui/app/Private/controller.ts`, add `FetchProgressEvent` to the `import type { … } from "./types.ts";` list. In `start()`, add the subscription right after `bindEvents(this);`:

```ts
    this.bridge.listen("fetch-progress", /*trackFetchProgress=*/ (payload) => this.onFetchProgress(payload));
```

Add these two methods right after `reloadViewData()`:

```ts
  /// Progress events can arrive after the command already settled (queued IPC);
  /// a finished fetch must not light the bar back up.
  onFetchProgress(payload: FetchProgressEvent): void {
    if (!this.state.fetch.active) return;
    this.state.fetch.phase = payload.phase;
    this.state.fetch.done = payload.done;
    this.state.fetch.total = payload.total;
    this.render();
  }

  async cancelFetch(): Promise<void> {
    if (!this.state.fetch.active) return;
    await this.bridge.invoke("cancel_fetch");
  }
```

- [ ] **Step 5: Manage the active flag in `fetchRemotes`**

In `ui/app/Private/discovery.ts`, replace the whole `fetchRemotes` function with:

```ts
/// Refresh fetches every configured remote before reloading local discovery
/// data. A failed fetch is surfaced as a warning, not a blocking error. The
/// active flag spans exactly the invoke so late progress events are dropped.
export async function fetchRemotes(controller: AppController): Promise<string> {
  const fetch = controller.state.fetch;
  fetch.active = true;
  fetch.phase = "";
  fetch.done = 0;
  fetch.total = 0;
  controller.render();
  try {
    await controller.bridge.invoke("fetch_remotes");
    return "";
  } catch (error) {
    return invokeError(error);
  } finally {
    fetch.active = false;
    controller.render();
  }
}
```

- [ ] **Step 6: Verify**

Run: `node --test test/ui-fetch.test.mjs`, then `npm run lint`, then the full `npm test`
Expected: PASS everywhere, including all pre-existing UI tests.

- [ ] **Step 7: Commit**

```powershell
git add ui test
git commit -m "feat: track fetch progress state and listen for desktop events"
```

---

### Task 4: Status bar progress bar and stop button

**Files:**
- Create: `ui/app/Private/views/status-bar.ts`
- Modify: `ui/app/Private/views/shell.ts` (import + footer usage + delete old `status()`), `ui/app/Private/event-tables.ts` (one entry), `ui/styles/workbench.css` (append styles)
- Test: `test/ui-fetch.test.mjs` (append tests)

**Interfaces:**
- Consumes: `state.fetch` (Task 3), `controller.cancelFetch` (Task 3), the delegated `CLICK` table in `event-tables.ts`, the CSS variables `--border` / `--accent` / `--bad` already defined in `workbench.css`.
- Produces: `statusBar(state: AppState): string` exported from `ui/app/Private/views/status-bar.ts`, used only by `shell.ts`; markup contract `data-event="cancel-fetch"` on the stop button, `role="progressbar"` with `aria-valuenow` on the bar.

- [ ] **Step 1: Write the failing tests**

Append to `test/ui-fetch.test.mjs` (add `renderShell` to the `../ui/app/index.ts` import and `CLICK` via `import { CLICK } from "../ui/app/Private/event-tables.ts";`):

```js
test("the status bar shows fetch progress with a stop button", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;
  controller.state.fetch = { active: true, phase: "Receiving objects", done: 45, total: 100 };

  const markup = renderShell(controller.state);

  assert.match(markup, /role="progressbar"/);
  assert.match(markup, /aria-valuenow="45"/);
  assert.match(markup, /width:45%/);
  assert.match(markup, /Receiving objects 45%/);
  assert.match(markup, /data-event="cancel-fetch"/);
  assert.doesNotMatch(markup, /Working…/);
});

test("the status bar shows an indeterminate fetch state before the first event", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;
  controller.state.fetch = { active: true, phase: "", done: 0, total: 0 };

  const markup = renderShell(controller.state);

  assert.match(markup, /Fetching remotes…/);
  assert.match(markup, /data-event="cancel-fetch"/);
  assert.doesNotMatch(markup, /role="progressbar"/);
});

test("the status bar keeps the busy spinner when no fetch is active", () => {
  const controller = controllerWith(stubBridge());
  controller.state.busy = true;

  const markup = renderShell(controller.state);

  assert.match(markup, /Working…/);
  assert.doesNotMatch(markup, /role="progressbar"/);
});

test("the cancel-fetch click action invokes the desktop cancel command", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => { commands.push(command); return []; };
  controller.state.fetch.active = true;

  await CLICK["cancel-fetch"](controller, "");

  assert.deepEqual(commands, ["cancel_fetch"]);
});
```

Run: `node --test test/ui-fetch.test.mjs`
Expected: FAIL — the old footer renders "Working…" and no `cancel-fetch` markup exists.

- [ ] **Step 2: Create the status bar view**

Create `ui/app/Private/views/status-bar.ts`:

```ts
import { esc } from "../dom.ts";
import type { AppState } from "../types.ts";

/// The fetch bar outranks the generic busy spinner: the fetch runs inside the
/// busy window, and the user needs its progress and stop control instead.
export function statusBar(state: AppState): string {
  if (state.fetch.active) return fetchStatus(state);
  if (state.busy) return `<span class="spinner" aria-hidden="true"></span>Working…`;
  if (state.review) return "Review pending — nothing has been written yet.";
  return "Ready";
}

function fetchStatus(state: AppState): string {
  const { phase, done, total } = state.fetch;
  if (!phase || !total) {
    return `<span class="spinner" aria-hidden="true"></span><span>Fetching remotes…</span>${stopButton()}`;
  }
  const percent = Math.min(100, Math.round((done / total) * 100));
  return `<span class="fetch-progress" role="progressbar" aria-label="Fetch progress"
      aria-valuemin="0" aria-valuemax="100" aria-valuenow="${percent}"><span class="fetch-fill" style="width:${percent}%"></span></span>
    <span class="fetch-label">${esc(phase)} ${percent}%</span>${stopButton()}`;
}

/// Never disabled: the whole point is interrupting the busy window.
function stopButton(): string {
  return `<button class="fetch-stop" data-event="cancel-fetch" title="Stop the fetch" aria-label="Stop the fetch">&times;</button>`;
}
```

- [ ] **Step 3: Use it from the shell**

In `ui/app/Private/views/shell.ts`:
- Add `import { statusBar } from "./status-bar.ts";` after the `reviewPane` import.
- In `main()`, replace `<footer class="status" role="status">${status(state)}</footer>` with `<footer class="status" role="status">${statusBar(state)}</footer>`.
- Delete the now-unused `function status(state: AppState): string {…}` block at the bottom of the file.

- [ ] **Step 4: Register the click action**

In `ui/app/Private/event-tables.ts`, add right after the `refresh:` entry:

```ts
  "cancel-fetch": (controller) => controller.cancelFetch(),
```

- [ ] **Step 5: Style the bar and the stop button**

Append to `ui/styles/workbench.css` (after the `@keyframes spin` block at the end of the file):

```css
.fetch-progress {
  width: 160px;
  height: 6px;
  display: inline-block;
  background: var(--border);
  border-radius: 3px;
  overflow: hidden;
}

.fetch-fill {
  display: block;
  height: 100%;
  background: var(--accent);
  transition: width 0.2s ease-out;
}

.fetch-stop {
  padding: 0 4px;
  border: none;
  border-radius: var(--radius);
  background: transparent;
  color: var(--bad);
  font-size: 14px;
  line-height: 1;
}

.fetch-stop:hover {
  background: color-mix(in srgb, var(--bad) 12%, transparent);
}
```

- [ ] **Step 6: Verify**

Run: `node --test test/ui-fetch.test.mjs`, then `npm run lint`, then the full `npm test`
Expected: PASS everywhere.

- [ ] **Step 7: Commit**

```powershell
git add ui test
git commit -m "feat: show fetch progress with a stop button in the status bar"
```

---

### Task 5: Paint the current repository before the fetch completes

**Files:**
- Modify: `ui/app/Private/controller.ts` (`refresh` pre-paint), `ui/app/Private/repository-switcher.ts` (`openRepositoryPath` reorder)
- Test: `test/ui-fetch.test.mjs` (append Refresh ordering test), `test/ui-repo-switcher.test.mjs` (stateful stub + 2 new ordering tests)

**Interfaces:**
- Consumes: `fetchRemotes` (unchanged signature, Task 3), `controller.reload(snapshot?)` (in `discovery.ts`: called with a snapshot it skips `load_snapshot`; called without one it invokes `load_snapshot`), the returned-snapshot contract of `open_repository`.
- Produces: ordering guarantees — `refresh()`: when `state.snapshot` is null and no snapshot argument is passed, `load_snapshot` runs before `fetch_remotes`; `openRepositoryPath`: `open_repository` → reload from returned snapshot → `fetch_remotes` → reload via `load_snapshot` → `list_recent_repositories`.

- [ ] **Step 1: Make the repo-switcher test stub stateful (refactor, no behavior change)**

In `test/ui-repo-switcher.test.mjs`, replace the `withRecents` helper with a version whose `load_snapshot` returns whichever repository was opened last (the new flow reloads once more after the fetch, so the stub must model "the desktop remembers the open repo"):

```js
function withRecents(extra = {}) {
  let current = snapshotWith({});
  const controller = controllerWith({
    async invoke(command, args) {
      if (command === "list_recent_repositories") return [...RECENTS];
      if (command === "remove_recent_repository") {
        return RECENTS.filter((entry) => entry.path !== args.path);
      }
      if (command === "open_repository") {
        current = snapshotWith({ path: args.request.path, name: args.request.path.split("/").pop() });
        return current;
      }
      if (command === "load_snapshot") return current;
      return [];
    },
  });
  controller.state.recentRepositories = [...RECENTS];
  Object.assign(controller.state, extra);
  return controller;
}
```

Run: `node --test test/ui-repo-switcher.test.mjs`
Expected: PASS — existing tests still pass against the old flow (the returned snapshot and the reloaded one carry the same content here).

- [ ] **Step 2: Write the failing ordering tests**

Append to `test/ui-repo-switcher.test.mjs`:

```js
async function flushUntil(condition) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (condition()) return;
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  throw new Error("condition was never met");
}

test("the new repository is visible while its fetch is still running", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const original = controller.bridge.invoke.bind(controller.bridge);
  let finishFetch;
  controller.bridge.invoke = (command, args) => {
    if (command !== "fetch_remotes") return original(command, args);
    return new Promise((resolve) => { finishFetch = () => resolve(null); });
  };

  const opening = openRecentRepository(controller, "C:/work/beta");
  await flushUntil(() => controller.state.fetch.active);

  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");

  finishFetch();
  await opening;
  assert.equal(controller.state.snapshot.overview.path, "C:/work/beta");
  assert.equal(controller.state.fetch.active, false);
});

test("opening a repository reloads once more after the fetch", async () => {
  const controller = withRecents({ repoMenuOpen: true });
  const commands = [];
  const original = controller.bridge.invoke.bind(controller.bridge);
  controller.bridge.invoke = async (command, args) => {
    commands.push(command);
    return original(command, args);
  };

  await openRecentRepository(controller, "C:/work/beta");

  const fetchAt = commands.indexOf("fetch_remotes");
  const loads = commands
    .map((command, index) => (command === "load_snapshot" ? index : -1))
    .filter((index) => index >= 0);
  assert.ok(fetchAt > commands.indexOf("open_repository"));
  assert.deepEqual(loads.length, 1);
  assert.ok(loads[0] > fetchAt);
});
```

Append to `test/ui-fetch.test.mjs`:

```js
test("refresh paints local state before the first fetch of a session", async () => {
  const commands = [];
  const controller = controllerWith(stubBridge());
  controller.bridge.invoke = async (command) => {
    commands.push(command);
    if (command === "load_snapshot") return snapshotWith({});
    return [];
  };
  controller.state.snapshot = null;

  await controller.refresh();

  const firstLoad = commands.indexOf("load_snapshot");
  const fetchAt = commands.indexOf("fetch_remotes");
  assert.ok(firstLoad !== -1 && firstLoad < fetchAt);
  assert.ok(commands.lastIndexOf("load_snapshot") > fetchAt);
});
```

Run: `node --test test/ui-repo-switcher.test.mjs` and `node --test test/ui-fetch.test.mjs`
Expected: FAIL — during the pending fetch the snapshot still shows the previous repo, and `refresh` invokes `fetch_remotes` before any `load_snapshot`.

- [ ] **Step 3: Reorder the open-repository flow**

In `ui/app/Private/repository-switcher.ts`, inside `openRepositoryPath`, replace this block:

```ts
      controller.state.draft = createDraft();
      controller.state.outcome = null;
      controller.state.expanded.clear();
      // Same contract as Refresh: learn whether remotes are reachable for the
      // newly opened repo, without blocking the local snapshot reload.
      controller.state.warning = await fetchRemotes(controller);
      if (controller.state.warning) controller.announce(controller.state.warning);
      await controller.reload(snapshot);
      await loadRecentRepositories(controller);
```

with:

```ts
      controller.state.draft = createDraft();
      controller.state.outcome = null;
      controller.state.expanded.clear();
      // Paint the new repository from the returned snapshot first; the fetch
      // then runs behind the progress bar instead of hiding the switch.
      await controller.reload(snapshot);
      // Same contract as Refresh: learn whether remotes are reachable for the
      // newly opened repo, without blocking the local snapshot reload.
      controller.state.warning = await fetchRemotes(controller);
      if (controller.state.warning) controller.announce(controller.state.warning);
      // The fetch moved remote-tracking refs, so the snapshot is stale now.
      await controller.reload();
      await loadRecentRepositories(controller);
```

- [ ] **Step 4: Paint local state first in `refresh` when nothing is on screen**

In `ui/app/Private/controller.ts`, replace the `refresh` method with:

```ts
  refresh(snapshot: RepositorySnapshot | null = null): Promise<void> {
    return this.run(async () => {
      // With nothing on screen yet (app start), paint local state first so a
      // slow fetch never hides a repository we already know. A visible
      // repository skips this: its data is already on screen.
      if (!this.state.snapshot && !snapshot) await this.reload();
      this.state.warning = await fetchRemotes(this);
      if (this.state.warning) this.announce(this.state.warning);
      await this.reload(snapshot);
    });
  }
```

- [ ] **Step 5: Verify**

Run: `node --test test/ui-repo-switcher.test.mjs`, `node --test test/ui-fetch.test.mjs`, then `npm run lint` and the full `npm test`
Expected: PASS everywhere, including the pre-existing smoke test "refresh fetches remotes first and warns without blocking when fetch fails" (its controller already holds a snapshot, so the pre-paint branch is skipped there).

- [ ] **Step 6: Commit**

```powershell
git add ui test
git commit -m "feat: paint the repository before the fetch completes"
```

---

### Task 6: Documentation and full verification

**Files:**
- Modify: `docs/architecture/workbench-ui.md`, `docs/architecture/git-core.md`, `docs/flows/switch-repository.md`, `docs/flows/README.md`, `docs/lessons-learned/README.md`
- Create: `docs/flows/fetch-progress.md`, `docs/lessons-learned/fetch-progress-streaming.md`

**Interfaces:**
- Consumes: everything from Tasks 1–5.
- Produces: documentation matching the new behavior; a fully green verification run.

- [ ] **Step 1: Update `docs/architecture/workbench-ui.md`**

Replace the "Refresh fetches before it reloads." state rule bullet with:

```markdown
- **Refresh paints local state before it waits on the network.** With nothing on screen yet (app start, repository switch), the controller reloads from local refs first, then runs `git fetch --all --no-tags --no-recurse-submodules --progress` with no review, then reloads again so the snapshot reflects the moved remote-tracking refs. While the fetch runs, the status footer shows a progress bar fed by the desktop `fetch-progress` event plus a stop button that invokes `cancel_fetch`; a failed fetch sets `state.warning` and shows a dismissible banner, a cancelled fetch is silent. `state.fetch` holds `{ active, phase, done, total }`; events arriving after the command settled are dropped so a stale event cannot resurrect the bar.
```

Add a row to the file table at the top (after the `views/repo-menu.ts` row):

```markdown
| `Private/views/status-bar.ts` | Status footer: busy spinner, fetch progress bar with stop button, review hint |
```

- [ ] **Step 2: Update `docs/architecture/git-core.md`**

Add a bullet after the `GitRunner` bullet near the top:

```markdown
- `inspection/fetch` spawns `git fetch --progress` and streams its stderr: `\r`/`\n`-separated `phase: N% (done/total)` fragments become `FetchProgress` events for the desktop layer, while non-progress lines are kept as the error tail. `FetchControl` — an `Arc` cancel flag plus the child handle — lets the desktop `cancel_fetch` command kill the process; a cancelled fetch reports `FetchStatus::Cancelled` instead of a Git error.
```

- [ ] **Step 3: Update `docs/flows/switch-repository.md`**

Replace step 6 with:

```markdown
6. The controller clears draft / outcome / expanded, reloads from the returned snapshot immediately so the new repository is on screen before any network wait, then fetches remotes for the new repository (same as Refresh — status-bar progress bar and stop button), reloads once more so moved remote-tracking refs are reflected, refreshes the recent list, and clears the in-flight selection. A failed fetch sets `state.warning` and still loads the local snapshot. A failed open restores the previous repository as the visible selection.
```

Update the Side effects bullet about fetch to:

```markdown
- A remote fetch runs after a successful open, streaming progress to the status bar with a stop button that kills it; unreachable remotes become a dismissible **Fetch failed** warning, not a blocked open
```

Add to Files to inspect: `- src/inspection/fetch/` and `- ui/app/Private/views/status-bar.ts`.

- [ ] **Step 4: Create `docs/flows/fetch-progress.md`**

```markdown
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
5. The stop button invokes `cancel_fetch`, which kills the child through the shared `FetchControl`. The killed process closes stderr; the loop sees EOF and reports `FetchStatus::Cancelled`, which the command maps to success — no warning banner.
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
```

Add the index row to `docs/flows/README.md`:

```markdown
| [fetch-progress.md](./fetch-progress.md) | Refresh/open fetch: progress bar, stop button, paint-before-fetch ordering |
```

- [ ] **Step 5: Add the lessons-learned entry**

Create `docs/lessons-learned/fetch-progress-streaming.md`:

```markdown
# Fetch progress streaming and cancellation

Git only emits progress on stderr, and only when it thinks a terminal is watching — a piped fetch is silent unless `--progress` is passed. Updates to the same meter are separated by `\r`, not `\n`, so a reader must split on both.

Cancellation cannot hold the child lock across a blocking `wait`: the cancel command needs that same lock to kill. Take the stderr handle out of the child, register the child in a shared slot, read to EOF (killing the process is what unblocks the read), then remove the child from the slot before waiting.

A backend event can arrive after the invoking command already settled (queued IPC). The UI must drop progress events received while no fetch is active, or a finished bar lights back up.
```

Add the index row to `docs/lessons-learned/README.md` (topical order is not strict; append at the bottom of the table):

```markdown
| [fetch-progress-streaming.md](./fetch-progress-streaming.md) | Git progress needs `--progress` and `\r`-splitting; cancel kills via a child slot, never a lock held across wait | 2026-11-08 |
```

- [ ] **Step 6: Run the full verification suite**

Run, in order:

```powershell
npm run lint
npm test
cargo test
npx fallow audit
```

Expected: all PASS; `fallow audit` reports no new issues (`statusBar`, the new bridge members, and the new Rust exports are all consumed).

- [ ] **Step 7: Manual verification in the desktop app**

Run: `npm run tauri dev`, then:

1. Open a repository whose remote has unfetched history (or throttle the network). The status footer shows the progress bar with phase and percentage, plus the red ✕.
2. Click ✕ mid-fetch: the bar disappears, the footer returns to "Ready", and **no** "Fetch failed" banner appears.
3. Switch to another recent repository: the new repository's branch/Base/worktree facts appear immediately; the fetch continues behind the progress bar; the bar finishes and the snapshot quietly refreshes.
4. Press Refresh with a healthy remote: bar appears and completes; with the network down: the dismissible **Fetch failed** warning appears as before.
5. Restart the app on a big repository: the repository paints from local state first, then updates when the fetch lands.

- [ ] **Step 8: Commit**

```powershell
git add docs
git commit -m "docs: fetch progress, cancellation, and paint-before-fetch"
```

---

## Out of scope (deliberate)

- **Sync's internal fetch and Quick switch's pull fetch** (`src/sync/start.rs`, `src/switch/pull.rs`) keep their current blocking behavior. They run inside reviewed multi-command transactions with oplog phases; streaming progress there is a separate, larger change.
- **Any form of shallow/filtered/refs-only fetch** — see the "sped up?" section above for why each is unsafe here.














