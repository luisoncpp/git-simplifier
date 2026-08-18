use git_helper_core::{
    CleanupPlan, CommitMergePlan, ExcludeSubmodulePlan, ForcePushPlan, HistorySwitchPlan,
    HistorySwitchResult, ObjectId, PublishBranchPlan, QuickSwitchPlan, QuickSwitchResult, RefName,
    RevertPlan, RewritePlan, SplitBranchPlan, SubmoduleCleanupPlan, SyncPhase, SyncRequest,
    SyncResult,
};

use super::data::{OperationOutcome, PendingOperation};
use super::repository::with_repository;
use super::state::AppState;

pub(super) fn apply(
    state: &AppState,
    operation: PendingOperation,
) -> Result<OperationOutcome, String> {
    match operation {
        PendingOperation::Uncommit { plan, .. } | PendingOperation::EditMessage { plan, .. } => {
            rewrite(state, plan)
        }
        PendingOperation::Revert { plan, .. } => revert(state, plan),
        PendingOperation::Exclude { plan, .. } => exclude(state, plan),
        PendingOperation::SubmoduleCleanup { plan, .. } => submodule_cleanup(state, plan),
        PendingOperation::Split { plan, .. } => split_branch(state, plan),
        PendingOperation::Publish { plan, .. } => publish_branch(state, plan),
        PendingOperation::QuickSwitch { plan, .. } => quick_switch(state, plan),
        PendingOperation::HistorySwitch { plan, .. } => history_switch(state, plan),
        PendingOperation::ResolveQuickSwitchPull { resolution, .. } => {
            resolve_pull(state, resolution)
        }
        PendingOperation::ForcePush { plan, .. } => force_push(state, plan),
        PendingOperation::Cleanup { plan, .. } => cleanup(state, plan),
        PendingOperation::Sync { base, head, .. } => sync(state, base, head),
        PendingOperation::Restore { head, .. } => restore(state, head),
        PendingOperation::Delete { branch, head, .. } => delete(state, branch, head),
        PendingOperation::Resume { operation_id, .. } => resume(state, operation_id),
        PendingOperation::CommitMerge { plan, head, .. } => commit_merge(state, plan, head),
    }
}

fn rewrite(state: &AppState, plan: RewritePlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_rewrite(&plan).map_err(|e| e.to_string())
    })?;
    let mut details = vec![format!("{} now points at {}", plan.branch, result.new_head)];
    if !result.dropped_commits.is_empty() {
        details.push(format!(
            "{} commit(s) were dropped entirely",
            result.dropped_commits.len()
        ));
    }
    let mut outcome = bare("rewrite", "History rewritten", details);
    outcome.offer_force_push = true;
    Ok(outcome)
}

fn revert(state: &AppState, plan: RevertPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_revert(&plan).map_err(|e| e.to_string())
    })?;
    Ok(bare(
        "revert",
        "Paths reverted",
        vec![format!(
            "{} path(s) now match {} in the index and working tree",
            result.paths.len(),
            result.source
        )],
    ))
}

fn exclude(state: &AppState, plan: ExcludeSubmodulePlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_exclude_submodule(&plan)
            .map_err(|e| e.to_string())
    })?;
    let mut details = Vec::new();
    if result.config_changed {
        details.push(format!("{} is hidden from local status", result.path));
    }
    if result.hook_changed {
        details.push(format!(
            "Commit guard installed in {}",
            plan.hook_path.display()
        ));
    }
    if details.is_empty() {
        details.push("The exclusion was already in place".to_string());
    }
    Ok(bare("exclude_submodule", "Submodule excluded", details))
}

fn submodule_cleanup(
    state: &AppState,
    plan: SubmoduleCleanupPlan,
) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_submodule_cleanup(&plan).map_err(|e| e.to_string())
    })?;
    let mut details = Vec::new();
    if result.uncommitted > 0 {
        details.push(format!(
            "{} submodule pointer(s) removed from the Editable range",
            result.uncommitted
        ));
    }
    if result.reverted > 0 {
        details.push(format!(
            "{} submodule checkout(s) aligned to HEAD",
            result.reverted
        ));
    }
    let mut outcome = bare("cleanup_submodules", "Submodules cleaned up", details);
    if result.uncommitted > 0 {
        outcome.offer_force_push = true;
    }
    Ok(outcome)
}

fn split_branch(state: &AppState, plan: SplitBranchPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_split_branch(&plan).map_err(|e| e.to_string())
    })?;
    let mut outcome = bare(
        "split_branch",
        "Branch created",
        vec![
            format!("{} points at {}", result.branch, result.commit),
            format!(
                "{} still carries the same {} file(s)",
                plan.source_branch,
                result.paths.len()
            ),
        ],
    );
    outcome.offer_publish_branch = Some(result.branch.clone());
    Ok(outcome)
}

fn publish_branch(state: &AppState, plan: PublishBranchPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_publish_branch(&plan).map_err(|e| e.to_string())
    })?;
    Ok(bare(
        "publish_branch",
        "Branch published",
        vec![
            format!("{} now exists on {}", plan.branch_name, result.remote),
            format!("{} tracks {}", plan.branch_name, result.upstream),
        ],
    ))
}

