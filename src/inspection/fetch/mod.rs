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
                kill_fetch_child(child);
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
        // Cancel may have raced between spawn and register with an empty slot.
        if self.was_cancelled() {
            self.cancel();
        }
    }

    /// The child leaves the slot before `wait`, so a cancel never blocks on a
    /// lock held across a blocking wait.
    fn take_child(&self) -> Option<Child> {
        self.child.lock().ok()?.take()
    }
}

/// On Windows, `Child::kill` only terminates `git.exe`; the remote helper that
/// owns the transfer (and often the stderr pipe) keeps running. Kill the tree.
fn kill_fetch_child(child: &mut Child) {
    #[cfg(windows)]
    kill_fetch_tree(child);
    #[cfg(not(windows))]
    {
        let _ = child.kill();
    }
}

#[cfg(windows)]
fn kill_fetch_tree(child: &mut Child) {
    use std::process::{Command, Stdio};
    let pid = child.id().to_string();
    let _ = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
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
