mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::GitCommand;
use support::fixture_repo::FixtureRepo;

#[test]
fn plan_refuses_when_no_merge_is_in_progress() {
    let fixture = FixtureRepo::new();
    let error = fixture.repo.plan_commit_merge().unwrap_err();
    assert!(error.to_string().contains("no merge in progress"));
}

#[test]
fn plan_refuses_unmerged_paths() {
    let fixture = FixtureRepo::new();
    start_readme_merge_conflict(&fixture);
    let error = fixture.repo.plan_commit_merge().unwrap_err();
    assert!(error.to_string().contains("Resolve merge conflicts first"));
}

#[test]
fn plan_refuses_rebase() {
    let fixture = FixtureRepo::new();
    fs::write(fixture.root.path().join(".git").join("rebase-merge"), "dummy").unwrap();
    let error = fixture.repo.plan_commit_merge().unwrap_err();
    assert!(error.to_string().contains("rebase"));
}

#[test]
fn plan_tree_omits_a_staged_file_that_was_not_part_of_the_merge() {
    let fixture = FixtureRepo::new();
    start_readme_merge_conflict(&fixture);
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("wip.txt", "unrelated\n");
    fixture.stage_file("wip.txt");

    let plan = fixture.repo.plan_commit_merge().unwrap();

    let listed = name_only(&fixture, &["diff", "--no-relative", "--name-only", "-z", "MERGE_HEAD", plan.tree.as_str()]);
    assert!(!bytes_has_path(&listed, "wip.txt"), "{listed:?}");
    assert!(plan.excluded_paths.iter().any(|path| path.as_str() == "wip.txt"));
}

#[test]
fn apply_commits_the_merge_and_leaves_unrelated_work_uncommitted() {
    let fixture = FixtureRepo::new();
    start_readme_merge_conflict(&fixture);
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("wip.txt", "unrelated\n");
    fixture.stage_file("wip.txt");
    let old_head = fixture.head();
    let plan = fixture.repo.plan_commit_merge().unwrap();

    let result = fixture.repo.apply_commit_merge(&plan).unwrap();

    assert_ne!(result.new_head.as_str(), old_head);
    assert!(!fixture.root.path().join(".git").join("MERGE_HEAD").exists());
    assert_eq!(read_worktree(&fixture, "wip.txt"), "unrelated\n");
    let committed = name_only(
        &fixture,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "-z",
            result.merge_head.as_str(),
            result.new_head.as_str(),
        ],
    );
    assert!(!bytes_has_path(&committed, "wip.txt"));
    let parents = String::from_utf8(read_git(&fixture, &["rev-list", "--parents", "-n", "1", "HEAD"])).unwrap();
    assert_eq!(parents.split_whitespace().count(), 3);
}

#[test]
fn apply_does_not_add_paths_to_base_triple_dot_head() {
    let fixture = FixtureRepo::new();
    fixture.set_config("githelper.base", "refs/remotes/origin/base");
    start_readme_merge_conflict(&fixture);
    let before = name_only(
        &fixture,
        &["diff", "--name-only", "--no-relative", "-z", "refs/remotes/origin/base...HEAD"],
    );
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("secret.txt", "should not land in the PR\n");
    fixture.stage_file("secret.txt");
    let plan = fixture.repo.plan_commit_merge().unwrap();
    fixture.repo.apply_commit_merge(&plan).unwrap();

    let after = name_only(
        &fixture,
        &["diff", "--name-only", "--no-relative", "-z", "refs/remotes/origin/base...HEAD"],
    );
    for path in after.split(|byte| *byte == 0).filter(|part| !part.is_empty()) {
        assert!(
            before.split(|byte| *byte == 0).any(|old| old == path),
            "extra path in Base...HEAD: {}",
            String::from_utf8_lossy(path)
        );
    }
}

#[test]
fn overview_reports_merge_in_progress() {
    let fixture = FixtureRepo::new();
    assert!(!fixture.repo.overview().unwrap().merge_in_progress);
    start_readme_merge_conflict(&fixture);
    assert!(fixture.repo.overview().unwrap().merge_in_progress);
}

fn start_readme_merge_conflict(fixture: &FixtureRepo) {
    fixture.commit_file("README.md", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base changed\n", "base change");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    let result = fixture.repo.run(GitCommand::write(vec![
        OsString::from("merge"),
        OsString::from("--no-edit"),
        OsString::from("base"),
    ]));
    assert!(result.is_err());
}

fn name_only(fixture: &FixtureRepo, args: &[&str]) -> Vec<u8> {
    let values = args.iter().map(|value| OsString::from(*value)).collect();
    fixture
        .repo
        .run(GitCommand::read(values))
        .unwrap()
        .stdout
}

fn read_git(fixture: &FixtureRepo, args: &[&str]) -> Vec<u8> {
    name_only(fixture, args)
}

fn read_worktree(fixture: &FixtureRepo, path: &str) -> String {
    std::fs::read_to_string(fixture.root.path().join(path)).unwrap()
}

fn bytes_has_path(bytes: &[u8], path: &str) -> bool {
    bytes
        .split(|byte| *byte == 0)
        .any(|entry| entry == path.as_bytes())
}
