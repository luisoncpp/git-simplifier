use serde::{Deserialize, Serialize};

use crate::rewrite::ObjectId;

use super::model::SavedWork;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistorySwitchRequest {
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub until: Option<String>,
    #[serde(default)]
    pub carry_changes: bool,
    #[serde(default)]
    pub merge_untracked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistorySwitchPlan {
    pub source_branch: String,
    pub source_head: ObjectId,
    pub target_commit: ObjectId,
    pub saved_work_reference: String,
    pub has_tracked_changes: bool,
    pub carry_changes: bool,
    pub untracked_conflicts: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistorySwitchResult {
    pub source_branch: String,
    pub target_commit: ObjectId,
    pub saved_work: Option<SavedWork>,
    pub carried_index: Option<bool>,
    pub carry_warning: Option<String>,
    pub untracked_merge_warning: Option<String>,
    pub present_branch: String,
}
