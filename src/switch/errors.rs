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
    #[error(
        "Saved work was applied with conflicts. Resolve the conflict markers, then delete Saved \
         work when the result is correct; the backup was kept."
    )]
    SavedWorkConflict,
    #[error("untracked files would be overwritten on the target branch: {0}")]
    UntrackedConflict(String),
    #[error("quick branch switch recording failed: {0}")]
    Recording(String),
}
