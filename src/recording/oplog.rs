use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecordingError {
    #[error("operation log I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("operation log JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("operation log entry was not found: {0}")]
    Missing(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct OperationRecord {
    pub id: String,
    pub operation: String,
    pub started: String,
    #[serde(default)]
    pub finished: Option<String>,
    pub refs_before: BTreeMap<String, String>,
    pub refs_after: BTreeMap<String, String>,
    #[serde(default)]
    pub snapshots: BTreeMap<String, String>,
    #[serde(default)]
    pub details: BTreeMap<String, String>,
    #[serde(default)]
    pub phase: Option<String>,
    pub commands: Vec<String>,
    pub reversible: bool,
}

pub(crate) struct Oplog {
    path: PathBuf,
}

impl Oplog {
    pub(crate) fn open(git_dir: &Path) -> Result<Self, RecordingError> {
        let folder = git_dir.join("githelper");
        fs::create_dir_all(&folder)?;
        Ok(Self {
            path: folder.join("oplog.json"),
        })
    }

    pub(crate) fn open_existing(git_dir: &Path) -> Self {
        Self {
            path: git_dir.join("githelper").join("oplog.json"),
        }
    }

    pub(crate) fn begin(&self, record: OperationRecord) -> Result<(), RecordingError> {
        let mut records = self.read()?;
        records.push(record);
        self.write(&records)
    }

    pub(crate) fn entries(&self) -> Result<Vec<OperationRecord>, RecordingError> {
        self.read()
    }

    pub(crate) fn finish(
        &self,
        id: &str,
        after: BTreeMap<String, String>,
    ) -> Result<(), RecordingError> {
        let mut records = self.read()?;
        let record = records.iter_mut().find(|record| record.id == id);
        let Some(record) = record else {
            return Err(RecordingError::Missing(id.to_string()));
        };
        record.finished = Some(timestamp());
        record.refs_after = after;
        self.write(&records)
    }

    pub(crate) fn update_phase(
        &self,
        id: &str,
        phase: &str,
        snapshots: BTreeMap<String, String>,
    ) -> Result<(), RecordingError> {
        let mut records = self.read()?;
        let record = records.iter_mut().find(|record| record.id == id);
        let Some(record) = record else {
            return Err(RecordingError::Missing(id.to_string()));
        };
        record.phase = Some(phase.to_string());
        record.snapshots = snapshots;
        self.write(&records)
    }

    pub(crate) fn active(
        &self,
        operation: &str,
    ) -> Result<Option<OperationRecord>, RecordingError> {
        Ok(self
            .read()?
            .into_iter()
            .rev()
            .find(|record| record.operation == operation && record.finished.is_none()))
    }

    fn read(&self) -> Result<Vec<OperationRecord>, RecordingError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(&self.path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn write(&self, records: &[OperationRecord]) -> Result<(), RecordingError> {
        let bytes = serde_json::to_vec_pretty(records)?;
        fs::write(&self.path, bytes)?;
        Ok(())
    }
}

pub(crate) fn timestamp() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    nanos.to_string()
}
