use git_helper_core::{
    EditMessageRequest, ExcludeSubmoduleRequest, ObjectId, RefName, RepoPath, RewritePlan,
    UncommitRequest,
};

use super::super::data::{
    EditMessageInput, ExcludeSubmoduleInput, OperationReview, PendingOperation, UncommitInput,
};
use super::super::repository::with_repository;
use super::super::review_commands;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn uncommit(
    state: &AppState,
    id: String,
    input: UncommitInput,
) -> Result<Prepared, String> {
    let paths = input
        .paths
        .into_iter()
        .map(RepoPath::new)
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        return Err("select at least one changed path to uncommit".to_string());
    }
    let names = paths
        .iter()
        .map(|path| path.as_str().to_string())
        .collect::<Vec<_>>();
    let request = UncommitRequest {
        base: RefName::new(input.base)?,
        paths,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_uncommit(request).map_err(|e| e.to_string())
    })?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "uncommit".to_string(),
        title: format!("Uncommit {}", summarize(&names)),
        impact: rewrite_impact(&plan, &names),
        preserves: vec![
            "Working-tree contents of the selected paths".to_string(),
            "Unrelated staged and unstaged changes".to_string(),
        ],
        warnings: rewrite_warnings(&plan),
        commands: review_commands::rewrite(&plan),
        apply_label: "Apply uncommit".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::Uncommit { id, plan },
    })
}

pub(super) fn edit_message(
    state: &AppState,
    id: String,
    input: EditMessageInput,
) -> Result<Prepared, String> {
    if input.message.trim().is_empty() {
        return Err("a commit message cannot be empty".to_string());
    }
    let request = EditMessageRequest {
        base: RefName::new(input.base)?,
        commit: ObjectId::new(input.commit)?,
        message: input.message.into_bytes(),
    };
    let plan = with_repository(state, |repo| {
        repo.plan_edit_message(request).map_err(|e| e.to_string())
    })?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "edit_message".to_string(),
        title: "Edit commit message".to_string(),
        impact: vec![
            format!(
                "Rebuild {} commit(s) on {}",
                plan.commits.len(),
                plan.branch
            ),
            "Replace the message of the selected commit".to_string(),
        ],
        preserves: vec![
            "Commit trees, parents, and authorship".to_string(),
            "The index and the working tree".to_string(),
        ],
        warnings: rewrite_warnings(&plan),
        commands: review_commands::rewrite(&plan),
        apply_label: "Apply message edit".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::EditMessage { id, plan },
    })
}

pub(super) fn exclude(
    state: &AppState,
    id: String,
    input: ExcludeSubmoduleInput,
) -> Result<Prepared, String> {
    let request = ExcludeSubmoduleRequest {
        path: RepoPath::new(input.path)?,
        install_hook: input.install_hook,
        disable_recurse: input.disable_recurse,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_exclude_submodule(request)
            .map_err(|e| e.to_string())
    })?;
    let mut impact = vec![format!("Hide {} from local status and diffs", plan.path)];
    if plan.hook_will_change {
        impact.push(format!("Guard commits with {}", plan.hook_path.display()));
    }
    if plan.disable_recurse {
        impact.push("Stop Git commands from recursing into submodules".to_string());
    }
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "exclude_submodule".to_string(),
        title: format!("Exclude {}", plan.path),
        impact,
        preserves: vec![
            "Existing hook contents; the guard block is appended".to_string(),
            "The submodule checkout and its own history".to_string(),
        ],
        warnings: vec![
            "A pointer already committed inside the Editable range needs a separate Uncommit."
                .to_string(),
        ],
        commands: review_commands::exclude_submodule(&plan),
        apply_label: "Apply exclusion".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::Exclude { id, plan },
    })
}

pub(super) fn force_push(state: &AppState, id: String) -> Result<Prepared, String> {
    let plan = with_repository(state, |repo| {
        repo.plan_force_push().map_err(|e| e.to_string())
    })?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "force_push".to_string(),
        title: format!("Force push {} to {}", plan.branch, plan.upstream),
        impact: vec![format!(
            "Move {} to the rewritten {}",
            plan.upstream, plan.source_head
        )],
        preserves: vec![format!(
            "The push is refused unless {} still points at {}",
            plan.upstream, plan.expected_remote
        )],
        warnings: vec![
            "Anyone who already pulled the old commits has to reset onto the new ones.".to_string(),
        ],
        commands: vec![plan.command.clone()],
        apply_label: "Force push with lease".to_string(),
    };
    Ok(Prepared {
        review,
        pending: PendingOperation::ForcePush { id, plan },
    })
}

fn rewrite_impact(plan: &RewritePlan, paths: &[String]) -> Vec<String> {
    let mut impact = vec![format!(
        "Rebuild {} commit(s) on {}",
        plan.commits.len(),
        plan.branch
    )];
    impact.push(format!(
        "Restore {} to its {} content in every rebuilt commit",
        summarize(paths),
        plan.base_ref
    ));
    if !plan.dropped_commits.is_empty() {
        impact.push(format!(
            "Drop {} commit(s) that only touched these paths",
            plan.dropped_commits.len()
        ));
    }
    impact
}

fn rewrite_warnings(plan: &RewritePlan) -> Vec<String> {
    vec![
        format!("Every commit after {} gets a new SHA.", plan.base_ref),
        "Already-pushed commits need a force push afterwards.".to_string(),
    ]
}

fn summarize(paths: &[String]) -> String {
    match paths {
        [] => "no paths".to_string(),
        [single] => single.clone(),
        [first, ..] => format!("{first} and {} more path(s)", paths.len() - 1),
    }
}
