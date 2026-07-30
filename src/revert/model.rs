use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName, RepoPath};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevertTarget {
    Head,
    Base,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RevertRequest {
    pub base: RefName,
    pub paths: Vec<RepoPath>,
    pub target: RevertTarget,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RevertPlan {
    pub paths: Vec<RepoPath>,
    pub target: RevertTarget,
    pub source: String,
    pub base_ref: RefName,
    pub commands: Vec<String>,
    pub(crate) source_head: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RevertResult {
    pub paths: Vec<RepoPath>,
    pub source: String,
}
