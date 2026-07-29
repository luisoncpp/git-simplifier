use std::ffi::OsString;

use crate::git::GitCommand;

use super::errors::SwitchError;
use crate::rewrite::ObjectId;

pub(super) struct PopOutcome {
    pub applied_index: bool,
    pub warning: Option<String>,
}

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

pub(super) fn push_tracked(runner: &crate::git::GitRunner) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(stash_args(&[
        "push",
        "-m",
        "git-helper carry",
    ])))?;
    Ok(())
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
    if has_unmerged_paths(runner)? {
        return Err(SwitchError::SavedWorkConflict);
    }
    runner.run_unlocked(GitCommand::write(stash_args(&["apply", reference])))?;
    Ok(false)
}

fn has_unmerged_paths(runner: &crate::git::GitRunner) -> Result<bool, SwitchError> {
    let output = runner.run_unlocked(GitCommand::read(args(&[
        "diff",
        "--name-only",
        "--diff-filter=U",
    ])))?;
    Ok(!output.stdout.is_empty())
}

pub(super) fn pop_carry(runner: &crate::git::GitRunner) -> Result<PopOutcome, SwitchError> {
    if runner
        .run_unlocked(GitCommand::write(stash_args(&["pop", "--index"])))
        .is_ok()
    {
        return Ok(PopOutcome {
            applied_index: true,
            warning: None,
        });
    }
    if runner
        .run_unlocked(GitCommand::write(stash_args(&["pop"])))
        .is_ok()
    {
        return Ok(PopOutcome {
            applied_index: false,
            warning: None,
        });
    }
    Ok(PopOutcome {
        applied_index: false,
        warning: Some(
            "Carried changes could not be popped cleanly. Resolve any conflict markers in the \
             working tree, then run git stash drop if an entry is still listed."
                .to_string(),
        ),
    })
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
