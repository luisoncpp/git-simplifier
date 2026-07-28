use crate::git::{GitCommand, GitRunner};
use crate::rewrite::RefName;

use super::errors::SyncError;
use super::state::{args, text};

pub(crate) fn untracked_conflicts(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<String>, SyncError> {
    let untracked = read_untracked(runner)?;
    let changed = read_base_changes(runner, base)?;
    Ok(untracked
        .into_iter()
        .filter(|path| changed.iter().any(|target| paths_overlap(path, target)))
        .collect())
}

fn read_untracked(runner: &GitRunner) -> Result<Vec<String>, SyncError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=all",
        "--ignore-submodules=none",
    ])))?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| record.starts_with(b"? "))
        .map(|record| text(&record[2..]))
        .collect()
}

fn read_base_changes(runner: &GitRunner, base: &RefName) -> Result<Vec<String>, SyncError> {
    let output = runner.run(GitCommand::read(args(&[
        "diff",
        "--name-only",
        "-z",
        "--diff-filter=ACMRTUXB",
        "HEAD",
        base.as_str(),
    ])))?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(text)
        .collect()
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
