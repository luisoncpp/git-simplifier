use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{QuickSwitchPlan, QuickSwitchRequest, SavedWork};
use super::{preflight, state};

pub(crate) fn create(
    runner: &GitRunner,
    request: QuickSwitchRequest,
) -> Result<QuickSwitchPlan, SwitchError> {
    state::validate_branch_name(runner, &request.target_branch)?;
    state::ensure_no_operation(runner)?;
    let source_branch = state::read_branch(runner)?;
    if source_branch == request.target_branch {
        return Err(SwitchError::InvalidState(
            "target branch is already checked out".to_string(),
        ));
    }
    let source_head = state::read_id(runner, "HEAD")?;
    let target_head = state::read_id(runner, &state::branch_ref(&request.target_branch))?;
    let saved_work_reference = state::wip_ref(&source_branch);
    if !request.carry_changes
        && state::optional_id(runner, &saved_work_reference)?.is_some()
    {
        return Err(SwitchError::ExistingSavedWork(source_branch));
    }
    let untracked = preflight::read_untracked(runner)?;
    preflight::ensure_untracked_safe(runner, &request.target_branch, &untracked)?;
    let target_saved_work = read_saved_work(runner, &request.target_branch)?;
    Ok(QuickSwitchPlan {
        source_branch,
        source_head,
        target_branch: request.target_branch,
        target_head,
        saved_work_reference,
        has_tracked_changes: state::read_tracked_changes(runner)?,
        carry_changes: request.carry_changes,
        target_saved_work,
    })
}

pub(crate) fn verify_current(
    runner: &GitRunner,
    plan: &QuickSwitchPlan,
) -> Result<(), SwitchError> {
    state::ensure_no_operation(runner)?;
    if state::read_branch(runner)? != plan.source_branch {
        return Err(SwitchError::StalePlan);
    }
    if state::read_id(runner, "HEAD")? != plan.source_head {
        return Err(SwitchError::StalePlan);
    }
    if state::read_id(runner, &state::branch_ref(&plan.target_branch))? != plan.target_head {
        return Err(SwitchError::StalePlan);
    }
    if !plan.carry_changes
        && state::optional_id(runner, &plan.saved_work_reference)?.is_some()
    {
        return Err(SwitchError::ExistingSavedWork(plan.source_branch.clone()));
    }
    if state::read_tracked_changes(runner)? != plan.has_tracked_changes {
        return Err(SwitchError::StalePlan);
    }
    let untracked = preflight::read_untracked(runner)?;
    preflight::ensure_untracked_safe(runner, &plan.target_branch, &untracked)?;
    Ok(())
}

pub(crate) fn list_saved_work(runner: &GitRunner) -> Result<Vec<SavedWork>, SwitchError> {
    let format = "%(refname)%00%(objectname)";
    let output = runner.run(GitCommand::read(state::args(&[
        "for-each-ref",
        &format!("--format={format}"),
        state::WIP_PREFIX,
    ])))?;
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(parse_saved_work)
        .collect()
}

pub(crate) fn read_saved_work(
    runner: &GitRunner,
    branch: &str,
) -> Result<Option<SavedWork>, SwitchError> {
    let reference = state::wip_ref(branch);
    let Some(snapshot) = state::optional_id(runner, &reference)? else {
        return Ok(None);
    };
    Ok(Some(SavedWork {
        branch: branch.to_string(),
        reference,
        snapshot,
    }))
}

fn parse_saved_work(line: &[u8]) -> Result<SavedWork, SwitchError> {
    let fields = line.split(|byte| *byte == 0).collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err(SwitchError::InvalidState(
            "saved work ref output was malformed".to_string(),
        ));
    }
    let reference = state::text(fields[0])?;
    let branch = reference
        .strip_prefix(state::WIP_PREFIX)
        .ok_or_else(|| SwitchError::InvalidState("invalid saved work ref".to_string()))?;
    let snapshot = ObjectId::new(state::text(fields[1])?).map_err(SwitchError::InvalidState)?;
    Ok(SavedWork {
        branch: branch.to_string(),
        reference,
        snapshot,
    })
}
