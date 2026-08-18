mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{
    GitCommand, HistorySwitchRequest, QuickSwitchRequest, SwitchError,
};
use support::fixture_repo::FixtureRepo;

#[test]
fn history_detaches_and_leaves_the_branch_at_present() {
    let (fixture, old) = fixture_with_past();
    let tip = fixture.head();
    let plan = fixture
        .repo
        .plan_history_switch(commit_request(&old))
        .unwrap();
    let result = fixture.repo.apply_history_switch(&plan).unwrap();

    assert_eq!(result.present_branch, "feature");
    assert_eq!(result.target_commit.as_str(), old);
    assert!(symbolic_ref(&fixture, "HEAD").is_none());
    assert_eq!(fixture.head(), old);
    assert_eq!(branch_tip(&fixture, "feature"), tip);
    assert_eq!(present_branch(&fixture).as_deref(), Some("feature"));
}

#[test]
fn history_date_picks_the_newest_commit_at_or_before_until() {
    let fixture = FixtureRepo::new();
    commit_dated(&fixture, "README.md", "old\n", "old state", "2020-01-01T12:00:00");
    let old = fixture.head();
    commit_dated(&fixture, "README.md", "new\n", "new state", "2024-06-01T12:00:00");

    let plan = fixture
        .repo
        .plan_history_switch(until_request("2021-01-01T00:00:00"))
        .unwrap();
    assert_eq!(plan.target_commit.as_str(), old);
}

#[test]
fn history_date_before_the_first_commit_fails() {
    let fixture = FixtureRepo::new();
    let error = fixture
        .repo
        .plan_history_switch(until_request("1970-01-01T00:00:00"))
        .unwrap_err();
    assert!(error.to_string().contains("No commit on this branch"));
}

#[test]
fn history_refuses_a_sha_not_on_the_current_branch() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "on feature\n", "feature commit");
    fixture.branch("other");
    fixture.commit_file("OTHER.md", "only other\n", "other only");
    let other = fixture.head();
    fixture.checkout("feature");

    let error = fixture
        .repo
        .plan_history_switch(commit_request(&other))
        .unwrap_err();
    assert!(error.to_string().contains("not on the current branch"));
}

#[test]
fn history_refuses_the_current_head() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "now\n", "now");
    let error = fixture
        .repo
        .plan_history_switch(commit_request(&fixture.head()))
        .unwrap_err();
    assert!(error.to_string().contains("already at this commit"));
}

#[test]
fn history_refuses_when_already_detached() {
    let (fixture, old) = fixture_with_past();
    fixture
        .repo
        .apply_history_switch(
            &fixture
                .repo
                .plan_history_switch(commit_request(&old))
                .unwrap(),
        )
        .unwrap();
    let error = fixture
        .repo
        .plan_history_switch(commit_request(&old))
        .unwrap_err();
    assert!(error.to_string().contains("detached"));
}

#[test]
fn history_parks_tracked_changes_when_carry_is_off() {
    let (fixture, old) = fixture_with_past();
    fixture.write_worktree_file("README.md", "wip\n");
    let result = fixture
        .repo
        .apply_history_switch(
            &fixture
                .repo
                .plan_history_switch(commit_request(&old))
                .unwrap(),
        )
        .unwrap();
    assert!(result.saved_work.is_some());
    assert_eq!(fixture.repo.list_saved_work().unwrap().len(), 1);
    assert_eq!(read_worktree(&fixture, "README.md"), "old\n");
}

#[test]
fn history_carries_tracked_changes_onto_the_detached_commit() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("notes.txt", "same\n", "notes");
    let with_notes = fixture.head();
    fixture.commit_file("README.md", "later\n", "later");
    fixture.write_worktree_file("notes.txt", "wip notes\n");
    let mut request = commit_request(&with_notes);
    request.carry_changes = true;
    fixture
        .repo
        .apply_history_switch(&fixture.repo.plan_history_switch(request).unwrap())
        .unwrap();
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
    assert_eq!(read_worktree(&fixture, "notes.txt"), "wip notes\n");
}

#[test]
fn history_blocks_untracked_overlap_without_merge() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("extra.txt", "tracked\n", "add extra");
    let with_extra = fixture.head();
    fixture.remove_file("extra.txt", "drop extra");
    fixture.write_worktree_file("extra.txt", "untracked\n");
    let error = fixture
        .repo
        .plan_history_switch(commit_request(&with_extra))
        .unwrap_err();
    assert!(matches!(error, SwitchError::UntrackedOverlap(_)));
}

#[test]
fn history_merges_untracked_overlap_when_asked() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("extra.txt", "tracked\n", "add extra");
    let with_extra = fixture.head();
    fixture.remove_file("extra.txt", "drop extra");
    fixture.write_worktree_file("extra.txt", "untracked\n");
    let mut request = commit_request(&with_extra);
    request.merge_untracked = true;
    fixture
        .repo
        .apply_history_switch(&fixture.repo.plan_history_switch(request).unwrap())
        .unwrap();
    assert!(symbolic_ref(&fixture, "HEAD").is_none());
}

