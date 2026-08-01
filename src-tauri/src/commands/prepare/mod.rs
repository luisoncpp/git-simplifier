mod branch;
mod cleanup;
mod history;
mod revert;
mod saved_work;
mod switch;
mod worktree;

use git_helper_core::{ObjectId, SavedWork};

use super::data::{OperationReview, PendingOperation, PrepareOperationRequest};
use super::repository::with_repository;
use super::state::AppState;

pub(super) struct Prepared {
    pub review: OperationReview,
    pub pending: PendingOperation,
}

pub(super) fn prepare(
    state: &AppState,
    id: String,
    request: PrepareOperationRequest,
) -> Result<Prepared, String> {
    match request {
        PrepareOperationRequest::Uncommit(input) => history::uncommit(state, id, input),
        PrepareOperationRequest::Revert(input) => revert::revert(state, id, input),
        PrepareOperationRequest::EditMessage(input) => history::edit_message(state, id, input),
        PrepareOperationRequest::ExcludeSubmodule(input) => history::exclude(state, id, input),
        PrepareOperationRequest::ForcePush => history::force_push(state, id),
        PrepareOperationRequest::SplitBranch(input) => branch::split_branch(state, id, input),
        PrepareOperationRequest::PublishBranch(input) => branch::publish_branch(state, id, input),
        PrepareOperationRequest::QuickSwitch(input) => switch::quick_switch(state, id, input),
        PrepareOperationRequest::ResolveQuickSwitchPull(input) => {
            switch::resolve_pull(state, id, input)
        }
        PrepareOperationRequest::Sync(input) => worktree::sync(state, id, input),
        PrepareOperationRequest::Cleanup(input) => cleanup::cleanup(state, id, input),
        PrepareOperationRequest::ResumeSync => worktree::resume_sync(state, id),
        PrepareOperationRequest::RestoreSavedWork => saved_work::restore(state, id),
        PrepareOperationRequest::DeleteSavedWork(input) => saved_work::delete(state, id, input),
    }
}

pub(super) fn current_branch(state: &AppState) -> Result<String, String> {
    with_repository(state, |repository| {
        let overview = repository.overview().map_err(|error| error.to_string())?;
        overview
            .branch
            .ok_or_else(|| "HEAD is detached; check out a branch first".to_string())
    })
}

pub(super) fn head_of(state: &AppState) -> Result<ObjectId, String> {
    with_repository(state, |repository| {
        Ok(repository
            .overview()
            .map_err(|error| error.to_string())?
            .head)
    })
}

pub(super) fn saved_work(state: &AppState, branch: &str) -> Result<SavedWork, String> {
    let all = with_repository(state, |repository| {
        repository
            .list_saved_work()
            .map_err(|error| error.to_string())
    })?;
    all.into_iter()
        .find(|saved| saved.branch == branch)
        .ok_or_else(|| format!("no Saved work is stored for {branch}"))
}
