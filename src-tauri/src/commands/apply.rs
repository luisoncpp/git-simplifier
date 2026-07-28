use git_helper_core::{
    ExcludeSubmodulePlan, ForcePushPlan, ObjectId, QuickSwitchPlan, RefName, RewritePlan,
    SplitBranchPlan, SyncRequest,
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
        PendingOperation::Exclude { plan, .. } => exclude(state, plan),
        PendingOperation::Split { plan, .. } => split_branch(state, plan),
        PendingOperation::QuickSwitch { plan, .. } => quick_switch(state, plan),
        PendingOperation::ForcePush { plan, .. } => force_push(state, plan),
        PendingOperation::Sync { base, head, .. } => sync(state, base, head),
        PendingOperation::Restore { head, .. } => restore(state, head),
        PendingOperation::Delete { branch, head, .. } => delete(state, branch, head),
        PendingOperation::Resume { operation_id, .. } => resume(state, operation_id),
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
    Ok(OperationOutcome {
        kind: "rewrite".to_string(),
        headline: "History rewritten".to_string(),
        details,
        offer_force_push: true,
    })
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
    Ok(OperationOutcome {
        kind: "exclude_submodule".to_string(),
        headline: "Submodule excluded".to_string(),
        details,
        offer_force_push: false,
    })
}

fn split_branch(state: &AppState, plan: SplitBranchPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_split_branch(&plan).map_err(|e| e.to_string())
    })?;
    Ok(OperationOutcome {
        kind: "split_branch".to_string(),
        headline: "Branch created".to_string(),
        details: vec![
            format!("{} points at {}", result.branch, result.commit),
            format!(
                "{} still carries the same {} file(s)",
                plan.source_branch,
                result.paths.len()
            ),
        ],
        offer_force_push: false,
    })
}

fn quick_switch(state: &AppState, plan: QuickSwitchPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_quick_switch(&plan).map_err(|e| e.to_string())
    })?;
    let mut details = vec![format!("Now on {}", result.target_branch)];
    if let Some(saved) = &result.saved_work {
        details.push(format!("Tracked changes saved for {}", saved.branch));
    }
    if result.target_saved_work.is_some() {
        details.push(format!(
            "Saved work for {} is ready to restore",
            result.target_branch
        ));
    }
    Ok(OperationOutcome {
        kind: "quick_switch".to_string(),
        headline: "Branch switched".to_string(),
        details,
        offer_force_push: false,
    })
}

fn force_push(state: &AppState, plan: ForcePushPlan) -> Result<OperationOutcome, String> {
    let result = with_repository(state, |repo| {
        repo.apply_force_push(&plan).map_err(|e| e.to_string())
    })?;
    Ok(OperationOutcome {
        kind: "force_push".to_string(),
        headline: "Force push completed".to_string(),
        details: vec![format!(
            "{} on {} now points at {}",
            result.branch, result.remote, result.new_head
        )],
        offer_force_push: false,
    })
}

fn sync(state: &AppState, base: RefName, head: ObjectId) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "sync")?;
    let result = with_repository(state, |repo| {
        repo.sync(SyncRequest { base }).map_err(|e| e.to_string())
    })?;
    Ok(OperationOutcome {
        kind: "sync".to_string(),
        headline: "Sync completed".to_string(),
        details: vec![format!("HEAD now points at {}", result.new_head)],
        offer_force_push: false,
    })
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
    Ok(OperationOutcome {
        kind: "restore_saved_work".to_string(),
        headline: "Saved work restored".to_string(),
        details,
        offer_force_push: false,
    })
}

fn delete(state: &AppState, branch: String, head: ObjectId) -> Result<OperationOutcome, String> {
    ensure_unchanged(state, &head, "delete")?;
    let result = with_repository(state, |repo| {
        repo.delete_saved_work(branch).map_err(|e| e.to_string())
    })?;
    Ok(OperationOutcome {
        kind: "delete_saved_work".to_string(),
        headline: "Saved work deleted".to_string(),
        details: vec![format!("Removed {}", result.reference)],
        offer_force_push: false,
    })
}

fn resume(state: &AppState, operation_id: String) -> Result<OperationOutcome, String> {
    let status = with_repository(state, |repo| repo.sync_status().map_err(|e| e.to_string()))?;
    if status.as_ref().map(|value| value.operation_id.as_str()) != Some(operation_id.as_str()) {
        return Err("the recorded sync changed since the review; prepare again".to_string());
    }
    let result = with_repository(state, |repo| repo.resume_sync().map_err(|e| e.to_string()))?;
    Ok(OperationOutcome {
        kind: "resume_sync".to_string(),
        headline: "Sync finished".to_string(),
        details: vec![format!("HEAD now points at {}", result.new_head)],
        offer_force_push: false,
    })
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
