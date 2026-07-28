use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("sync state is invalid: {0}")]
    InvalidState(String),
    #[error("untracked files would be overwritten by the base merge: {0}")]
    UntrackedConflict(String),
    #[error("base merge is unresolved: {source}")]
    BaseMergeConflict {
        #[source]
        source: GitError,
    },
    #[error("Saved work could not be reapplied: {source}")]
    WipReapplyConflict {
        #[source]
        source: GitError,
    },
    #[error("sync recording failed: {0}")]
    Recording(String),
}
