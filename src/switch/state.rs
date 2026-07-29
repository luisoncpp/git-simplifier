use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;

pub(super) const WIP_PREFIX: &str = "refs/githelper/wip/";

pub(super) fn read_branch(runner: &GitRunner) -> Result<String, SwitchError> {
    let output = runner
        .run(GitCommand::read(args(&[
            "symbolic-ref",
            "--quiet",
            "--short",
            "HEAD",
        ])))
        .map_err(|_| SwitchError::InvalidState("HEAD is detached".to_string()))?;
    text(&output.stdout).map(|value| value.trim().to_string())
}

pub(super) fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, SwitchError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SwitchError::InvalidState)
}

pub(super) fn optional_id(runner: &GitRunner, name: &str) -> Result<Option<ObjectId>, SwitchError> {
    let Ok(output) = runner.run(GitCommand::read(args(&[
        "rev-parse",
        "--verify",
        &format!("{name}^{{commit}}"),
    ]))) else {
        return Ok(None);
    };
    Ok(Some(
        ObjectId::new(text(&output.stdout)?.trim().to_string())
            .map_err(SwitchError::InvalidState)?,
    ))
}

pub(super) fn read_tracked_changes(runner: &GitRunner) -> Result<bool, SwitchError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=no",
        "--ignore-submodules=all",
    ])))?;
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .any(|record| !record.is_empty()))
}

pub(super) fn ensure_no_operation(runner: &GitRunner) -> Result<(), SwitchError> {
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
    ] {
        if git_path(runner, marker)?.exists() {
            return Err(SwitchError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, SwitchError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--git-path", marker])))?;
    let path = PathBuf::from(text(&output.stdout)?.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        runner.repo_path().join(path)
    })
}

pub(super) fn validate_branch_name(runner: &GitRunner, branch: &str) -> Result<(), SwitchError> {
    if branch.is_empty() || branch.starts_with('-') || branch.contains('\0') {
        return Err(SwitchError::InvalidState(format!(
            "invalid branch name: {branch}"
        )));
    }
    runner.run(GitCommand::read(args(&[
        "check-ref-format",
        "--branch",
        branch,
    ])))?;
    Ok(())
}

pub(super) fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

pub(super) fn wip_ref(branch: &str) -> String {
    format!("{WIP_PREFIX}{branch}")
}

pub(super) fn carry_ref(operation_id: &str) -> String {
    format!("refs/githelper/carry/{operation_id}")
}

/// Accepts `origin/feature` or `refs/remotes/origin/feature`.
pub(super) fn remote_tracking_ref(remote: &str) -> Result<String, SwitchError> {
    if remote.starts_with("refs/remotes/") {
        return Ok(remote.to_string());
    }
    if remote.is_empty() || remote.starts_with('-') || !remote.contains('/') {
        return Err(SwitchError::InvalidState(format!(
            "invalid remote-tracking name: {remote}"
        )));
    }
    Ok(format!("refs/remotes/{remote}"))
}

/// Prefer `origin/<branch>`, otherwise the first `refs/remotes/*/<branch>`.
pub(super) fn same_named_remote(
    runner: &GitRunner,
    branch: &str,
) -> Result<Option<String>, SwitchError> {
    let preferred = format!("refs/remotes/origin/{branch}");
    if optional_id(runner, &preferred)?.is_some() {
        return Ok(Some(preferred));
    }
    let output = runner.run(GitCommand::read(args(&[
        "for-each-ref",
        "--format=%(refname)",
        "refs/remotes",
    ])))?;
    let suffix = format!("/{branch}");
    for line in output.stdout.split(|byte| *byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let reference = text(line)?;
        if reference.ends_with("/HEAD") {
            continue;
        }
        if reference.ends_with(&suffix) {
            return Ok(Some(reference));
        }
    }
    Ok(None)
}

pub(super) fn text(bytes: &[u8]) -> Result<String, SwitchError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SwitchError::InvalidState("Git output is not UTF-8".to_string()))
}

pub(super) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
