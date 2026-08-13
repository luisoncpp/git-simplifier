use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::CommitMergeError;
use super::model::CommitMergePlan;

pub(crate) fn begin(oplog: &Oplog, plan: &CommitMergePlan) -> Result<String, CommitMergeError> {
    let started = timestamp();
    let id = format!("commit-merge-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "commit-merge".to_string(),
        started,
        finished: None,
        refs_before: BTreeMap::from([(plan.branch.to_string(), plan.source_head.to_string())]),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: BTreeMap::new(),
        phase: None,
        commands: plan.commands.clone(),
        reversible: true,
    };
    oplog
        .begin(record)
        .map_err(|error| CommitMergeError::Recording(error.to_string()))?;
    Ok(id)
}
