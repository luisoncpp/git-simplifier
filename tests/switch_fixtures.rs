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
fn restore_reports_conflicts_without_retrying_over_the_conflicted_index() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "saved edit\n");
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();
    fixture.checkout("feature");
    fixture.commit_file("README.md", "upstream edit\n", "change readme");

    let error = fixture.repo.restore_saved_work().unwrap_err();

    assert!(matches!(error, SwitchError::SavedWorkConflict));
    assert!(read_worktree(&fixture, "README.md").contains("<<<<<<<"));
    assert_eq!(fixture.repo.list_saved_work().unwrap().len(), 1);
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

/// A pop that cannot complete used to abandon the entry on the shared stash
/// stack, where the app never looks: the panel reported "No Saved work" while
/// the only copy of the carried changes sat in `refs/stash`.
#[test]
fn carry_that_cannot_pop_becomes_listable_saved_work() {
    let fixture = fixture_with_target("other");
    fixture.checkout("other");
    fixture.commit_file("README.md", "target version\n", "target readme");
    fixture.checkout("feature");
    fixture.write_worktree_file("README.md", "my edit\n");

    let plan = fixture.repo.plan_quick_switch(carry_request("other")).unwrap();
    fixture.repo.apply_quick_switch(&plan).unwrap();

    let saved = fixture.repo.list_saved_work().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].branch, "feature");
    assert_eq!(fixture.stash_entries(), 0);
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

#[test]
fn switch_creates_a_local_branch_from_a_remote_tracking_ref() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    let head = fixture.head();
    run(
        &fixture.repo,
        &["update-ref", "refs/remotes/origin/from-remote", &head],
    );

    let plan = fixture
        .repo
        .plan_quick_switch(QuickSwitchRequest {
            target_branch: "from-remote".to_string(),
            carry_changes: false,
            pull_after_switch: false,
            create_from_remote: Some("refs/remotes/origin/from-remote".to_string()),
        })
        .unwrap();
    fixture.repo.apply_quick_switch(&plan).unwrap();

    assert_eq!(current_branch(&fixture), "from-remote");
    let remote = String::from_utf8(
        fixture
            .repo
            .run(GitCommand::read(vec![
                OsString::from("config"),
                OsString::from("--get"),
                OsString::from("branch.from-remote.remote"),
            ]))
            .unwrap()
            .stdout,
    )
    .unwrap();
    let merge = String::from_utf8(
        fixture
            .repo
            .run(GitCommand::read(vec![
                OsString::from("config"),
                OsString::from("--get"),
                OsString::from("branch.from-remote.merge"),
            ]))
            .unwrap()
            .stdout,
    )
    .unwrap();
    assert_eq!(remote.trim(), "origin");
    assert_eq!(merge.trim(), "refs/heads/from-remote");
}

#[test]
fn switch_pulls_same_named_remote_with_ff_only() {
    let fixture = fixture_with_target("other");
    let _remote = fixture.add_bare_origin();
    run(&fixture.repo, &["push", "-u", "origin", "other:other"]);
    fixture.checkout("other");
    fixture.commit_file("README.md", "remote ahead\n", "ahead on other");
    run(&fixture.repo, &["push", "origin", "other"]);
    run(&fixture.repo, &["reset", "--hard", "HEAD~1"]);
    fixture.checkout("feature");

    let plan = fixture
        .repo
        .plan_quick_switch(QuickSwitchRequest {
            target_branch: "other".to_string(),
            carry_changes: false,
            pull_after_switch: true,
            create_from_remote: None,
        })
        .unwrap();
    assert_eq!(
        plan.pull_remote_ref.as_deref(),
        Some("refs/remotes/origin/other")
    );
    let result = fixture.repo.apply_quick_switch(&plan).unwrap();

    assert!(result.pulled);
    assert!(!result.pull_decision_needed);
    assert_eq!(current_branch(&fixture), "other");
    assert_eq!(read_worktree(&fixture, "README.md"), "remote ahead\n");
}

#[test]
fn diverged_pull_pauses_for_a_user_decision() {
    let fixture = fixture_with_target("other");
    let _remote = fixture.add_bare_origin();
    run(&fixture.repo, &["push", "-u", "origin", "other:other"]);
    fixture.checkout("other");
    fixture.commit_file("README.md", "remote edit\n", "remote change");
    run(&fixture.repo, &["push", "origin", "other"]);
    run(&fixture.repo, &["reset", "--hard", "HEAD~1"]);
    fixture.commit_file("README.md", "local edit\n", "local change");
    fixture.checkout("feature");

    let plan = fixture
        .repo
        .plan_quick_switch(QuickSwitchRequest {
            target_branch: "other".to_string(),
            carry_changes: false,
            pull_after_switch: true,
            create_from_remote: None,
        })
        .unwrap();
    let result = fixture.repo.apply_quick_switch(&plan).unwrap();

    assert!(result.pull_decision_needed);
    assert_eq!(current_branch(&fixture), "other");
    assert!(fixture.repo.quick_switch_status().unwrap().is_some());

    let resolved = fixture
        .repo
        .resolve_quick_switch_pull(git_helper_core::PullResolution::ReplaceWithRemote)
        .unwrap();
    assert!(resolved.pulled);
    assert_eq!(read_worktree(&fixture, "README.md"), "remote edit\n");
    assert!(fixture.repo.quick_switch_status().unwrap().is_none());
}

fn fixture_with_target(target: &str) -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.branch(target);
    fixture.checkout("feature");
    fixture
}

#[test]
fn preview_saved_work_lists_apply_delta_on_current_branch() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "saved edit\n");
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();
    fixture.checkout("feature");

    let preview = fixture
        .repo
        .preview_saved_work_apply("feature".to_string())
        .unwrap();
    assert!(preview.on_current_branch);
    let files = fixture
        .repo
        .saved_work_apply_files_diff(preview.before_tree, preview.after_tree)
        .unwrap();
    assert!(files
        .iter()
        .any(|file| file.path.as_str() == "README.md"));
}

#[test]
fn preview_saved_work_on_other_branch_uses_that_tip() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "saved edit\n");
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();

    let preview = fixture
        .repo
        .preview_saved_work_apply("feature".to_string())
        .unwrap();
    assert!(!preview.on_current_branch);
    let files = fixture
        .repo
        .saved_work_apply_files_diff(preview.before_tree, preview.after_tree)
        .unwrap();
    assert!(files
        .iter()
        .any(|file| file.path.as_str() == "README.md"));
}

#[test]
fn preview_saved_work_flags_worktree_conflicts() {
    let fixture = fixture_with_target("other");
    fixture.write_worktree_file("README.md", "saved edit\n");
    fixture
        .repo
        .apply_quick_switch(&fixture.repo.plan_quick_switch(request("other")).unwrap())
        .unwrap();
    fixture.checkout("feature");
    fixture.commit_file("README.md", "upstream edit\n", "change readme");

    let preview = fixture
        .repo
        .preview_saved_work_apply("feature".to_string())
        .unwrap();
    assert!(preview.worktree_conflicts);
}

fn request(target_branch: &str) -> QuickSwitchRequest {
    QuickSwitchRequest {
        target_branch: target_branch.to_string(),
        carry_changes: false,
        pull_after_switch: false,
        create_from_remote: None,
    }
}

fn carry_request(target_branch: &str) -> QuickSwitchRequest {
    QuickSwitchRequest {
        target_branch: target_branch.to_string(),
        carry_changes: true,
        pull_after_switch: false,
        create_from_remote: None,
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
