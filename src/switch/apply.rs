use std::collections::BTreeMap;

use crate::git::GitRunner;
use crate::recording::Oplog;

use super::errors::SwitchError;
use super::model::{QuickSwitchPlan, QuickSwitchResult, SavedWork};
use super::{carry, checkout, plan, pull, record, state, stash, untracked};

struct TrackedPrep {
    saved_work: Option<SavedWork>,
    carry_pushed: bool,
}

struct SwitchPrep {
    tracked: TrackedPrep,
    untracked_park: Option<untracked::ParkRef>,
}

struct SwitchCtx<'a> {
    runner: &'a GitRunner,
    oplog: &'a Oplog,
    operation_id: &'a str,
    plan: &'a QuickSwitchPlan,
}

pub(crate) fn switch(
    runner: &GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<QuickSwitchResult, SwitchError> {
    plan::verify_current(runner, switch_plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = record::begin_switch(&oplog, switch_plan)?;
    let ctx = SwitchCtx {
        runner,
        oplog: &oplog,
        operation_id: &operation_id,
        plan: switch_plan,
    };
    let prep = SwitchPrep {
        tracked: prepare_tracked_changes(runner, switch_plan)?,
        untracked_park: park_untracked(runner, switch_plan, &operation_id)?,
    };
    checkout::switch_branch(runner, switch_plan)?;
    if let Some(remote_ref) = &switch_plan.pull_remote_ref {
        if !pull::fast_forward(runner, remote_ref)? {
            return pause_for_pull(&ctx, &prep);
        }
        return finish_switch(&ctx, &prep, /*pulled=*/true);
    }
    finish_switch(&ctx, &prep, /*pulled=*/false)
}

fn park_untracked(
    runner: &GitRunner,
    switch_plan: &QuickSwitchPlan,
    operation_id: &str,
) -> Result<Option<untracked::ParkRef>, SwitchError> {
    if switch_plan.untracked_conflicts.is_empty() {
        return Ok(None);
    }
    untracked::park(runner, &switch_plan.untracked_conflicts, operation_id).map(Some)
}

fn pause_for_pull(
    ctx: &SwitchCtx<'_>,
    prep: &SwitchPrep,
) -> Result<QuickSwitchResult, SwitchError> {
    let carry_reference = if prep.tracked.carry_pushed {
        Some(pull::anchor_carry(ctx.runner, ctx.operation_id)?)
    } else {
        None
    };
    let untracked_merge_reference = prep
        .untracked_park
        .as_ref()
        .map(|park| park.reference.clone());
    record::mark_pull_failed(
        ctx.oplog,
        ctx.operation_id,
        record::PullPauseSnapshots {
            carry: carry_reference,
            untracked_merge: untracked_merge_reference,
        },
    )?;
    Ok(QuickSwitchResult {
        source_branch: ctx.plan.source_branch.clone(),
        target_branch: ctx.plan.target_branch.clone(),
        saved_work: prep.tracked.saved_work.clone(),
        carried_index: None,
        carry_warning: None,
        target_saved_work: ctx.plan.target_saved_work.clone(),
        pulled: false,
        pull_warning: Some(
            "Pull could not fast-forward. Choose how to update from the remote.".to_string(),
        ),
        pull_decision_needed: true,
        untracked_merge_warning: None,
    })
}

fn finish_switch(
    ctx: &SwitchCtx<'_>,
    prep: &SwitchPrep,
    pulled: bool,
) -> Result<QuickSwitchResult, SwitchError> {
    let carry = carry::pop(ctx.runner, ctx.plan, prep.tracked.carry_pushed)?;
    let untracked_merge_warning = prep
        .untracked_park
        .as_ref()
        .map(|park| untracked::reapply(ctx.runner, &park.reference))
        .transpose()?
        .flatten();
    let saved_work = prep.tracked.saved_work.clone().or(carry.saved_work);
    let mut after = BTreeMap::new();
    after.insert(
        "HEAD".to_string(),
        state::read_id(ctx.runner, "HEAD")?.to_string(),
    );
    if let Some(saved) = &saved_work {
        after.insert(saved.reference.clone(), saved.snapshot.to_string());
    }
    ctx.oplog
        .finish(ctx.operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(QuickSwitchResult {
        source_branch: ctx.plan.source_branch.clone(),
        target_branch: ctx.plan.target_branch.clone(),
        saved_work,
        carried_index: carry.carried_index,
        carry_warning: carry.warning,
        target_saved_work: ctx.plan.target_saved_work.clone(),
        pulled,
        pull_warning: None,
        pull_decision_needed: false,
        untracked_merge_warning,
    })
}

fn prepare_tracked_changes(
    runner: &GitRunner,
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
    state::update_ref(runner, &switch_plan.saved_work_reference, &snapshot, "")?;
    Ok(TrackedPrep {
        saved_work: Some(SavedWork {
            branch: switch_plan.source_branch.clone(),
            reference: switch_plan.saved_work_reference.clone(),
            snapshot,
        }),
        carry_pushed: false,
    })
}

pub(crate) fn restore(runner: &GitRunner) -> Result<super::model::RestoreSavedWorkResult, SwitchError> {
    super::saved::restore(runner)
}

pub(crate) fn delete(
    runner: &GitRunner,
    branch: &str,
) -> Result<super::model::DeleteSavedWorkResult, SwitchError> {
    super::saved::delete(runner, branch)
}
