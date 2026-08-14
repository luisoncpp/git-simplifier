use std::collections::BTreeSet;

use crate::git::{GitCommand, GitRunner};

use super::errors::SwitchError;
use super::state::{args, text};

pub(super) fn read_untracked(runner: &GitRunner) -> Result<Vec<String>, SwitchError> {
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

pub(super) fn classify(
    runner: &GitRunner,
    target_commitish: &str,
    untracked: &[String],
) -> Result<(Vec<String>, Vec<String>), SwitchError> {
    if untracked.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let target_paths = read_target_paths(runner, target_commitish)?;
    Ok(classify_conflicts(untracked, &target_paths))
}

fn classify_conflicts(
    untracked: &[String],
    target_paths: &BTreeSet<String>,
) -> (Vec<String>, Vec<String>) {
    let mut mergeable = Vec::new();
    let mut hard = Vec::new();
    for untracked_path in untracked {
        for target in target_paths {
            if untracked_path == target {
                mergeable.push(untracked_path.clone());
                break;
            }
            if prefix_overlap(untracked_path, target) {
                hard.push(untracked_path.clone());
                break;
            }
        }
    }
    (mergeable, hard)
}

fn read_target_paths(
    runner: &GitRunner,
    target_commitish: &str,
) -> Result<BTreeSet<String>, SwitchError> {
    let output = runner.run(GitCommand::read(args(&[
        "ls-tree",
        "-r",
        "--name-only",
        "-z",
        "--full-tree",
        target_commitish,
    ])))?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(text)
        .collect()
}

fn prefix_overlap(left: &str, right: &str) -> bool {
    left != right
        && (left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
            || right
                .strip_prefix(left)
                .is_some_and(|suffix| suffix.starts_with('/')))
}
