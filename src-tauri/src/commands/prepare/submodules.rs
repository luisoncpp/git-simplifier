use git_helper_core::{RefName, RepoPath, SubmoduleCleanupRequest};

use super::super::data::{
    CleanupSubmodulesInput, OperationReview, PendingOperation,
};
use super::super::repository::with_repository;
use super::super::review_commands;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn cleanup_submodules(
    state: &AppState,
    id: String,
    input: CleanupSubmodulesInput,
) -> Result<Prepared, String> {
    let paths = input
        .paths
        .into_iter()
        .map(RepoPath::new)
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        return Err("select at least one dirty submodule".to_string());
    }
    if !input.uncommit && !input.revert {
        return Err("select at least one cleanup action".to_string());
    }
    let request = SubmoduleCleanupRequest {
        base: RefName::new(input.base)?,
        paths: paths.clone(),
        uncommit: input.uncommit,
        revert: input.revert,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_submodule_cleanup(request)
            .map_err(|e| e.to_string())
    })?;
    let names = paths
        .iter()
        .map(|path| path.as_str().to_string())
        .collect::<Vec<_>>();
    let mut impact = Vec::new();
    if plan.uncommit && !plan.uncommit_paths.is_empty() {
        impact.push(format!(
            "Uncommit {} from {}",
            summarize(&plan.uncommit_paths.iter().map(|p| p.as_str().to_string()).collect::<Vec<_>>()),
            plan.base_ref
        ));
    } else if plan.uncommit {
        impact.push(
            "No selected submodule differs from Base in the Editable range; Uncommit is skipped."
                .to_string(),
        );
    }
    if plan.revert && !plan.revert_paths.is_empty() {
        impact.push(format!(
            "Restore {} to HEAD in the index and working tree, then sync nested checkouts",
            summarize(&names)
        ));
    }
    let mut warnings = Vec::new();
    if plan.uncommit && plan.revert {
        warnings.push(
            "Uncommit runs first; Revert then aligns the gitlink and nested checkout to the rewritten HEAD."
                .to_string(),
        );
    } else if plan.uncommit {
        warnings.push(
            "Uncommit leaves nested submodule checkouts untouched until you Revert separately."
                .to_string(),
        );
    }
    if plan.uncommit && !plan.uncommit_paths.is_empty() {
        warnings.push("Every commit after Base gets a new SHA.".to_string());
        warnings.push("Already-pushed commits need a force push afterwards.".to_string());
    }
    let mut commands = Vec::new();
    if let Some(uncommit_plan) = plan.uncommit_plan() {
        commands.extend(review_commands::rewrite(uncommit_plan));
    }
    commands.extend(plan.commands.clone());
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "cleanup_submodules".to_string(),
        title: format!("Cleanup {}", summarize(&names)),
        impact,
        preserves: vec![
            "Unrelated paths and submodule histories".to_string(),
            "Excluded submodule standing rules".to_string(),
        ],
        warnings,
        commands,
        apply_label: "Apply cleanup".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::SubmoduleCleanup { id, plan }),
        block: None,
    })
}

fn summarize(paths: &[String]) -> String {
    match paths {
        [] => "no submodules".to_string(),
        [single] => single.clone(),
        [first, ..] => format!("{first} and {} more submodule(s)", paths.len() - 1),
    }
}
