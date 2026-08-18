use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::SwitchError;
use super::history_model::HistorySwitchPlan;
use super::record_commands;

pub(super) fn begin_history(
    oplog: &Oplog,
    plan: &HistorySwitchPlan,
) -> Result<String, SwitchError> {
    let started = timestamp();
    let id = format!("history-switch-{started}-{}", std::process::id());
    let details = BTreeMap::from([
        ("source_branch".to_string(), plan.source_branch.clone()),
        ("target_commit".to_string(), plan.target_commit.to_string()),
        ("present_ref".to_string(), super::present::PRESENT_REF.to_string()),
        ("present_branch".to_string(), plan.source_branch.clone()),
    ]);
    let record = OperationRecord {
        id: id.clone(),
        operation: "history-switch".to_string(),
        started,
        finished: None,
        refs_before: BTreeMap::from([("HEAD".to_string(), plan.source_head.to_string())]),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details,
        phase: None,
        commands: record_commands::history_commands(plan),
        reversible: true,
    };
    oplog
        .begin(record)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(id)
}
