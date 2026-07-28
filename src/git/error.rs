use std::ffi::OsString;
use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("failed to start git: {source}")]
    Spawn {
        #[source]
        source: io::Error,
    },
    #[error("git command failed with exit code {exit_code:?}")]
    Command {
        args: Vec<OsString>,
        exit_code: Option<i32>,
        stderr: Vec<u8>,
    },
    #[error("git version is below the supported minimum: {raw:?}")]
    UnsupportedVersion { raw: Vec<u8> },
    #[error("git output could not be parsed: {message}")]
    Parse { message: String },
    #[error("repository write lock was poisoned")]
    LockPoisoned,
}

impl GitError {
    pub fn raw_stderr(&self) -> Option<&[u8]> {
        match self {
            Self::Command { stderr, .. } => Some(stderr),
            _ => None,
        }
    }
}
