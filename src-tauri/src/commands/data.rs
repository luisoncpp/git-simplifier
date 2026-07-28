use git_helper_core::{ObjectId, RefName};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct RepositorySnapshot {
    pub overview: git_helper_core::RepositoryOverview,
    pub saved_work_count: usize,
    pub operation_count: usize,
    pub sync_in_progress: bool,
    pub saved_work: Vec<git_helper_core::SavedWork>,
    pub operations: Vec<git_helper_core::RecoveryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct BaseRequest {
    pub base: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenRepositoryInput {
    pub path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrepareOperationRequest {
    Uncommit {
        base: String,
        paths: Vec<String>,
    },
    EditMessage {
        base: String,
        commit: String,
        message: String,
    },
    ExcludeSubmodule {
        path: String,
        install_hook: bool,
        disable_recurse: bool,
    },
    QuickSwitch {
        target_branch: String,
    },
    Sync {
        base: String,
    },
    RestoreSavedWork,
    DeleteSavedWork {
        branch: String,
        snapshot: Option<String>,
    },
    ResumeSync,
    ForcePush,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationReview {
    pub plan_id: String,
    pub kind: String,
    pub title: String,
    pub impact: Vec<String>,
    pub preserves: Vec<String>,
    pub warnings: Vec<String>,
    pub commands: Vec<String>,
    pub apply_label: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationOutcome {
    pub kind: String,
    pub headline: String,
    pub details: Vec<String>,
    pub offer_force_push: bool,
}

#[derive(Clone, Debug)]
pub enum PendingOperation {
    Uncommit {
        id: String,
        plan: git_helper_core::RewritePlan,
    },
    EditMessage {
        id: String,
        plan: git_helper_core::RewritePlan,
    },
    Exclude {
        id: String,
        plan: git_helper_core::ExcludeSubmodulePlan,
    },
    QuickSwitch {
        id: String,
        plan: git_helper_core::QuickSwitchPlan,
    },
    ForcePush {
        id: String,
        plan: git_helper_core::ForcePushPlan,
    },
    Sync {
        id: String,
        base: RefName,
        head: ObjectId,
    },
    Restore {
        id: String,
        head: ObjectId,
    },
    Delete {
        id: String,
        branch: String,
        head: ObjectId,
    },
    Resume {
        id: String,
        operation_id: String,
    },
}

impl PendingOperation {
    pub fn id(&self) -> &str {
        match self {
            Self::Uncommit { id, .. }
            | Self::EditMessage { id, .. }
            | Self::Exclude { id, .. }
            | Self::QuickSwitch { id, .. }
            | Self::ForcePush { id, .. }
            | Self::Sync { id, .. }
            | Self::Restore { id, .. }
            | Self::Delete { id, .. }
            | Self::Resume { id, .. } => id,
        }
    }
}
