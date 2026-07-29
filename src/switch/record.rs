use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{QuickSwitchPlan, SavedWork};

pub(super) fn begin_switch(
    oplog: &Oplog,
    switch_plan: &QuickSwitchPlan,
) -> Result<String, SwitchError> {
    let started = timestamp();
    let id = format!("quick-switch-{started}-{}", std::process::id());
    let mut commands = vec!["git stash create".to_string()];
    if switch_plan.has_tracked_changes && !switch_plan.carry_changes {
        commands.push(format!(
            "git update-ref {} <snapshot>",
            switch_plan.saved_work_reference
        ));
    }
    commands.push("git reset --hard HEAD".to_string());
    commands.push(format!(
        "git switch --no-guess -- {}",
        switch_plan.target_branch
    ));
    if switch_plan.has_tracked_changes && switch_plan.carry_changes {
        commands.push("git stash apply --index <snapshot>".to_string());
    }
    let record = OperationRecord {
        id: id.clone(),
        operation: "quick-switch".to_string(),
        started,
        finished: None,
        refs_before: map_entry("HEAD", &switch_plan.source_head),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: BTreeMap::new(),
        phase: None,
        commands,
        reversible: true,
    };
    begin(oplog, record, id)
}

pub(super) fn begin_restore(oplog: &Oplog, saved: &SavedWork) -> Result<String, SwitchError> {
    begin_simple(
        oplog,
        SimpleRecord {
            operation: "restore-saved-work",
            refs_before: map_entry(&saved.reference, &saved.snapshot),
            commands: vec![format!("git stash apply --index {}", saved.reference)],
            reversible: false,
        },
    )
}

pub(super) fn begin_delete(oplog: &Oplog, saved: &SavedWork) -> Result<String, SwitchError> {
    begin_simple(
        oplog,
        SimpleRecord {
            operation: "delete-saved-work",
            refs_before: map_entry(&saved.reference, &saved.snapshot),
            commands: vec![format!("git update-ref -d {}", saved.reference)],
            reversible: false,
        },
    )
}

struct SimpleRecord {
    operation: &'static str,
    refs_before: BTreeMap<String, String>,
    commands: Vec<String>,
    reversible: bool,
}

fn begin_simple(oplog: &Oplog, draft: SimpleRecord) -> Result<String, SwitchError> {
    let started = timestamp();
    let id = format!("{}-{started}-{}", draft.operation, std::process::id());
    begin(
        oplog,
        OperationRecord {
            id: id.clone(),
            operation: draft.operation.to_string(),
            started,
            finished: None,
            refs_before: draft.refs_before,
            refs_after: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            details: BTreeMap::new(),
            phase: None,
            commands: draft.commands,
            reversible: draft.reversible,
        },
        id,
    )
}

fn begin(oplog: &Oplog, record: OperationRecord, id: String) -> Result<String, SwitchError> {
    oplog
        .begin(record)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    Ok(id)
}

fn map_entry(key: &str, value: &ObjectId) -> BTreeMap<String, String> {
    BTreeMap::from([(key.to_string(), value.to_string())])
}
