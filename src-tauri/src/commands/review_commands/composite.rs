use git_helper_core::{QuickSwitchPlan, RefName};

pub(crate) fn quick_switch(plan: &QuickSwitchPlan) -> Vec<String> {
    let switch = format!(
        "git switch --no-recurse-submodules --no-guess -- {}",
        plan.target_branch
    );
    if !plan.has_tracked_changes {
        return vec![switch];
    }
    vec![
        "git -c submodule.recurse=false stash create".to_string(),
        format!(
            "git update-ref {} <snapshot> \"\"",
            plan.saved_work_reference
        ),
        "git reset --hard --no-recurse-submodules HEAD".to_string(),
        switch,
    ]
}

pub(crate) fn sync(base: &RefName) -> Result<Vec<String>, String> {
    let (remote, branch) = base_parts(base)?;
    let saved_work = "refs/githelper/backup/sync-<id>-wip";
    Ok(vec![
        format!("git fetch --no-tags --no-recurse-submodules {remote} +{branch}:{base}"),
        "git -c submodule.recurse=false stash create".to_string(),
        format!("git update-ref {saved_work} <snapshot> \"\""),
        "git reset --hard --no-recurse-submodules HEAD".to_string(),
        format!("git -c submodule.recurse=false merge --no-edit {base}"),
        format!("git -c submodule.recurse=false stash apply --index {saved_work}"),
        format!("git -c submodule.recurse=false stash apply {saved_work}  # fallback"),
    ])
}

fn base_parts(base: &RefName) -> Result<(&str, &str), String> {
    let value = base
        .as_str()
        .strip_prefix("refs/remotes/")
        .ok_or_else(|| "Base must be a remote-tracking ref".to_string())?;
    value
        .split_once('/')
        .filter(|(remote, branch)| !remote.is_empty() && !branch.is_empty())
        .ok_or_else(|| "Base must include a remote and branch".to_string())
}

#[cfg(test)]
mod tests {
    use git_helper_core::{ObjectId, QuickSwitchPlan, RefName};

    use super::{quick_switch, sync};

    #[test]
    fn quick_switch_lists_the_saved_work_sequence() {
        let plan = switch_plan(/*has_tracked_changes=*/ true);

        assert_eq!(
            quick_switch(&plan),
            vec![
                "git -c submodule.recurse=false stash create",
                "git update-ref refs/githelper/wip/feature <snapshot> \"\"",
                "git reset --hard --no-recurse-submodules HEAD",
                "git switch --no-recurse-submodules --no-guess -- other",
            ]
        );
    }

    #[test]
    fn clean_quick_switch_only_lists_the_switch() {
        let plan = switch_plan(/*has_tracked_changes=*/ false);

        assert_eq!(
            quick_switch(&plan),
            vec!["git switch --no-recurse-submodules --no-guess -- other"]
        );
    }

    #[test]
    fn sync_lists_fetch_save_merge_and_reapply() {
        let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();

        assert_eq!(
            sync(&base).unwrap(),
            vec![
                "git fetch --no-tags --no-recurse-submodules origin +base:refs/remotes/origin/base",
                "git -c submodule.recurse=false stash create",
                "git update-ref refs/githelper/backup/sync-<id>-wip <snapshot> \"\"",
                "git reset --hard --no-recurse-submodules HEAD",
                "git -c submodule.recurse=false merge --no-edit refs/remotes/origin/base",
                "git -c submodule.recurse=false stash apply --index refs/githelper/backup/sync-<id>-wip",
                "git -c submodule.recurse=false stash apply refs/githelper/backup/sync-<id>-wip  # fallback",
            ]
        );
    }

    fn switch_plan(has_tracked_changes: bool) -> QuickSwitchPlan {
        QuickSwitchPlan {
            source_branch: "feature".to_string(),
            source_head: object_id(),
            target_branch: "other".to_string(),
            target_head: object_id(),
            saved_work_reference: "refs/githelper/wip/feature".to_string(),
            has_tracked_changes,
            target_saved_work: None,
        }
    }

    fn object_id() -> ObjectId {
        ObjectId::new("1111111111111111111111111111111111111111".to_string()).unwrap()
    }
}
