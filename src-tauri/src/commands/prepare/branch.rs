use git_helper_core::{RefName, RepoPath, SplitBranchPlan, SplitBranchRequest};

use super::super::data::{OperationReview, PendingOperation, SplitBranchInput};
use super::super::repository::with_repository;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn split_branch(
    state: &AppState,
    id: String,
    input: SplitBranchInput,
) -> Result<Prepared, String> {
    if input.new_branch.trim().is_empty() {
        return Err("name the branch the selected changes will land on".to_string());
    }
    if input.paths.is_empty() {
        return Err("select at least one changed path to split out".to_string());
    }
    let request = SplitBranchRequest {
        base: RefName::new(input.base)?,
        new_branch: input.new_branch.trim().to_string(),
        paths: input
            .paths
            .into_iter()
            .map(RepoPath::new)
            .collect::<Result<Vec<_>, _>>()?,
        message: (!input.message.trim().is_empty()).then(|| input.message.into_bytes()),
    };
    let plan = with_repository(state, |repo| {
        repo.plan_split_branch(request).map_err(|e| e.to_string())
    })?;
    Ok(Prepared {
        review: review(id.clone(), &plan),
        pending: PendingOperation::Split { id, plan },
    })
}

fn review(id: String, plan: &SplitBranchPlan) -> OperationReview {
    OperationReview {
        plan_id: id,
        kind: "split_branch".to_string(),
        title: format!("Split {} onto {}", summarize(plan), plan.new_branch),
        impact: impact(plan),
        preserves: vec![
            format!("{} keeps every one of those changes", plan.source_branch),
            "The index and the working tree".to_string(),
        ],
        warnings: warnings(plan),
        // The core planner already derived this sequence from the plan being
        // applied; rebuilding it here would be a second, drifting source.
        commands: plan.commands.clone(),
        apply_label: "Create the branch".to_string(),
    }
}

fn impact(plan: &SplitBranchPlan) -> Vec<String> {
    let mut impact = vec![
        format!(
            "Create {} at {} with one commit",
            plan.new_branch, plan.merge_base
        ),
        format!(
            "Copy {} changed file(s) out of {}",
            plan.changed_paths.len(),
            plan.source_branch
        ),
    ];
    if !plan.companion_paths.is_empty() {
        impact.push(format!(
            "Also carry {} companion file(s) that cannot travel alone: {}",
            plan.companion_paths.len(),
            names(&plan.companion_paths)
        ));
    }
    impact
}

/// Copy mode leaves the change in two places. Saying so is the whole point:
/// a user who expected a move would otherwise discover it much later.
fn warnings(plan: &SplitBranchPlan) -> Vec<String> {
    let mut warnings = vec![format!(
        "This copies. The selected changes stay on {} as well, and removing them there is a separate decision.",
        plan.source_branch
    )];
    if plan.message_is_derived {
        warnings.push(format!(
            "No message was given, so the commit will read “{}”.",
            String::from_utf8_lossy(&plan.message).trim()
        ));
    }
    warnings
}

fn summarize(plan: &SplitBranchPlan) -> String {
    match plan.changed_paths.as_slice() {
        [single] => single.to_string(),
        paths => format!("{} file(s)", paths.len()),
    }
}

fn names(paths: &[RepoPath]) -> String {
    paths
        .iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use git_helper_core::ObjectId;

    use super::*;

    #[test]
    fn the_review_shows_the_plan_commands_instead_of_rebuilding_them() {
        let plan = split_plan();

        let review = review("op-1".to_string(), &plan);

        assert_eq!(review.commands, plan.commands);
    }

    /// Copy versus move is the one thing a user cannot recover from being wrong
    /// about, so the review has to say it before anything is created.
    #[test]
    fn the_review_states_that_the_source_branch_keeps_the_changes() {
        let review = review("op-1".to_string(), &split_plan());

        assert!(review
            .warnings
            .iter()
            .any(|warning| warning.contains("This copies")));
        assert!(review
            .preserves
            .iter()
            .any(|entry| entry.contains("feature")));
    }

    #[test]
    fn a_derived_message_is_quoted_in_the_review() {
        let review = review("op-1".to_string(), &split_plan());

        assert!(review
            .warnings
            .iter()
            .any(|warning| warning.contains("Split 1 file from feature")));
    }

    #[test]
    fn companions_are_named_so_an_unexpected_file_is_never_a_surprise() {
        let mut plan = split_plan();
        plan.companion_paths = vec![repo_path("Assets/hero.png.meta")];
        plan.changed_paths.push(repo_path("Assets/hero.png.meta"));

        let review = review("op-1".to_string(), &plan);

        assert!(review
            .impact
            .iter()
            .any(|entry| entry.contains("Assets/hero.png.meta")));
    }

    fn split_plan() -> SplitBranchPlan {
        SplitBranchPlan {
            source_branch: "feature".to_string(),
            source_head: object_id("1"),
            base_ref: RefName::new("refs/remotes/origin/main".to_string()).unwrap(),
            base: object_id("2"),
            merge_base: object_id("3"),
            new_branch: "carved".to_string(),
            new_branch_ref: "refs/heads/carved".to_string(),
            selected_paths: vec![repo_path("Assets/hero.png")],
            changed_paths: vec![repo_path("Assets/hero.png")],
            companion_paths: Vec::new(),
            message: b"Split 1 file from feature\n".to_vec(),
            message_is_derived: true,
            commands: vec!["git worktree add --detach <worktree> 333".to_string()],
        }
    }

    fn repo_path(value: &str) -> RepoPath {
        RepoPath::new(value.to_string()).unwrap()
    }

    fn object_id(digit: &str) -> ObjectId {
        ObjectId::new(digit.repeat(40)).unwrap()
    }
}
