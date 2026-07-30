mod diff;
mod errors;
mod model;
mod patch;
mod queries;

pub use errors::InspectionError;
pub use model::{
    ChangedPath, DiffCompare, DiffHunk, DiffLine, DiffLineKind, EditableCommit, FileDiff,
    FileDiffStatus, LocalBranchChoice, RemoteBaseChoice, RepositoryOverview, SubmoduleChoice,
    WorktreeSummary,
};

use crate::git::GitRunner;
use crate::rewrite::{RefName, RepoPath};

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
pub(crate) fn branch_diff(
    runner: &GitRunner,
    base: &RefName,
    compare: model::DiffCompare,
) -> Result<String, InspectionError> {
    diff::branch_diff(runner, base, compare)
}
pub(crate) fn files_diff(
    runner: &GitRunner,
    base: &RefName,
    compare: model::DiffCompare,
) -> Result<Vec<FileDiff>, InspectionError> {
    diff::files_diff(runner, base, compare)
}
pub(crate) fn full_file_diff(
    runner: &GitRunner,
    base: &RefName,
    path: &RepoPath,
    compare: model::DiffCompare,
) -> Result<Option<FileDiff>, InspectionError> {
    diff::full_file_diff(runner, base, path, compare)
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
