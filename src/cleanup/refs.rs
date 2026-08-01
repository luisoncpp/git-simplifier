use std::collections::BTreeSet;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::CleanupError;
use super::state::{args, text, WIP_PREFIX};

/// Every atom below is single-line by construction. A `%(contents)`-style atom
/// must never be added: a field carrying a newline would break the record split.
const LOCAL_FORMAT: &str = concat!(
    "--format=%(refname)%00%(objectname)%00%(HEAD)%00%(worktreepath)",
    "%00%(authoremail:trim)%00%(upstream:remotename)%00%(upstream:remoteref)%00%(upstream)"
);
const LOCAL_FIELDS: usize = 8;
const REMOTE_FORMAT: &str =
    "--format=%(refname)%00%(objectname)%00%(symref)%00%(authoremail:trim)";
const REMOTE_FIELDS: usize = 4;

pub(super) struct LocalRef {
    pub reference: String,
    pub head: ObjectId,
    /// `%(HEAD)` is `"*"` for the checked-out branch and a single space
    /// otherwise — never empty, so it must be compared rather than tested.
    pub is_head: bool,
    pub worktree: String,
    pub author_email: String,
    pub upstream_remote: String,
    pub upstream_remote_ref: String,
    pub upstream: String,
}

pub(super) struct RemoteRef {
    pub reference: String,
    pub head: ObjectId,
    pub symref: String,
    pub author_email: String,
}

pub(super) fn merged_locals(
    runner: &GitRunner,
    base_head: &ObjectId,
) -> Result<Vec<LocalRef>, CleanupError> {
    let output = runner.run(GitCommand::read(args(&[
        "for-each-ref",
        "--merged",
        base_head.as_str(),
        LOCAL_FORMAT,
        "refs/heads",
    ])))?;
    records(&output.stdout)
        .map(|record| parse_local(record))
        .collect()
}

pub(super) fn remotes(runner: &GitRunner) -> Result<Vec<RemoteRef>, CleanupError> {
    let output = runner.run(GitCommand::read(args(&[
        "for-each-ref",
        REMOTE_FORMAT,
        "refs/remotes",
    ])))?;
    records(&output.stdout)
        .map(|record| parse_remote(record))
        .collect()
}

pub(super) fn merged_remote_names(
    runner: &GitRunner,
    base_head: &ObjectId,
) -> Result<BTreeSet<String>, CleanupError> {
    let output = runner.run(GitCommand::read(args(&[
        "for-each-ref",
        "--merged",
        base_head.as_str(),
        "--format=%(refname)",
        "refs/remotes",
    ])))?;
    lines(&output.stdout)
}

pub(super) fn remote_names(runner: &GitRunner) -> Result<Vec<String>, CleanupError> {
    let output = runner.run(GitCommand::read(args(&["remote"])))?;
    Ok(lines(&output.stdout)?.into_iter().collect())
}

/// Read with `%(refname)` and stripped in Rust: `%(refname:strip=N)` counts the
/// branch name as a component, so it empties a simple name and truncates a
/// slashed one. This is a safety input, so it cannot afford to under-report.
pub(super) fn saved_work_branches(runner: &GitRunner) -> Result<BTreeSet<String>, CleanupError> {
    let output = runner.run(GitCommand::read(args(&[
        "for-each-ref",
        "--format=%(refname)",
        "refs/githelper/wip",
    ])))?;
    let mut branches = BTreeSet::new();
    for reference in lines(&output.stdout)? {
        let Some(branch) = reference.strip_prefix(WIP_PREFIX) else {
            continue;
        };
        branches.insert(branch.to_string());
    }
    Ok(branches)
}

fn records(stdout: &[u8]) -> impl Iterator<Item = &[u8]> {
    stdout
        .split(|byte| *byte == b'\n')
        .filter(|record| !record.is_empty())
}

fn lines(stdout: &[u8]) -> Result<BTreeSet<String>, CleanupError> {
    records(stdout)
        .map(|record| text(record).map(|value| value.trim().to_string()))
        .collect()
}

fn fields(record: &[u8], expected: usize) -> Result<Vec<String>, CleanupError> {
    let parts = record.split(|byte| *byte == 0).collect::<Vec<_>>();
    if parts.len() != expected {
        return Err(CleanupError::Parse(
            "branch listing output was malformed".to_string(),
        ));
    }
    parts.into_iter().map(text).collect()
}

fn parse_local(record: &[u8]) -> Result<LocalRef, CleanupError> {
    let values = fields(record, LOCAL_FIELDS)?;
    Ok(LocalRef {
        reference: values[0].clone(),
        head: object_id(&values[1])?,
        is_head: values[2] == "*",
        worktree: values[3].clone(),
        author_email: values[4].clone(),
        upstream_remote: values[5].clone(),
        upstream_remote_ref: values[6].clone(),
        upstream: values[7].clone(),
    })
}

fn parse_remote(record: &[u8]) -> Result<RemoteRef, CleanupError> {
    let values = fields(record, REMOTE_FIELDS)?;
    Ok(RemoteRef {
        reference: values[0].clone(),
        head: object_id(&values[1])?,
        symref: values[2].clone(),
        author_email: values[3].clone(),
    })
}

fn object_id(value: &str) -> Result<ObjectId, CleanupError> {
    ObjectId::new(value.trim().to_string()).map_err(CleanupError::InvalidState)
}
