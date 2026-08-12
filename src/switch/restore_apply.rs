use std::ffi::OsString;

use crate::git::GitCommand;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::state;
use super::stash::{self, ApplyOutcome};

const PARK_CONFLICT: &str = "Saved work was restored, but recent edits conflicted with it. \
    Resolve the conflict markers in the working tree.";

const SAVED_CONFLICT_WITH_PARK: &str = "Saved work was applied with conflicts. Resolve the \
    conflict markers, then delete Saved work when the result is correct. Your recent edits are \
    parked at {park}; reapply them after resolving.";

struct ParkedDirt {
    reference: String,
    snapshot: ObjectId,
}

/// Apply Saved work; if a dirty tree would be overwritten, park dirt and merge.
pub(super) fn apply(
    runner: &crate::git::GitRunner,
    reference: &str,
    operation_id: &str,
) -> Result<ApplyOutcome, SwitchError> {
    match stash::try_apply(runner, reference) {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            if stash::has_unmerged_paths(runner)? {
                return Ok(stash::conflict(
                    "Saved work was applied with conflicts. Resolve the conflict markers, then \
                     delete Saved work when the result is correct; the backup was kept.",
                    /*consumed=*/ false,
                ));
            }
            if !state::read_tracked_changes(runner)? {
                return Err(error);
            }
            apply_via_park(runner, reference, operation_id)
        }
    }
}

fn apply_via_park(
    runner: &crate::git::GitRunner,
    reference: &str,
    operation_id: &str,
) -> Result<ApplyOutcome, SwitchError> {
    let park = park_current(runner, operation_id)?;
    stash::reset_tracked(runner)?;
    let saved = stash::try_apply(runner, reference)?;
    if saved.conflict {
        let warning = SAVED_CONFLICT_WITH_PARK.replace("{park}", &park.reference);
        return Ok(stash::conflict(&warning, /*consumed=*/ false));
    }
    merge_park(runner, &park, saved.applied_index)
}

fn park_current(
    runner: &crate::git::GitRunner,
    operation_id: &str,
) -> Result<ParkedDirt, SwitchError> {
    let reference = format!("refs/githelper/restore-park/{operation_id}");
    let snapshot = stash::snapshot(runner)?;
    create_ref(runner, &reference, &snapshot)?;
    Ok(ParkedDirt {
        reference,
        snapshot,
    })
}

fn merge_park(
    runner: &crate::git::GitRunner,
    park: &ParkedDirt,
    saved_index: bool,
) -> Result<ApplyOutcome, SwitchError> {
    // Plain stash apply leaves the worktree dirty vs the index; park apply then
    // hard-refuses. Stage so the second apply can three-way merge.
    stage_tracked(runner)?;
    let outcome = match stash::try_apply(runner, &park.reference) {
        Ok(outcome) => outcome,
        Err(_) if stash::has_unmerged_paths(runner)? => {
            stash::conflict(PARK_CONFLICT, /*consumed=*/ true)
        }
        Err(error) => return Err(error),
    };
    delete_ref(runner, &park.reference, &park.snapshot)?;
    if outcome.conflict {
        return Ok(stash::conflict(PARK_CONFLICT, /*consumed=*/ true));
    }
    Ok(ApplyOutcome {
        applied_index: saved_index && outcome.applied_index,
        conflict: false,
        warning: None,
        consumed: true,
    })
}

fn stage_tracked(runner: &crate::git::GitRunner) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("add"),
        OsString::from("-u"),
        OsString::from("--"),
    ]))?;
    Ok(())
}

fn create_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    snapshot: &ObjectId,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper park-restore-dirt"),
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
        OsString::from("git-helper drop-restore-park"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
    ]))?;
    Ok(())
}
