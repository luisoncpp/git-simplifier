use thiserror::Error;

use crate::git::GitError;
use crate::rewrite::RewriteError;
use crate::revert::RevertError;

#[derive(Debug, Error)]
pub enum SubmoduleCleanupError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Rewrite(#[from] RewriteError),
    #[error(transparent)]
    Apply(#[from] crate::rewrite::ApplyError),
    #[error(transparent)]
    Revert(#[from] RevertError),
    #[error("submodule cleanup is invalid: {0}")]
    InvalidState(String),
    #[error("submodule cleanup plan is stale")]
    StalePlan,
    #[error("submodule cleanup recording failed: {0}")]
    Recording(String),
}
