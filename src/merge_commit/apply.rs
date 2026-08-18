use std::collections::BTreeMap;

use crate::git::{GitCommand, GitRunner};
use crate::recording::Oplog;
use crate::rewrite::ObjectId;

use super::errors::CommitMergeError;
use super::model::{CommitMergePlan, CommitMergeResult};
use super::plan::{check_pr_subset, verify_current};
use super::record;
use super::state::{args, read_id, read_tree_id};

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &CommitMergePlan,
) -> Result<CommitMergeResult, CommitMergeError> {
    verify_current(runner, plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| CommitMergeError::Recording(error.to_string()))?;
    let operation_id = record::begin(&oplog, plan)?;
    install_tree(runner, plan)?;
    let new_head = commit_merge(runner)?;
    verify_merge_commit(runner, plan, &new_head)?;
    check_pr_subset(runner, plan)?;
    oplog
        .finish(
            &operation_id,
            BTreeMap::from([(plan.branch.to_string(), new_head.to_string())]),
        )
        .map_err(|error| CommitMergeError::Recording(error.to_string()))?;
    Ok(CommitMergeResult {
        old_head: plan.source_head.clone(),
        new_head,
        merge_head: plan.merge_head.clone(),
        excluded_paths: plan.excluded_paths.clone(),
    })
}

fn install_tree(runner: &GitRunner, plan: &CommitMergePlan) -> Result<(), CommitMergeError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "read-tree",
        plan.tree.as_str(),
    ])))?;
    Ok(())
}

fn commit_merge(runner: &GitRunner) -> Result<ObjectId, CommitMergeError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "commit",
        "--no-edit",
        "--no-verify",
    ])))?;
    read_id(runner, "HEAD")
}

fn verify_merge_commit(
    runner: &GitRunner,
    plan: &CommitMergePlan,
    new_head: &ObjectId,
) -> Result<(), CommitMergeError> {
    let parent_one = read_id(runner, &format!("{new_head}^1"))?;
    let parent_two = read_id(runner, &format!("{new_head}^2"))?;
    if parent_one != plan.source_head || parent_two != plan.merge_head {
        return Err(CommitMergeError::InvalidState(
            "merge commit parents do not match HEAD and MERGE_HEAD".to_string(),
        ));
    }
    let tree = read_tree_id(runner, new_head)?;
    if tree != plan.tree {
        return Err(CommitMergeError::InvalidState(
            "merge commit tree does not match the planned tree".to_string(),
        ));
    }
    Ok(())
}
