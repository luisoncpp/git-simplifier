use git_helper_core::{ExcludeSubmodulePlan, RewriteAction, RewritePlan, TreeEntry};

/// The rewrite engine never runs `git rebase`. It rebuilds every commit in the
/// Editable range through a temporary index, so the review has to list that
/// sequence instead of a plausible-looking rebase.
const TEMP: &str = "GIT_INDEX_FILE=<temp-index>";

pub(crate) fn rewrite(plan: &RewritePlan) -> Vec<String> {
    let rebuilt = plan
        .commits
        .iter()
        .filter(|commit| commit.action == RewriteAction::Rebuild)
        .count();
    let mut commands = vec![format!(
        "# repeated for each of the {rebuilt} rebuilt commit(s), oldest first:"
    )];
    commands.push(format!("{TEMP} git read-tree <commit-tree>"));
    commands.extend(index_updates(plan));
    commands.push(format!("{TEMP} git write-tree"));
    commands.push("git commit-tree <new-tree> -F - -p <rewritten-parent>".to_string());
    commands.push("# once, after the range is rebuilt:".to_string());
    commands.push(update_ref(plan));
    if !plan.selected_paths.is_empty() {
        commands.push(reset_paths(plan));
    }
    commands
}

fn index_updates(plan: &RewritePlan) -> Vec<String> {
    plan.base_entries
        .iter()
        .map(|(path, entry)| index_update(path.as_str(), entry.as_ref()))
        .collect()
}

fn index_update(path: &str, entry: Option<&TreeEntry>) -> String {
    let Some(entry) = entry else {
        return format!("{TEMP} git update-index --force-remove -- {path}");
    };
    format!(
        "{TEMP} git update-index --add --cacheinfo {} {} {path}",
        entry.mode, entry.object
    )
}

fn update_ref(plan: &RewritePlan) -> String {
    format!(
        "git update-ref -m \"git-helper {}\" {} <new-head> {}",
        plan.operation.label(),
        plan.branch,
        plan.source_head
    )
}

fn reset_paths(plan: &RewritePlan) -> String {
    let paths = plan
        .selected_paths
        .iter()
        .map(|path| path.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    format!("git reset --mixed <new-head> -- {paths}")
}

pub(crate) fn exclude_submodule(plan: &ExcludeSubmodulePlan) -> Vec<String> {
    let mut commands = plan.config_lines.clone();
    if plan.hook_will_change {
        let verb = if plan.hook_exists { "append to" } else { "create" };
        commands.push(format!("# {verb} {}", plan.hook_path.display()));
        commands.extend(plan.hook_preview.lines().map(|line| format!("  {line}")));
    }
    commands.push(plan.staging_command.clone());
    commands
}