#[test]
fn quick_switch_from_history_returns_to_present_and_clears_the_marker() {
    let (fixture, old) = fixture_with_past();
    let tip = fixture.head();
    fixture
        .repo
        .apply_history_switch(
            &fixture
                .repo
                .plan_history_switch(commit_request(&old))
                .unwrap(),
        )
        .unwrap();
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(switch_request("feature")).unwrap())
        .unwrap();

    assert_eq!(symbolic_ref(&fixture, "HEAD").as_deref(), Some("feature"));
    assert_eq!(fixture.head(), tip);
    assert!(present_branch(&fixture).is_none());
}

#[test]
fn quick_switch_from_history_to_another_branch_clears_the_marker() {
    let (fixture, old) = fixture_with_past();
    fixture.branch("other");
    fixture.checkout("feature");
    fixture
        .repo
        .apply_history_switch(
            &fixture
                .repo
                .plan_history_switch(commit_request(&old))
                .unwrap(),
        )
        .unwrap();
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(switch_request("other")).unwrap())
        .unwrap();

    assert_eq!(symbolic_ref(&fixture, "HEAD").as_deref(), Some("other"));
    assert!(present_branch(&fixture).is_none());
}

#[test]
fn list_history_commits_omits_head() {
    let (fixture, old) = fixture_with_past();
    let commits = fixture.repo.list_history_commits().unwrap();
    assert!(commits.iter().any(|commit| commit.id.as_str() == old));
    assert!(!commits.iter().any(|commit| commit.id.as_str() == fixture.head()));
}

fn fixture_with_past() -> (FixtureRepo, String) {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "old\n", "old state");
    let old = fixture.head();
    fixture.commit_file("README.md", "new\n", "new state");
    (fixture, old)
}

fn commit_request(commit: &str) -> HistorySwitchRequest {
    HistorySwitchRequest {
        commit: Some(commit.to_string()),
        until: None,
        carry_changes: false,
        merge_untracked: false,
    }
}

fn until_request(until: &str) -> HistorySwitchRequest {
    HistorySwitchRequest {
        commit: None,
        until: Some(until.to_string()),
        carry_changes: false,
        merge_untracked: false,
    }
}

fn switch_request(target_branch: &str) -> QuickSwitchRequest {
    QuickSwitchRequest {
        target_branch: target_branch.to_string(),
        carry_changes: false,
        pull_after_switch: false,
        create_from_remote: None,
        merge_untracked: false,
    }
}

fn commit_dated(fixture: &FixtureRepo, path: &str, content: &str, message: &str, date: &str) {
    fixture.write_worktree_file(path, content);
    fixture.stage_file(path);
    let command = GitCommand::write(vec![
        OsString::from("commit"),
        OsString::from("-m"),
        OsString::from(message),
    ])
    .with_environment(OsString::from("GIT_AUTHOR_DATE"), OsString::from(date))
    .with_environment(OsString::from("GIT_COMMITTER_DATE"), OsString::from(date));
    fixture.repo.run(command).unwrap();
}

fn symbolic_ref(fixture: &FixtureRepo, name: &str) -> Option<String> {
    let args = [
        OsString::from("symbolic-ref"),
        OsString::from("--quiet"),
        OsString::from("--short"),
        OsString::from(name),
    ];
    fixture
        .repo
        .run(GitCommand::read(args.to_vec()))
        .ok()
        .map(|output| String::from_utf8(output.stdout).unwrap().trim().to_string())
        .filter(|value| !value.is_empty())
}

fn present_branch(fixture: &FixtureRepo) -> Option<String> {
    let args = [
        OsString::from("symbolic-ref"),
        OsString::from("--quiet"),
        OsString::from("refs/githelper/present"),
    ];
    fixture
        .repo
        .run(GitCommand::read(args.to_vec()))
        .ok()
        .and_then(|output| {
            String::from_utf8(output.stdout)
                .ok()
                .map(|value| value.trim().to_string())
        })
        .and_then(|value| value.strip_prefix("refs/heads/").map(str::to_string))
}

fn branch_tip(fixture: &FixtureRepo, branch: &str) -> String {
    let spec = format!("refs/heads/{branch}");
    let args = [
        OsString::from("rev-parse"),
        OsString::from(spec.as_str()),
    ];
    String::from_utf8(
        fixture
            .repo
            .run(GitCommand::read(args.to_vec()))
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string()
}

fn read_worktree(fixture: &FixtureRepo, path: &str) -> String {
    fs::read_to_string(fixture.root.path().join(path))
        .unwrap()
        .replace("\r\n", "\n")
}
