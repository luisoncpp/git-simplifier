use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::RevertError;
use super::model::{RevertPlan, RevertResult};
use super::paths::literal;
use super::plan;

pub(super) fn apply(runner: &GitRunner, plan: &RevertPlan) -> Result<RevertResult, RevertError> {
    plan::verify_current(runner, plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| RevertError::Recording(error.to_string()))?;
    let operation_id = begin_record(&oplog, plan)?;
    restore_paths(runner, plan)?;
    oplog
        .finish(&operation_id, Default::default())
        .map_err(|error| RevertError::Recording(error.to_string()))?;
    Ok(RevertResult {
        paths: plan.paths.clone(),
        source: plan.source.clone(),
    })
}

fn restore_paths(runner: &GitRunner, plan: &RevertPlan) -> Result<(), RevertError> {
    let mut values = vec![
        "-c".to_string(),
        "submodule.recurse=false".to_string(),
        "restore".to_string(),
        format!("--source={}", plan.source),
        "--staged".to_string(),
        "--worktree".to_string(),
        "--".to_string(),
    ];
    for path in &plan.paths {
        values.push(literal(path.as_str()));
    }
    let args = values
        .iter()
        .map(|value| OsString::from(value.as_str()))
        .collect();
    runner.run_unlocked(GitCommand::write(args))?;
    Ok(())
}

fn begin_record(oplog: &Oplog, plan: &RevertPlan) -> Result<String, RevertError> {
    let started = timestamp();
    let id = format!("revert-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "revert".to_string(),
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
        .map_err(|error| RevertError::Recording(error.to_string()))?;
    Ok(id)
}
