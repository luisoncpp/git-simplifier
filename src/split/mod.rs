mod apply;
mod errors;
mod model;
mod paths;
mod plan;
mod record;
mod review;
mod state;
mod worktree;

pub use errors::SplitError;
pub use model::{SplitBranchPlan, SplitBranchRequest, SplitBranchResult};

use crate::git::GitRunner;

pub(crate) fn create_plan(
    runner: &GitRunner,
    request: SplitBranchRequest,
) -> Result<SplitBranchPlan, SplitError> {
    plan::create(runner, request)
}

pub(crate) fn apply_plan(
    runner: &GitRunner,
    plan: &SplitBranchPlan,
) -> Result<SplitBranchResult, SplitError> {
    apply::split(runner, plan)
}
