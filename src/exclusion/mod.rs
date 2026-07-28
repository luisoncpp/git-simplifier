mod apply;
mod errors;
mod hook;
mod model;
mod plan;

pub use errors::ExclusionError;
pub use model::{ExcludeSubmodulePlan, ExcludeSubmoduleRequest, ExcludeSubmoduleResult};

use crate::git::GitRunner;

pub(crate) fn create(
    runner: &GitRunner,
    request: ExcludeSubmoduleRequest,
) -> Result<ExcludeSubmodulePlan, ExclusionError> {
    plan::create(runner, request)
}

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &ExcludeSubmodulePlan,
) -> Result<ExcludeSubmoduleResult, ExclusionError> {
    apply::apply(runner, plan)
}
