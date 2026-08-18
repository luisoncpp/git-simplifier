fn main() {
    let r = git_helper_core::RefName::new("refs/remotes/origin/base".into()).unwrap();
    println!("ref: {}", serde_json::to_string(&r).unwrap());
    let o = git_helper_core::RepositoryOverview {
        path: "p".into(),
        name: "n".into(),
        branch: Some("feature".into()),
        base: Some(r),
        upstream: None,
        head: git_helper_core::ObjectId::new("a".repeat(40)).unwrap(),
        git_version: "2".into(),
        worktree: git_helper_core::WorktreeSummary { staged: 0, unstaged: 0, untracked: 0, conflicts: 0 },
        merge_in_progress: false,
        saved_work_count: 0,
        recovery_count: 0,
        sync_status: None,
        quick_switch_status: None,
        present_branch: None,
    };
    println!("{}", serde_json::to_string_pretty(&o).unwrap());
}
