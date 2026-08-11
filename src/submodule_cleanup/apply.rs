use std::ffi::OsString;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::recording::{timestamp, OperationRecord, Oplog};
use crate::revert::literal;

use super::errors::SubmoduleCleanupError;
use super::model::{SubmoduleCleanupPlan, SubmoduleCleanupResult};
use super::plan;

pub(super) fn apply(
    runner: &GitRunner,
    plan: &SubmoduleCleanupPlan,
) -> Result<SubmoduleCleanupResult, SubmoduleCleanupError> {
    plan::verify_current(runner, plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| SubmoduleCleanupError::Recording(error.to_string()))?;
    let operation_id = begin_record(&oplog, plan)?;
    let mut uncommitted = 0usize;
    if let Some(uncommit_plan) = &plan.uncommit_plan {
        crate::rewrite::apply(runner, uncommit_plan)?;
        uncommitted = plan.uncommit_paths.len();
    }
    let mut reverted = 0usize;
    if !plan.revert_paths.is_empty() {
        revert_submodules(runner, plan)?;
        reverted = plan.revert_paths.len();
    }
    oplog
        .finish(&operation_id, Default::default())
        .map_err(|error| SubmoduleCleanupError::Recording(error.to_string()))?;
    Ok(SubmoduleCleanupResult {
        paths: plan.paths.clone(),
        uncommitted,
        reverted,
    })
}

fn revert_submodules(
    runner: &GitRunner,
    plan: &SubmoduleCleanupPlan,
) -> Result<(), SubmoduleCleanupError> {
    for path in &plan.revert_paths {
        if head_has_gitlink(runner, path.as_str())? {
            sync_gitlink(runner, path.as_str())?;
        } else {
            remove_checkout(runner, path.as_str())?;
        }
    }
    Ok(())
}

fn sync_gitlink(runner: &GitRunner, path: &str) -> Result<(), SubmoduleCleanupError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "restore",
        "--source=HEAD",
        "--staged",
        "--worktree",
        "--",
        &literal(path),
    ])))?;
    runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "submodule",
        "update",
        "--force",
        "--",
        path,
    ])))?;
    runner.run_unlocked(GitCommand::write(args(&[
        "-C",
        path,
        "checkout",
        "--force",
        "HEAD",
    ])))?;
    runner.run_unlocked(GitCommand::write(args(&[
        "-C",
        path,
        "clean",
        "-fd",
    ])))?;
    Ok(())
}

fn remove_checkout(runner: &GitRunner, path: &str) -> Result<(), SubmoduleCleanupError> {
    let _ = runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "submodule",
        "deinit",
        "-f",
        "--",
        path,
    ])));
    let worktree = worktree_path(runner, path)?;
    if worktree.exists() {
        std::fs::remove_dir_all(worktree).map_err(|error| {
            SubmoduleCleanupError::InvalidState(error.to_string())
        })?;
    }
    Ok(())
}

fn head_has_gitlink(runner: &GitRunner, path: &str) -> Result<bool, SubmoduleCleanupError> {
    let output = runner.run(GitCommand::read(args(&["ls-tree", "HEAD", "--", path])))?;
    Ok(!output.stdout.is_empty())
}

fn worktree_path(runner: &GitRunner, path: &str) -> Result<PathBuf, SubmoduleCleanupError> {
    Ok(runner.repo_path().join(path))
}

fn begin_record(oplog: &Oplog, plan: &SubmoduleCleanupPlan) -> Result<String, SubmoduleCleanupError> {
    let started = timestamp();
    let id = format!("submodule-cleanup-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "cleanup_submodules".to_string(),
        started,
        finished: None,
        refs_before: Default::default(),
        refs_after: Default::default(),
        snapshots: Default::default(),
        details: Default::default(),
        phase: None,
        commands: plan.commands.clone(),
        reversible: false,
    };
    oplog
        .begin(record)
        .map_err(|error| SubmoduleCleanupError::Recording(error.to_string()))?;
    Ok(id)
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
