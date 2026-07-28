use std::sync::atomic::{AtomicU64, Ordering};

use git_helper_core::{
    EditMessageRequest, ExcludeSubmoduleRequest, ObjectId, RefName, RepoPath, SyncPhase,
    SyncRequest, UncommitRequest,
};
use tauri::State;

use super::data::{
    BaseRequest, OpenRepositoryInput, OperationOutcome, OperationReview, PendingOperation,
    PrepareOperationRequest, RepositorySnapshot,
};
use super::repository::{self, with_repository};
use super::review_commands;
use super::state::AppState;

static NEXT_PLAN: AtomicU64 = AtomicU64::new(1);

fn plan_id() -> String {
    format!("op-{}", NEXT_PLAN.fetch_add(1, Ordering::Relaxed))
}
#[allow(clippy::too_many_arguments)]
fn review(
    id: String,
    kind: &str,
    title: &str,
    impact: Vec<String>,
    preserves: Vec<String>,
    warnings: Vec<String>,
    commands: Vec<String>,
    label: &str,
) -> OperationReview {
    OperationReview {
        plan_id: id,
        kind: kind.to_string(),
        title: title.to_string(),
        impact,
        preserves,
        warnings,
        commands,
        apply_label: label.to_string(),
    }
}

#[tauri::command]
pub fn app_ready() -> &'static str {
    "git-helper-ui-ready"
}

#[tauri::command(async)]
pub fn open_repository(
    state: State<'_, AppState>,
    request: OpenRepositoryInput,
) -> Result<RepositorySnapshot, String> {
    state.open_path(request.path.into());
    repository::snapshot(state.inner())
}

#[tauri::command(async)]
pub fn load_snapshot(state: State<'_, AppState>) -> Result<RepositorySnapshot, String> {
    repository::snapshot(state.inner())
}

