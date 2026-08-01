use serde::{Deserialize, Serialize};

use crate::rewrite::ObjectId;

fn default_pull_after_switch() -> bool {
    true
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchRequest {
    pub target_branch: String,
    #[serde(default)]
    pub carry_changes: bool,
    #[serde(default = "default_pull_after_switch")]
    pub pull_after_switch: bool,
    /// Short remote-tracking name such as `origin/feature`. When set, the local
    /// `target_branch` is created to track it.
    #[serde(default)]
    pub create_from_remote: Option<String>,
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
    pub pull_after_switch: bool,
    pub create_from_remote: Option<String>,
    /// Remote-tracking ref to fast-forward from after the switch, when present.
    pub pull_remote_ref: Option<String>,
    pub target_saved_work: Option<SavedWork>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum QuickSwitchPhase {
    PullFastForwardFailed,
}

impl QuickSwitchPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PullFastForwardFailed => "pull-ff-failed",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "pull-ff-failed" => Some(Self::PullFastForwardFailed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PullResolution {
    ReplaceWithRemote,
    MergePull,
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchStatus {
    pub operation_id: String,
    pub target_branch: String,
    pub remote_ref: String,
    pub phase: QuickSwitchPhase,
    pub carry_reference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuickSwitchResult {
    pub source_branch: String,
    pub target_branch: String,
    pub saved_work: Option<SavedWork>,
    pub carried_index: Option<bool>,
    pub carry_warning: Option<String>,
    pub target_saved_work: Option<SavedWork>,
    pub pulled: bool,
    pub pull_warning: Option<String>,
    /// Set when `git pull --ff-only` failed and the user must choose a resolution.
    pub pull_decision_needed: bool,
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SavedWorkApplyPreview {
    pub branch: String,
    pub on_current_branch: bool,
    pub before_tree: ObjectId,
    pub after_tree: ObjectId,
    pub worktree_conflicts: bool,
    pub index_conflicts: bool,
}
