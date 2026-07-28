use serde::{Deserialize, Serialize};

use crate::rewrite::{ObjectId, RefName};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ForcePushPlan {
    pub branch: RefName,
    pub upstream: RefName,
    pub remote: String,
    pub remote_branch: RefName,
    pub expected_remote: ObjectId,
    pub source_head: ObjectId,
    pub command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ForcePushResult {
    pub branch: RefName,
    pub remote: String,
    pub new_head: ObjectId,
}

/// Publishing is the first push of a branch that the remote has never seen, so
/// it carries no lease against a previous value and no rewrite warning.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishBranchPlan {
    pub branch: RefName,
    pub branch_name: String,
    pub remote: String,
    pub remote_branch: RefName,
    pub upstream: RefName,
    pub source_head: ObjectId,
    pub command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PublishBranchResult {
    pub branch: RefName,
    pub remote: String,
    pub upstream: RefName,
    pub head: ObjectId,
}
