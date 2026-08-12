use std::ffi::OsString;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum GitError {
    Spawn {
        source: io::Error,
    },
    Io {
        source: io::Error,
    },
    Command {
        args: Vec<OsString>,
        exit_code: Option<i32>,
        stderr: Vec<u8>,
    },
    UnsupportedVersion {
        raw: Vec<u8>,
    },
    Parse {
        message: String,
    },
    LockPoisoned,
}

impl fmt::Display for GitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { source } => write!(formatter, "failed to start git: {source}"),
            Self::Io { source } => write!(formatter, "git I/O failed: {source}"),
            Self::Command {
                args,
                exit_code,
                stderr,
            } => {
                write!(
                    formatter,
                    "git {} failed with exit code {exit_code:?}",
                    command_summary(args)
                )?;
                let detail = String::from_utf8_lossy(stderr);
                let detail = detail.trim();
                if !detail.is_empty() {
                    write!(formatter, ": {detail}")?;
                }
                Ok(())
            }
            Self::UnsupportedVersion { raw } => write!(
                formatter,
                "git version is below the supported minimum: {raw:?}"
            ),
            Self::Parse { message } => {
                write!(formatter, "git output could not be parsed: {message}")
            }
            Self::LockPoisoned => formatter.write_str("repository write lock was poisoned"),
        }
    }
}

impl std::error::Error for GitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source } | Self::Io { source } => Some(source),
            _ => None,
        }
    }
}

impl GitError {
    pub fn raw_stderr(&self) -> Option<&[u8]> {
        match self {
            Self::Command { stderr, .. } => Some(stderr),
            _ => None,
        }
    }
}

fn command_summary(args: &[OsString]) -> String {
    args.iter()
        .take(3)
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ")
}
