use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum SplitError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("split branch I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("split branch state is invalid: {0}")]
    InvalidState(String),
    #[error("split branch plan is stale")]
    StalePlan,
    #[error("branch already exists: {0}")]
    ExistingBranch(String),
    #[error("no paths were selected for the split branch")]
    EmptySelection,
    #[error("the selected paths carry no changes over the Base")]
    NoChanges,
    #[error("split branch recording failed: {0}")]
    Recording(String),
}
