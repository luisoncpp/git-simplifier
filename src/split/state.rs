use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SplitError;

pub(super) fn read_branch(runner: &GitRunner) -> Result<String, SplitError> {
    let output = runner
        .run(GitCommand::read(args(&[
            "symbolic-ref",
            "--quiet",
            "--short",
            "HEAD",
        ])))
        .map_err(|_| SplitError::InvalidState("HEAD is detached".to_string()))?;
    text(&output.stdout).map(|value| value.trim().to_string())
}

pub(super) fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, SplitError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SplitError::InvalidState)
}

pub(super) fn merge_base(
    runner: &GitRunner,
    base: &ObjectId,
    head: &ObjectId,
) -> Result<ObjectId, SplitError> {
    let output = runner
        .run(GitCommand::read(args(&[
            "merge-base",
            base.as_str(),
            head.as_str(),
        ])))
        .map_err(|_| {
            SplitError::InvalidState(format!("no common ancestor between {base} and {head}"))
        })?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SplitError::InvalidState)
}

pub(super) fn branch_exists(runner: &GitRunner, reference: &str) -> Result<bool, SplitError> {
    Ok(runner
        .run(GitCommand::read(args(&[
            "show-ref", "--verify", "--quiet", reference,
        ])))
        .is_ok())
}

pub(super) fn validate_branch_name(runner: &GitRunner, branch: &str) -> Result<(), SplitError> {
    if branch.is_empty() || branch.starts_with('-') || branch.contains('\0') {
        return Err(SplitError::InvalidState(format!(
            "invalid branch name: {branch}"
        )));
    }
    runner.run(GitCommand::read(args(&[
        "check-ref-format",
        "--branch",
        branch,
    ])))?;
    Ok(())
}

pub(super) fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

pub(super) fn literal(path: &str) -> String {
    format!(":(literal){path}")
}

pub(super) fn text(bytes: &[u8]) -> Result<String, SplitError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SplitError::InvalidState("Git output is not UTF-8".to_string()))
}

pub(super) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
