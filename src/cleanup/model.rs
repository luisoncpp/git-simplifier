use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName};

/// Whether a row is a local branch (which may also own a server counterpart) or
/// a merged remote-tracking branch with no local at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupKind {
    Local,
    RemoteOnly,
}

/// The branch as it exists on a server. `remote_ref` is the name *there*, taken
/// from `%(upstream:remoteref)`, so a non-trivial refspec cannot mislead a delete.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteCounterpart {
    pub remote: String,
    pub tracking_ref: String,
    pub remote_ref: String,
    pub head: ObjectId,
    /// The counterpart is itself an ancestor of Base. Only these may be deleted.
    pub merged: bool,
}

/// One row of the checklist. `reference` is the identity the plan echoes back.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupChoice {
    pub branch: String,
    pub reference: String,
    pub head: ObjectId,
    pub kind: CleanupKind,
    pub author_email: String,
    pub mine: bool,
    /// A well-known shared name. Offered, but never ticked by default.
    pub protected: bool,
    pub remote: Option<RemoteCounterpart>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExclusionReason {
    CurrentBranch,
    CheckedOutInWorktree,
    BaseBranch,
    SavedWork,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupExclusion {
    pub branch: String,
    pub reason: ExclusionReason,
}

/// The maximal offerable set. The three display toggles filter this in the UI;
/// they are never sent to Git, so changing one costs no repository work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupDiscovery {
    pub base: RefName,
    pub base_head: ObjectId,
    pub identity: Option<String>,
    pub choices: Vec<CleanupChoice>,
    /// Branches a safety rule removed, so the UI can explain an absence.
    pub excluded: Vec<CleanupExclusion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupRequest {
    pub base: RefName,
    /// `CleanupChoice::reference` values the user left ticked.
    pub chosen: Vec<String>,
    pub include_remote_counterparts: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalDeletion {
    pub reference: String,
    pub head: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteDeletion {
    pub remote: String,
    pub remote_ref: String,
    pub tracking_ref: String,
    pub head: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupBranchPlan {
    pub branch: String,
    pub local: Option<LocalDeletion>,
    pub remote: Option<RemoteDeletion>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeptReason {
    /// The server branch has commits Base does not contain.
    NotMerged,
    NoUpstream,
    /// The user turned remote counterparts off.
    Disabled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeptRemote {
    pub branch: String,
    pub tracking_ref: String,
    pub reason: KeptReason,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupPlan {
    pub base: RefName,
    pub base_head: ObjectId,
    pub branches: Vec<CleanupBranchPlan>,
    pub kept_remotes: Vec<KeptRemote>,
    pub local_count: usize,
    pub remote_count: usize,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted_local: Vec<String>,
    pub deleted_remote: Vec<String>,
    pub kept_remotes: Vec<String>,
}
