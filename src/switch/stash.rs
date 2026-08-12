use std::ffi::OsString;

use crate::git::GitCommand;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;

pub(super) struct PopOutcome {
    pub applied_index: bool,
    pub warning: Option<String>,
}

pub(super) struct ApplyOutcome {
    pub applied_index: bool,
    pub conflict: bool,
    pub warning: Option<String>,
    /// When true, the Saved work snapshot is already in the tree; delete the WIP ref.
    pub consumed: bool,
}

const CONFLICT_WARNING: &str = "Saved work was applied with conflicts. Resolve the conflict \
    markers, then delete Saved work when the result is correct; the backup was kept.";

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

/// Apply a stash-shaped ref without parking local dirt (carry / pull paths).
pub(super) fn apply(
    runner: &crate::git::GitRunner,
    reference: &str,
) -> Result<ApplyOutcome, SwitchError> {
    try_apply(runner, reference)
}

pub(super) fn try_apply(
    runner: &crate::git::GitRunner,
    reference: &str,
) -> Result<ApplyOutcome, SwitchError> {
    if runner
        .run_unlocked(GitCommand::write(stash_args(&[
            "apply", "--index", reference,
        ])))
        .is_ok()
    {
        return Ok(clean(/*applied_index=*/ true));
    }
    if has_unmerged_paths(runner)? {
        return Ok(conflict(CONFLICT_WARNING, /*consumed=*/ false));
    }
    match runner.run_unlocked(GitCommand::write(stash_args(&["apply", reference]))) {
        Ok(_) => Ok(clean(/*applied_index=*/ false)),
        Err(source) => {
            if has_unmerged_paths(runner)? {
                return Ok(conflict(CONFLICT_WARNING, /*consumed=*/ false));
            }
            Err(SwitchError::Git(source))
        }
    }
}

pub(super) fn has_unmerged_paths(runner: &crate::git::GitRunner) -> Result<bool, SwitchError> {
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

pub(super) fn drop_top(runner: &crate::git::GitRunner) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(stash_args(&["drop", "stash@{0}"])))?;
    Ok(())
}

fn clean(applied_index: bool) -> ApplyOutcome {
    ApplyOutcome {
        applied_index,
        conflict: false,
        warning: None,
        consumed: true,
    }
}

pub(super) fn conflict(warning: &str, consumed: bool) -> ApplyOutcome {
    ApplyOutcome {
        applied_index: false,
        conflict: true,
        warning: Some(warning.to_string()),
        consumed,
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
