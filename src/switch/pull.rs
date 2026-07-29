use std::ffi::OsString;

use crate::git::GitCommand;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::state;

/// Returns `true` when the fast-forward succeeded (or was already up to date).
pub(crate) fn fast_forward(
    runner: &crate::git::GitRunner,
    remote_ref: &str,
) -> Result<bool, SwitchError> {
    let (remote, branch) = remote_parts(remote_ref)?;
    let result = runner.run_unlocked(GitCommand::write(vec![
        OsString::from("pull"),
        OsString::from("--ff-only"),
        OsString::from("--no-recurse-submodules"),
        OsString::from("--no-tags"),
        OsString::from(remote),
        OsString::from(branch),
    ]));
    Ok(result.is_ok())
}

pub(crate) fn replace_with_remote(
    runner: &crate::git::GitRunner,
    remote_ref: &str,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("reset"),
        OsString::from("--hard"),
        OsString::from("--no-recurse-submodules"),
        OsString::from(remote_ref),
    ]))?;
    Ok(())
}

/// Merge pull that may leave conflicts. Returns `true` when clean.
pub(crate) fn merge_pull(
    runner: &crate::git::GitRunner,
    remote_ref: &str,
) -> Result<bool, SwitchError> {
    let (remote, branch) = remote_parts(remote_ref)?;
    let result = runner.run_unlocked(GitCommand::write(vec![
        OsString::from("pull"),
        OsString::from("--no-ff"),
        OsString::from("--no-rebase"),
        OsString::from("--no-recurse-submodules"),
        OsString::from("--no-tags"),
        OsString::from(remote),
        OsString::from(branch),
    ]));
    Ok(result.is_ok())
}

/// Move the top stash entry onto a durable carry ref and drop it from the stack.
pub(crate) fn anchor_carry(
    runner: &crate::git::GitRunner,
    operation_id: &str,
) -> Result<String, SwitchError> {
    let output = runner.run_unlocked(GitCommand::read(state::args(&[
        "rev-parse",
        "--verify",
        "stash@{0}",
    ])))?;
    let snapshot = ObjectId::new(state::text(&output.stdout)?.trim().to_string())
        .map_err(SwitchError::InvalidState)?;
    let reference = state::carry_ref(operation_id);
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper carry"),
        OsString::from(&reference),
        OsString::from(snapshot.as_str()),
        OsString::from(""),
    ]))?;
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("-c"),
        OsString::from("submodule.recurse=false"),
        OsString::from("stash"),
        OsString::from("drop"),
        OsString::from("stash@{0}"),
    ]))?;
    Ok(reference)
}

pub(crate) fn apply_carry_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
) -> Result<(bool, Option<String>), SwitchError> {
    let applied = super::stash::apply(runner, reference);
    match applied {
        Ok(indexed) => {
            delete_carry(runner, reference)?;
            Ok((indexed, None))
        }
        Err(SwitchError::SavedWorkConflict) => Ok((
            false,
            Some(
                "Carried changes conflicted after the pull. Resolve the markers, then delete the \
                 carry ref when done."
                    .to_string(),
            ),
        )),
        Err(error) => Err(error),
    }
}

fn delete_carry(runner: &crate::git::GitRunner, reference: &str) -> Result<(), SwitchError> {
    let Some(snapshot) = state::optional_id(runner, reference)? else {
        return Ok(());
    };
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-d"),
        OsString::from("-m"),
        OsString::from("git-helper drop-carry"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
    ]))?;
    Ok(())
}

fn remote_parts(remote_ref: &str) -> Result<(&str, &str), SwitchError> {
    let value = remote_ref
        .strip_prefix("refs/remotes/")
        .ok_or_else(|| SwitchError::InvalidState(format!("not a remote-tracking ref: {remote_ref}")))?;
    value.split_once('/').filter(|(remote, branch)| {
        !remote.is_empty() && !branch.is_empty()
    }).ok_or_else(|| {
        SwitchError::InvalidState(format!("remote-tracking ref is incomplete: {remote_ref}"))
    })
}
