use std::collections::BTreeSet;

use crate::git::{GitCommand, GitRunner};

use super::errors::SwitchError;
use super::state::{args, branch_ref, text};

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

pub(super) fn ensure_untracked_safe(
    runner: &GitRunner,
    target_branch: &str,
    untracked: &[String],
) -> Result<(), SwitchError> {
    if untracked.is_empty() {
        return Ok(());
    }
    let target_paths = read_target_paths(runner, target_branch)?;
    let conflicts = untracked
        .iter()
        .filter(|untracked_path| {
            target_paths
                .iter()
                .any(|target| paths_overlap(untracked_path, target))
        })
        .cloned()
        .collect::<Vec<_>>();
    if conflicts.is_empty() {
        return Ok(());
    }
    Err(SwitchError::UntrackedConflict(conflicts.join(", ")))
}

fn read_target_paths(
    runner: &GitRunner,
    target_branch: &str,
) -> Result<BTreeSet<String>, SwitchError> {
    let target = branch_ref(target_branch);
    let output = runner.run(GitCommand::read(args(&[
        "ls-tree",
        "-r",
        "--name-only",
        "-z",
        "--full-tree",
        &target,
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
