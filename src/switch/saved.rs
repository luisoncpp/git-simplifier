use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::GitCommand;
use crate::recording::Oplog;
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{DeleteSavedWorkResult, RestoreSavedWorkResult};
use super::{plan, record, restore_apply, state};

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
    let outcome = restore_apply::apply(runner, &saved.reference, &operation_id)?;
    if outcome.consumed {
        delete_ref(runner, &saved.reference, &saved.snapshot)?;
    }
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
        applied_index: outcome.applied_index,
        warning: outcome.warning,
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

fn delete_ref(
    runner: &crate::git::GitRunner,
    reference: &str,
    snapshot: &ObjectId,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-d"),
        OsString::from("-m"),
        OsString::from("git-helper delete-saved-work"),
        OsString::from(reference),
        OsString::from(snapshot.as_str()),
    ]))?;
    Ok(())
}
