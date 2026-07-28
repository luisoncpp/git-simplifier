mod apply;
mod errors;
mod model;
mod plan;

pub use errors::ForcePushError;
pub use model::{ForcePushPlan, ForcePushResult};

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
