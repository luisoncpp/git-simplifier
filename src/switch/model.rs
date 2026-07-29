use serde::{Deserialize, Serialize};

use crate::rewrite::ObjectId;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchRequest {
    pub target_branch: String,
    #[serde(default)]
    pub carry_changes: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SavedWork {
    pub branch: String,
    pub reference: String,
    pub snapshot: ObjectId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchPlan {
    pub source_branch: String,
    pub source_head: ObjectId,
    pub target_branch: String,
    pub target_head: ObjectId,
    pub saved_work_reference: String,
    pub has_tracked_changes: bool,
    pub carry_changes: bool,
    pub target_saved_work: Option<SavedWork>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchResult {
    pub source_branch: String,
    pub target_branch: String,
    pub saved_work: Option<SavedWork>,
    pub carried_index: Option<bool>,
    pub target_saved_work: Option<SavedWork>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RestoreSavedWorkResult {
    pub branch: String,
    pub reference: String,
    pub applied_index: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeleteSavedWorkResult {
    pub branch: String,
    pub reference: String,
}
