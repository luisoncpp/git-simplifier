use std::collections::BTreeMap;

use crate::git::GitRunner;
use crate::recording::Oplog;

use super::errors::SwitchError;
use super::model::{PullResolution, QuickSwitchPhase, QuickSwitchResult, QuickSwitchStatus};
use super::{pull, record, state};

pub(crate) fn status(runner: &GitRunner) -> Result<Option<QuickSwitchStatus>, SwitchError> {
    let oplog = Oplog::open_existing(&runner.git_dir()?);
    let Some(context) = record::active_pull_decision(&oplog)? else {
        return Ok(None);
    };
    Ok(Some(QuickSwitchStatus {
        operation_id: context.id,
        target_branch: context.target_branch,
        remote_ref: context.remote_ref,
        phase: QuickSwitchPhase::PullFastForwardFailed,
        carry_reference: context.carry_reference,
    }))
}

pub(crate) fn resolve(
    runner: &GitRunner,
    resolution: PullResolution,
) -> Result<QuickSwitchResult, SwitchError> {
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let Some(context) = record::active_pull_decision(&oplog)? else {
        return Err(SwitchError::InvalidState(
            "no interrupted quick switch pull exists".to_string(),
        ));
    };
    if state::read_branch(runner)? != context.target_branch {
        return Err(SwitchError::InvalidState(
            "the branch changed after the pull was interrupted".to_string(),
        ));
    }
    let (pulled, pull_warning) = apply_resolution(runner, resolution, &context.remote_ref)?;
    let (carried_index, carry_warning) = reapply_carry(runner, context.carry_reference.as_deref())?;
    let mut after = BTreeMap::new();
    after.insert(
        "HEAD".to_string(),
        state::read_id(runner, "HEAD")?.to_string(),
    );
    oplog
        .finish(&context.id, after)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(QuickSwitchResult {
        source_branch: context.source_branch,
        target_branch: context.target_branch,
        saved_work: None,
        carried_index,
        carry_warning,
        target_saved_work: None,
        pulled,
        pull_warning,
        pull_decision_needed: false,
    })
}

fn apply_resolution(
    runner: &GitRunner,
    resolution: PullResolution,
    remote_ref: &str,
) -> Result<(bool, Option<String>), SwitchError> {
    match resolution {
        PullResolution::Cancel => Ok((false, None)),
        PullResolution::ReplaceWithRemote => {
            pull::replace_with_remote(runner, remote_ref)?;
            Ok((true, None))
        }
        PullResolution::MergePull => {
            if pull::merge_pull(runner, remote_ref)? {
                return Ok((true, None));
            }
            Ok((
                false,
                Some(
                    "Merge pull left conflicts. Resolve them in the working tree, then continue \
                     your work."
                        .to_string(),
                ),
            ))
        }
    }
}

fn reapply_carry(
    runner: &GitRunner,
    carry_reference: Option<&str>,
) -> Result<(Option<bool>, Option<String>), SwitchError> {
    let Some(reference) = carry_reference else {
        return Ok((None, None));
    };
    if state::optional_id(runner, reference)?.is_none() {
        return Ok((None, None));
    }
    // Leave carry anchored when a merge left conflicts; applying over MERGE_HEAD is unsafe.
    if merge_in_progress(runner)? {
        return Ok((
            None,
            Some(format!(
                "Carried changes remain at {reference} until the merge conflicts are resolved."
            )),
        ));
    }
    let (indexed, warning) = pull::apply_carry_ref(runner, reference)?;
    Ok((Some(indexed), warning))
}

fn merge_in_progress(runner: &GitRunner) -> Result<bool, SwitchError> {
    Ok(runner.git_dir()?.join("MERGE_HEAD").exists()
        || state::optional_id(runner, "MERGE_HEAD")?.is_some())
}
