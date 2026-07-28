use std::io;

use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum ExclusionError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("excluded submodule I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("excluded submodule configuration is invalid: {0}")]
    InvalidState(String),
    #[error("excluded submodule plan is stale")]
    StalePlan,
    #[error("excluded submodule recording failed: {0}")]
    Recording(String),
}
