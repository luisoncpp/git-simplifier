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

#[derive(Debug, Error)]
pub enum PublishError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("branch does not exist: {0}")]
    MissingBranch(String),
    #[error("no remote is configured to publish to")]
    NoRemote,
    #[error("a local remote cannot be published to")]
    LocalRemote,
    #[error("{0} already exists on the remote; this publishes a new branch only")]
    ExistingRemoteBranch(String),
    #[error("publish state is invalid: {0}")]
    InvalidState(String),
    #[error("publish plan is stale")]
    StalePlan,
    #[error("publish recording failed: {0}")]
    Recording(String),
}
