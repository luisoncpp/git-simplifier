use std::collections::BTreeSet;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName, RepoPath};

use super::errors::CommitMergeError;
use super::state;

pub(crate) fn name_only_diff(
    runner: &GitRunner,
    left: &str,
    right: &str,
) -> Result<Vec<RepoPath>, CommitMergeError> {
    let output = runner.run(GitCommand::read(state::args(&[
        "diff",
        "--name-only",
        "--no-relative",
        "--ignore-submodules=none",
        "-z",
        left,
        right,
    ])))?;
    parse_paths(&output.stdout)
}

pub(crate) fn pr_paths_before(
    runner: &GitRunner,
    base: &RefName,
    merge_head: &ObjectId,
) -> Result<Vec<RepoPath>, CommitMergeError> {
    let base_commit = state::optional_id(runner, base.as_str())?;
    if base_commit.as_ref() != Some(merge_head) {
        return Ok(Vec::new());
    }
    triple_dot_paths(runner, base)
}

pub(crate) fn triple_dot_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<RepoPath>, CommitMergeError> {
    let range = format!("{}...HEAD", base.as_str());
    let output = runner.run(GitCommand::read(state::args(&[
        "diff",
        "--name-only",
        "--no-relative",
        "--ignore-submodules=none",
        "-z",
        &range,
    ])))?;
    parse_paths(&output.stdout)
}

pub(crate) fn literal(path: &str) -> String {
    format!(":(top,literal){path}")
}

pub(crate) fn staged_vs_merge_head(runner: &GitRunner) -> Result<Vec<RepoPath>, CommitMergeError> {
    let output = runner.run(GitCommand::read(state::args(&[
        "diff",
        "--cached",
        "--name-only",
        "--no-relative",
        "--ignore-submodules=none",
        "-z",
        "MERGE_HEAD",
    ])))?;
    parse_paths(&output.stdout)
}

pub(crate) fn excluded_paths(
    runner: &GitRunner,
    merge_head: &ObjectId,
    tree: &ObjectId,
) -> Result<Vec<RepoPath>, CommitMergeError> {
    let staged = staged_vs_merge_head(runner)?;
    let in_tree = name_only_diff(runner, merge_head.as_str(), tree.as_str())?;
    let tree_set = in_tree.into_iter().collect::<BTreeSet<_>>();
    Ok(staged
        .into_iter()
        .filter(|path| !tree_set.contains(path))
        .collect())
}

pub(crate) fn extras_vs_pr(before: &[RepoPath], after: &[RepoPath]) -> Vec<RepoPath> {
    let before_set = before.iter().collect::<BTreeSet<_>>();
    after
        .iter()
        .filter(|path| !before_set.contains(path))
        .cloned()
        .collect()
}

fn parse_paths(bytes: &[u8]) -> Result<Vec<RepoPath>, CommitMergeError> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let text = String::from_utf8(entry.to_vec())
                .map_err(|_| CommitMergeError::InvalidState("path is not UTF-8".to_string()))?;
            RepoPath::new(text).map_err(CommitMergeError::InvalidState)
        })
        .collect()
}
