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

/// Which side of an Inspection diff is compared against the merge base of Base
/// and HEAD.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffCompare {
    /// `Base...HEAD`: committed changes from the merge base to HEAD.
    #[default]
    Head,
    /// Merge base → working tree: committed branch work plus uncommitted tracked
    /// changes.
    Local,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileDiffStatus {
    Added,
    Deleted,
    Modified,
    /// Unreachable while the patch argv keeps `--no-renames`, which reports a
    /// rename as a delete plus an add.
    Renamed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLineKind {
    Context,
    Add,
    Del,
}

/// One changed file of a `Base...HEAD` patch.
///
/// `complete` is the contract the expansion query fulfils: true means `hunks`
/// already hold every line of the file, so a viewer has nothing left to fetch
/// and must stop offering context-expansion controls.
///
/// A mode change is not a status — a file can be modified and chmod'ed in the
/// same patch — so callers read "mode only" as empty `hunks` with differing
/// modes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileDiff {
    pub path: RepoPath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<RepoPath>,
    pub status: FileDiffStatus,
    /// Absent on the side where the file does not exist.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_mode: Option<String>,
    pub binary: bool,
    pub complete: bool,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiffHunk {
    /// Git's own convention: the start is 0 when the line count is 0.
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    /// The section heading after the closing `@@`, empty when Git printed none.
    pub heading: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub old_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_line: Option<u32>,
    /// Content without the `+`/`-`/space marker and without the line
    /// terminator. A CRLF file keeps its trailing `\r` here.
    pub text: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_newline: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}
