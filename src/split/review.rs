use crate::rewrite::RepoPath;

use super::model::SplitBranchPlan;
use super::state::literal;

/// The exact ordered Git write sequence a Split branch performs. Every line is
/// derived from the plan so a review can never drift from what runs.
pub(super) fn commands(plan: &SplitBranchPlan) -> Vec<String> {
    let pathspecs = pathspecs(&plan.changed_paths);
    vec![
        format!(
            "git -c submodule.recurse=false worktree add --detach <worktree> {}",
            plan.merge_base
        ),
        format!(
            "git diff --binary --no-relative --no-renames {} {} -- {pathspecs} | git -C <worktree> apply --index --binary",
            plan.merge_base, plan.source_head
        ),
        "git -C <worktree> write-tree".to_string(),
        format!("git commit-tree <tree> -p {}", plan.merge_base),
        format!("git update-ref {} <commit> ''", plan.new_branch_ref),
        "git worktree remove --force <worktree>".to_string(),
    ]
}

fn pathspecs(paths: &[RepoPath]) -> String {
    paths
        .iter()
        .map(|path| quote(&literal(path.as_str())))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
