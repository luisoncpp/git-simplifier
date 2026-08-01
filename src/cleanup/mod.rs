mod apply;
mod discover;
mod eligibility;
mod errors;
mod model;
mod plan;
mod record;
mod refs;
mod remote;
mod review;
mod state;

pub use errors::CleanupError;
pub use model::{
    CleanupBranchPlan, CleanupChoice, CleanupDiscovery, CleanupExclusion, CleanupKind, CleanupPlan,
    CleanupRequest, CleanupResult, ExclusionReason, KeptReason, KeptRemote, LocalDeletion,
    RemoteCounterpart, RemoteDeletion,
};

use crate::git::GitRunner;
use crate::rewrite::RefName;

pub(crate) fn discover_branches(
    runner: &GitRunner,
    base: &RefName,
) -> Result<CleanupDiscovery, CleanupError> {
    discover::eligible(runner, base)
}

pub(crate) fn create_plan(
    runner: &GitRunner,
    request: CleanupRequest,
) -> Result<CleanupPlan, CleanupError> {
    plan::create(runner, request)
}

pub(crate) fn apply_plan(
    runner: &GitRunner,
    plan: &CleanupPlan,
) -> Result<CleanupResult, CleanupError> {
    apply::cleanup(runner, plan)
}
