use super::history_model::HistorySwitchPlan;
use super::model::QuickSwitchPlan;

pub(super) fn switch_commands(switch_plan: &QuickSwitchPlan) -> Vec<String> {
    let mut commands = Vec::new();
    push_tracked_prep(&mut commands, switch_plan);
    push_untracked_park(&mut commands, &switch_plan.untracked_conflicts);
    push_switch_and_pull(&mut commands, switch_plan);
    if switch_plan.has_tracked_changes && switch_plan.carry_changes {
        commands.push("git stash pop --index".to_string());
        commands.push("git stash pop  # fallback".to_string());
    }
    push_untracked_reapply(&mut commands, &switch_plan.untracked_conflicts);
    commands
}

pub(super) fn history_commands(plan: &HistorySwitchPlan) -> Vec<String> {
    let mut commands = Vec::new();
    push_history_tracked(&mut commands, plan);
    push_untracked_park(&mut commands, &plan.untracked_conflicts);
    commands.push(format!(
        "git symbolic-ref {} refs/heads/{}",
        super::present::PRESENT_REF,
        plan.source_branch
    ));
    commands.push(format!(
        "git switch --no-recurse-submodules --detach {}",
        plan.target_commit
    ));
    if plan.has_tracked_changes && plan.carry_changes {
        commands.push("git stash pop --index".to_string());
        commands.push("git stash pop  # fallback".to_string());
    }
    push_untracked_reapply(&mut commands, &plan.untracked_conflicts);
    commands
}

fn push_tracked_prep(commands: &mut Vec<String>, plan: &QuickSwitchPlan) {
    push_tracked(
        commands,
        TrackedReview {
            has_tracked_changes: plan.has_tracked_changes,
            carry_changes: plan.carry_changes,
            saved_work_reference: &plan.saved_work_reference,
        },
    );
}

fn push_history_tracked(commands: &mut Vec<String>, plan: &HistorySwitchPlan) {
    push_tracked(
        commands,
        TrackedReview {
            has_tracked_changes: plan.has_tracked_changes,
            carry_changes: plan.carry_changes,
            saved_work_reference: &plan.saved_work_reference,
        },
    );
}

struct TrackedReview<'a> {
    has_tracked_changes: bool,
    carry_changes: bool,
    saved_work_reference: &'a str,
}

fn push_tracked(commands: &mut Vec<String>, review: TrackedReview<'_>) {
    if review.has_tracked_changes && review.carry_changes {
        commands.push("git stash push -m \"git-helper carry\"".to_string());
        return;
    }
    if !review.has_tracked_changes {
        return;
    }
    commands.push("git stash create".to_string());
    commands.push(format!(
        "git update-ref {} <snapshot>",
        review.saved_work_reference
    ));
    commands.push("git reset --hard HEAD".to_string());
}

fn push_untracked_park(commands: &mut Vec<String>, conflicts: &[String]) {
    if conflicts.is_empty() {
        return;
    }
    for path in conflicts {
        commands.push(format!("git add -- :(top,literal){path}"));
    }
    commands.push("git stash create".to_string());
    commands.push(
        "git update-ref refs/githelper/untracked-merge/<operation-id> <snapshot>".to_string(),
    );
    commands.push("git restore --worktree --source=HEAD -- <paths>".to_string());
}

fn push_switch_and_pull(commands: &mut Vec<String>, plan: &QuickSwitchPlan) {
    if let Some(remote) = &plan.create_from_remote {
        let start = remote
            .strip_prefix("refs/remotes/")
            .unwrap_or(remote.as_str());
        commands.push(format!(
            "git switch --track -c {} {}",
            plan.target_branch, start
        ));
    } else {
        commands.push(format!("git switch --no-guess -- {}", plan.target_branch));
    }
    let Some(remote) = &plan.pull_remote_ref else {
        return;
    };
    let short = remote
        .strip_prefix("refs/remotes/")
        .unwrap_or(remote.as_str());
    let (remote_name, branch) = short.split_once('/').unwrap_or(("origin", short));
    commands.push(format!("git pull --ff-only {remote_name} {branch}"));
}

fn push_untracked_reapply(commands: &mut Vec<String>, conflicts: &[String]) {
    if conflicts.is_empty() {
        return;
    }
    commands.push(
        "git stash apply --index refs/githelper/untracked-merge/<operation-id>".to_string(),
    );
    commands.push(
        "git stash apply refs/githelper/untracked-merge/<operation-id>  # fallback".to_string(),
    );
}
