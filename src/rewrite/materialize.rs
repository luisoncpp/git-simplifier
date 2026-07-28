use std::collections::BTreeMap;
use std::ffi::OsString;

use tempfile::tempdir;

use crate::git::{GitCommand, GitRunner};
use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::ApplyError;
use super::materialize_steps::{build_history, parse_id};
use super::model::{ApplyResult, ObjectId, RewritePlan};

pub(crate) fn apply(runner: &GitRunner, plan: &RewritePlan) -> Result<ApplyResult, ApplyError> {
    verify_plan(runner, plan)?;
    let git_dir = runner.git_dir()?;
    let oplog = Oplog::open(&git_dir).map_err(|error| ApplyError::Recording(error.to_string()))?;
    let operation_id = begin_record(&oplog, plan)?;
    let index_dir = tempdir().map_err(|error| ApplyError::InvalidPlan(error.to_string()))?;
    let index = index_dir.path().join("index");
    let new_head = build_history(runner, plan, &index)?;
    update_branch(runner, plan, &new_head)?;
    if !plan.selected_paths.is_empty() {
        reset_paths(runner, plan, &new_head)?;
    }
    let mut after = BTreeMap::new();
    after.insert(plan.branch.to_string(), new_head.to_string());
    oplog
        .finish(&operation_id, after)
        .map_err(|error| ApplyError::Recording(error.to_string()))?;
    Ok(ApplyResult {
        old_head: plan.source_head.clone(),
        new_head,
        dropped_commits: plan.dropped_commits.clone(),
    })
}

fn verify_plan(runner: &GitRunner, plan: &RewritePlan) -> Result<(), ApplyError> {
    if plan.commits.is_empty() {
        return Err(ApplyError::InvalidPlan(
            "plan contains no commits".to_string(),
        ));
    }
    let branch = read_text(runner, vec!["symbolic-ref", "--quiet", "HEAD"])?;
    if branch.trim() != plan.branch.as_str() {
        return Err(ApplyError::StalePlan);
    }
    let head = read_id(runner, vec!["rev-parse", "--verify", "HEAD^{commit}"])?;
    if head != plan.source_head {
        return Err(ApplyError::StalePlan);
    }
    let base_arg = format!("{}^{{commit}}", plan.base_ref);
    let base = read_id(runner, vec!["rev-parse", "--verify", &base_arg])?;
    if base != plan.base {
        return Err(ApplyError::StalePlan);
    }
    Ok(())
}

fn update_branch(
    runner: &GitRunner,
    plan: &RewritePlan,
    new_head: &ObjectId,
) -> Result<(), ApplyError> {
    let reflog_message = format!("git-helper {}", plan.operation.label());
    let values = vec![
        "update-ref",
        "-m",
        reflog_message.as_str(),
        plan.branch.as_str(),
        new_head.as_str(),
        plan.source_head.as_str(),
    ];
    runner.run_unlocked(GitCommand::write(GitRunner::command_args(&values)))?;
    Ok(())
}

fn reset_paths(
    runner: &GitRunner,
    plan: &RewritePlan,
    new_head: &ObjectId,
) -> Result<(), ApplyError> {
    let mut args = vec![
        OsString::from("reset"),
        OsString::from("--mixed"),
        OsString::from(new_head.as_str()),
        OsString::from("--"),
    ];
    args.extend(
        plan.selected_paths
            .iter()
            .map(|path| OsString::from(path.as_str())),
    );
    runner.run_unlocked(GitCommand::write(args))?;
    Ok(())
}

fn begin_record(oplog: &Oplog, plan: &RewritePlan) -> Result<String, ApplyError> {
    let started = timestamp();
    let id = format!(
        "{}-{started}-{}",
        plan.operation.label(),
        std::process::id()
    );
    let mut before = BTreeMap::new();
    before.insert(plan.branch.to_string(), plan.source_head.to_string());
    let record = OperationRecord {
        id: id.clone(),
        operation: plan.operation.label().to_string(),
        started,
        finished: None,
        refs_before: before,
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: BTreeMap::new(),
        phase: None,
        commands: vec![format!(
            "git update-ref {} <new> {}",
            plan.branch, plan.source_head
        )],
        reversible: true,
    };
    oplog
        .begin(record)
        .map_err(|error| ApplyError::Recording(error.to_string()))?;
    Ok(id)
}

fn read_id(runner: &GitRunner, values: Vec<&str>) -> Result<ObjectId, ApplyError> {
    let output = runner.run_unlocked(GitCommand::read(GitRunner::command_args(&values)))?;
    parse_id(&output.stdout)
}

fn read_text(runner: &GitRunner, values: Vec<&str>) -> Result<String, ApplyError> {
    let output = runner.run_unlocked(GitCommand::read(GitRunner::command_args(&values)))?;
    String::from_utf8(output.stdout)
        .map_err(|_| ApplyError::InvalidPlan("Git output is not UTF-8".to_string()))
}
