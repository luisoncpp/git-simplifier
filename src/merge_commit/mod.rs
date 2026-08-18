mod apply;
mod errors;
mod model;
mod paths;
mod plan;
mod preflight;
mod record;
mod state;
mod tree;

pub use errors::CommitMergeError;
pub use model::{CommitMergePlan, CommitMergeResult};

use crate::git::GitRunner;

pub fn merge_in_progress(runner: &GitRunner) -> Result<bool, errors::CommitMergeError> {
    state::merge_in_progress(runner)
}

pub fn create(runner: &GitRunner) -> Result<CommitMergePlan, errors::CommitMergeError> {
    plan::create(runner)
}

pub fn apply(
    runner: &GitRunner,
    plan: &CommitMergePlan,
) -> Result<CommitMergeResult, errors::CommitMergeError> {
    apply::apply(runner, plan)
}
