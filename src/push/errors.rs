use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum ForcePushError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("current branch is detached")]
    DetachedHead,
    #[error("current branch has no upstream configured")]
    NoUpstream,
    #[error("a local upstream cannot be force-pushed")]
    LocalUpstream,
    #[error("upstream branch configuration is invalid: {0}")]
    InvalidState(String),
    #[error("force-push plan is stale")]
    StalePlan,
    #[error("force-push recording failed: {0}")]
    Recording(String),
}
