use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::errors::CommitMergeError;

pub(crate) fn merge_in_progress(runner: &GitRunner) -> Result<bool, CommitMergeError> {
    Ok(git_path(runner, "MERGE_HEAD")?.exists())
}

pub(crate) fn read_branch(runner: &GitRunner) -> Result<RefName, CommitMergeError> {
    let output = runner
        .run(GitCommand::read(args(&["symbolic-ref", "--quiet", "HEAD"])))
        .map_err(|_| CommitMergeError::InvalidState("HEAD is detached".to_string()))?;
    RefName::new(text(&output.stdout)?.trim().to_string())
        .map_err(CommitMergeError::InvalidState)
}

pub(crate) fn read_tree_id(runner: &GitRunner, commit: &ObjectId) -> Result<ObjectId, CommitMergeError> {
    let spec = format!("{commit}^{{tree}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    parse_id(&output.stdout)
}

pub(crate) fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, CommitMergeError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    parse_id(&output.stdout)
}

pub(crate) fn optional_id(runner: &GitRunner, name: &str) -> Result<Option<ObjectId>, CommitMergeError> {
    let spec = format!("{name}^{{commit}}");
    let Ok(output) = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec]))) else {
        return Ok(None);
    };
    Ok(Some(parse_id(&output.stdout)?))
}

pub(crate) fn optional_base(runner: &GitRunner) -> Result<Option<RefName>, CommitMergeError> {
    let output = runner.run(GitCommand::read(args(&[
        "config",
        "--local",
        "--get",
        "githelper.base",
    ])));
    match output {
        Ok(value) => {
            let text = text(&value.stdout)?.trim().to_string();
            if text.is_empty() {
                Ok(None)
            } else {
                Ok(Some(RefName::new(text).map_err(CommitMergeError::InvalidState)?))
            }
        }
        Err(_) => Ok(None),
    }
}

pub(crate) fn merge_base(
    runner: &GitRunner,
    ours: &ObjectId,
    theirs: &ObjectId,
) -> Result<ObjectId, CommitMergeError> {
    let output = runner.run(GitCommand::read(args(&[
        "merge-base",
        ours.as_str(),
        theirs.as_str(),
    ])))?;
    parse_id(&output.stdout).map_err(|_| {
        CommitMergeError::InvalidState("HEAD and MERGE_HEAD have no merge base".to_string())
    })
}

pub(crate) fn has_unmerged_entries(runner: &GitRunner) -> Result<bool, CommitMergeError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=no",
    ])))?;
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .any(|record| record.starts_with(b"u ")))
}

pub(crate) fn refuse_other_operations(runner: &GitRunner) -> Result<(), CommitMergeError> {
    for marker in ["CHERRY_PICK_HEAD", "BISECT_LOG", "rebase-merge", "rebase-apply"] {
        if git_path(runner, marker)?.exists() {
            return Err(CommitMergeError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn temp_index_path(runner: &GitRunner) -> Result<PathBuf, CommitMergeError> {
    let dir = runner.git_dir()?.join("githelper");
    std::fs::create_dir_all(&dir).map_err(|error| CommitMergeError::InvalidState(error.to_string()))?;
    Ok(dir.join(format!("merge-index-{}", std::process::id())))
}

pub(crate) fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, CommitMergeError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--git-path", marker])))?;
    let path = PathBuf::from(text(&output.stdout)?.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        runner.repo_path().join(path)
    })
}

pub(crate) fn index_command(index: &Path, cmd: &[&str]) -> GitCommand {
    GitCommand::write(args(cmd))
        .with_environment(OsString::from("GIT_INDEX_FILE"), index.as_os_str().to_os_string())
}

pub(crate) fn text(bytes: &[u8]) -> Result<String, CommitMergeError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| CommitMergeError::InvalidState("Git output is not UTF-8".to_string()))
}

pub(crate) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn parse_id(bytes: &[u8]) -> Result<ObjectId, CommitMergeError> {
    let line = bytes.split(|byte| *byte == b'\n').next().unwrap_or(bytes);
    ObjectId::from_bytes(trim_line(line)).map_err(CommitMergeError::InvalidState)
}

fn trim_line(line: &[u8]) -> &[u8] {
    line.trim_ascii_end()
}
