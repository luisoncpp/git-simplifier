use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RepoPath};

use super::errors::RevertError;
use super::model::{RevertPlan, RevertRequest, RevertTarget};
use super::paths::{literal, revertible_paths};

pub(super) fn create(runner: &GitRunner, request: RevertRequest) -> Result<RevertPlan, RevertError> {
    if request.paths.is_empty() {
        return Err(RevertError::InvalidState(
            "select at least one path to revert".to_string(),
        ));
    }
    ensure_no_operation(runner)?;
    let source_head = read_id(runner, "HEAD")?;
    let eligible = revertible_paths(runner, &request.base)?
        .into_iter()
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();
    for path in &request.paths {
        if !eligible.contains(path) {
            return Err(RevertError::InvalidState(format!(
                "path is not locally dirty and does not differ from Base: {path}"
            )));
        }
    }
    let source = source_for(&request)?;
    let commands = vec![restore_command(&source, &request.paths)];
    Ok(RevertPlan {
        paths: request.paths,
        target: request.target,
        source,
        base_ref: request.base,
        commands,
        source_head,
    })
}

pub(super) fn verify_current(runner: &GitRunner, plan: &RevertPlan) -> Result<(), RevertError> {
    if read_id(runner, "HEAD")? != plan.source_head {
        return Err(RevertError::StalePlan);
    }
    ensure_no_operation(runner)?;
    let eligible = revertible_paths(runner, &plan.base_ref)?
        .into_iter()
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();
    for path in &plan.paths {
        if !eligible.contains(path) {
            return Err(RevertError::StalePlan);
        }
    }
    Ok(())
}

fn source_for(request: &RevertRequest) -> Result<String, RevertError> {
    match request.target {
        RevertTarget::Head => Ok("HEAD".to_string()),
        RevertTarget::Base => Ok(request.base.as_str().to_string()),
    }
}

fn restore_command(source: &str, paths: &[RepoPath]) -> String {
    let mut parts = vec![
        "git".to_string(),
        "-c".to_string(),
        "submodule.recurse=false".to_string(),
        "restore".to_string(),
        format!("--source={source}"),
        "--staged".to_string(),
        "--worktree".to_string(),
        "--".to_string(),
    ];
    for path in paths {
        parts.push(literal(path.as_str()));
    }
    parts.join(" ")
}

fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, RevertError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(RevertError::InvalidState)
}

fn ensure_no_operation(runner: &GitRunner) -> Result<(), RevertError> {
    for marker in [
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "BISECT_LOG",
        "rebase-merge",
        "rebase-apply",
    ] {
        if git_path(runner, marker)?.exists() {
            return Err(RevertError::InvalidState(format!(
                "Git operation is in progress: {marker}"
            )));
        }
    }
    Ok(())
}

fn git_path(runner: &GitRunner, marker: &str) -> Result<PathBuf, RevertError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--git-path", marker])))?;
    let path = PathBuf::from(text(&output.stdout)?.trim());
    Ok(if path.is_absolute() {
        path
    } else {
        runner.repo_path().join(path)
    })
}

fn text(bytes: &[u8]) -> Result<String, RevertError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| RevertError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
