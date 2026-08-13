use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName, RepoPath};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommitMergePlan {
    pub branch: RefName,
    pub source_head: ObjectId,
    pub merge_head: ObjectId,
    pub merge_base: ObjectId,
    pub tree: ObjectId,
    pub base: Option<RefName>,
    pub conflicted_paths: Vec<RepoPath>,
    pub excluded_paths: Vec<RepoPath>,
    pub pr_paths_before: Vec<RepoPath>,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CommitMergeResult {
    pub old_head: ObjectId,
    pub new_head: ObjectId,
    pub merge_head: ObjectId,
    pub excluded_paths: Vec<RepoPath>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MergeParents {
    pub base: ObjectId,
    pub ours: ObjectId,
    pub theirs: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexEntry {
    pub mode: String,
    pub object: ObjectId,
    pub path: RepoPath,
}
