mod diff;
mod errors;
mod model;
mod queries;

pub use errors::InspectionError;
pub use model::{
    ChangedPath, EditableCommit, LocalBranchChoice, RemoteBaseChoice, RepositoryOverview,
    SubmoduleChoice, WorktreeSummary,
};

use crate::git::GitRunner;
use crate::rewrite::RefName;

pub(crate) fn overview(runner: &GitRunner) -> Result<RepositoryOverview, InspectionError> {
    queries::overview(runner)
}
pub(crate) fn base_choices(runner: &GitRunner) -> Result<Vec<RemoteBaseChoice>, InspectionError> {
    queries::base_choices(runner)
}
pub(crate) fn changed_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<ChangedPath>, InspectionError> {
    queries::changed_paths(runner, base)
}
pub(crate) fn branch_diff(runner: &GitRunner, base: &RefName) -> Result<String, InspectionError> {
    diff::branch_diff(runner, base)
}
pub(crate) fn editable_commits(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<EditableCommit>, InspectionError> {
    queries::editable_commits(runner, base)
}
pub(crate) fn local_branches(
    runner: &GitRunner,
) -> Result<Vec<LocalBranchChoice>, InspectionError> {
    queries::local_branches(runner)
}
pub(crate) fn submodules(runner: &GitRunner) -> Result<Vec<SubmoduleChoice>, InspectionError> {
    queries::submodules(runner)
}
pub(crate) fn set_base(runner: &GitRunner, base: RefName) -> Result<(), InspectionError> {
    queries::set_base(runner, base)
}
