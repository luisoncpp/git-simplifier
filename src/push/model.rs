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
