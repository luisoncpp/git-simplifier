use git_helper_core::HistorySwitchPlan;

pub(crate) fn history_switch(plan: &HistorySwitchPlan) -> Vec<String> {
    let mut commands = Vec::new();
    push_tracked(&mut commands, plan);
    push_untracked_park(&mut commands, plan);
    commands.push(format!(
        "git symbolic-ref refs/githelper/present refs/heads/{}",
        plan.source_branch
    ));
    commands.push(format!(
        "git switch --no-recurse-submodules --detach {}",
        plan.target_commit
    ));
    if plan.has_tracked_changes && plan.carry_changes {
        commands.push(
            "git -c submodule.recurse=false stash pop --index".to_string(),
        );
        commands.push("git -c submodule.recurse=false stash pop  # fallback".to_string());
    }
    push_untracked_reapply(&mut commands, plan);
    commands
}

fn push_tracked(commands: &mut Vec<String>, plan: &HistorySwitchPlan) {
    if plan.has_tracked_changes && plan.carry_changes {
        commands.push(
            "git -c submodule.recurse=false stash push -m \"git-helper carry\"".to_string(),
        );
        return;
    }
    if !plan.has_tracked_changes {
        return;
    }
    commands.push("git -c submodule.recurse=false stash create".to_string());
    commands.push(format!(
        "git update-ref {} <snapshot> \"\"",
        plan.saved_work_reference
    ));
    commands.push("git reset --hard --no-recurse-submodules HEAD".to_string());
}

fn push_untracked_park(commands: &mut Vec<String>, plan: &HistorySwitchPlan) {
    if plan.untracked_conflicts.is_empty() {
        return;
    }
    for path in &plan.untracked_conflicts {
        commands.push(format!("git add -- :(top,literal){path}"));
    }
    commands.push("git -c submodule.recurse=false stash create".to_string());
    commands.push(
        "git update-ref refs/githelper/untracked-merge/<operation-id> <snapshot> \"\"".to_string(),
    );
    commands.push("git restore --worktree --source=HEAD -- <paths>".to_string());
}

fn push_untracked_reapply(commands: &mut Vec<String>, plan: &HistorySwitchPlan) {
    if plan.untracked_conflicts.is_empty() {
        return;
    }
    commands.push(
        "git -c submodule.recurse=false stash apply --index refs/githelper/untracked-merge/<operation-id>"
            .to_string(),
    );
    commands.push(
        "git -c submodule.recurse=false stash apply refs/githelper/untracked-merge/<operation-id>  # fallback"
            .to_string(),
    );
}

#[cfg(test)]
mod tests {
    use git_helper_core::{HistorySwitchPlan, ObjectId};

    use super::history_switch;

    #[test]
    fn history_lists_present_ref_and_detach_without_pull() {
        let commands = history_switch(&plan());
        assert!(commands.iter().any(|command| command.contains("refs/githelper/present")));
        assert!(commands.iter().any(|command| {
            command.contains("switch --no-recurse-submodules --detach")
        }));
        assert!(!commands.iter().any(|command| command.contains("pull")));
    }

    fn plan() -> HistorySwitchPlan {
        HistorySwitchPlan {
            source_branch: "feature".to_string(),
            source_head: object_id("1"),
            target_commit: object_id("2"),
            saved_work_reference: "refs/githelper/wip/feature".to_string(),
            has_tracked_changes: false,
            carry_changes: false,
            untracked_conflicts: Vec::new(),
        }
    }

    fn object_id(digit: &str) -> ObjectId {
        ObjectId::new(digit.repeat(40)).unwrap()
    }
}
