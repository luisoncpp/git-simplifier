use git_helper_core::{HistorySwitchRequest, SwitchError};

use super::super::data::{
    HistorySwitchInput, OperationBlock, OperationReview, PendingOperation,
};
use super::super::repository::with_repository;
use super::super::review_commands;
use super::Prepared;
use crate::commands::state::AppState;

enum PlanOutcome {
    Plan(git_helper_core::HistorySwitchPlan),
    Block(OperationBlock),
}

pub(super) fn history_switch(
    state: &AppState,
    id: String,
    input: HistorySwitchInput,
) -> Result<Prepared, String> {
    let request = HistorySwitchRequest {
        commit: input.commit,
        until: input.until,
        carry_changes: input.carry_changes,
        merge_untracked: input.merge_untracked,
    };
    let outcome = with_repository(state, |repo| match repo.plan_history_switch(request) {
        Ok(plan) => Ok(PlanOutcome::Plan(plan)),
        Err(SwitchError::UntrackedOverlap(paths)) => Ok(PlanOutcome::Block(OperationBlock {
            kind: "untracked_overwrite".to_string(),
            message: "Untracked files would be overwritten at the target commit.".to_string(),
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

fn build_prepared(
    id: String,
    plan: &git_helper_core::HistorySwitchPlan,
) -> Result<Prepared, String> {
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "history".to_string(),
        title: format!("Switch to {}", plan.target_commit),
        impact: history_impact(plan),
        preserves: vec![
            format!("The branch pointer on {}", plan.source_branch),
            "Untracked files and submodule checkouts".to_string(),
        ],
        warnings: history_warnings(plan),
        commands: review_commands::history_switch(plan),
        apply_label: "Switch to commit".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::HistorySwitch { id, plan: plan.clone() }),
        block: None,
    })
}

fn history_impact(plan: &git_helper_core::HistorySwitchPlan) -> Vec<String> {
    let mut impact = vec![format!(
        "Detach HEAD at {} while {} stays at present",
        plan.target_commit, plan.source_branch
    )];
    if plan.has_tracked_changes {
        if plan.carry_changes {
            impact.push("Carry tracked changes onto the checked-out commit".to_string());
        } else {
            impact.push(format!(
                "Store tracked changes as Saved work under {}",
                plan.saved_work_reference
            ));
        }
    }
    if !plan.untracked_conflicts.is_empty() {
        impact.push(format!(
            "Merge {} untracked file(s) after checkout",
            plan.untracked_conflicts.len()
        ));
    }
    impact
}

fn history_warnings(plan: &git_helper_core::HistorySwitchPlan) -> Vec<String> {
    let mut warnings = Vec::new();
    if !plan.untracked_conflicts.is_empty() {
        warnings.push(
            "Overlapping untracked files will be merged with the target commit and may leave \
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
    warnings
}
