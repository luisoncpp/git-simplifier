use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::{QuickSwitchPhase, QuickSwitchPlan, SavedWork};

pub(super) struct PullContext {
    pub id: String,
    pub source_branch: String,
    pub target_branch: String,
    pub remote_ref: String,
    pub carry_reference: Option<String>,
}

pub(super) fn begin_switch(
    oplog: &Oplog,
    switch_plan: &QuickSwitchPlan,
) -> Result<String, SwitchError> {
    let started = timestamp();
    let id = format!("quick-switch-{started}-{}", std::process::id());
    let mut details = BTreeMap::from([
        ("source_branch".to_string(), switch_plan.source_branch.clone()),
        ("target_branch".to_string(), switch_plan.target_branch.clone()),
    ]);
    if let Some(remote) = &switch_plan.pull_remote_ref {
        details.insert("remote_ref".to_string(), remote.clone());
    }
    let record = OperationRecord {
        id: id.clone(),
        operation: "quick-switch".to_string(),
        started,
        finished: None,
        refs_before: map_entry("HEAD", &switch_plan.source_head),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details,
        phase: None,
        commands: switch_commands(switch_plan),
        reversible: true,
    };
    begin(oplog, record, id)
}

pub(super) fn mark_pull_failed(
    oplog: &Oplog,
    id: &str,
    remote_ref: &str,
    carry_reference: Option<&str>,
) -> Result<(), SwitchError> {
    let mut snapshots = BTreeMap::new();
    if let Some(reference) = carry_reference {
        snapshots.insert("carry".to_string(), reference.to_string());
    }
    oplog
        .update_phase(id, QuickSwitchPhase::PullFastForwardFailed.as_str(), snapshots)
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let _ = remote_ref;
    Ok(())
}

pub(super) fn active_pull_decision(oplog: &Oplog) -> Result<Option<PullContext>, SwitchError> {
    let record = oplog
        .active("quick-switch")
        .map_err(|error| SwitchError::Recording(error.to_string()))?;
    let Some(record) = record else {
        return Ok(None);
    };
    let phase = record.phase.as_deref().and_then(QuickSwitchPhase::parse);
    if phase != Some(QuickSwitchPhase::PullFastForwardFailed) {
        return Ok(None);
    }
    Ok(Some(PullContext {
        id: record.id.clone(),
        source_branch: detail(&record, "source_branch")?,
        target_branch: detail(&record, "target_branch")?,
        remote_ref: detail(&record, "remote_ref")?,
        carry_reference: record.snapshots.get("carry").cloned(),
    }))
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

fn switch_commands(switch_plan: &QuickSwitchPlan) -> Vec<String> {
    let mut commands = Vec::new();
    if switch_plan.has_tracked_changes && switch_plan.carry_changes {
        commands.push("git stash push -m \"git-helper carry\"".to_string());
    } else if switch_plan.has_tracked_changes {
        commands.push("git stash create".to_string());
        commands.push(format!(
            "git update-ref {} <snapshot>",
            switch_plan.saved_work_reference
        ));
        commands.push("git reset --hard HEAD".to_string());
    }
    if let Some(remote) = &switch_plan.create_from_remote {
        let start = remote
            .strip_prefix("refs/remotes/")
            .unwrap_or(remote.as_str());
        commands.push(format!(
            "git switch --track -c {} {}",
            switch_plan.target_branch, start
        ));
    } else {
        commands.push(format!(
            "git switch --no-guess -- {}",
            switch_plan.target_branch
        ));
    }
    if let Some(remote) = &switch_plan.pull_remote_ref {
        let short = remote
            .strip_prefix("refs/remotes/")
            .unwrap_or(remote.as_str());
        let (remote_name, branch) = short.split_once('/').unwrap_or(("origin", short));
        commands.push(format!("git pull --ff-only {remote_name} {branch}"));
    }
    if switch_plan.has_tracked_changes && switch_plan.carry_changes {
        commands.push("git stash pop --index".to_string());
        commands.push("git stash pop  # fallback".to_string());
    }
    commands
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

fn detail(record: &OperationRecord, key: &str) -> Result<String, SwitchError> {
    record
        .details
        .get(key)
        .cloned()
        .ok_or_else(|| SwitchError::InvalidState(format!("quick-switch detail missing: {key}")))
}

fn map_entry(key: &str, value: &ObjectId) -> BTreeMap<String, String> {
    BTreeMap::from([(key.to_string(), value.to_string())])
}
