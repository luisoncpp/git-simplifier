use crate::git::GitRunner;

use super::errors::SwitchError;
use super::model::{QuickSwitchPlan, QuickSwitchRequest};
use super::{present, state};

pub(super) fn branch_for_switch(
    runner: &GitRunner,
    request: &QuickSwitchRequest,
) -> Result<String, SwitchError> {
    if let Some(branch) = state::optional_branch(runner)? {
        return Ok(branch);
    }
    if let Some(present) = present::read(runner)? {
        return Ok(present);
    }
    if state::read_tracked_changes(runner)? && !request.carry_changes {
        return Err(SwitchError::InvalidState(
            "HEAD is detached; carry tracked changes or return to a branch first".to_string(),
        ));
    }
    Ok(String::new())
}

pub(super) fn already_on_target(
    runner: &GitRunner,
    target_branch: &str,
) -> Result<bool, SwitchError> {
    Ok(state::optional_branch(runner)?.as_deref() == Some(target_branch))
}

pub(super) fn verify_source(
    runner: &GitRunner,
    plan: &QuickSwitchPlan,
) -> Result<(), SwitchError> {
    let current = state::optional_branch(runner)?;
    match current {
        Some(branch) if branch != plan.source_branch => Err(SwitchError::StalePlan),
        _ => Ok(()),
    }
}
