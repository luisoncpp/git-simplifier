use crate::git::{GitCommand, GitError, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::history_model::{HistorySwitchPlan, HistorySwitchRequest};
use super::{plan, preflight, state};

pub(crate) fn create(
    runner: &GitRunner,
    request: HistorySwitchRequest,
) -> Result<HistorySwitchPlan, SwitchError> {
    state::ensure_no_operation(runner)?;
    let source_branch = state::read_branch(runner)?;
    let source_head = state::read_id(runner, "HEAD")?;
    let target_commit = resolve_target(runner, &request, &source_head)?;
    let saved_work_reference = state::wip_ref(&source_branch);
    refuse_existing_saved_work(runner, &request, &source_branch, &saved_work_reference)?;
    let untracked = preflight::read_untracked(runner)?;
    let untracked_conflicts = plan::resolve_untracked_conflicts(
        runner,
        plan::UntrackedInput {
            tree_ref: target_commit.as_str(),
            paths: &untracked,
            merge: request.merge_untracked,
        },
    )?;
    Ok(HistorySwitchPlan {
        source_branch,
        source_head,
        target_commit,
        saved_work_reference,
        has_tracked_changes: state::read_tracked_changes(runner)?,
        carry_changes: request.carry_changes,
        untracked_conflicts,
    })
}

pub(crate) fn verify(runner: &GitRunner, plan: &HistorySwitchPlan) -> Result<(), SwitchError> {
    state::ensure_no_operation(runner)?;
    if state::read_branch(runner)? != plan.source_branch {
        return Err(SwitchError::StalePlan);
    }
    if state::read_id(runner, "HEAD")? != plan.source_head {
        return Err(SwitchError::StalePlan);
    }
    if state::read_tracked_changes(runner)? != plan.has_tracked_changes {
        return Err(SwitchError::StalePlan);
    }
    verify_untracked(runner, plan)
}

fn refuse_existing_saved_work(
    runner: &GitRunner,
    request: &HistorySwitchRequest,
    source_branch: &str,
    saved_work_reference: &str,
) -> Result<(), SwitchError> {
    if request.carry_changes {
        return Ok(());
    }
    if state::optional_id(runner, saved_work_reference)?.is_some() {
        return Err(SwitchError::ExistingSavedWork(source_branch.to_string()));
    }
    Ok(())
}

fn verify_untracked(runner: &GitRunner, history: &HistorySwitchPlan) -> Result<(), SwitchError> {
    if !history.carry_changes
        && state::optional_id(runner, &history.saved_work_reference)?.is_some()
    {
        return Err(SwitchError::ExistingSavedWork(history.source_branch.clone()));
    }
    let untracked = preflight::read_untracked(runner)?;
    let merge = !history.untracked_conflicts.is_empty();
    let conflicts = plan::resolve_untracked_conflicts(
        runner,
        plan::UntrackedInput {
            tree_ref: history.target_commit.as_str(),
            paths: &untracked,
            merge,
        },
    )?;
    if conflicts != history.untracked_conflicts {
        return Err(SwitchError::StalePlan);
    }
    Ok(())
}

fn resolve_target(
    runner: &GitRunner,
    request: &HistorySwitchRequest,
    source_head: &ObjectId,
) -> Result<ObjectId, SwitchError> {
    match (request.commit.as_deref(), request.until.as_deref()) {
        (Some(commit), None) if !commit.trim().is_empty() => resolve_commit(runner, commit),
        (None, Some(until)) if !until.trim().is_empty() => resolve_until(runner, until),
        _ => Err(SwitchError::InvalidState(
            "choose a commit or a date and time".to_string(),
        )),
    }
    .and_then(|target| refuse_current_head(target, source_head))
}

fn refuse_current_head(target: ObjectId, source_head: &ObjectId) -> Result<ObjectId, SwitchError> {
    if &target == source_head {
        return Err(SwitchError::InvalidState(
            "already at this commit".to_string(),
        ));
    }
    Ok(target)
}

fn resolve_commit(runner: &GitRunner, commit: &str) -> Result<ObjectId, SwitchError> {
    let id = state::read_id(runner, commit)?;
    if !is_ancestor(runner, id.as_str())? {
        return Err(SwitchError::InvalidState(
            "commit is not on the current branch".to_string(),
        ));
    }
    Ok(id)
}

fn resolve_until(runner: &GitRunner, until: &str) -> Result<ObjectId, SwitchError> {
    let spec = format!("--until={until}");
    let output = runner.run(GitCommand::read(state::args(&[
        "rev-list",
        "-1",
        "--first-parent",
        &spec,
        "HEAD",
    ])))?;
    let value = state::text(&output.stdout)?.trim().to_string();
    if value.is_empty() {
        return Err(SwitchError::InvalidState(
            "No commit on this branch at or before that time.".to_string(),
        ));
    }
    ObjectId::new(value).map_err(SwitchError::InvalidState)
}

fn is_ancestor(runner: &GitRunner, commit: &str) -> Result<bool, SwitchError> {
    match runner.run(GitCommand::read(state::args(&[
        "merge-base",
        "--is-ancestor",
        commit,
        "HEAD",
    ]))) {
        Ok(_) => Ok(true),
        Err(GitError::Command {
            exit_code: Some(1), ..
        }) => Ok(false),
        Err(error) => Err(error.into()),
    }
}
