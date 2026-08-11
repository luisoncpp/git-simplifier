mod apply;
mod errors;
mod model;
mod plan;

pub use errors::SubmoduleCleanupError;
pub use model::{
    SubmoduleCleanupPlan, SubmoduleCleanupRequest, SubmoduleCleanupResult,
};

use crate::git::GitRunner;

pub(crate) fn create(
    runner: &GitRunner,
    request: SubmoduleCleanupRequest,
) -> Result<SubmoduleCleanupPlan, SubmoduleCleanupError> {
    plan::create(runner, request)
}

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &SubmoduleCleanupPlan,
) -> Result<SubmoduleCleanupResult, SubmoduleCleanupError> {
    apply::apply(runner, plan)
}
