use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum RevertError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("revert is invalid: {0}")]
    InvalidState(String),
    #[error("revert plan is stale")]
    StalePlan,
    #[error("revert recording failed: {0}")]
    Recording(String),
}
