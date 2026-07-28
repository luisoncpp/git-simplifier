use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::SplitError;
use super::model::SplitBranchPlan;

pub(super) fn begin(oplog: &Oplog, plan: &SplitBranchPlan) -> Result<String, SplitError> {
    let started = timestamp();
    let id = format!("split-branch-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "split-branch".to_string(),
        started,
        finished: None,
        refs_before: refs_before(plan),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: details(plan),
        phase: None,
        commands: plan.commands.clone(),
        reversible: true,
    };
    oplog
        .begin(record)
        .map_err(|error| SplitError::Recording(error.to_string()))?;
    Ok(id)
}

/// The new branch is recorded with an empty previous value: it did not exist,
/// so recovery deletes it rather than restoring a former target.
fn refs_before(plan: &SplitBranchPlan) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HEAD".to_string(), plan.source_head.to_string()),
        (plan.new_branch_ref.clone(), String::new()),
    ])
}

fn details(plan: &SplitBranchPlan) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("source-branch".to_string(), plan.source_branch.clone()),
        ("merge-base".to_string(), plan.merge_base.to_string()),
        (
            "paths".to_string(),
            plan.changed_paths
                .iter()
                .map(|path| path.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ])
}
