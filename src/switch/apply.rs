use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::GitCommand;
use crate::recording::Oplog;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{
    DeleteSavedWorkResult, QuickSwitchPlan, QuickSwitchResult, RestoreSavedWorkResult, SavedWork,
};
use super::{plan, record, state, stash};

struct TrackedPrep {
    saved_work: Option<SavedWork>,
    carry_snapshot: Option<ObjectId>,
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
    switch_branch(runner, &switch_plan.target_branch)?;
    let carried_index =
        reapply_carried_changes(runner, &tracked.carry_snapshot, &switch_plan.target_branch)?;
    let mut after = BTreeMap::new();
    after.insert("HEAD".to_string(), switch_plan.target_head.to_string());
    if let Some(saved) = &tracked.saved_work {
        after.insert(saved.reference.clone(), saved.snapshot.to_string());
    }
    oplog
        .finish(&operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(QuickSwitchResult {
        source_branch: switch_plan.source_branch.clone(),
        target_branch: switch_plan.target_branch.clone(),
        saved_work: tracked.saved_work,
        carried_index,
        target_saved_work: switch_plan.target_saved_work.clone(),
    })
}

fn prepare_tracked_changes(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<TrackedPrep, SwitchError> {
    if !switch_plan.has_tracked_changes {
        return Ok(TrackedPrep {
            saved_work: None,
            carry_snapshot: None,
        });
    }
    let snapshot = stash::snapshot(runner)?;
    stash::reset_tracked(runner)?;
    if switch_plan.carry_changes {
        update_ref(runner, state::CARRY_REF, &snapshot, "")?;
        return Ok(TrackedPrep {
            saved_work: None,
            carry_snapshot: Some(snapshot),
        });
    }
    update_ref(
        runner,
        &switch_plan.saved_work_reference,
        &snapshot,
        "",
    )?;
    Ok(TrackedPrep {
        saved_work: Some(SavedWork {
            branch: switch_plan.source_branch.clone(),
            reference: switch_plan.saved_work_reference.clone(),
            snapshot,
        }),
        carry_snapshot: None,
    })
}

fn reapply_carried_changes(
    runner: &crate::git::GitRunner,
    carry_snapshot: &Option<ObjectId>,
    target_branch: &str,
) -> Result<Option<bool>, SwitchError> {
    let Some(snapshot) = carry_snapshot else {
        return Ok(None);
    };
    let applied_index = stash::apply_carry(runner, state::CARRY_REF, target_branch)?;
    delete_ref(runner, state::CARRY_REF, snapshot)?;
    Ok(Some(applied_index))
}

fn switch_branch(runner: &crate::git::GitRunner, target_branch: &str) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("switch"),
        OsString::from("--no-recurse-submodules"),
        OsString::from("--no-guess"),
        OsString::from("--"),
        OsString::from(target_branch),
    ]))?;
    Ok(())
}

pub(crate) fn restore(
    runner: &crate::git::GitRunner,
) -> Result<RestoreSavedWorkResult, SwitchError> {
    state::ensure_no_operation(runner)?;
    let branch = state::read_branch(runner)?;
    let Some(saved) = plan::read_saved_work(runner, &branch)? else {
        return Err(SwitchError::MissingSavedWork(branch));
    };
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = record::begin_restore(&oplog, &saved)?;
    let applied_index = stash::apply(runner, &saved.reference)?;
    delete_ref(runner, &saved.reference, &saved.snapshot)?;
    let mut after = BTreeMap::new();
    after.insert(
        "HEAD".to_string(),
        state::read_id(runner, "HEAD")?.to_string(),
    );
    oplog
        .finish(&operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(RestoreSavedWorkResult {
        branch,
        reference: saved.reference,
        applied_index,
    })
}

pub(crate) fn delete(
    runner: &crate::git::GitRunner,
    branch: &str,
) -> Result<DeleteSavedWorkResult, SwitchError> {
    state::validate_branch_name(runner, branch)?;
    state::ensure_no_operation(runner)?;
    let Some(saved) = plan::read_saved_work(runner, branch)? else {
        return Err(SwitchError::MissingSavedWork(branch.to_string()));
    };
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = record::begin_delete(&oplog, &saved)?;
    delete_ref(runner, &saved.reference, &saved.snapshot)?;
    oplog
        .finish(&operation_id, BTreeMap::new())
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(DeleteSavedWorkResult {
        branch: saved.branch,
        reference: saved.reference,
    })
}

fn update_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    value: &ObjectId,
    old: &str,
) -> Result<(), SwitchError> {
    let values = vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper save-work"),
        OsString::from(reference),
        OsString::from(value.as_str()),
        OsString::from(old),
    ];
    runner.run_unlocked(GitCommand::write(values))?;
    Ok(())
}

fn delete_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    snapshot: &ObjectId,
) -> Result<(), SwitchError> {
    let values = vec![
        OsString::from("update-ref"),
        OsString::from("-d"),
        OsString::from("-m"),
        OsString::from("git-helper delete-saved-work"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
    ];
    runner.run_unlocked(GitCommand::write(values))?;
    Ok(())
}
