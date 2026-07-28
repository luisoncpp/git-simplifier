mod apply;
mod errors;
mod model;
mod plan;
mod publish;

pub use errors::{ForcePushError, PublishError};
pub use model::{ForcePushPlan, ForcePushResult, PublishBranchPlan, PublishBranchResult};

use crate::git::GitRunner;

pub(crate) fn create(runner: &GitRunner) -> Result<ForcePushPlan, ForcePushError> {
    plan::create(runner)
}

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &ForcePushPlan,
) -> Result<ForcePushResult, ForcePushError> {
    apply::apply(runner, plan)
}

pub(crate) fn create_publish(
    runner: &GitRunner,
    branch: String,
) -> Result<PublishBranchPlan, PublishError> {
    publish::create(runner, branch)
}

pub(crate) fn apply_publish(
    runner: &GitRunner,
    plan: &PublishBranchPlan,
) -> Result<PublishBranchResult, PublishError> {
    apply::publish(runner, plan)
}
