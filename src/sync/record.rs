use std::collections::BTreeMap;

use crate::recording::{timestamp, OperationRecord, Oplog};
use crate::rewrite::{ObjectId, RefName};

use super::errors::SyncError;
use super::model::SyncPhase;
use super::state;

pub(crate) struct BeginInput {
    pub branch: String,
    pub base: RefName,
    pub source_head: ObjectId,
    pub base_before: Option<ObjectId>,
}

pub(crate) struct Context {
    pub id: String,
    pub branch: String,
    pub base: RefName,
    pub source_head: ObjectId,
    pub phase: SyncPhase,
    pub snapshot_reference: Option<String>,
}

pub(crate) struct PhaseUpdate {
    pub phase: SyncPhase,
    pub snapshot_reference: Option<String>,
}

pub(crate) fn begin(oplog: &Oplog, input: BeginInput) -> Result<String, SyncError> {
    let started = timestamp();
    let id = format!("sync-{started}-{}", std::process::id());
    let record = OperationRecord {
        id: id.clone(),
        operation: "sync".to_string(),
        started,
        finished: None,
        refs_before: refs_before(&input),
        refs_after: BTreeMap::new(),
        snapshots: BTreeMap::new(),
        details: BTreeMap::from([
            ("branch".to_string(), input.branch),
            ("base".to_string(), input.base.to_string()),
            ("source_head".to_string(), input.source_head.to_string()),
        ]),
        phase: Some(SyncPhase::Fetch.as_str().to_string()),
        commands: vec![
            format!("git fetch <remote> <branch>:{}", input.base),
            "git stash create".to_string(),
            "git reset --hard HEAD".to_string(),
            format!("git merge --no-edit {}", input.base),
            "git stash apply --index <saved-work>".to_string(),
        ],
        reversible: true,
    };
    oplog
        .begin(record)
        .map_err(|error| SyncError::Recording(error.to_string()))?;
    Ok(id)
}

pub(crate) fn update_phase(oplog: &Oplog, id: &str, update: PhaseUpdate) -> Result<(), SyncError> {
    let snapshots = update
        .snapshot_reference
        .map(|reference| BTreeMap::from([("wip".to_string(), reference)]))
        .unwrap_or_default();
    oplog
        .update_phase(id, update.phase.as_str(), snapshots)
        .map_err(|error| SyncError::Recording(error.to_string()))
}

pub(crate) fn finish(
    oplog: &Oplog,
    id: &str,
    after: BTreeMap<String, String>,
) -> Result<(), SyncError> {
    oplog
        .finish(id, after)
        .map_err(|error| SyncError::Recording(error.to_string()))
}

pub(crate) fn active(oplog: &Oplog) -> Result<Option<Context>, SyncError> {
    let record = oplog
        .active("sync")
        .map_err(|error| SyncError::Recording(error.to_string()))?;
    record.map(context).transpose()
}

fn context(record: OperationRecord) -> Result<Context, SyncError> {
    let base = RefName::new(detail(&record, "base")?).map_err(SyncError::InvalidState)?;
    let source_head =
        ObjectId::new(detail(&record, "source_head")?).map_err(SyncError::InvalidState)?;
    let phase = record
        .phase
        .as_deref()
        .and_then(SyncPhase::parse)
        .ok_or_else(|| SyncError::InvalidState("sync phase is missing or unknown".to_string()))?;
    let id = record.id.clone();
    Ok(Context {
        id,
        branch: detail(&record, "branch")?,
        base,
        source_head,
        phase,
        snapshot_reference: record.snapshots.get("wip").cloned(),
    })
}

fn detail(record: &OperationRecord, name: &str) -> Result<String, SyncError> {
    record
        .details
        .get(name)
        .cloned()
        .ok_or_else(|| SyncError::InvalidState(format!("sync record is missing {name}")))
}

fn refs_before(input: &BeginInput) -> BTreeMap<String, String> {
    let mut refs = BTreeMap::from([
        ("HEAD".to_string(), input.source_head.to_string()),
        (
            state::branch_ref(&input.branch),
            input.source_head.to_string(),
        ),
    ]);
    if let Some(base) = &input.base_before {
        refs.insert(input.base.to_string(), base.to_string());
    }
    refs
}
