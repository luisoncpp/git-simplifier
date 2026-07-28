mod errors;
mod materialize;
mod materialize_steps;
mod model;
mod objects;
mod planner;
mod preflight;

pub use errors::{ApplyError, RewriteError};
pub use model::{
    ApplyResult, CommitMetadata, CommitRewrite, EditMessageRequest, ObjectId, RefName, RepoPath,
    RewriteAction, RewriteOperation, RewritePlan, Signature, TreeEntry, UncommitRequest,
};

use crate::git::GitRunner;

pub(crate) fn plan(
    runner: &GitRunner,
    request: UncommitRequest,
) -> Result<RewritePlan, RewriteError> {
    planner::create(runner, request)
}

pub(crate) fn plan_edit_message(
    runner: &GitRunner,
    request: model::EditMessageRequest,
) -> Result<RewritePlan, RewriteError> {
    planner::create_edit_message(runner, request)
}

pub(crate) fn apply(runner: &GitRunner, plan: &RewritePlan) -> Result<ApplyResult, ApplyError> {
    materialize::apply(runner, plan)
}
