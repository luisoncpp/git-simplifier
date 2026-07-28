mod support;

use std::ffi::OsString;

use git_helper_core::{GitCommand, RefName, RepoPath, SplitBranchRequest, SplitError};
use support::fixture_repo::FixtureRepo;

#[test]
fn split_copies_only_the_selected_paths_onto_a_new_branch() {
    let fixture = fixture_with_two_changes();
    let head_before = fixture.head();

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt"]))
        .unwrap();
    let result = fixture.repo.apply_split_branch(&plan).unwrap();

    assert_eq!(result.branch, "carved");
    assert_eq!(result.merge_base, plan.merge_base);
    assert_eq!(
        show(&fixture, "carved:Assets/kept.txt"),
        b"kept change\n".to_vec()
    );
    assert!(!branch_has_path(&fixture, "carved", "Assets/other.txt"));
    assert_eq!(parent(&fixture, "carved"), plan.merge_base.to_string());
    assert_eq!(fixture.head(), head_before);
    assert!(fixture.status().is_empty());
}

#[test]
fn split_carries_a_changed_meta_file_with_its_asset() {
    let fixture = fixture_with_two_changes();
    fixture.commit_file("Assets/kept.txt.meta", "guid: 2\n", "meta change");

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt"]))
        .unwrap();
    fixture.repo.apply_split_branch(&plan).unwrap();

    assert_eq!(
        plan.companion_paths,
        vec![RepoPath::new("Assets/kept.txt.meta".to_string()).unwrap()]
    );
    assert!(branch_has_path(&fixture, "carved", "Assets/kept.txt.meta"));
}

#[test]
fn split_carries_a_selected_meta_file_with_its_asset() {
    let fixture = fixture_with_two_changes();
    fixture.commit_file("Assets/kept.txt.meta", "guid: 2\n", "meta change");

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt.meta"]))
        .unwrap();

    assert_eq!(
        plan.companion_paths,
        vec![RepoPath::new("Assets/kept.txt".to_string()).unwrap()]
    );
}

#[test]
fn split_selects_every_changed_file_under_a_selected_directory() {
    let fixture = fixture_with_two_changes();

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets"]))
        .unwrap();

    assert_eq!(plan.changed_paths.len(), 2);
}

#[test]
fn split_carries_a_deletion_made_on_the_source_branch() {
    let fixture = fixture_with_two_changes();
    fixture.remove_file("README.md", "drop readme");

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["README.md"]))
        .unwrap();
    fixture.repo.apply_split_branch(&plan).unwrap();

    assert!(!branch_has_path(&fixture, "carved", "README.md"));
}

#[test]
fn split_derives_a_message_when_the_caller_supplies_none() {
    let fixture = fixture_with_two_changes();

    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt"]))
        .unwrap();

    assert!(plan.message_is_derived);
    assert_eq!(plan.message, b"Split 1 file from feature\n".to_vec());
}

#[test]
fn split_keeps_a_caller_message_verbatim() {
    let fixture = fixture_with_two_changes();
    let mut request = request("carved", &["Assets/kept.txt"]);
    request.message = Some(b"chosen subject\n".to_vec());

    let plan = fixture.repo.plan_split_branch(request).unwrap();
    fixture.repo.apply_split_branch(&plan).unwrap();

    assert!(!plan.message_is_derived);
    assert_eq!(
        fixture.commit_message("carved"),
        b"chosen subject\n".to_vec()
    );
}

#[test]
fn split_rejects_a_branch_name_that_already_exists() {
    let fixture = fixture_with_two_changes();

    let result = fixture
        .repo
        .plan_split_branch(request("base", &["Assets/kept.txt"]));

    assert!(matches!(
        result,
        Err(SplitError::ExistingBranch(name)) if name == "base"
    ));
}

#[test]
fn split_rejects_a_selection_that_carries_no_change() {
    let fixture = fixture_with_two_changes();

    let result = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/absent.txt"]));

    assert!(matches!(result, Err(SplitError::NoChanges)));
}

#[test]
fn split_rejects_an_empty_selection() {
    let fixture = fixture_with_two_changes();

    let result = fixture.repo.plan_split_branch(request("carved", &[]));

    assert!(matches!(result, Err(SplitError::EmptySelection)));
}

#[test]
fn split_rejects_a_plan_whose_source_branch_moved() {
    let fixture = fixture_with_two_changes();
    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt"]))
        .unwrap();
    fixture.commit_file("Assets/late.txt", "late\n", "late change");

    let result = fixture.repo.apply_split_branch(&plan);

    assert!(matches!(result, Err(SplitError::StalePlan)));
}

#[test]
fn split_removes_its_temporary_worktree_and_records_a_reversible_operation() {
    let fixture = fixture_with_two_changes();
    let plan = fixture
        .repo
        .plan_split_branch(request("carved", &["Assets/kept.txt"]))
        .unwrap();

    fixture.repo.apply_split_branch(&plan).unwrap();

    let worktrees = read(&fixture, &["worktree", "list", "--porcelain"]);
    assert_eq!(
        String::from_utf8(worktrees)
            .unwrap()
            .matches("worktree ")
            .count(),
        1
    );
    let entry = fixture
        .repo
        .list_operations()
        .unwrap()
        .into_iter()
        .find(|entry| entry.operation == "split-branch")
        .unwrap();
    assert!(entry.finished.is_some());
    assert_eq!(
        entry.recovery_command,
        Some("git update-ref -d refs/heads/carved".to_string())
    );
    assert_eq!(entry.commands, plan.commands);
}

fn fixture_with_two_changes() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.commit_file("Assets/kept.txt", "kept change\n", "kept");
    fixture.commit_file("Assets/other.txt", "other change\n", "other");
    fixture
}

fn request(branch: &str, paths: &[&str]) -> SplitBranchRequest {
    SplitBranchRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
        new_branch: branch.to_string(),
        paths: paths
            .iter()
            .map(|path| RepoPath::new(path.to_string()).unwrap())
            .collect(),
        message: None,
    }
}

fn branch_has_path(fixture: &FixtureRepo, branch: &str, path: &str) -> bool {
    !read(fixture, &["ls-tree", "-r", "-z", branch, "--", path]).is_empty()
}

fn parent(fixture: &FixtureRepo, branch: &str) -> String {
    let spec = format!("{branch}^1");
    String::from_utf8(read(fixture, &["rev-parse", &spec]))
        .unwrap()
        .trim()
        .to_string()
}

fn show(fixture: &FixtureRepo, spec: &str) -> Vec<u8> {
    read(fixture, &["show", spec])
}

fn read(fixture: &FixtureRepo, values: &[&str]) -> Vec<u8> {
    let args: Vec<OsString> = values.iter().map(|value| OsString::from(*value)).collect();
    fixture.repo.run(GitCommand::read(args)).unwrap().stdout
}
