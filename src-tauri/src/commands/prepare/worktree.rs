use git_helper_core::{QuickSwitchRequest, RefName, SyncPhase};

use super::super::data::{BaseRequest, OperationReview, PendingOperation, QuickSwitchInput};
use super::super::repository::with_repository;
use super::super::review_commands;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn quick_switch(
    state: &AppState,
    id: String,
    input: QuickSwitchInput,
) -> Result<Prepared, String> {
    let request = QuickSwitchRequest {
        target_branch: input.target_branch,
        carry_changes: input.carry_changes,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_quick_switch(request).map_err(|e| e.to_string())
    })?;
    let mut impact = vec![format!(
        "Check out {} instead of {}",
        plan.target_branch, plan.source_branch
    )];
    if plan.has_tracked_changes {
        if plan.carry_changes {
            impact.push(format!(
                "Carry tracked changes onto {}",
                plan.target_branch
            ));
        } else {
            impact.push(format!(
                "Store tracked changes as Saved work under {}",
                plan.saved_work_reference
            ));
        }
    }
    if plan.target_saved_work.is_some() {
        impact.push(format!(
            "Saved work for {} is waiting and stays untouched until you restore it",
            plan.target_branch
        ));
    }
    let mut warnings = Vec::new();
    if plan.carry_changes && plan.has_tracked_changes {
        warnings.push(
            "Carry uses git stash push and stash pop. Conflicts are reported after the switch \
             instead of blocking the review."
                .to_string(),
        );
    }
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "quick_switch".to_string(),
        title: format!("Switch to {}", plan.target_branch),
        impact,
        preserves: vec![
            "Untracked files and submodule checkouts".to_string(),
            format!("The commits on {}", plan.source_branch),
        ],
        warnings,
        commands: review_commands::quick_switch(&plan),
        apply_label: "Switch branch".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::QuickSwitch { id, plan },
    })
}

pub(super) fn sync(state: &AppState, id: String, input: BaseRequest) -> Result<Prepared, String> {
    let base = RefName::new(input.base)?;
    let (head, branch) = with_repository(state, |repo| {
        let overview = repo.overview().map_err(|e| e.to_string())?;
        Ok((
            overview.head,
            overview.branch.unwrap_or_else(|| "HEAD".into()),
        ))
    })?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "sync".to_string(),
        title: format!("Sync {branch} with {base}"),
        impact: vec![
            format!("Fetch {base} and merge it into {branch}"),
            "Set tracked changes aside and reapply them after the merge".to_string(),
        ],
        preserves: vec![
            "Untracked files and submodule checkouts".to_string(),
            "A durable backup ref for the set-aside work".to_string(),
        ],
        warnings: vec![
            "Conflicts pause the operation; resolve them and resume from Actions.".to_string(),
        ],
        commands: review_commands::sync(&base)?,
        apply_label: "Start sync".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::Sync { id, base, head },
    })
}

pub(super) fn resume_sync(state: &AppState, id: String) -> Result<Prepared, String> {
    let status = with_repository(state, |repo| repo.sync_status().map_err(|e| e.to_string()))?
        .ok_or_else(|| "no sync operation needs resuming".to_string())?;
    let retrying_fetch = status.phase == SyncPhase::Fetch;
    let resolvable = matches!(
        status.phase,
        SyncPhase::BaseMergeConflict | SyncPhase::WipReapplyConflict
    );
    if !retrying_fetch && !resolvable {
        return Err(format!(
            "sync stopped during {}; inspect Recovery before continuing",
            phase_label(&status.phase)
        ));
    }
    let title = if retrying_fetch {
        "Retry sync"
    } else {
        "Resume sync"
    };
    let warning = if retrying_fetch {
        "The remote has to be reachable before retrying."
    } else {
        "Resolve every conflicted file in the working tree first."
    };
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "resume_sync".to_string(),
        title: title.to_string(),
        impact: vec![format!("Continue from {}", phase_label(&status.phase))],
        preserves: vec!["The recorded operation until it completes".to_string()],
        warnings: vec![warning.to_string()],
        commands: vec![format!(
            "# continue recorded operation {}",
            status.operation_id
        )],
        apply_label: title.to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::Resume {
            id,
            operation_id: status.operation_id,
        },
    })
}

pub(crate) fn phase_label(phase: &SyncPhase) -> &'static str {
    match phase {
        SyncPhase::Fetch => "an interrupted fetch",
        SyncPhase::Snapshot => "setting tracked work aside",
        SyncPhase::BaseMerge => "merging Base",
        SyncPhase::BaseMergeConflict => "conflicts while merging Base",
        SyncPhase::WipReapply => "reapplying Saved work",
        SyncPhase::WipReapplyConflict => "conflicts while reapplying Saved work",
    }
}
