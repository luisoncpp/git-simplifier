use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncRequest {
    pub base: RefName,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SyncPhase {
    Fetch,
    Snapshot,
    BaseMerge,
    BaseMergeConflict,
    WipReapply,
    WipReapplyConflict,
}

impl SyncPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fetch => "fetch",
            Self::Snapshot => "snapshot",
            Self::BaseMerge => "base-merge",
            Self::BaseMergeConflict => "base-merge-conflict",
            Self::WipReapply => "wip-reapply",
            Self::WipReapplyConflict => "wip-reapply-conflict",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "fetch" => Self::Fetch,
            "snapshot" => Self::Snapshot,
            "base-merge" => Self::BaseMerge,
            "base-merge-conflict" => Self::BaseMergeConflict,
            "wip-reapply" => Self::WipReapply,
            "wip-reapply-conflict" => Self::WipReapplyConflict,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncSnapshot {
    pub reference: String,
    pub snapshot: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncResult {
    pub branch: RefName,
    pub base: RefName,
    pub old_head: ObjectId,
    pub new_head: ObjectId,
    pub saved_work: Option<SyncSnapshot>,
    pub applied_index: bool,
    /// Set when the sync finished without the snapshot reaching the working
    /// tree. The snapshot ref is kept so the work stays recoverable.
    pub saved_work_warning: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncStatus {
    pub operation_id: String,
    pub branch: RefName,
    pub base: RefName,
    pub source_head: ObjectId,
    pub phase: SyncPhase,
    pub saved_work: Option<SyncSnapshot>,
}
