use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;

use crate::git::{GitCommand, GitRunner};
use crate::recording::Oplog;
use crate::rewrite::{ObjectId, RepoPath};

use super::errors::SplitError;
use super::model::{SplitBranchPlan, SplitBranchResult};
use super::state::{args, literal, text};
use super::{plan, record, worktree};

pub(crate) fn split(
    runner: &GitRunner,
    split_plan: &SplitBranchPlan,
) -> Result<SplitBranchResult, SplitError> {
    plan::verify_current(runner, split_plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SplitError::Recording(error.to_string()))?;
    let operation_id = record::begin(&oplog, split_plan)?;
    let tree = build_tree(runner, split_plan)?;
    let commit = commit_tree(runner, split_plan, &tree)?;
    create_branch(runner, split_plan, &commit)?;
    let after = BTreeMap::from([(split_plan.new_branch_ref.clone(), commit.to_string())]);
    oplog
        .finish(&operation_id, after)
        .map_err(|error| SplitError::Recording(error.to_string()))?;
    Ok(SplitBranchResult {
        branch: split_plan.new_branch.clone(),
        reference: split_plan.new_branch_ref.clone(),
        commit,
        merge_base: split_plan.merge_base.clone(),
        paths: split_plan.changed_paths.clone(),
    })
}

fn build_tree(runner: &GitRunner, split_plan: &SplitBranchPlan) -> Result<ObjectId, SplitError> {
    let patch = read_patch(runner, split_plan)?;
    worktree::with_temporary(runner, &split_plan.merge_base, /*action=*/ |path| {
        apply_patch(runner, path, patch)?;
        write_tree(runner, path)
    })
}

fn read_patch(runner: &GitRunner, split_plan: &SplitBranchPlan) -> Result<Vec<u8>, SplitError> {
    let mut values = args(&[
        "diff",
        "--binary",
        "--no-relative",
        "--no-renames",
        "--no-ext-diff",
        "--no-color",
        split_plan.merge_base.as_str(),
        split_plan.source_head.as_str(),
        "--",
    ]);
    values.extend(pathspecs(&split_plan.changed_paths));
    let output = runner.run(GitCommand::read(values))?;
    if output.stdout.is_empty() {
        return Err(SplitError::NoChanges);
    }
    Ok(output.stdout)
}

fn apply_patch(runner: &GitRunner, path: &Path, patch: Vec<u8>) -> Result<(), SplitError> {
    let mut values = vec![OsString::from("-C"), OsString::from(path)];
    values.extend(args(&[
        "-c",
        "submodule.recurse=false",
        "apply",
        "--index",
        "--binary",
        "--whitespace=nowarn",
        "-",
    ]));
    runner.run_unlocked(GitCommand::write(values).with_stdin(patch))?;
    Ok(())
}

fn write_tree(runner: &GitRunner, path: &Path) -> Result<ObjectId, SplitError> {
    let mut values = vec![OsString::from("-C"), OsString::from(path)];
    values.extend(args(&["write-tree"]));
    let output = runner.run_unlocked(GitCommand::write(values))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SplitError::InvalidState)
}

fn commit_tree(
    runner: &GitRunner,
    split_plan: &SplitBranchPlan,
    tree: &ObjectId,
) -> Result<ObjectId, SplitError> {
    let values = args(&[
        "commit-tree",
        tree.as_str(),
        "-p",
        split_plan.merge_base.as_str(),
    ]);
    let output =
        runner.run_unlocked(GitCommand::write(values).with_stdin(split_plan.message.clone()))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SplitError::InvalidState)
}

/// The empty old value makes Git refuse the write unless the branch is still
/// absent, so a racing branch creation fails instead of being overwritten.
fn create_branch(
    runner: &GitRunner,
    split_plan: &SplitBranchPlan,
    commit: &ObjectId,
) -> Result<(), SplitError> {
    let values = vec![
        OsString::from("update-ref"),
        OsString::from("-m"),
        OsString::from("git-helper split-branch"),
        OsString::from(&split_plan.new_branch_ref),
        OsString::from(commit.as_str()),
        OsString::from(""),
    ];
    runner.run_unlocked(GitCommand::write(values))?;
    Ok(())
}

fn pathspecs(paths: &[RepoPath]) -> Vec<OsString> {
    paths
        .iter()
        .map(|path| OsString::from(literal(path.as_str())))
        .collect()
}
