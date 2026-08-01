use super::model::CleanupPlan;
use super::remote;

/// The exact ordered Git write sequence a Cleanup performs, derived from the
/// plan so a review can never drift from what runs. Remote deletions come
/// first: the local branch is the backup for an irreversible server deletion,
/// so it must still exist while that push runs.
pub(super) fn commands(plan: &CleanupPlan) -> Vec<String> {
    let mut commands = remote_commands(plan);
    commands.extend(local_commands(plan));
    commands
}

pub(super) fn remote_commands(plan: &CleanupPlan) -> Vec<String> {
    remote::group(plan).iter().map(remote::push_command).collect()
}

pub(super) fn local_commands(plan: &CleanupPlan) -> Vec<String> {
    plan.branches
        .iter()
        .filter_map(|entry| entry.local.as_ref())
        .map(|local| {
            format!(
                "git update-ref -d -m 'git-helper cleanup' {} {}",
                local.reference, local.head
            )
        })
        .collect()
}
