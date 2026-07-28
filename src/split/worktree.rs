use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::git::{GitCommand, GitRunner};
use crate::recording::timestamp;
use crate::rewrite::ObjectId;

use super::errors::SplitError;
use super::state::args;

/// Runs `action` inside a detached worktree checked out at `at`, so the user's
/// own checkout never churns. The worktree is removed on every exit path.
pub(super) fn with_temporary<T>(
    runner: &GitRunner,
    at: &ObjectId,
    action: impl FnOnce(&Path) -> Result<T, SplitError>,
) -> Result<T, SplitError> {
    let path = reserve(runner)?;
    add(runner, &path, at)?;
    let result = action(&path);
    remove(runner, &path);
    result
}

fn reserve(runner: &GitRunner) -> Result<PathBuf, SplitError> {
    let folder = runner.git_dir()?.join("githelper").join("worktrees");
    fs::create_dir_all(&folder)?;
    Ok(folder.join(format!("split-{}-{}", timestamp(), std::process::id())))
}

fn add(runner: &GitRunner, path: &Path, at: &ObjectId) -> Result<(), SplitError> {
    let mut values = args(&[
        "-c",
        "submodule.recurse=false",
        "worktree",
        "add",
        "--detach",
    ]);
    values.push(OsString::from(path));
    values.push(OsString::from(at.as_str()));
    runner.run_unlocked(GitCommand::write(values))?;
    Ok(())
}

fn remove(runner: &GitRunner, path: &Path) {
    let mut values = args(&["worktree", "remove", "--force"]);
    values.push(OsString::from(path));
    let removed = runner.run_unlocked(GitCommand::write(values)).is_ok();
    if !removed {
        let _ = fs::remove_dir_all(path);
    }
    let _ = runner.run_unlocked(GitCommand::write(args(&["worktree", "prune"])));
}
