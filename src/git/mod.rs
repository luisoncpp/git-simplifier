mod command;
mod error;
mod lock;
mod process;
mod version;

pub use command::{AccessMode, GitCommand, GitOutput};
pub use error::GitError;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Clone)]
pub struct RepositoryConfig {
    pub path: PathBuf,
    pub git_executable: PathBuf,
}

pub struct GitRunner {
    repo: PathBuf,
    git: PathBuf,
    git_dir: OnceLock<PathBuf>,
    git_version: String,
    lock: lock::RepoLock,
}

impl GitRunner {
    pub fn open(config: RepositoryConfig) -> Result<Self, GitError> {
        let repo = config.path;
        let git = config.git_executable;
        let lock = lock::lock_for(&repo);
        let output =
            process::execute(&git, &repo, GitCommand::read(process::args(&["--version"])))?;
        version::validate(&output)?;
        let git_version = output_text(&output.stdout, "git version")?;
        Ok(Self {
            repo,
            git,
            git_dir: OnceLock::new(),
            git_version,
            lock,
        })
    }

    pub fn run(&self, command: GitCommand) -> Result<GitOutput, GitError> {
        if command.access == AccessMode::ReadOnly {
            return self.run_unlocked(command);
        }
        self.with_write_lock(|| self.run_unlocked(command))
    }

    pub(crate) fn run_unlocked(&self, command: GitCommand) -> Result<GitOutput, GitError> {
        process::execute(&self.git, &self.repo, command)
    }

    pub(crate) fn run_unlocked_allowing_exit(
        &self,
        command: GitCommand,
        allowed_exits: &[i32],
    ) -> Result<GitOutput, GitError> {
        process::execute_with_allowed_exits(&self.git, &self.repo, command, allowed_exits)
    }

    pub(crate) fn spawn_piped(&self, command: &GitCommand) -> Result<std::process::Child, GitError> {
        process::spawn_piped(&self.git, &self.repo, command)
    }

    pub(crate) fn with_write_lock<T, E>(
        &self,
        action: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E>
    where
        E: From<GitError>,
    {
        lock::with_lock(&self.lock, action)
    }

    pub(crate) fn repo_path(&self) -> &Path {
        &self.repo
    }

    pub(crate) fn git_dir(&self) -> Result<PathBuf, GitError> {
        if let Some(path) = self.git_dir.get() {
            return Ok(path.clone());
        }
        let output = self.run(GitCommand::read(process::args(&["rev-parse", "--git-dir"])))?;
        let path = resolve_git_dir(&self.repo, output_text(&output.stdout, "git-dir")?);
        if self.git_dir.set(path.clone()).is_ok() {
            return Ok(path);
        }
        Ok(self.git_dir.get().cloned().unwrap_or(path))
    }

    pub(crate) fn git_version(&self) -> &str {
        &self.git_version
    }

    pub(crate) fn command_args(values: &[&str]) -> Vec<OsString> {
        process::args(values)
    }
}

fn output_text(bytes: &[u8], name: &str) -> Result<String, GitError> {
    String::from_utf8(bytes.to_vec())
        .map(|value| value.trim().to_string())
        .map_err(|_| GitError::Parse {
            message: format!("{name} contained non-UTF-8 bytes"),
        })
}

fn resolve_git_dir(repo: &Path, raw: String) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        return path;
    }
    repo.join(path)
}
