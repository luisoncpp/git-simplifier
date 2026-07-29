mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{GitCommand, QuickSwitchRequest, SwitchError};
use support::fixture_repo::FixtureRepo;
use support::submodule::{add_submodule, head, run};

#[test]
fn switch_saves_tracked_work_and_restores_staged_state_explicitly() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "staged\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("README.md", "unstaged\n");

    let plan = fixture.repo.plan_quick_switch(request("other")).unwrap();
    let result = fixture.repo.apply_quick_switch(&plan).unwrap();

    assert_eq!(result.source_branch, "feature");
    assert_eq!(current_branch(&fixture), "other");
    assert_eq!(read_worktree(&fixture, "README.md"), "base\n");
    assert_eq!(fixture.repo.list_saved_work().unwrap().len(), 1);

    fixture.checkout("feature");
    let restored = fixture.repo.restore_saved_work().unwrap();

    assert!(restored.applied_index);
    assert_eq!(current_branch(&fixture), "feature");
    assert_eq!(read_worktree(&fixture, "README.md"), "unstaged\n");
    assert!(fixture
        .cached_paths()
        .split(|byte| *byte == 0)
        .any(|path| path == b"README.md"));
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
}

#[test]
fn switch_carries_tracked_changes_onto_the_target_branch() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "staged\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("README.md", "unstaged\n");

    let plan = fixture.repo.plan_quick_switch(carry_request("other")).unwrap();
    let result = fixture.repo.apply_quick_switch(&plan).unwrap();

    assert_eq!(result.source_branch, "feature");
    assert_eq!(current_branch(&fixture), "other");
    assert_eq!(read_worktree(&fixture, "README.md"), "unstaged\n");
    assert!(result.carried_index.is_some());
    assert!(result.saved_work.is_none());
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
}

#[test]
fn carry_allows_switch_when_source_already_has_saved_work() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "first\n");
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();
    fixture.checkout("feature");
    assert!(fixture.repo.list_saved_work().unwrap().len() == 1);
    fixture.write_worktree_file("README.md", "carried\n");

    assert!(matches!(
        fixture.repo.plan_quick_switch(request("other")),
        Err(SwitchError::ExistingSavedWork(branch)) if branch == "feature"
    ));

    let plan = fixture.repo.plan_quick_switch(carry_request("other")).unwrap();
    fixture.repo.apply_quick_switch(&plan).unwrap();

    assert_eq!(current_branch(&fixture), "other");
    assert_eq!(read_worktree(&fixture, "README.md"), "carried\n");
    assert_eq!(fixture.repo.list_saved_work().unwrap().len(), 1);
}

#[test]
fn carry_warns_when_pop_conflicts_on_divergent_files() {
    let fixture = fixture_with_target("other");
    fixture.checkout("other");
    fixture.commit_file("README.md", "target version\n", "target readme");
    fixture.checkout("feature");
    fixture.write_worktree_file("README.md", "my edit\n");

    let plan = fixture.repo.plan_quick_switch(carry_request("other")).unwrap();
    let result = fixture.repo.apply_quick_switch(&plan).unwrap();

    assert_eq!(current_branch(&fixture), "other");
    assert!(result.carry_warning.is_some());
}

#[test]
fn switch_rejects_an_untracked_file_that_target_would_overwrite() {
    let fixture = fixture_with_target("other");
    fixture.checkout("other");
    fixture.commit_file("collision.txt", "target\n", "target file");
    fixture.checkout("feature");
    fixture.write_worktree_file("collision.txt", "local\n");

    let result = fixture.repo.plan_quick_switch(request("other"));

    assert!(matches!(
        result,
        Err(SwitchError::UntrackedConflict(paths)) if paths.contains("collision.txt")
    ));
    assert_eq!(current_branch(&fixture), "feature");
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("collision.txt")).unwrap(),
        "local\n"
    );
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
}

#[test]
fn switch_leaves_non_conflicting_untracked_files_in_place() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("local-only.txt", "keep\n");

    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();

    assert_eq!(current_branch(&fixture), "other");
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("local-only.txt")).unwrap(),
        "keep\n"
    );
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
}

#[test]
fn switch_preserves_submodule_worktree_state() {
    let fixture = fixture_with_target("other");
    let child = add_submodule(&fixture);
    run(&fixture.repo, &["config", "submodule.recurse", "true"]);
    let child_head = head(&child);
    fixture.write_worktree_file("Modules/Engine/README.md", "local change\n");
    fixture.write_worktree_file("Modules/Engine/scratch.txt", "untracked\n");
    fixture.write_worktree_file("README.md", "outer local\n");

    let result = fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();

    assert!(result.saved_work.is_some());
    assert_eq!(head(&child), child_head);
    assert_eq!(
        read_worktree(&fixture, "Modules/Engine/README.md"),
        "local change\n"
    );
    assert_eq!(
        read_worktree(&fixture, "Modules/Engine/scratch.txt"),
        "untracked\n"
    );
    assert_eq!(read_worktree(&fixture, "README.md"), "base\n");

    run(
        &fixture.repo,
        &["-c", "submodule.recurse=false", "switch", "feature"],
    );
    fixture.repo.restore_saved_work().unwrap();

    assert_eq!(head(&child), child_head);
    assert_eq!(
        read_worktree(&fixture, "Modules/Engine/README.md"),
        "local change\n"
    );
    assert_eq!(read_worktree(&fixture, "README.md"), "outer local\n");
}

#[test]
fn switch_does_not_overwrite_existing_saved_work_for_a_branch() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "first\n");
    let plan = fixture.repo.plan_quick_switch(request("other")).unwrap();
    fixture.repo.apply_quick_switch(&plan).unwrap();
    fixture.checkout("feature");
    fixture.write_worktree_file("README.md", "second\n");

    let result = fixture.repo.plan_quick_switch(request("other"));

    assert!(matches!(result, Err(SwitchError::ExistingSavedWork(branch)) if branch == "feature"));
    assert_eq!(fixture.repo.list_saved_work().unwrap().len(), 1);
}

#[test]
fn saved_work_can_be_deleted_explicitly_without_switching() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "saved\n");
    let plan = fixture.repo.plan_quick_switch(request("other")).unwrap();
    fixture.repo.apply_quick_switch(&plan).unwrap();

    let deleted = fixture
        .repo
        .delete_saved_work("feature".to_string())
        .unwrap();

    assert_eq!(deleted.branch, "feature");
    assert!(fixture.repo.list_saved_work().unwrap().is_empty());
}

fn fixture_with_target(target: &str) -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.branch(target);
    fixture.checkout("feature");
    fixture
}

fn request(target_branch: &str) -> QuickSwitchRequest {
    QuickSwitchRequest {
        target_branch: target_branch.to_string(),
        carry_changes: false,
    }
}

fn carry_request(target_branch: &str) -> QuickSwitchRequest {
    QuickSwitchRequest {
        target_branch: target_branch.to_string(),
        carry_changes: true,
    }
}

fn current_branch(fixture: &FixtureRepo) -> String {
    let args = [
        OsString::from("symbolic-ref"),
        OsString::from("--short"),
        OsString::from("HEAD"),
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
