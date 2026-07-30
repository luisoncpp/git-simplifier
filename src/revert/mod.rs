mod apply;
mod errors;
mod model;
mod paths;
mod plan;

pub use errors::RevertError;
pub use model::{RevertPlan, RevertRequest, RevertResult, RevertTarget};

use crate::git::GitRunner;
use crate::inspection::ChangedPath;
use crate::rewrite::RefName;

pub(crate) fn list_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<ChangedPath>, RevertError> {
    paths::revertible_paths(runner, base)
}

pub(crate) fn create(
    runner: &GitRunner,
    request: RevertRequest,
) -> Result<RevertPlan, RevertError> {
    plan::create(runner, request)
}

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &RevertPlan,
) -> Result<RevertResult, RevertError> {
    apply::apply(runner, plan)
}
