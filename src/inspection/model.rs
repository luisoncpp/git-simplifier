use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName, RepoPath, Signature};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorktreeSummary {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicts: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepositoryOverview {
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
    pub base: Option<RefName>,
    pub upstream: Option<RefName>,
    pub head: ObjectId,
    pub git_version: String,
    pub worktree: WorktreeSummary,
    pub saved_work_count: usize,
    pub recovery_count: usize,
    pub sync_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quick_switch_status: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RemoteBaseChoice {
    pub reference: RefName,
    pub display: String,
    pub head: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChangedPath {
    pub path: RepoPath,
    pub previous_path: Option<RepoPath>,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EditableCommit {
    pub id: ObjectId,
    pub short_id: String,
    pub subject: String,
    pub message: String,
    pub author: Signature,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalBranchChoice {
    pub name: String,
    pub head: ObjectId,
    pub current: bool,
    pub saved_work: bool,
    /// When set, this is a remote-tracking branch with no same-named local;
    /// `name` is the local name created on switch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmoduleChoice {
    pub path: RepoPath,
    pub object: ObjectId,
    pub excluded: bool,
}
