use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::GitCommand;
use crate::recording::Oplog;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{
    DeleteSavedWorkResult, QuickSwitchPlan, QuickSwitchResult, RestoreSavedWorkResult, SavedWork,
};
use super::{carry, checkout, plan, pull, record, state, stash};

struct TrackedPrep {
    saved_work: Option<SavedWork>,
    carry_pushed: bool,
}

pub(crate) fn switch(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<QuickSwitchResult, SwitchError> {
    plan::verify_current(runner, switch_plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = record::begin_switch(&oplog, switch_plan)?;
    let tracked = prepare_tracked_changes(runner, switch_plan)?;
    checkout::switch_branch(runner, switch_plan)?;
    if let Some(remote_ref) = &switch_plan.pull_remote_ref {
        if !pull::fast_forward(runner, remote_ref)? {
            return pause_for_pull_decision(
                runner, &oplog, &operation_id, switch_plan, tracked, remote_ref,
            );
        }
        return finish_switch(
            runner, &oplog, &operation_id, switch_plan, tracked, /*pulled=*/true,
        );
    }
    finish_switch(
        runner, &oplog, &operation_id, switch_plan, tracked, /*pulled=*/false,
    )
}

fn pause_for_pull_decision(
    runner: &crate::git::GitRunner,
    oplog: &Oplog,
    operation_id: &str,
    switch_plan: &QuickSwitchPlan,
    tracked: TrackedPrep,
    remote_ref: &str,
) -> Result<QuickSwitchResult, SwitchError> {
    let carry_reference = if tracked.carry_pushed {
        Some(pull::anchor_carry(runner, operation_id)?)
    } else {
        None
    };
    record::mark_pull_failed(oplog, operation_id, remote_ref, carry_reference.as_deref())?;
    Ok(QuickSwitchResult {
        source_branch: switch_plan.source_branch.clone(),
        target_branch: switch_plan.target_branch.clone(),
        saved_work: tracked.saved_work,
        carried_index: None,
        carry_warning: None,
        target_saved_work: switch_plan.target_saved_work.clone(),
        pulled: false,
        pull_warning: Some(
            "Pull could not fast-forward. Choose how to update from the remote.".to_string(),
        ),
        pull_decision_needed: true,
    })
}

fn finish_switch(
    runner: &crate::git::GitRunner,
    oplog: &Oplog,
    operation_id: &str,
    switch_plan: &QuickSwitchPlan,
    tracked: TrackedPrep,
    pulled: bool,
) -> Result<QuickSwitchResult, SwitchError> {
    let carry = carry::pop(runner, switch_plan, tracked.carry_pushed)?;
    let saved_work = tracked.saved_work.or(carry.saved_work);
    let mut after = BTreeMap::new();
    after.insert(
        "HEAD".to_string(),
        state::read_id(runner, "HEAD")?.to_string(),
    );
    if let Some(saved) = &saved_work {
        after.insert(saved.reference.clone(), saved.snapshot.to_string());
    }
    oplog
        .finish(operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(QuickSwitchResult {
        source_branch: switch_plan.source_branch.clone(),
        target_branch: switch_plan.target_branch.clone(),
        saved_work,
        carried_index: carry.carried_index,
        carry_warning: carry.warning,
        target_saved_work: switch_plan.target_saved_work.clone(),
        pulled,
        pull_warning: None,
        pull_decision_needed: false,
    })
}

fn prepare_tracked_changes(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<TrackedPrep, SwitchError> {
    if !switch_plan.has_tracked_changes {
        return Ok(TrackedPrep {
            saved_work: None,
            carry_pushed: false,
        });
    }
    if switch_plan.carry_changes {
        stash::push_tracked(runner)?;
        return Ok(TrackedPrep {
            saved_work: None,
            carry_pushed: true,
        });
    }
    let snapshot = stash::snapshot(runner)?;
    stash::reset_tracked(runner)?;
    update_ref(runner, &switch_plan.saved_work_reference, &snapshot, "")?;
    Ok(TrackedPrep {
        saved_work: Some(SavedWork {
            branch: switch_plan.source_branch.clone(),
            reference: switch_plan.saved_work_reference.clone(),
            snapshot,
        }),
        carry_pushed: false,
    })
}

pub(crate) fn restore(
    runner: &crate::git::GitRunner,
) -> Result<RestoreSavedWorkResult, SwitchError> {
    super::saved::restore(runner)
}

pub(crate) fn delete(
    runner: &crate::git::GitRunner,
    branch: &str,
) -> Result<DeleteSavedWorkResult, SwitchError> {
    super::saved::delete(runner, branch)
}

fn update_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    value: &ObjectId,
    old: &str,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper save-work"),
        OsString::from(reference),
        OsString::from(value.as_str()),
        OsString::from(old),
    ]))?;
    Ok(())
}
