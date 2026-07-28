use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};

use super::errors::RewriteError;
use super::model::{ObjectId, RefName};

pub(crate) struct RepoState {
    pub branch: RefName,
    pub head: ObjectId,
    pub base: ObjectId,
    pub commits: Vec<ObjectId>,
}

pub(crate) fn inspect(runner: &GitRunner, base: &RefName) -> Result<RepoState, RewriteError> {
    ensure_remote_ref(base)?;
    ensure_no_operation(runner)?;
    let branch = read_branch(runner)?;
    let head = read_id(runner, vec!["rev-parse", "--verify", "HEAD^{commit}"])?;
    let base_id = read_id(
        runner,
        vec!["rev-parse", "--verify", &format!("{}^{{commit}}", base)],
    )?;
    ensure_shared_base(runner, &head, &base_id)?;
    let commits = read_range(runner, &head, &base_id)?;
    if commits.is_empty() {
        return Err(RewriteError::InvalidState(
            "Editable range is empty".to_string(),
        ));
    }
    Ok(RepoState {
        branch,
        head,
        base: base_id,
        commits,
    })
}

pub(crate) fn resolve_commit(
    runner: &GitRunner,
    commit: &ObjectId,
) -> Result<ObjectId, RewriteError> {
    let value = format!("{}^{{commit}}", commit);
    read_id(runner, vec!["rev-parse", "--verify", &value])
}

fn ensure_remote_ref(base: &RefName) -> Result<(), RewriteError> {
    if base.as_str().starts_with("refs/remotes/") {
        return Ok(());
    }
    Err(RewriteError::InvalidState(
        "Base must be a remote-tracking ref".to_string(),
    ))
}

fn read_branch(runner: &GitRunner) -> Result<RefName, RewriteError> {
    let output = runner
        .run(GitCommand::read(args(&["symbolic-ref", "--quiet", "HEAD"])))
        .map_err(|_| RewriteError::InvalidState("HEAD is detached".to_string()))?;
    let branch = text(&output.stdout)?;
    RefName::new(branch.trim().to_string()).map_err(RewriteError::InvalidState)
}

fn read_id(runner: &GitRunner, values: Vec<&str>) -> Result<ObjectId, RewriteError> {
    let output = runner.run(GitCommand::read(args(&values)))?;
    let line = output
        .stdout
        .split(|byte| *byte == b'\n')
        .next()
        .unwrap_or_default();
    ObjectId::from_bytes(trim_line(line)).map_err(RewriteError::Parse)
}

fn ensure_shared_base(
    runner: &GitRunner,
    head: &ObjectId,
    base: &ObjectId,
) -> Result<(), RewriteError> {
    let values = vec!["merge-base", base.as_str(), head.as_str()];
    if runner.run(GitCommand::read(args(&values))).is_ok() {
        return Ok(());
    }
    Err(RewriteError::InvalidState(
        "HEAD and Base have no merge base".to_string(),
    ))
}

fn read_range(
    runner: &GitRunner,
    head: &ObjectId,
    base: &ObjectId,
) -> Result<Vec<ObjectId>, RewriteError> {
    let values = vec![
        "rev-list",
        "--first-parent",
        "--reverse",
        head.as_str(),
        "--not",
        base.as_str(),
    ];
    let output = runner.run(GitCommand::read(args(&values)))?;
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| ObjectId::from_bytes(trim_line(line)).map_err(RewriteError::Parse))
        .collect()
}

fn ensure_no_operation(runner: &GitRunner) -> Result<(), RewriteError> {
    let markers = [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
    ];
    for marker in markers {
        let path = git_path(runner, marker)?;
        if path.exists() {
            return Err(RewriteError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, RewriteError> {
    let values = vec!["rev-parse", "--git-path", marker];
    let output = runner.run(GitCommand::read(args(&values)))?;
    let value = text(&output.stdout)?;
    let path = PathBuf::from(value.trim());
    if path.is_absolute() {
        return Ok(path);
    }
    Ok(runner.repo_path().join(path))
}

fn text(bytes: &[u8]) -> Result<String, RewriteError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| RewriteError::Parse("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn trim_line(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .rposition(|byte| !matches!(byte, b' ' | b'\r' | b'\n'));
    end.map(|index| &bytes[..=index]).unwrap_or_default()
}
