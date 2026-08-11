use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName, RepoPath, RewritePlan};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmoduleCleanupRequest {
    pub base: RefName,
    pub paths: Vec<RepoPath>,
    pub uncommit: bool,
    pub revert: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmoduleCleanupPlan {
    pub paths: Vec<RepoPath>,
    pub uncommit: bool,
    pub revert: bool,
    pub uncommit_paths: Vec<RepoPath>,
    pub revert_paths: Vec<RepoPath>,
    pub base_ref: RefName,
    pub commands: Vec<String>,
    pub(crate) uncommit_plan: Option<RewritePlan>,
    pub(crate) source_head: ObjectId,
}

impl SubmoduleCleanupPlan {
    pub fn uncommit_plan(&self) -> Option<&RewritePlan> {
        self.uncommit_plan.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SubmoduleCleanupResult {
    pub paths: Vec<RepoPath>,
    pub uncommitted: usize,
    pub reverted: usize,
}
