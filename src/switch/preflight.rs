use std::collections::BTreeSet;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::state::{args, branch_ref, text};

pub(super) fn ensure_carry_safe(
    runner: &GitRunner,
    source_head: &ObjectId,
    target_head: &ObjectId,
) -> Result<(), SwitchError> {
    let conflicts = carry_conflicts(runner, source_head, target_head)?;
    if conflicts.is_empty() {
        return Ok(());
    }
    Err(SwitchError::CarryConflict(conflicts.join(", ")))
}

fn carry_conflicts(
    runner: &GitRunner,
    source_head: &ObjectId,
    target_head: &ObjectId,
) -> Result<Vec<String>, SwitchError> {
    let changed = read_tracked_paths(runner)?;
    Ok(changed
        .into_iter()
        .filter(|path| trees_differ(runner, source_head, target_head, path))
        .collect())
}

fn read_tracked_paths(runner: &GitRunner) -> Result<Vec<String>, SwitchError> {
    let mut paths = read_diff_paths(runner, &["diff", "--name-only", "-z", "HEAD"])?;
    paths.extend(read_diff_paths(
        runner,
        &["diff", "--cached", "--name-only", "-z"],
    )?);
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn read_diff_paths(runner: &GitRunner, command: &[&str]) -> Result<Vec<String>, SwitchError> {
    let output = runner.run(GitCommand::read(args(command)))?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(text)
        .collect()
}

fn trees_differ(
    runner: &GitRunner,
    source_head: &ObjectId,
    target_head: &ObjectId,
    path: &str,
) -> bool {
    blob_at(runner, source_head, path) != blob_at(runner, target_head, path)
}

fn blob_at(runner: &GitRunner, commit: &ObjectId, path: &str) -> Option<String> {
    let spec = format!("{}:{}", commit.as_str(), path);
    runner
        .run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))
        .ok()
        .and_then(|output| text(&output.stdout).ok())
        .map(|value| value.trim().to_string())
}

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
