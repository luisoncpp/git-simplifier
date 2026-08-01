use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::errors::CleanupError;

pub(super) const HEADS_PREFIX: &str = "refs/heads/";
pub(super) const REMOTES_PREFIX: &str = "refs/remotes/";
pub(super) const WIP_PREFIX: &str = "refs/githelper/wip/";

/// Well-known shared names. These are offered but never pre-ticked, so a bulk
/// apply cannot take one without a deliberate click.
pub(super) const PROTECTED: [&str; 5] = ["main", "master", "develop", "dev", "trunk"];

pub(super) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

pub(super) fn text(bytes: &[u8]) -> Result<String, CleanupError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| CleanupError::Parse("Git output is not UTF-8".to_string()))
}

pub(super) fn ensure_remote_base(base: &RefName) -> Result<(), CleanupError> {
    if !base.as_str().starts_with(REMOTES_PREFIX) {
        return Err(CleanupError::InvalidBase(base.to_string()));
    }
    Ok(())
}

pub(super) fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, CleanupError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(CleanupError::InvalidState)
}

/// The identity comes from the full config cascade, **not** `--local`: every
/// other config read in this codebase is repo-scoped, but `user.email` normally
/// lives in the global file, so a `--local` copy would match nothing.
pub(super) fn identity(runner: &GitRunner) -> Result<Option<String>, CleanupError> {
    let Ok(output) = runner.run(GitCommand::read(args(&["config", "--get", "user.email"]))) else {
        return Ok(None);
    };
    let email = text(&output.stdout)?.trim().to_string();
    if email.is_empty() {
        return Ok(None);
    }
    Ok(Some(email))
}

pub(super) fn ensure_no_operation(runner: &GitRunner) -> Result<(), CleanupError> {
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
    ] {
        if git_path(runner, marker)?.exists() {
            return Err(CleanupError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, CleanupError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--git-path", marker])))?;
    let path = PathBuf::from(text(&output.stdout)?.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        runner.repo_path().join(path)
    })
}

/// `refs/remotes/origin/main` -> `main`, used to protect the local branch Base
/// tracks. This one *does* fall back to a first-slash split when no configured
/// remote matches, because the two ways of being wrong are not symmetric: an
/// over-broad name merely protects an extra branch, while failing to resolve it
/// would offer to delete the mainline.
pub(super) fn base_branch_name(base: &RefName, remotes: &[String]) -> Option<String> {
    let short = base.as_str().strip_prefix(REMOTES_PREFIX)?;
    if let Some((_, branch)) = split_remote(short, remotes) {
        return Some(branch);
    }
    let (_, branch) = short.split_once('/')?;
    if branch.is_empty() {
        return None;
    }
    Some(branch.to_string())
}

/// Splits `origin/feature` into its remote and branch halves, preferring the
/// longest configured remote name so a remote whose own name contains a slash
/// still resolves. Strict on purpose: callers that turn the result back into a
/// server ref must skip a branch they cannot name rather than guess at one.
pub(super) fn split_remote(short: &str, remotes: &[String]) -> Option<(String, String)> {
    let mut best: Option<(String, String)> = None;
    for remote in remotes {
        let Some(branch) = short.strip_prefix(&format!("{remote}/")) else {
            continue;
        };
        if branch.is_empty() {
            continue;
        }
        let longer = best
            .as_ref()
            .map(|(current, _)| remote.len() > current.len())
            .unwrap_or(true);
        if longer {
            best = Some((remote.clone(), branch.to_string()));
        }
    }
    best
}

pub(super) fn is_protected(branch: &str) -> bool {
    PROTECTED
        .iter()
        .any(|name| branch.eq_ignore_ascii_case(name))
}