fn quick_switch(state: &AppState, plan: QuickSwitchPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_quick_switch(&plan).map_err(|e| e.to_string())
    })?;
    let offer_restore = result.target_saved_work.is_some() && !result.pull_decision_needed;
    let has_warning = result.carry_warning.is_some()
        || result.pull_warning.is_some()
        || result.untracked_merge_warning.is_some();
    let headline = if result.pull_decision_needed {
        "Pull needs a decision"
    } else if has_warning {
        "Branch switched with conflicts"
    } else {
        "Branch switched"
    };
    let mut outcome = bare("quick_switch", headline, switch_details(&result));
    outcome.offer_resolve_pull = result.pull_decision_needed;
    outcome.offer_restore_saved_work = offer_restore;
    outcome.has_warning = has_warning;
    Ok(outcome)
}

fn history_switch(
    state: &AppState,
    plan: HistorySwitchPlan,
) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_history_switch(&plan).map_err(|e| e.to_string())
    })?;
    let has_warning = result.carry_warning.is_some() || result.untracked_merge_warning.is_some();
    let headline = if has_warning {
        "History switch completed with conflicts"
    } else {
        "Now in History"
    };
    let mut outcome = bare("history", headline, history_details(&result));
    outcome.offer_switch_to_present = Some(result.present_branch);
    outcome.has_warning = has_warning;
    Ok(outcome)
}

fn history_details(result: &HistorySwitchResult) -> Vec<String> {
    let mut details = vec![
        format!("HEAD is at {}", result.target_commit),
        format!("{} remains at present", result.present_branch),
    ];
    if let Some(saved) = &result.saved_work {
        details.push(format!("Tracked changes saved for {}", saved.branch));
    }
    if result.carried_index.is_some() && result.carry_warning.is_none() {
        details.push("Tracked changes were carried onto this commit".to_string());
    }
    if let Some(warning) = &result.carry_warning {
        details.push(warning.clone());
    }
    if let Some(warning) = &result.untracked_merge_warning {
        details.push(warning.clone());
    }
    details
}

fn resolve_pull(
    state: &AppState,
    resolution: git_helper_core::PullResolution,
) -> Result<OperationOutcome, String> {
    let (result, offer_restore) = with_repository(state, |repo| {
        let result = repo
            .resolve_quick_switch_pull(resolution)
            .map_err(|e| e.to_string())?;
        let offer_restore = repo
            .list_saved_work()
            .map_err(|e| e.to_string())?
            .iter()
            .any(|saved| saved.branch == result.target_branch);
        Ok((result, offer_restore))
    })?;
    let mut details = resolve_details(&result);
    if offer_restore {
        details.push(format!(
            "Saved work for {} is ready to restore",
            result.target_branch
        ));
    }
    let has_warning = result.carry_warning.is_some()
        || result.pull_warning.is_some()
        || result.untracked_merge_warning.is_some();
    let headline = if has_warning {
        "Pull decision applied with conflicts"
    } else {
        "Pull decision applied"
    };
    let mut outcome = bare("resolve_quick_switch_pull", headline, details);
    outcome.offer_restore_saved_work = offer_restore;
    outcome.has_warning = has_warning;
    Ok(outcome)
}

fn switch_details(result: &QuickSwitchResult) -> Vec<String> {
    let mut details = vec![format!("Now on {}", result.target_branch)];
    if let Some(saved) = &result.saved_work {
        details.push(format!("Tracked changes saved for {}", saved.branch));
    }
    if result.pulled {
        details.push("Fast-forwarded from the remote-tracking branch".to_string());
    }
    if result.carried_index.is_some() && result.carry_warning.is_none() {
        details.push(format!(
            "Tracked changes carried onto {}",
            result.target_branch
        ));
    }
    push_warnings(&mut details, result);
    if result.target_saved_work.is_some() {
        details.push(format!(
            "Saved work for {} is ready to restore",
            result.target_branch
        ));
    }
    details
}

fn resolve_details(result: &QuickSwitchResult) -> Vec<String> {
    let mut details = vec![format!("Still on {}", result.target_branch)];
    if result.pulled {
        details.push("Local branch now matches the chosen remote update".to_string());
    }
    if let Some(warning) = &result.pull_warning {
        details.push(warning.clone());
    }
    if result.carried_index.is_some() && result.carry_warning.is_none() {
        details.push("Carried changes were reapplied".to_string());
    }
    if let Some(warning) = &result.carry_warning {
        details.push(warning.clone());
    }
    if let Some(warning) = &result.untracked_merge_warning {
        details.push(warning.clone());
    }
    details
}

fn push_warnings(details: &mut Vec<String>, result: &QuickSwitchResult) {
    if let Some(warning) = &result.carry_warning {
        details.push(warning.clone());
    }
    if let Some(warning) = &result.pull_warning {
        details.push(warning.clone());
    }
    if let Some(warning) = &result.untracked_merge_warning {
        details.push(warning.clone());
    }
}

