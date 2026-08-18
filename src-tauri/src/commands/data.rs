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

#[derive(Clone, Debug, Deserialize)]
pub struct DirtySubmodulesRequest {
    #[serde(default)]
    pub base: Option<String>,
}

/// Inspection diff requests carry a compare mode; `BaseRequest` stays for Sync
/// and other actions that only need the ref. Local compare may send untracked
/// discovery filters so `ls-files` is constrained before the search, not after.
#[derive(Clone, Debug, Deserialize)]
pub struct DiffRequest {
    pub base: String,
    #[serde(default)]
    pub compare: git_helper_core::DiffCompare,
    #[serde(default)]
    pub untracked_filters: git_helper_core::UntrackedFilters,
}

/// The diff viewer expands one file at a time, which `DiffRequest` cannot say.
#[derive(Clone, Debug, Deserialize)]
pub struct FilePathRequest {
    pub base: String,
    pub path: String,
    #[serde(default)]
    pub compare: git_helper_core::DiffCompare,
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
pub struct RevertInput {
    pub base: String,
    pub paths: Vec<String>,
    pub target: String,
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

#[derive(Clone, Debug, Deserialize)]
pub struct CleanupSubmodulesInput {
    pub base: String,
    pub paths: Vec<String>,
    #[serde(default = "default_true")]
    pub uncommit: bool,
    #[serde(default = "default_true")]
    pub revert: bool,
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
pub struct PublishBranchInput {
    pub branch: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QuickSwitchInput {
    pub target_branch: String,
    #[serde(default)]
    pub carry_changes: bool,
    #[serde(default = "default_true")]
    pub pull_after_switch: bool,
    #[serde(default)]
    pub create_from_remote: Option<String>,
    #[serde(default)]
    pub merge_untracked: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HistorySwitchInput {
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub until: Option<String>,
    #[serde(default)]
    pub carry_changes: bool,
    #[serde(default)]
    pub merge_untracked: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResolvePullInput {
    pub resolution: git_helper_core::PullResolution,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DeleteSavedWorkInput {
    pub branch: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SavedWorkFilePathInput {
    pub path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CleanupInput {
    pub base: String,
    /// Full ref names of the rows the user left ticked. The planner re-derives
    /// eligibility from these rather than trusting any metadata sent with them.
    pub references: Vec<String>,
    #[serde(default = "default_true")]
    pub delete_remotes: bool,
}

/// Newtype variants keep the JSON payload flat (`{"kind": "uncommit", …}`)
/// while giving each prepare step a single typed input argument.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PrepareOperationRequest {
    Uncommit(UncommitInput),
    Revert(RevertInput),
    EditMessage(EditMessageInput),
    ExcludeSubmodule(ExcludeSubmoduleInput),
    CleanupSubmodules(CleanupSubmodulesInput),
    SplitBranch(SplitBranchInput),
    PublishBranch(PublishBranchInput),
    QuickSwitch(QuickSwitchInput),
    History(HistorySwitchInput),
    ResolveQuickSwitchPull(ResolvePullInput),
    Sync(BaseRequest),
    Cleanup(CleanupInput),
    RestoreSavedWork,
    DeleteSavedWork(DeleteSavedWorkInput),
    ResumeSync,
    CommitMerge,
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
pub struct OperationBlock {
    pub kind: String,
    pub message: String,
    pub paths: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PrepareResult {
    pub review: Option<OperationReview>,
    pub block: Option<OperationBlock>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationOutcome {
    pub kind: String,
    pub headline: String,
    pub details: Vec<String>,
    pub offer_force_push: bool,
    /// The branch a follow-up publish would push. A rewrite offers a force push;
    /// a freshly created branch offers its first push, and needs to name it.
    pub offer_publish_branch: Option<String>,
    /// When set, the quick-switch pull could not fast-forward and the user must choose.
    pub offer_resolve_pull: bool,
    /// When set, the current branch has Saved work ready for the restore review.
    pub offer_restore_saved_work: bool,
    /// When set, a Sync paused at base-merge-conflict can resume after the merge commit.
    #[serde(default)]
    pub offer_resume_sync: bool,
    /// When set, History left this branch as present; Quick switch returns there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offer_switch_to_present: Option<String>,
    /// Conflict or warning details are present; the result banner should not look like success.
    pub has_warning: bool,
}

#[derive(Clone, Debug)]
pub enum PendingOperation {
    Uncommit {
        id: String,
        plan: git_helper_core::RewritePlan,
    },
    Revert {
        id: String,
        plan: git_helper_core::RevertPlan,
    },
    EditMessage {
        id: String,
        plan: git_helper_core::RewritePlan,
    },
    Exclude {
        id: String,
        plan: git_helper_core::ExcludeSubmodulePlan,
    },
    SubmoduleCleanup {
        id: String,
        plan: git_helper_core::SubmoduleCleanupPlan,
    },
    Split {
        id: String,
        plan: git_helper_core::SplitBranchPlan,
    },
    Publish {
        id: String,
        plan: git_helper_core::PublishBranchPlan,
    },
    QuickSwitch {
        id: String,
        plan: git_helper_core::QuickSwitchPlan,
    },
    HistorySwitch {
        id: String,
        plan: git_helper_core::HistorySwitchPlan,
    },
    ResolveQuickSwitchPull {
        id: String,
        resolution: git_helper_core::PullResolution,
    },
    ForcePush {
        id: String,
        plan: git_helper_core::ForcePushPlan,
    },
    Cleanup {
        id: String,
        plan: git_helper_core::CleanupPlan,
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
    CommitMerge {
        id: String,
        plan: git_helper_core::CommitMergePlan,
        head: ObjectId,
    },
}

impl PendingOperation {
    pub fn id(&self) -> &str {
        match self {
            Self::Uncommit { id, .. }
            | Self::Revert { id, .. }
            | Self::EditMessage { id, .. }
            | Self::Exclude { id, .. }
            | Self::SubmoduleCleanup { id, .. }
            | Self::Split { id, .. }
            | Self::Publish { id, .. }
            | Self::QuickSwitch { id, .. }
            | Self::HistorySwitch { id, .. }
            | Self::ResolveQuickSwitchPull { id, .. }
            | Self::ForcePush { id, .. }
            | Self::Cleanup { id, .. }
            | Self::Sync { id, .. }
            | Self::Restore { id, .. }
            | Self::Delete { id, .. }
            | Self::Resume { id, .. } => id,
            Self::CommitMerge { id, .. } => id,
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
            r#"{"kind":"revert","base":"refs/remotes/origin/main","paths":["a.txt"],"target":"head"}"#,
            r#"{"kind":"edit_message","base":"refs/remotes/origin/main","commit":"abcdef1","message":"new"}"#,
            r#"{"kind":"exclude_submodule","path":"vendor/sdk","install_hook":true,"disable_recurse":false}"#,
            r#"{"kind":"cleanup_submodules","base":"refs/remotes/origin/main","paths":["Modules/Engine"],"uncommit":true,"revert":true}"#,
            r#"{"kind":"split_branch","base":"refs/remotes/origin/main","new_branch":"carved","paths":["a.txt"],"message":""}"#,
            r#"{"kind":"publish_branch","branch":"carved"}"#,
            r#"{"kind":"quick_switch","target_branch":"develop"}"#,
            r#"{"kind":"history","commit":"abcdef1"}"#,
            r#"{"kind":"history","until":"2021-01-01T00:00:00"}"#,
            r#"{"kind":"resolve_quick_switch_pull","resolution":"cancel"}"#,
            r#"{"kind":"sync","base":"refs/remotes/origin/main"}"#,
            r#"{"kind":"cleanup","base":"refs/remotes/origin/main","references":["refs/heads/spike"],"delete_remotes":true}"#,
            r#"{"kind":"restore_saved_work"}"#,
            r#"{"kind":"delete_saved_work","branch":"feature"}"#,
            r#"{"kind":"commit_merge"}"#,
            r#"{"kind":"resume_sync"}"#,
            r#"{"kind":"force_push"}"#,
        ];

        for payload in payloads {
            serde_json::from_str::<PrepareOperationRequest>(payload)
                .unwrap_or_else(|error| panic!("{payload} failed to deserialize: {error}"));
        }
    }
}
