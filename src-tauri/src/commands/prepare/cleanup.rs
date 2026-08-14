use git_helper_core::{CleanupPlan, CleanupRequest, KeptReason, RefName};

use super::super::data::{CleanupInput, OperationReview, PendingOperation};
use super::super::repository::with_repository;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn cleanup(
    state: &AppState,
    id: String,
    input: CleanupInput,
) -> Result<Prepared, String> {
    if input.references.is_empty() {
        return Err("select at least one branch to delete".to_string());
    }
    let request = CleanupRequest {
        base: RefName::new(input.base)?,
        chosen: input.references,
        include_remote_counterparts: input.delete_remotes,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_cleanup(request).map_err(|e| e.to_string())
    })?;
    Ok(Prepared {
        review: Some(review(id.clone(), &plan)),
        pending: Some(PendingOperation::Cleanup { id, plan }),
        block: None,
    })
}

fn review(id: String, plan: &CleanupPlan) -> OperationReview {
    OperationReview {
        plan_id: id,
        kind: "cleanup".to_string(),
        title: title(plan),
        impact: impact(plan),
        preserves: vec![
            "The current branch, the branch Base tracks, and anything with Saved work".to_string(),
            "Every commit: these branches are already contained in Base".to_string(),
        ],
        warnings: warnings(plan),
        // Derived by the core planner from the plan being applied; rebuilding
        // it here would be a second, drifting source.
        commands: plan.commands.clone(),
        apply_label: "Delete the branches".to_string(),
    }
}

fn title(plan: &CleanupPlan) -> String {
    let local = plan.local_count;
    if plan.remote_count == 0 {
        return format!("Delete {} merged {}", local, branch_word(local));
    }
    format!(
        "Delete {} merged {} and {} remote {}",
        local,
        branch_word(local),
        plan.remote_count,
        branch_word(plan.remote_count)
    )
}

fn impact(plan: &CleanupPlan) -> Vec<String> {
    let mut impact = Vec::new();
    for entry in &plan.branches {
        if entry.local.is_some() {
            impact.push(format!("Delete the local branch {}", entry.branch));
        }
        let Some(remote) = &entry.remote else {
            continue;
        };
        impact.push(format!(
            "Delete {} on {}",
            remote.remote_ref, remote.remote
        ));
    }
    impact
}

fn warnings(plan: &CleanupPlan) -> Vec<String> {
    let mut warnings = Vec::new();
    if plan.remote_count > 0 {
        warnings.push(
            "Deleting a branch on a server cannot be undone from this app. The local branches can be restored from the Recovery panel; the remote ones cannot."
                .to_string(),
        );
    }
    warnings.extend(plan.kept_remotes.iter().filter_map(kept_warning));
    warnings
}

fn kept_warning(kept: &git_helper_core::KeptRemote) -> Option<String> {
    match kept.reason {
        KeptReason::NotMerged => Some(format!(
            "{} has commits Base does not contain, so it is left on the server",
            kept.tracking_ref
        )),
        KeptReason::NoUpstream => Some(format!(
            "{} tracks no remote branch, so nothing is deleted on a server for it",
            kept.branch
        )),
        KeptReason::Disabled => None,
    }
}

fn branch_word(count: usize) -> &'static str {
    if count == 1 {
        "branch"
    } else {
        "branches"
    }
}

#[cfg(test)]
mod tests {
    use git_helper_core::{
        CleanupBranchPlan, CleanupPlan, KeptRemote, LocalDeletion, ObjectId, RefName,
        RemoteDeletion,
    };

    use super::{review, KeptReason};

    #[test]
    fn a_local_only_cleanup_carries_no_irreversible_warning() {
        let plan = plan(/*with_remote=*/ false);

        let review = review("op-1".to_string(), &plan);

        assert_eq!(review.title, "Delete 1 merged branch");
        assert!(review.warnings.is_empty());
        assert_eq!(review.impact, vec!["Delete the local branch spike"]);
    }

    #[test]
    fn a_remote_cleanup_warns_that_the_server_deletion_cannot_be_undone() {
        let plan = plan(/*with_remote=*/ true);

        let review = review("op-1".to_string(), &plan);

        assert_eq!(review.title, "Delete 1 merged branch and 1 remote branch");
        assert!(review.warnings[0].contains("cannot be undone"));
        assert_eq!(review.impact[1], "Delete refs/heads/spike on origin");
    }

    #[test]
    fn a_remote_left_behind_explains_itself() {
        let mut plan = plan(/*with_remote=*/ false);
        plan.kept_remotes = vec![KeptRemote {
            branch: "spike".to_string(),
            tracking_ref: "refs/remotes/origin/spike".to_string(),
            reason: KeptReason::NotMerged,
        }];

        let review = review("op-1".to_string(), &plan);

        assert!(review.warnings[0].contains("Base does not contain"));
    }

    fn plan(with_remote: bool) -> CleanupPlan {
        let head = ObjectId::new("1".repeat(40)).unwrap();
        let remote = with_remote.then(|| RemoteDeletion {
            remote: "origin".to_string(),
            remote_ref: "refs/heads/spike".to_string(),
            tracking_ref: "refs/remotes/origin/spike".to_string(),
            head: head.clone(),
        });
        CleanupPlan {
            base: RefName::new("refs/remotes/origin/main".to_string()).unwrap(),
            base_head: head.clone(),
            branches: vec![CleanupBranchPlan {
                branch: "spike".to_string(),
                local: Some(LocalDeletion {
                    reference: "refs/heads/spike".to_string(),
                    head,
                }),
                remote,
            }],
            kept_remotes: Vec::new(),
            local_count: 1,
            remote_count: usize::from(with_remote),
            commands: Vec::new(),
        }
    }
}
