use git_helper_core::{RefName, RepoPath, RevertRequest, RevertTarget};

use super::super::data::{OperationReview, PendingOperation, RevertInput};
use super::super::repository::with_repository;
use super::Prepared;
use crate::commands::state::AppState;

pub(super) fn revert(state: &AppState, id: String, input: RevertInput) -> Result<Prepared, String> {
    let paths = input
        .paths
        .into_iter()
        .map(RepoPath::new)
        .collect::<Result<Vec<_>, _>>()?;
    if paths.is_empty() {
        return Err("select at least one path to revert".to_string());
    }
    let names = paths
        .iter()
        .map(|path| path.as_str().to_string())
        .collect::<Vec<_>>();
    let target = parse_target(&input.target)?;
    let source_label = match target {
        RevertTarget::Head => "HEAD".to_string(),
        RevertTarget::Base => input.base.clone(),
    };
    let request = RevertRequest {
        base: RefName::new(input.base)?,
        paths,
        target,
    };
    let plan = with_repository(state, |repo| {
        repo.plan_revert(request).map_err(|e| e.to_string())
    })?;
    let review = OperationReview {
        plan_id: id.clone(),
        kind: "revert".to_string(),
        title: format!("Revert {} to {}", summarize(&names), source_label),
        impact: vec![
            format!(
                "Overwrite the index and working tree for {} from {}",
                summarize(&names),
                plan.source
            ),
            "Leave branch history and commit SHAs unchanged".to_string(),
        ],
        preserves: vec![
            "Every commit on the current branch".to_string(),
            "Unrelated paths not selected in this review".to_string(),
        ],
        warnings: vec![
            "Uncommitted content on the selected paths is discarded.".to_string(),
            "This is not Uncommit: history is left alone.".to_string(),
        ],
        commands: plan.commands.clone(),
        apply_label: "Apply revert".to_string(),
    };
    Ok(Prepared {
        review: Some(review),
        pending: Some(PendingOperation::Revert { id, plan }),
        block: None,
    })
}

fn parse_target(value: &str) -> Result<RevertTarget, String> {
    match value {
        "head" => Ok(RevertTarget::Head),
        "base" => Ok(RevertTarget::Base),
        other => Err(format!("unknown revert target: {other}")),
    }
}

fn summarize(paths: &[String]) -> String {
    match paths {
        [] => "no paths".to_string(),
        [single] => single.clone(),
        [first, ..] => format!("{first} and {} more path(s)", paths.len() - 1),
    }
}
