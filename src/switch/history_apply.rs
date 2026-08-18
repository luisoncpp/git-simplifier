use std::collections::BTreeMap;

use crate::git::GitRunner;
use crate::recording::Oplog;

use super::errors::SwitchError;
use super::history_model::{HistorySwitchPlan, HistorySwitchResult};
use super::prep::{TrackedPrep, TrackedSpec};
use super::{carry, checkout, history_plan, history_record, prep, present, state, untracked};

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &HistorySwitchPlan,
) -> Result<HistorySwitchResult, SwitchError> {
    history_plan::verify(runner, plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = history_record::begin_history(&oplog, plan)?;
    let tracked = prep::prepare_tracked(
        runner,
        TrackedSpec {
            source_branch: &plan.source_branch,
            saved_work_reference: &plan.saved_work_reference,
            has_tracked_changes: plan.has_tracked_changes,
            carry_changes: plan.carry_changes,
        },
    )?;
    let park = park_untracked(runner, plan, &operation_id)?;
    detach(runner, plan)?;
    finish(&FinishCtx {
        runner,
        oplog: &oplog,
        operation_id: &operation_id,
        plan,
        tracked: &tracked,
        park: park.as_ref(),
    })
}

struct FinishCtx<'a> {
    runner: &'a GitRunner,
    oplog: &'a Oplog,
    operation_id: &'a str,
    plan: &'a HistorySwitchPlan,
    tracked: &'a TrackedPrep,
    park: Option<&'a untracked::ParkRef>,
}

fn park_untracked(
    runner: &GitRunner,
    plan: &HistorySwitchPlan,
    operation_id: &str,
) -> Result<Option<untracked::ParkRef>, SwitchError> {
    if plan.untracked_conflicts.is_empty() {
        return Ok(None);
    }
    untracked::park(runner, &plan.untracked_conflicts, operation_id).map(Some)
}

fn detach(runner: &GitRunner, plan: &HistorySwitchPlan) -> Result<(), SwitchError> {
    present::write(runner, &plan.source_branch)?;
    if let Err(error) = checkout::switch_detach(runner, plan.target_commit.as_str()) {
        let _ = present::delete(runner);
        return Err(error);
    }
    Ok(())
}

fn finish(ctx: &FinishCtx<'_>) -> Result<HistorySwitchResult, SwitchError> {
    let carry = carry::pop(
        ctx.runner,
        carry::CarrySource {
            branch: &ctx.plan.source_branch,
            saved_work_reference: &ctx.plan.saved_work_reference,
        },
        ctx.tracked.carry_pushed,
    )?;
    let untracked_merge_warning = ctx
        .park
        .map(|park| untracked::reapply(ctx.runner, &park.reference))
        .transpose()?
        .flatten();
    let saved_work = ctx.tracked.saved_work.clone().or(carry.saved_work);
    write_after(ctx, saved_work.as_ref())?;
    Ok(HistorySwitchResult {
        source_branch: ctx.plan.source_branch.clone(),
        target_commit: ctx.plan.target_commit.clone(),
        saved_work,
        carried_index: carry.carried_index,
        carry_warning: carry.warning,
        untracked_merge_warning,
        present_branch: ctx.plan.source_branch.clone(),
    })
}

fn write_after(
    ctx: &FinishCtx<'_>,
    saved_work: Option<&super::model::SavedWork>,
) -> Result<(), SwitchError> {
    let mut after = BTreeMap::new();
    after.insert(
        "HEAD".to_string(),
        state::read_id(ctx.runner, "HEAD")?.to_string(),
    );
    if let Some(saved) = saved_work {
        after.insert(saved.reference.clone(), saved.snapshot.to_string());
    }
    ctx.oplog
        .finish(ctx.operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))
}
