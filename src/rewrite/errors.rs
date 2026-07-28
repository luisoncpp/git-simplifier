use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum RewriteError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("rewrite preflight failed: {0}")]
    InvalidState(String),
    #[error("rewrite input could not be parsed: {0}")]
    Parse(String),
}

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("rewrite plan is stale")]
    StalePlan,
    #[error("rewrite recording failed: {0}")]
    Recording(String),
    #[error("rewrite plan could not be applied: {0}")]
    InvalidPlan(String),
}
