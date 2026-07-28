use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::git::{GitError, GitRunner};

use super::{OperationRecord, Oplog};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEntry {
    pub id: String,
    pub operation: String,
    pub started: String,
    pub finished: Option<String>,
    pub refs_before: BTreeMap<String, String>,
    pub refs_after: BTreeMap<String, String>,
    pub snapshots: BTreeMap<String, String>,
    pub details: BTreeMap<String, String>,
    pub phase: Option<String>,
    pub commands: Vec<String>,
    pub reversible: bool,
    pub recovery_command: Option<String>,
}

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("could not locate the repository Git directory: {0}")]
    Git(#[from] GitError),
    #[error("could not read the operation log: {0}")]
    Recording(String),
}

pub(crate) fn list(runner: &GitRunner) -> Result<Vec<RecoveryEntry>, RecoveryError> {
    let oplog = Oplog::open_existing(&runner.git_dir()?);
    let records = oplog
        .entries()
        .map_err(|error| RecoveryError::Recording(error.to_string()))?;
    Ok(records.into_iter().map(RecoveryEntry::from).collect())
}

impl From<OperationRecord> for RecoveryEntry {
    fn from(record: OperationRecord) -> Self {
        let command = restore_refs(&record.refs_before);
        let recovery_command = (record.reversible && !command.is_empty()).then_some(command);
        Self {
            id: record.id,
            operation: record.operation,
            started: record.started,
            finished: record.finished,
            refs_before: record.refs_before,
            refs_after: record.refs_after,
            snapshots: record.snapshots,
            details: record.details,
            phase: record.phase,
            commands: record.commands,
            reversible: record.reversible,
            recovery_command,
        }
    }
}

fn restore_refs(refs: &BTreeMap<String, String>) -> String {
    refs.iter()
        .filter(|(name, _)| name.starts_with("refs/"))
        .map(|(name, value)| match value.is_empty() {
            // An empty recorded value means the ref did not exist before.
            true => format!("git update-ref -d {name}"),
            false => format!("git update-ref {name} {value}"),
        })
        .collect::<Vec<_>>()
        .join(" && ")
}
