use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::ForcePushError;
use super::model::{ForcePushPlan, ForcePushResult};
use super::plan;

pub(crate) fn apply(
    runner: &GitRunner,
    plan: &ForcePushPlan,
) -> Result<ForcePushResult, ForcePushError> {
    plan::verify_current(runner, plan)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| ForcePushError::Recording(error.to_string()))?;
    let operation_id = begin_record(&oplog, plan)?;
    runner.run_unlocked(GitCommand::write(push_args(plan)))?;
    let after = BTreeMap::from([(plan.branch.to_string(), plan.source_head.to_string())]);
    oplog
        .finish(&operation_id, after)
        .map_err(|error| ForcePushError::Recording(error.to_string()))?;
    Ok(ForcePushResult {
        branch: plan.branch.clone(),
        remote: plan.remote.clone(),
        new_head: plan.source_head.clone(),
    })
}

fn begin_record(oplog: &Oplog, plan: &ForcePushPlan) -> Result<String, ForcePushError> {
    let started = timestamp();
    let id = format!("force-push-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "force-push".to_string(),
        started,
        finished: None,
        refs_before: BTreeMap::from([
            (plan.branch.to_string(), plan.source_head.to_string()),
            (plan.upstream.to_string(), plan.expected_remote.to_string()),
        ]),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: BTreeMap::from([
            ("remote".to_string(), plan.remote.clone()),
            ("remote_branch".to_string(), plan.remote_branch.to_string()),
        ]),
        phase: None,
        commands: vec![plan.command.clone()],
        reversible: false,
    };
    oplog
        .begin(record)
        .map_err(|error| ForcePushError::Recording(error.to_string()))?;
    Ok(id)
}

fn push_args(plan: &ForcePushPlan) -> Vec<OsString> {
    let lease = format!(
        "--force-with-lease={}:{}",
        plan.remote_branch, plan.expected_remote
    );
    let refspec = format!("HEAD:{}", plan.remote_branch);
    vec![
        OsString::from("push"),
        OsString::from(lease),
        OsString::from(plan.remote.as_str()),
        OsString::from(refspec),
    ]
}
