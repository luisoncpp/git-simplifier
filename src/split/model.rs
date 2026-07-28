use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName, RepoPath};

/// A Split branch request always copies: the source branch is never rewritten
/// and never receives a revert commit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SplitBranchRequest {
    pub base: RefName,
    pub new_branch: String,
    pub paths: Vec<RepoPath>,
    pub message: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SplitBranchPlan {
    pub source_branch: String,
    pub source_head: ObjectId,
    pub base_ref: RefName,
    pub base: ObjectId,
    pub merge_base: ObjectId,
    pub new_branch: String,
    pub new_branch_ref: String,
    /// The paths the caller asked for, normalized. May name directories.
    pub selected_paths: Vec<RepoPath>,
    /// Exact changed files the split will carry, selection plus companions.
    pub changed_paths: Vec<RepoPath>,
    /// Companion `.meta` files pulled in because their asset was selected,
    /// or assets pulled in because their `.meta` was selected.
    pub companion_paths: Vec<RepoPath>,
    pub message: Vec<u8>,
    pub message_is_derived: bool,
    pub commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SplitBranchResult {
    pub branch: String,
    pub reference: String,
    pub commit: ObjectId,
    pub merge_base: ObjectId,
    pub paths: Vec<RepoPath>,
}
