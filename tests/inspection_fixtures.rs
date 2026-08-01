mod support;

use std::ffi::OsString;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use git_helper_core::{DiffCompare, GitCommand, LocalBranchChoice, RefName};
use support::fixture_repo::FixtureRepo;

#[test]
fn editable_commit_discovery_ignores_git_log_record_whitespace() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "editable message");
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();

    let commits = fixture.repo.list_editable_commits(base).unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "editable message");
    assert_eq!(commits[0].message, "editable message");
}

#[test]
fn setting_base_completes_without_relocking_the_repository() {
    let fixture = FixtureRepo::new();
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        sender.send(fixture.repo.set_base(base)).unwrap();
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("set_base deadlocked while acquiring the repository write lock");
    result.unwrap();
}

#[test]
fn branch_diff_is_a_stable_patch_of_committed_changes_since_base() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "base\ncommitted\n", "extend readme");
    fixture.write_worktree_file("README.md", "working tree only\n");
    fixture.set_config("color.ui", "always");
    fixture.set_config("diff.noprefix", "true");
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();

    let diff = fixture.repo.branch_diff(base, DiffCompare::Head).unwrap();

    assert!(diff.contains("--- a/README.md"), "{diff}");
    assert!(diff.contains("+++ b/README.md"), "{diff}");
    assert!(diff.contains("+committed"));
    assert!(!diff.contains("working tree only"));
    assert!(!diff.contains("\u{1b}["));
}

#[test]
fn branch_diff_local_includes_worktree_dirt_and_committed_changes() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "base\ncommitted\n", "extend readme");
    fixture.write_worktree_file("README.md", "working tree only\n");
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();

    let diff = fixture.repo.branch_diff(base, DiffCompare::Local).unwrap();

    assert!(diff.contains("+committed") || diff.contains("working tree only"), "{diff}");
    assert!(diff.contains("working tree only"), "{diff}");
}

/// `%(refname:strip=N)` counts the branch name as a component, so reading Saved
/// work names that way returned an empty string for every simple branch and lost
/// the first segment of a slashed one.
#[test]
fn saved_work_is_reported_for_simple_and_slashed_branch_names() {
    let fixture = FixtureRepo::new();
    fixture.branch("team/thing");
    fixture.checkout("feature");
    let head = fixture.head();
    write_ref(&fixture, "refs/githelper/wip/feature", &head);
    write_ref(&fixture, "refs/githelper/wip/team/thing", &head);

    let branches = fixture.repo.list_local_branches().unwrap();

    assert!(has_saved_work(&branches, "feature"));
    assert!(has_saved_work(&branches, "team/thing"));
    assert!(!has_saved_work(&branches, "base"));
}

#[test]
fn fetch_remotes_succeeds_when_no_remotes_are_configured() {
    let fixture = FixtureRepo::new();
    fixture.repo.fetch_remotes().unwrap();
}

#[test]
fn fetch_remotes_picks_up_new_refs_on_a_configured_remote() {
    let fixture = FixtureRepo::new();
    let remote = fixture.add_bare_origin();
    let commit = fixture.head();
    let git_dir = remote.path().to_str().unwrap();
    std::process::Command::new("git")
        .args([
            "--git-dir",
            git_dir,
            "update-ref",
            "refs/heads/extra",
            &commit,
        ])
        .status()
        .expect("failed to seed remote ref");

    let missing = fixture
        .repo
        .run(GitCommand::read(args(&["rev-parse", "--verify", "refs/remotes/origin/extra"])));
    assert!(missing.is_err());

    fixture.repo.fetch_remotes().unwrap();

    let fetched = fixture
        .repo
        .run(GitCommand::read(args(&["rev-parse", "refs/remotes/origin/extra"])))
        .unwrap();
    assert_eq!(
        String::from_utf8(fetched.stdout).unwrap().trim(),
        commit
    );
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn write_ref(fixture: &FixtureRepo, reference: &str, value: &str) {
    let args = ["update-ref", reference, value]
        .iter()
        .map(|value| OsString::from(*value))
        .collect();
    fixture.repo.run(GitCommand::write(args)).unwrap();
}

fn has_saved_work(branches: &[LocalBranchChoice], name: &str) -> bool {
    branches
        .iter()
        .any(|branch| branch.name == name && branch.saved_work)
}
