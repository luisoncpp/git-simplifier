use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::CleanupError;
use super::model::CleanupPlan;
use super::remote::{self, RemotePush};
use super::review;

/// Shared by both phases so the recovery panel can pair the records of one run.
pub(super) fn group_id() -> String {
    format!("cleanup-{}-{}", timestamp(), std::process::id())
}

/// A remote deletion is recorded as irreversible and with an empty `refs_before`,
/// so it fails **both** gates `recovery::restore_refs` checks. A remote-tracking
/// ref in `refs_before` would yield an `update-ref` that recreates a local
/// pointer to a server branch that no longer exists — a recovery command that
/// lies. The real repair goes in `details` instead.
pub(super) fn begin_remote(
    oplog: &Oplog,
    push: &RemotePush,
    group: &str,
) -> Result<String, CleanupError> {
    let started = timestamp();
    let id = format!("cleanup-remote-{started}-{}", std::process::id());
    begin(
        oplog,
        OperationRecord {
            id: id.clone(),
            operation: "cleanup-remote-branches".to_string(),
            started,
            finished: None,
            refs_before: BTreeMap::new(),
            refs_after: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            details: remote_details(push, group),
            phase: None,
            commands: vec![remote::push_command(push)],
            reversible: false,
        },
    )?;
    Ok(id)
}

/// Local deletions record their old SHA, which is exactly what
/// `recovery::restore_refs` turns into a working `git update-ref` restore. The
/// commits survive until `gc`, so that command really does bring the branch back.
pub(super) fn begin_local(
    oplog: &Oplog,
    plan: &CleanupPlan,
    group: &str,
) -> Result<String, CleanupError> {
    let started = timestamp();
    let id = format!("cleanup-local-{started}-{}", std::process::id());
    let refs_before = plan
        .branches
        .iter()
        .filter_map(|entry| entry.local.as_ref())
        .map(|local| (local.reference.clone(), local.head.to_string()))
        .collect::<BTreeMap<_, _>>();
    begin(
        oplog,
        OperationRecord {
            id: id.clone(),
            operation: "cleanup-local-branches".to_string(),
            started,
            finished: None,
            refs_before,
            refs_after: BTreeMap::new(),
            snapshots: BTreeMap::new(),
            details: BTreeMap::from([
                ("cleanup".to_string(), group.to_string()),
                ("base".to_string(), plan.base.to_string()),
                ("base-head".to_string(), plan.base_head.to_string()),
            ]),
            phase: None,
            commands: review::local_commands(plan),
            reversible: true,
        },
    )?;
    Ok(id)
}

pub(super) fn finish(oplog: &Oplog, id: &str) -> Result<(), CleanupError> {
    oplog
        .finish(id, BTreeMap::new())
        .map_err(|error| CleanupError::Recording(error.to_string()))
}

fn remote_details(push: &RemotePush, group: &str) -> BTreeMap<String, String> {
    let deleted = push
        .deletions
        .iter()
        .map(|entry| format!("{}@{}", entry.tracking_ref, entry.head))
        .collect::<Vec<_>>()
        .join(", ");
    let restore = push
        .deletions
        .iter()
        .map(|entry| {
            format!(
                "git push {} {}:{}",
                push.remote, entry.head, entry.remote_ref
            )
        })
        .collect::<Vec<_>>()
        .join(" && ");
    BTreeMap::from([
        ("cleanup".to_string(), group.to_string()),
        ("remote".to_string(), push.remote.clone()),
        ("deleted".to_string(), deleted),
        ("restore".to_string(), restore),
        (
            "irreversible".to_string(),
            "Deleting a branch on a server cannot be undone from this app.".to_string(),
        ),
    ])
}

fn begin(oplog: &Oplog, record: OperationRecord) -> Result<(), CleanupError> {
    oplog
        .begin(record)
        .map_err(|error| CleanupError::Recording(error.to_string()))
}
