use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum CleanupError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("cleanup state is invalid: {0}")]
    InvalidState(String),
    #[error("Base must be a remote-tracking ref: {0}")]
    InvalidBase(String),
    #[error("Git has no user.email, so branches cannot be matched to you")]
    NoIdentity,
    #[error("{0} is not eligible for cleanup")]
    NotEligible(String),
    #[error("no branches were selected for cleanup")]
    EmptySelection,
    #[error("cleanup plan is stale")]
    StalePlan,
    #[error("{remote} refused the branch deletion: {stderr}")]
    RemoteRejected { remote: String, stderr: String },
    #[error("cleanup output was malformed: {0}")]
    Parse(String),
    #[error("cleanup recording failed: {0}")]
    Recording(String),
}
