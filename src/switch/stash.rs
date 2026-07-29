use std::ffi::OsString;

use crate::git::{GitCommand, GitError};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;

pub(super) fn snapshot(runner: &crate::git::GitRunner) -> Result<ObjectId, SwitchError> {
    let output = runner.run_unlocked(GitCommand::write(stash_args(&["create"])))?;
    let value = text(&output.stdout)?.trim().to_string();
    if value.is_empty() {
        return Err(SwitchError::InvalidState(
            "Git did not create Saved work for tracked changes".to_string(),
        ));
    }
    ObjectId::new(value).map_err(SwitchError::InvalidState)
}

pub(super) fn reset_tracked(runner: &crate::git::GitRunner) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "reset",
        "--hard",
        "--no-recurse-submodules",
        "HEAD",
    ])))?;
    Ok(())
}

pub(super) fn apply(runner: &crate::git::GitRunner, reference: &str) -> Result<bool, SwitchError> {
    let indexed = runner
        .run_unlocked(GitCommand::write(stash_args(&[
            "apply", "--index", reference,
        ])))
        .is_ok();
    if indexed {
        return Ok(true);
    }
    runner.run_unlocked(GitCommand::write(stash_args(&["apply", reference])))?;
    Ok(false)
}

pub(super) fn apply_carry(
    runner: &crate::git::GitRunner,
    reference: &str,
    target_branch: &str,
) -> Result<bool, SwitchError> {
    apply(runner, reference).map_err(|error| carry_apply_failed(error, target_branch))
}

fn carry_apply_failed(error: SwitchError, target_branch: &str) -> SwitchError {
    let SwitchError::Git(git) = error else {
        return error;
    };
    let detail = git_detail(&git);
    SwitchError::CarryReapplyFailed(format!(
        "You're on {target_branch}, but replaying the carried changes failed: {detail} \
         The snapshot is still at refs/githelper/carry/pending."
    ))
}

fn git_detail(error: &GitError) -> String {
    match error {
        GitError::Command { stderr, .. } => {
            let text = String::from_utf8_lossy(stderr).trim().to_string();
            if text.is_empty() {
                error.to_string()
            } else {
                text
            }
        }
        other => other.to_string(),
    }
}

fn text(bytes: &[u8]) -> Result<String, SwitchError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SwitchError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn stash_args(values: &[&str]) -> Vec<OsString> {
    let mut command = args(&["-c", "submodule.recurse=false", "stash"]);
    command.extend(args(values));
    command
}
