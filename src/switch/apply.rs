use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::GitCommand;
use crate::recording::Oplog;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{
    DeleteSavedWorkResult, QuickSwitchPlan, QuickSwitchResult, RestoreSavedWorkResult, SavedWork,
};
use super::{plan, record, state};

pub(crate) fn switch(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<QuickSwitchResult, SwitchError> {
    plan::verify_current(runner, switch_plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let operation_id = record::begin_switch(&oplog, switch_plan)?;
    let saved_work = save_tracked_changes(runner, switch_plan)?;
    switch_branch(runner, &switch_plan.target_branch)?;
    let mut after = BTreeMap::new();
    after.insert("HEAD".to_string(), switch_plan.target_head.to_string());
    if let Some(saved) = &saved_work {
        after.insert(saved.reference.clone(), saved.snapshot.to_string());
    }
    oplog
        .finish(&operation_id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(QuickSwitchResult {
        source_branch: switch_plan.source_branch.clone(),
        target_branch: switch_plan.target_branch.clone(),
        saved_work,
        target_saved_work: switch_plan.target_saved_work.clone(),
    })
}

fn save_tracked_changes(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<Option<SavedWork>, SwitchError> {
    if !switch_plan.has_tracked_changes {
        return Ok(None);
    }
    let output = runner.run_unlocked(GitCommand::write(stash_args(&["create"])))?;
    let value = text(&output.stdout)?.trim().to_string();
    if value.is_empty() {
        return Err(SwitchError::InvalidState(
            "Git did not create Saved work for tracked changes".to_string(),
        ));
    }
    let snapshot = ObjectId::new(value).map_err(SwitchError::InvalidState)?;
    update_ref(runner, &switch_plan.saved_work_reference, &snapshot, "")?;
    runner.run_unlocked(GitCommand::write(args(&[
        "reset",
        "--hard",
        "--no-recurse-submodules",
        "HEAD",
    ])))?;
    Ok(Some(SavedWork {
        branch: switch_plan.source_branch.clone(),
        reference: switch_plan.saved_work_reference.clone(),
        snapshot,
    }))
}

fn switch_branch(runner: &crate::git::GitRunner, target_branch: &str) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "switch",
        "--no-recurse-submodules",
        "--no-guess",
        "--",
        target_branch,
    ])))?;
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
    let applied_index = apply_stash(runner, &saved.reference)?;
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

fn apply_stash(runner: &crate::git::GitRunner, reference: &str) -> Result<bool, SwitchError> {
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
