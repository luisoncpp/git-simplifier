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
    let (target_head, create_from_remote) =
        resolve_target(runner, &request.target_branch, request.create_from_remote.as_deref())?;
    let saved_work_reference = state::wip_ref(&source_branch);
    if !request.carry_changes
        && state::optional_id(runner, &saved_work_reference)?.is_some()
    {
        return Err(SwitchError::ExistingSavedWork(source_branch));
    }
    let untracked = preflight::read_untracked(runner)?;
    let tree_ref = create_from_remote
        .clone()
        .unwrap_or_else(|| state::branch_ref(&request.target_branch));
    let untracked_conflicts = resolve_untracked_conflicts(
        runner,
        UntrackedInput {
            tree_ref: &tree_ref,
            paths: &untracked,
            merge: request.merge_untracked,
        },
    )?;
    let pull_remote_ref = if request.pull_after_switch {
        resolve_pull_remote(runner, &request.target_branch, create_from_remote.as_deref())?
    } else {
        None
    };
    let target_saved_work = read_saved_work(runner, &request.target_branch)?;
    Ok(QuickSwitchPlan {
        source_branch,
        source_head,
        target_branch: request.target_branch,
        target_head,
        saved_work_reference,
        has_tracked_changes: state::read_tracked_changes(runner)?,
        carry_changes: request.carry_changes,
        pull_after_switch: request.pull_after_switch,
        create_from_remote,
        pull_remote_ref,
        target_saved_work,
        untracked_conflicts,
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
    let expected = match &plan.create_from_remote {
        Some(remote) => state::optional_id(runner, remote)?,
        None => Some(state::read_id(runner, &state::branch_ref(&plan.target_branch))?),
    };
    if expected != Some(plan.target_head.clone()) {
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
    let tree_ref = plan
        .create_from_remote
        .clone()
        .unwrap_or_else(|| state::branch_ref(&plan.target_branch));
    let merge_untracked = !plan.untracked_conflicts.is_empty();
    let conflicts = resolve_untracked_conflicts(
        runner,
        UntrackedInput {
            tree_ref: &tree_ref,
            paths: &untracked,
            merge: merge_untracked,
        },
    )?;
    if conflicts != plan.untracked_conflicts {
        return Err(SwitchError::StalePlan);
    }
    Ok(())
}

struct UntrackedInput<'a> {
    tree_ref: &'a str,
    paths: &'a [String],
    merge: bool,
}

fn resolve_untracked_conflicts(
    runner: &GitRunner,
    input: UntrackedInput<'_>,
) -> Result<Vec<String>, SwitchError> {
    let (mergeable, hard) = preflight::classify(runner, input.tree_ref, input.paths)?;
    if !hard.is_empty() {
        return Err(SwitchError::UntrackedConflict(hard.join(", ")));
    }
    if !mergeable.is_empty() && !input.merge {
        return Err(SwitchError::UntrackedOverlap(mergeable));
    }
    Ok(mergeable)
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

fn resolve_target(
    runner: &GitRunner,
    target_branch: &str,
    create_from_remote: Option<&str>,
) -> Result<(ObjectId, Option<String>), SwitchError> {
    let Some(remote) = create_from_remote else {
        let head = state::read_id(runner, &state::branch_ref(target_branch))?;
        return Ok((head, None));
    };
    if state::optional_id(runner, &state::branch_ref(target_branch))?.is_some() {
        return Err(SwitchError::InvalidState(format!(
            "local branch {target_branch} already exists"
        )));
    }
    let remote_ref = state::remote_tracking_ref(remote)?;
    let head = state::read_id(runner, &remote_ref)?;
    Ok((head, Some(remote_ref)))
}

fn resolve_pull_remote(
    runner: &GitRunner,
    target_branch: &str,
    create_from_remote: Option<&str>,
) -> Result<Option<String>, SwitchError> {
    if let Some(remote) = create_from_remote {
        return Ok(Some(remote.to_string()));
    }
    state::same_named_remote(runner, target_branch)
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
