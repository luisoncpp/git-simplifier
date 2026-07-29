use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum SwitchError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("quick branch switch state is invalid: {0}")]
    InvalidState(String),
    #[error("quick branch switch plan is stale")]
    StalePlan,
    #[error("saved work does not exist for branch: {0}")]
    MissingSavedWork(String),
    #[error("saved work for branch already exists: {0}")]
    ExistingSavedWork(String),
    #[error("untracked files would be overwritten on the target branch: {0}")]
    UntrackedConflict(String),
    #[error(
        "carried changes would conflict on the target branch because these files differ between the branches: {0}"
    )]
    CarryConflict(String),
    #[error("{0}")]
    CarryReapplyFailed(String),
    #[error("quick branch switch recording failed: {0}")]
    Recording(String),
}
