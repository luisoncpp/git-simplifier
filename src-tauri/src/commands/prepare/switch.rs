use git_helper_core::{QuickSwitchRequest, SwitchError};

use super::super::data::{
    OperationBlock, OperationReview, PendingOperation, QuickSwitchInput, ResolvePullInput,
};
use super::super::repository::with_repository;
use super::super::review_commands;
use super::Prepared;
use crate::commands::state::AppState;

enum PlanOutcome {
    Plan(git_helper_core::QuickSwitchPlan),
    Block(OperationBlock),
}

pub(super) fn quick_switch(
    state: &AppState,
    id: String,
    input: QuickSwitchInput,
) -> Result<Prepared, String> {
    let request = QuickSwitchRequest {
        target_branch: input.target_branch,
        carry_changes: input.carry_changes,
        pull_after_switch: input.pull_after_switch,
        create_from_remote: input.create_from_remote,
        merge_untracked: input.merge_untracked,
    };
    let outcome = with_repository(state, |repo| match repo.plan_quick_switch(request) {
        Ok(plan) => Ok(PlanOutcome::Plan(plan)),
        Err(SwitchError::UntrackedOverlap(paths)) => Ok(PlanOutcome::Block(OperationBlock {
            kind: "untracked_overwrite".to_string(),
            message: "Untracked files would be overwritten on the target branch.".to_string(),
            paths,
        })),
        Err(error) => Err(error.to_string()),
    })?;
    match outcome {
        PlanOutcome::Plan(plan) => build_prepared(id, &plan),
        PlanOutcome::Block(block) => Ok(Prepared {
            review: None,
            pending: None,
            block: Some(block),
        }),
    }
}

pub(super) fn resolve_pull(
    state: &AppState,
    id: String,
    input: ResolvePullInput,
) -> Result<Prepared, String> {
    let status = with_repository(state, |repo| {
        repo.quick_switch_status()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "no interrupted quick-switch pull exists".to_string())
    })?;
    let review = resolution_review(&id, &status, &input.resolution);
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::ResolveQuickSwitchPull {
            id,
            resolution: input.resolution,
        }),
        block: None,
    })
}

fn build_prepared(id: String, plan: &git_helper_core::QuickSwitchPlan) -> Result<Prepared, String> {
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "quick_switch".to_string(),
        title: format!("Switch to {}", plan.target_branch),
        impact: switch_impact(plan),
        preserves: vec![
            "Untracked files and submodule checkouts".to_string(),
            format!("The commits on {}", plan.source_branch),
        ],
        warnings: switch_warnings(plan),
        commands: review_commands::quick_switch(plan),
        apply_label: "Switch branch".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::QuickSwitch { id, plan: plan.clone() }),
        block: None,
    })
}

fn switch_impact(plan: &git_helper_core::QuickSwitchPlan) -> Vec<String> {
    let mut impact = vec![format!(
        "Check out {} instead of {}",
        plan.target_branch, plan.source_branch
    )];
    if plan.create_from_remote.is_some() {
        impact.push(format!(
            "Create local {} tracking the remote-tracking branch",
            plan.target_branch
        ));
    }
    if plan.has_tracked_changes {
        if plan.carry_changes {
            impact.push(format!("Carry tracked changes onto {}", plan.target_branch));
        } else {
            impact.push(format!(
                "Store tracked changes as Saved work under {}",
                plan.saved_work_reference
            ));
        }
    }
    if !plan.untracked_conflicts.is_empty() {
        impact.push(format!(
            "Merge {} untracked file(s) with the target branch after checkout",
            plan.untracked_conflicts.len()
        ));
    }
    if let Some(remote) = &plan.pull_remote_ref {
        impact.push(format!("Fast-forward from {remote} after the switch"));
    }
    if plan.target_saved_work.is_some() {
        impact.push(format!(
            "Saved work for {} is waiting and stays untouched until you restore it",
            plan.target_branch
        ));
    }
    impact
}

fn switch_warnings(plan: &git_helper_core::QuickSwitchPlan) -> Vec<String> {
    let mut warnings = Vec::new();
    if !plan.untracked_conflicts.is_empty() {
        warnings.push(
            "Overlapping untracked files will be merged with the target branch and may leave \
             conflict markers."
                .to_string(),
        );
    }
    if plan.carry_changes && plan.has_tracked_changes {
        warnings.push(
            "Carry uses git stash push and stash pop. Conflicts are reported after the switch \
             instead of blocking the review."
                .to_string(),
        );
    }
    if plan.pull_remote_ref.is_some() {
        warnings.push(
            "If the fast-forward fails, you will choose whether to replace the local branch, \
             merge (possibly with conflicts), or cancel the pull."
                .to_string(),
        );
    }
    warnings
}

fn resolution_review(
    id: &str,
    status: &git_helper_core::QuickSwitchStatus,
    resolution: &git_helper_core::PullResolution,
) -> OperationReview {
    let (title, impact, commands, apply_label) = match resolution {
        git_helper_core::PullResolution::ReplaceWithRemote => (
            format!("Replace {} with remote", status.target_branch),
            vec![format!(
                "Reset {} hard to {}",
                status.target_branch, status.remote_ref
            )],
            vec![format!(
                "git reset --hard --no-recurse-submodules {}",
                status.remote_ref
            )],
            "Replace with remote".to_string(),
        ),
        git_helper_core::PullResolution::MergePull => (
            format!("Merge-pull into {}", status.target_branch),
            vec![format!(
                "Pull {} allowing merge conflicts",
                status.remote_ref
            )],
            vec![format!("git pull --no-rebase {}", status.remote_ref)],
            "Pull with merge".to_string(),
        ),
        git_helper_core::PullResolution::Cancel => (
            "Cancel pull update".to_string(),
            vec![format!(
                "Leave {} at its current commit and finish the switch",
                status.target_branch
            )],
            vec!["# skip pull and finish".to_string()],
            "Cancel pull".to_string(),
        ),
    };
    let mut warnings = Vec::new();
    if status.carry_reference.is_some() {
        warnings.push(
            "Carried changes are anchored and will be reapplied after this choice when safe."
                .to_string(),
        );
    }
    if status.untracked_merge_reference.is_some() {
        warnings.push(
            "Untracked overlap merge is anchored and will be reapplied after this choice when safe."
                .to_string(),
        );
    }
    OperationReview {
        plan_id: id.to_string(),
        kind: "resolve_quick_switch_pull".to_string(),
        title,
        impact,
        preserves: vec!["The checkout already on the target branch".to_string()],
        warnings,
        commands,
        apply_label,
    }
}
