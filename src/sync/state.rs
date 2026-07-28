use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::errors::SyncError;

pub(crate) struct BaseSpec {
    pub remote: String,
    pub branch: String,
}

pub(crate) fn base_spec(base: &RefName) -> Result<BaseSpec, SyncError> {
    let Some(suffix) = base.as_str().strip_prefix("refs/remotes/") else {
        return Err(SyncError::InvalidState(
            "Base must be a remote-tracking ref".to_string(),
        ));
    };
    let Some((remote, branch)) = suffix.split_once('/') else {
        return Err(SyncError::InvalidState(
            "Base must include a remote and branch".to_string(),
        ));
    };
    if remote.is_empty() || branch.is_empty() {
        return Err(SyncError::InvalidState(
            "Base ref is incomplete".to_string(),
        ));
    }
    Ok(BaseSpec {
        remote: remote.to_string(),
        branch: branch.to_string(),
    })
}

pub(crate) fn read_branch(runner: &GitRunner) -> Result<String, SyncError> {
    let output = runner
        .run(GitCommand::read(args(&[
            "symbolic-ref",
            "--quiet",
            "--short",
            "HEAD",
        ])))
        .map_err(|_| SyncError::InvalidState("HEAD is detached".to_string()))?;
    text(&output.stdout).map(|value| value.trim().to_string())
}

pub(crate) fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, SyncError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SyncError::InvalidState)
}

pub(crate) fn optional_id(runner: &GitRunner, name: &str) -> Result<Option<ObjectId>, SyncError> {
    let spec = format!("{name}^{{commit}}");
    let Ok(output) = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec]))) else {
        return Ok(None);
    };
    Ok(Some(
        ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SyncError::InvalidState)?,
    ))
}

pub(crate) fn ensure_no_operation(runner: &GitRunner) -> Result<(), SyncError> {
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
    ] {
        if git_path(runner, marker)?.exists() {
            return Err(SyncError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn has_unmerged_entries(runner: &GitRunner) -> Result<bool, SyncError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=no",
    ])))?;
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .any(|record| record.starts_with(b"u ")))
}

pub(crate) fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, SyncError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--git-path", marker])))?;
    let path = PathBuf::from(text(&output.stdout)?.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        runner.repo_path().join(path)
    })
}

pub(crate) fn text(bytes: &[u8]) -> Result<String, SyncError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SyncError::InvalidState("Git output is not UTF-8".to_string()))
}

pub(crate) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
