//! Park overlapping untracked files and reapply them onto the target branch.

use std::ffi::OsString;

use crate::git::GitCommand;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::state;
use super::stash;

const MERGE_WARNING: &str = "Untracked files were merged with the target branch and may leave \
    conflict markers. Resolve them before continuing.";

pub(super) struct ParkRef {
    pub reference: String,
}

pub(super) fn park(
    runner: &crate::git::GitRunner,
    paths: &[String],
    operation_id: &str,
) -> Result<ParkRef, SwitchError> {
    stage_paths(runner, paths)?;
    let snapshot = stash::snapshot(runner)?;
    let reference = state::untracked_merge_ref(operation_id);
    write_ref(runner, &reference, &snapshot)?;
    clear_paths(runner, paths)?;
    Ok(ParkRef {
        reference,
    })
}

pub(super) fn reapply(
    runner: &crate::git::GitRunner,
    reference: &str,
) -> Result<Option<String>, SwitchError> {
    let Some(snapshot) = state::optional_id(runner, reference)? else {
        return Ok(None);
    };
    let outcome = stash::try_apply(runner, reference)?;
    if outcome.consumed {
        delete_ref(runner, reference, &snapshot)?;
    }
    if outcome.conflict || outcome.warning.is_some() {
        return Ok(Some(
            outcome
                .warning
                .unwrap_or_else(|| MERGE_WARNING.to_string()),
        ));
    }
    Ok(None)
}

fn stage_paths(runner: &crate::git::GitRunner, paths: &[String]) -> Result<(), SwitchError> {
    for path in paths {
        let spec = format!(":(top,literal){path}");
        runner.run_unlocked(GitCommand::write(vec![
            OsString::from("add"),
            OsString::from("--"),
            OsString::from(spec),
        ]))?;
    }
    Ok(())
}

fn clear_paths(runner: &crate::git::GitRunner, paths: &[String]) -> Result<(), SwitchError> {
    for path in paths {
        runner.run_unlocked(GitCommand::write(vec![
            OsString::from("restore"),
            OsString::from("--worktree"),
            OsString::from("--source=HEAD"),
            OsString::from("--"),
            OsString::from(path),
        ]))?;
        runner.run_unlocked(GitCommand::write(vec![
            OsString::from("reset"),
            OsString::from("HEAD"),
            OsString::from("--"),
            OsString::from(path),
        ]))?;
    }
    Ok(())
}

fn write_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    snapshot: &ObjectId,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper untracked-merge"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
        OsString::from(""),
    ]))?;
    Ok(())
}

fn delete_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    snapshot: &ObjectId,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-d"),
        OsString::from("-m"),
        OsString::from("git-helper drop-untracked-merge"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
    ]))?;
    Ok(())
}
