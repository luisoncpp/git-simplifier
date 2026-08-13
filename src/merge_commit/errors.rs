use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum CommitMergeError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("commit merge is invalid: {0}")]
    InvalidState(String),
    #[error("commit merge plan is stale")]
    StalePlan,
    #[error("commit merge recording failed: {0}")]
    Recording(String),
}
