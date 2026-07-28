use git_helper_core::{ObjectId, RefName};
use serde::{Deserialize, Serialize};

/// `overview` already carries the Saved work count, recovery count, and sync
/// phase, so the snapshot does not repeat them under second names.
#[derive(Clone, Debug, Serialize)]
pub struct RepositorySnapshot {
    pub overview: git_helper_core::RepositoryOverview,
    pub saved_work: Vec<git_helper_core::SavedWork>,
    pub operations: Vec<git_helper_core::RecoveryEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BaseRequest {
    pub base: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenRepositoryInput {
    pub path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UncommitInput {
    pub base: String,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EditMessageInput {
    pub base: String,
    pub commit: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExcludeSubmoduleInput {
    pub path: String,
    pub install_hook: bool,
    pub disable_recurse: bool,
}

/// An empty `message` means the planner derives one; the review shows what it
/// chose, so the caller never has to guess.
#[derive(Clone, Debug, Deserialize)]
pub struct SplitBranchInput {
    pub base: String,
    pub new_branch: String,
    pub paths: Vec<String>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuickSwitchInput {
    pub target_branch: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeleteSavedWorkInput {
    pub branch: String,
}

/// Newtype variants keep the JSON payload flat (`{"kind": "uncommit", …}`)
/// while giving each prepare step a single typed input argument.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrepareOperationRequest {
    Uncommit(UncommitInput),
    EditMessage(EditMessageInput),
    ExcludeSubmodule(ExcludeSubmoduleInput),
    SplitBranch(SplitBranchInput),
    QuickSwitch(QuickSwitchInput),
    Sync(BaseRequest),
    RestoreSavedWork,
    DeleteSavedWork(DeleteSavedWorkInput),
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
    Split {
        id: String,
        plan: git_helper_core::SplitBranchPlan,
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
            | Self::Split { id, .. }
            | Self::QuickSwitch { id, .. }
            | Self::ForcePush { id, .. }
            | Self::Sync { id, .. }
            | Self::Restore { id, .. }
            | Self::Delete { id, .. }
            | Self::Resume { id, .. } => id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PrepareOperationRequest;

    /// The UI sends one flat object per operation. A tagged enum that stopped
    /// accepting these payloads would break every button at runtime only.
    #[test]
    fn every_operation_payload_the_ui_sends_deserializes() {
        let payloads = [
            r#"{"kind":"uncommit","base":"refs/remotes/origin/main","paths":["a.txt"]}"#,
            r#"{"kind":"edit_message","base":"refs/remotes/origin/main","commit":"abcdef1","message":"new"}"#,
            r#"{"kind":"exclude_submodule","path":"vendor/sdk","install_hook":true,"disable_recurse":false}"#,
            r#"{"kind":"split_branch","base":"refs/remotes/origin/main","new_branch":"carved","paths":["a.txt"],"message":""}"#,
            r#"{"kind":"quick_switch","target_branch":"develop"}"#,
            r#"{"kind":"sync","base":"refs/remotes/origin/main"}"#,
            r#"{"kind":"restore_saved_work"}"#,
            r#"{"kind":"delete_saved_work","branch":"feature"}"#,
            r#"{"kind":"resume_sync"}"#,
            r#"{"kind":"force_push"}"#,
        ];

        for payload in payloads {
            serde_json::from_str::<PrepareOperationRequest>(payload)
                .unwrap_or_else(|error| panic!("{payload} failed to deserialize: {error}"));
        }
    }
}
