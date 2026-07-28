use thiserror::Error;

use crate::git::GitError;

#[derive(Debug, Error)]
pub enum InspectionError {
    #[error("Git inspection failed: {0}")]
    Git(#[from] GitError),
    #[error("Git inspection output was invalid: {0}")]
    Parse(String),
    #[error("invalid Base: {0}")]
    InvalidBase(String),
}