#[tauri::command(async)]
pub fn list_operations(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::RecoveryEntry>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_operations()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_saved_work(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::SavedWork>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_saved_work()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_base_choices(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::RemoteBaseChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_base_choices()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn set_base(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<RepositorySnapshot, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository.set_base(base).map_err(|error| error.to_string())
    })?;
    repository::snapshot(state.inner())
}

#[tauri::command(async)]
pub fn list_changed_paths(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<Vec<git_helper_core::ChangedPath>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_changed_paths(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_editable_commits(
    state: State<'_, AppState>,
    request: BaseRequest,
) -> Result<Vec<git_helper_core::EditableCommit>, String> {
    let base = RefName::new(request.base).map_err(|error| error.to_string())?;
    with_repository(state.inner(), |repository| {
        repository
            .list_editable_commits(base)
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_local_branches(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::LocalBranchChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_local_branches()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn list_submodules(
    state: State<'_, AppState>,
) -> Result<Vec<git_helper_core::SubmoduleChoice>, String> {
    with_repository(state.inner(), |repository| {
        repository
            .list_submodules()
            .map_err(|error| error.to_string())
    })
}

#[tauri::command(async)]
pub fn prepare_operation(
    state: State<'_, AppState>,
    request: PrepareOperationRequest,
) -> Result<OperationReview, String> {
    let id = plan_id();
    match request {
        PrepareOperationRequest::Uncommit { base, paths } => {
            let base_ref = RefName::new(base).map_err(|e| e.to_string())?;
            let selected = paths
                .into_iter()
                .map(RepoPath::new)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            let plan = with_repository(state.inner(), |repo| {
                repo.plan_uncommit(UncommitRequest {
                    base: base_ref,
                    paths: selected,
                })
                .map_err(|e| e.to_string())
            })?;
            let commands = vec![format!("git rebase --rebase-merges {}", plan.base_ref)];
            let result = review(
                id.clone(),
                "uncommit",
                "Uncommit selected paths",
                vec![
                    format!("Rewrite {} commit(s)", plan.commits.len()),
                    format!("Drop {} commit(s)", plan.dropped_commits.len()),
                ],
                vec!["Preserve unrelated worktree and index state".into()],
                vec!["This changes published history; review before applying.".into()],
                commands,
                "Apply uncommit",
            );
            state.set_pending(PendingOperation::Uncommit { id, plan })?;
            Ok(result)
        }
        PrepareOperationRequest::EditMessage {
            base,
            commit,
            message,
        } => {
            let request = EditMessageRequest {
                base: RefName::new(base).map_err(|e| e.to_string())?,
                commit: ObjectId::new(commit).map_err(|e| e.to_string())?,
                message: message.into_bytes(),
            };
            let plan = with_repository(state.inner(), |repo| {
                repo.plan_edit_message(request).map_err(|e| e.to_string())
            })?;
            let result = review(
                id.clone(),
                "edit_message",
                "Edit commit message",
                vec![format!("Rewrite {} commit(s)", plan.commits.len())],
                vec!["Preserve commit trees and unrelated work".into()],
                vec!["The commit SHA will change.".into()],
                vec!["git rebase --rebase-merges <base>".into()],
                "Apply message edit",
            );
            state.set_pending(PendingOperation::EditMessage { id, plan })?;
            Ok(result)
        }
        PrepareOperationRequest::ExcludeSubmodule {
            path,
            install_hook,
            disable_recurse,
        } => {
            let plan = with_repository(state.inner(), |repo| {
                repo.plan_exclude_submodule(ExcludeSubmoduleRequest {
                    path: RepoPath::new(path).map_err(|e| e.to_string())?,
                    install_hook,
                    disable_recurse,
                })
                .map_err(|e| e.to_string())
            })?;
            let result = review(
                id.clone(),
                "exclude_submodule",
                "Exclude submodule changes",
                plan.config_lines.clone(),
                vec!["Keep the gitlink staged for a separate commit".into()],
                vec![
                    "If cleanup rewrites history, it requires a second reviewed operation.".into(),
                ],
                vec![plan.staging_command.clone()],
                "Apply exclusion",
            );
            state.set_pending(PendingOperation::Exclude { id, plan })?;
            Ok(result)
        }
        PrepareOperationRequest::QuickSwitch { target_branch } => {
            let plan = with_repository(state.inner(), |repo| {
                repo.plan_quick_switch(git_helper_core::QuickSwitchRequest { target_branch })
                    .map_err(|e| e.to_string())
            })?;
            let mut impact = vec![format!(
                "Switch from {} to {}",
                plan.source_branch, plan.target_branch
            )];
            if plan.has_tracked_changes {
                impact.push("Save tracked work before switching".into());
            }
            if plan.target_saved_work.is_some() {
                impact.push("Saved work is available after arrival".into());
            }
            let commands = review_commands::quick_switch(&plan);
            let result = review(
                id.clone(),
                "quick_switch",
                "Switch local branch",
                impact,
                vec!["Do not overwrite the source branch".into()],
                vec![],
                commands,
                "Switch branch",
            );
            state.set_pending(PendingOperation::QuickSwitch { id, plan })?;
            Ok(result)
        }
        PrepareOperationRequest::ForcePush => {
            let plan = with_repository(state.inner(), |repo| {
                repo.plan_force_push().map_err(|e| e.to_string())
            })?;
            let result = review(
                id.clone(),
                "force_push",
                "Force push rewritten history",
                vec![format!("Update {}", plan.upstream)],
                vec![format!(
                    "Lease against observed remote SHA {}",
                    plan.expected_remote
                )],
                vec!["Only run this after the rewrite succeeds.".into()],
                vec![plan.command.clone()],
                "Force push with lease",
            );
            state.set_pending(PendingOperation::ForcePush { id, plan })?;
            Ok(result)
        }
        PrepareOperationRequest::Sync { base } => {
            let base_ref = RefName::new(base).map_err(|e| e.to_string())?;
            let (head, branch) = with_repository(state.inner(), |repo| {
                let o = repo.overview().map_err(|e| e.to_string())?;
                Ok((o.head, o.branch.unwrap_or_else(|| "HEAD".into())))
            })?;
            let commands = review_commands::sync(&base_ref)?;
            let result = review(
                id.clone(),
                "sync",
                "Sync with Base",
                vec![
                    format!("Fetch and merge {} into {}", base_ref, branch),
                    "Temporarily save and reapply tracked work".into(),
                ],
                vec!["Record a resumable phase if conflicts occur".into()],
                vec![
                    "The workbench will stay busy until sync finishes or needs resolution.".into(),
                ],
                commands,
                "Start sync",
            );
            state.set_pending(PendingOperation::Sync {
                id,
                base: base_ref,
                head,
            })?;
            Ok(result)
        }
        PrepareOperationRequest::RestoreSavedWork => {
            let (head, branch) = with_repository(state.inner(), |repo| {
                let o = repo.overview().map_err(|e| e.to_string())?;
                Ok((o.head, o.branch.unwrap_or_default()))
            })?;
            let result = review(
                id.clone(),
                "restore_saved_work",
                "Restore saved work",
                vec![format!("Apply the saved snapshot for {}", branch)],
                vec!["Keep the saved ref until restore succeeds".into()],
                vec![],
                vec!["git stash apply <saved-work-ref>".into()],
                "Restore saved work",
            );
            state.set_pending(PendingOperation::Restore { id, head })?;
            Ok(result)
        }
        PrepareOperationRequest::DeleteSavedWork {
            branch,
            snapshot: _snapshot,
        } => {
            let head = with_repository(state.inner(), |repo| {
                Ok(repo.overview().map_err(|e| e.to_string())?.head)
            })?;
            let result = review(
                id.clone(),
                "delete_saved_work",
                "Delete saved work",
                vec![format!("Delete the saved snapshot for {}", branch)],
                vec![],
                vec!["This removes the recovery ref and cannot restore the snapshot.".into()],
                vec![format!("git update-ref -d refs/githelper/saved/{}", branch)],
                "Delete saved work",
            );
            state.set_pending(PendingOperation::Delete { id, branch, head })?;
            Ok(result)
        }
        PrepareOperationRequest::ResumeSync => {
            let status = with_repository(state.inner(), |repo| {
                repo.sync_status().map_err(|e| e.to_string())
            })?
            .ok_or_else(|| "no sync operation needs resuming".to_string())?;
            let retrying_fetch = status.phase == SyncPhase::Fetch;
            if !retrying_fetch
                && !matches!(
                    status.phase,
                    SyncPhase::BaseMergeConflict | SyncPhase::WipReapplyConflict
                )
            {
                return Err(format!(
                    "sync interrupted during {}; inspect Recovery before continuing",
                    status.phase.as_str()
                ));
            }
            let (title, warning) = if retrying_fetch {
                (
                    "Retry sync",
                    "The remote must be reachable before retrying.",
                )
            } else {
                (
                    "Resume sync",
                    "Resolve conflicts in the worktree before resuming.",
                )
            };
            let result = review(
                /*id=*/ id.clone(),
                /*kind=*/ "resume_sync",
                /*title=*/ title,
                /*impact=*/
                vec![format!("Continue recorded {} phase", status.phase.as_str())],
                /*preserves=*/
                vec!["Keep the recorded operation until completion".into()],
                /*warnings=*/ vec![warning.into()],
                /*commands=*/
                vec![format!("git-helper sync resume {}", status.operation_id)],
                /*apply_label=*/ title,
            );
            state.set_pending(PendingOperation::Resume {
                id,
                operation_id: status.operation_id,
            })?;
            Ok(result)
        }
    }
}

#[tauri::command(async)]
pub fn cancel_operation(state: State<'_, AppState>, plan_id: String) -> Result<(), String> {
    state.cancel_pending(&plan_id)
}

#[tauri::command(async)]
pub fn apply_operation(
    state: State<'_, AppState>,
    plan_id: String,
) -> Result<OperationOutcome, String> {
    let operation = state.take_pending(&plan_id)?;
    let (kind, headline, details, force) = match operation {
        PendingOperation::Uncommit { plan, .. } | PendingOperation::EditMessage { plan, .. } => {
            let result = with_repository(state.inner(), |repo| {
                repo.apply_rewrite(&plan).map_err(|e| e.to_string())
            })?;
            (
                "rewrite",
                "Rewrite applied",
                vec![format!("New HEAD: {}", result.new_head)],
                true,
            )
        }
        PendingOperation::Exclude { plan, .. } => {
            let result = with_repository(state.inner(), |repo| {
                repo.apply_exclude_submodule(&plan)
                    .map_err(|e| e.to_string())
            })?;
            (
                "exclude_submodule",
                "Submodule exclusion applied",
                vec![format!("Config changed: {}", result.config_changed)],
                false,
            )
        }
        PendingOperation::QuickSwitch { plan, .. } => {
            let result = with_repository(state.inner(), |repo| {
                repo.apply_quick_switch(&plan).map_err(|e| e.to_string())
            })?;
            (
                "quick_switch",
                "Branch switched",
                vec![format!("Arrived on {}", result.target_branch)],
                false,
            )
        }
        PendingOperation::ForcePush { plan, .. } => {
            let result = with_repository(state.inner(), |repo| {
                repo.apply_force_push(&plan).map_err(|e| e.to_string())
            })?;
            (
                "force_push",
                "Force push completed",
                vec![format!(
                    "Remote {} updated to {}",
                    result.remote, result.new_head
                )],
                false,
            )
        }
        PendingOperation::Sync { base, head, .. } => {
            let current = with_repository(state.inner(), |repo| {
                Ok(repo.overview().map_err(|e| e.to_string())?.head)
            })?;
            if current != head {
                return Err("repository changed since sync review; prepare again".into());
            }
            let result = with_repository(state.inner(), |repo| {
                repo.sync(SyncRequest { base }).map_err(|e| e.to_string())
            })?;
            (
                "sync",
                "Sync completed",
                vec![format!("New HEAD: {}", result.new_head)],
                false,
            )
        }
        PendingOperation::Restore { head, .. } => {
            let current = with_repository(state.inner(), |repo| {
                Ok(repo.overview().map_err(|e| e.to_string())?.head)
            })?;
            if current != head {
                return Err("repository changed since restore review; prepare again".into());
            }
            let result = with_repository(state.inner(), |repo| {
                repo.restore_saved_work().map_err(|e| e.to_string())
            })?;
            (
                "restore_saved_work",
                "Saved work restored",
                vec![format!("Snapshot: {}", result.reference)],
                false,
            )
        }
        PendingOperation::Delete { branch, head, .. } => {
            let current = with_repository(state.inner(), |repo| {
                Ok(repo.overview().map_err(|e| e.to_string())?.head)
            })?;
            if current != head {
                return Err("repository changed since delete review; prepare again".into());
            }
            let result = with_repository(state.inner(), |repo| {
                repo.delete_saved_work(branch).map_err(|e| e.to_string())
            })?;
            (
                "delete_saved_work",
                "Saved work deleted",
                vec![format!("Removed {}", result.reference)],
                false,
            )
        }
        PendingOperation::Resume { operation_id, .. } => {
            let status = with_repository(state.inner(), |repo| {
                repo.sync_status().map_err(|e| e.to_string())
            })?;
            if status.as_ref().map(|s| s.operation_id.as_str()) != Some(operation_id.as_str()) {
                return Err("sync operation changed; prepare again".into());
            }
            let result = with_repository(state.inner(), |repo| {
                repo.resume_sync().map_err(|e| e.to_string())
            })?;
            (
                "resume_sync",
                "Sync resumed",
                vec![format!("New HEAD: {}", result.new_head)],
                false,
            )
        }
    };
    Ok(OperationOutcome {
        kind: kind.into(),
        headline: headline.into(),
        details,
        offer_force_push: force,
    })
}