fn force_push(state: &AppState, plan: ForcePushPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_force_push(&plan).map_err(|e| e.to_string())
    })?;
    Ok(bare(
        "force_push",
        "Force push completed",
        vec![format!(
            "{} on {} now points at {}",
            result.branch, result.remote, result.new_head
        )],
    ))
}

fn cleanup(state: &AppState, plan: CleanupPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_cleanup(&plan).map_err(|e| e.to_string())
    })?;
    let mut details = vec![format!(
        "{} local branch(es) deleted",
        result.deleted_local.len()
    )];
    if !result.deleted_remote.is_empty() {
        details.push(format!(
            "{} branch(es) deleted on their remotes",
            result.deleted_remote.len()
        ));
    }
    if !result.kept_remotes.is_empty() {
        details.push(format!(
            "{} remote branch(es) were left in place",
            result.kept_remotes.len()
        ));
    }
    details.push("Local deletions can be restored from the Recovery panel".to_string());
    Ok(bare("cleanup", "Branches deleted", details))
}

fn sync(state: &AppState, base: RefName, head: ObjectId) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "sync")?;
    let result = with_repository(state, |repo| {
        repo.sync(SyncRequest { base }).map_err(|e| e.to_string())
    })?;
    Ok(sync_outcome("sync", "Sync completed", result))
}

/// Saved work that never reached the tree must not read as a clean success:
/// the ref is the only remaining copy, so the banner has to name it.
fn sync_outcome(kind: &str, headline: &str, result: SyncResult) -> OperationOutcome {
    let mut details = vec![format!("HEAD now points at {}", result.new_head)];
    let Some(warning) = result.saved_work_warning else {
        return bare(kind, headline, details);
    };
    details.push(warning);
    let mut outcome = bare(kind, "Sync finished without restoring Saved work", details);
    outcome.has_warning = true;
    outcome
}

fn restore(state: &AppState, head: ObjectId) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "restore")?;
    let result = with_repository(state, |repo| {
        repo.restore_saved_work().map_err(|e| e.to_string())
    })?;
    let mut details = vec![format!(
        "Saved work for {} is back in the working tree",
        result.branch
    )];
    if !result.applied_index {
        details.push("The staged split could not be restored; everything is unstaged".to_string());
    }
    let Some(warning) = result.warning else {
        return Ok(bare("restore_saved_work", "Saved work restored", details));
    };
    details.push(warning);
    let mut outcome = bare(
        "restore_saved_work",
        "Saved work restored with conflicts",
        details,
    );
    outcome.has_warning = true;
    Ok(outcome)
}

fn delete(state: &AppState, branch: String, head: ObjectId) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "delete")?;
    let result = with_repository(state, |repo| {
        repo.delete_saved_work(branch).map_err(|e| e.to_string())
    })?;
    Ok(bare(
        "delete_saved_work",
        "Saved work deleted",
        vec![format!("Removed {}", result.reference)],
    ))
}

fn resume(state: &AppState, operation_id: String) -> Result<OperationOutcome, String> {
    let status = with_repository(state, |repo| repo.sync_status().map_err(|e| e.to_string()))?;
    if status.as_ref().map(|value| value.operation_id.as_str()) != Some(operation_id.as_str()) {
        return Err("the recorded sync changed since the review; prepare again".to_string());
    }
    let result = with_repository(state, |repo| repo.resume_sync().map_err(|e| e.to_string()))?;
    Ok(sync_outcome("resume_sync", "Sync finished", result))
}

fn commit_merge(
    state: &AppState,
    plan: CommitMergePlan,
    head: ObjectId,
) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "commit merge")?;
    let result = with_repository(state, |repo| {
        repo.apply_commit_merge(&plan).map_err(|e| e.to_string())
    })?;
    let mut details = vec![format!("HEAD now points at {}", result.new_head)];
    if !result.excluded_paths.is_empty() {
        details.push(format!(
            "{} unrelated path(s) stayed uncommitted",
            result.excluded_paths.len()
        ));
    }
    let mut outcome = bare("commit_merge", "Merge committed", details);
    let status = with_repository(state, |repo| repo.sync_status().map_err(|e| e.to_string()))?;
    if status
        .as_ref()
        .is_some_and(|value| value.phase == SyncPhase::BaseMergeConflict)
    {
        outcome.offer_resume_sync = true;
    }
    Ok(outcome)
}

fn ensure_unchanged(state: &AppState, head: &ObjectId, label: &str) -> Result<(), String> {
    let current = with_repository(state, |repo| {
        Ok(repo.overview().map_err(|e| e.to_string())?.head)
    })?;
    if &current != head {
        return Err(format!(
            "the repository changed since the {label} review; prepare again"
        ));
    }
    Ok(())
}

fn bare(kind: &str, headline: &str, details: Vec<String>) -> OperationOutcome {
    OperationOutcome {
        kind: kind.to_string(),
        headline: headline.to_string(),
        details,
        offer_force_push: false,
        offer_publish_branch: None,
        offer_resolve_pull: false,
        offer_restore_saved_work: false,
        offer_resume_sync: false,
        offer_switch_to_present: None,
        has_warning: false,
    }
}
