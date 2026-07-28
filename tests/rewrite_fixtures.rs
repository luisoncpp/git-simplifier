mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{
    ApplyError, EditMessageRequest, GitCommand, RefName, RepoPath, RewriteAction, UncommitRequest,
};
use support::fixture_repo::FixtureRepo;

#[test]
fn plan_is_read_only_and_marks_empty_commits() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    let accidental = fixture.head();
    fixture.commit_file("other.txt", "other\n", "other");
    let before = fixture.head();

    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    assert_eq!(fixture.head(), before);
    assert_eq!(plan.commits.len(), 2);
    assert!(plan
        .dropped_commits
        .iter()
        .any(|id| id.as_str() == accidental));
    assert_eq!(plan.commits[0].action, RewriteAction::Drop);
    assert_eq!(plan.commits[1].action, RewriteAction::Rebuild);
}

#[test]
fn apply_rewrite_preserves_worktree_and_resets_the_index_path() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    let result = fixture.repo.apply_rewrite(&plan).unwrap();
    assert_eq!(result.dropped_commits.len(), 1);
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("accidental.txt")).unwrap(),
        "keep locally\n"
    );
    assert!(fixture
        .status()
        .windows("accidental.txt".len())
        .any(|window| window == b"accidental.txt"));
    assert!(fixture
        .root
        .path()
        .join(".git/githelper/oplog.json")
        .exists());
}

#[test]
fn base_merge_is_rebuilt_with_its_second_parent() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    fixture.switch_to_base();
    fixture.commit_file("base-change.txt", "base\n", "base change");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    fixture.merge("base", "merge base");

    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    assert!(plan
        .commits
        .iter()
        .any(|commit| !commit.additional_parents.is_empty()));
    assert_eq!(plan.base.as_str(), fixture.base_head().as_str());
}

#[test]
fn teammate_merge_commit_is_not_in_the_editable_range() {
    let fixture = FixtureRepo::new();
    fixture.switch_to_base();
    fixture.branch("teammate");
    fixture.commit_file("teammate.txt", "teammate\n", "teammate");
    let teammate = fixture.head();
    fixture.checkout("feature");
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    fixture.merge("teammate", "merge teammate");

    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    assert!(!plan
        .commits
        .iter()
        .any(|commit| commit.source.as_str() == teammate));
}

#[test]
fn repeated_path_edits_are_planned_for_each_first_parent_commit() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "one\n", "first");
    fixture.commit_file("accidental.txt", "two\n", "second");

    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    assert_eq!(plan.commits.len(), 2);
    assert!(plan
        .commits
        .iter()
        .all(|commit| commit.action == RewriteAction::Drop));
}

#[test]
fn submodule_gitlink_is_treated_as_a_tree_entry() {
    let fixture = FixtureRepo::new();
    let child = FixtureRepo::new();
    let child_head = child.head();
    fixture.add_gitlink("Modules/Engine", &child_head, "add submodule pointer");

    let plan = fixture
        .repo
        .plan_uncommit(request("Modules/Engine".to_string()))
        .unwrap();

    assert!(plan
        .base_entries
        .get(&RepoPath::new("Modules/Engine".to_string()).unwrap())
        .unwrap()
        .is_none());
    assert_eq!(plan.commits[0].action, RewriteAction::Drop);
}

#[test]
fn special_path_is_passed_literally_and_omitted_from_resulting_tree() {
    let fixture = FixtureRepo::new();
    let path = "folder/name with spaces & [glob].txt";
    fixture.commit_file(path, "keep locally\n", "special path");
    let plan = fixture
        .repo
        .plan_uncommit(request(path.to_string()))
        .unwrap();

    fixture.repo.apply_rewrite(&plan).unwrap();

    assert!(!fixture.tree_has_path(path));
    assert_eq!(
        fs::read_to_string(fixture.root.path().join(path)).unwrap(),
        "keep locally\n"
    );
}

#[test]
fn stale_head_plan_is_rejected_without_moving_the_branch() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();
    fixture.commit_file("later.txt", "later\n", "later");
    let before = fixture.head();

    let result = fixture.repo.apply_rewrite(&plan);

    assert!(matches!(result, Err(ApplyError::StalePlan)));
    assert_eq!(fixture.head(), before);
    assert!(!fixture
        .root
        .path()
        .join(".git/githelper/oplog.json")
        .exists());
}

#[test]
fn stale_base_plan_is_rejected_without_moving_the_branch() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();
    let before = fixture.head();
    fixture.switch_to_base();
    fixture.commit_file("base-moved.txt", "base moved\n", "move base");
    fixture.set_base_ref();
    fixture.switch_to_feature();

    let result = fixture.repo.apply_rewrite(&plan);

    assert!(matches!(result, Err(ApplyError::StalePlan)));
    assert_eq!(fixture.head(), before);
    assert!(!fixture
        .root
        .path()
        .join(".git/githelper/oplog.json")
        .exists());
}

#[test]
fn existing_base_path_is_restored_in_history_but_not_in_worktree() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("README.md", "feature\n", "change base path");
    fixture.write_worktree_file("README.md", "local dirty\n");
    let plan = fixture
        .repo
        .plan_uncommit(request("README.md".to_string()))
        .unwrap();

    fixture.repo.apply_rewrite(&plan).unwrap();

    assert_eq!(fixture.head_path("README.md"), b"base\n");
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("README.md")).unwrap(),
        "local dirty\n"
    );
}

#[test]
fn unrelated_staged_changes_survive_the_rewrite() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental");
    fixture.write_worktree_file("unrelated.txt", "staged\n");
    fixture.stage_file("unrelated.txt");
    let plan = fixture
        .repo
        .plan_uncommit(request("accidental.txt".to_string()))
        .unwrap();

    fixture.repo.apply_rewrite(&plan).unwrap();

    assert!(fixture
        .cached_paths()
        .split(|byte| *byte == 0)
        .any(|path| path == b"unrelated.txt"));
}

#[test]
fn edit_message_rebuilds_the_chain_without_touching_tree_or_index() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("first.txt", "first\n", "bad first message");
    let target = fixture.head();
    fixture.commit_file("second.txt", "second\n", "second message");
    fixture.write_worktree_file("unrelated.txt", "staged\n");
    fixture.stage_file("unrelated.txt");
    let before_tree = fixture.commit_tree(&target);
    let before_head_tree = fixture.commit_tree(&fixture.head());

    let plan = fixture
        .repo
        .plan_edit_message(edit_message(
            target.clone(),
            b"corrected message\n".to_vec(),
        ))
        .unwrap();

    assert!(plan.dropped_commits.is_empty());
    assert_eq!(plan.commits.len(), 2);
    assert_eq!(plan.commits[0].metadata.message, b"corrected message\n");
    fixture.repo.apply_rewrite(&plan).unwrap();

    assert_eq!(fixture.commit_message(&fixture.head()), b"second message\n");
    assert_eq!(fixture.commit_message("HEAD^"), b"corrected message\n");
    assert_eq!(fixture.commit_message(&target), b"bad first message\n");
    assert_eq!(fixture.commit_tree(&fixture.head()), before_head_tree);
    assert_eq!(
        fs::read_to_string(fixture.root.path().join("unrelated.txt")).unwrap(),
        "staged\n"
    );
    assert_eq!(
        fixture.commit_tree(plan.commits[0].source.as_str()),
        before_tree
    );
    assert!(fixture
        .cached_paths()
        .split(|byte| *byte == 0)
        .any(|path| path == b"unrelated.txt"));
    let log = fs::read_to_string(fixture.root.path().join(".git/githelper/oplog.json")).unwrap();
    assert!(log.contains("\"operation\": \"edit-message\""));
}

#[test]
fn edit_message_preserves_merge_parents_and_changes_only_the_target_message() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "feature message");
    let target = fixture.head();
    fixture.switch_to_base();
    fixture.commit_file("base-change.txt", "base\n", "base change");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    fixture.merge("base", "merge base");

    let plan = fixture
        .repo
        .plan_edit_message(edit_message(target, b"edited feature\n".to_vec()))
        .unwrap();

    assert!(plan
        .commits
        .iter()
        .any(|commit| !commit.additional_parents.is_empty()));
    fixture.repo.apply_rewrite(&plan).unwrap();
    assert_eq!(fixture.commit_message(&fixture.head()), b"merge base\n");
}

#[test]
fn edit_message_rejects_a_commit_already_on_base() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "feature");
    let base = fixture.base_head();
    let target = git_helper_core::ObjectId::new(base).unwrap();

    let result = fixture
        .repo
        .plan_edit_message(edit_message(target.to_string(), b"not allowed\n".to_vec()));

    assert!(result
        .unwrap_err()
        .to_string()
        .contains("outside the Editable range"));
}

fn request(path: String) -> UncommitRequest {
    UncommitRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
        paths: vec![RepoPath::new(path).unwrap()],
    }
}

fn edit_message(commit: String, message: Vec<u8>) -> EditMessageRequest {
    EditMessageRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
        commit: git_helper_core::ObjectId::new(commit).unwrap(),
        message,
    }
}

trait FixtureExt {
    fn base_head(&self) -> String;
}

impl FixtureExt for FixtureRepo {
    fn base_head(&self) -> String {
        let args = vec![
            OsString::from("rev-parse"),
            OsString::from("refs/remotes/origin/base"),
        ];
        String::from_utf8(self.repo.run(GitCommand::read(args)).unwrap().stdout)
            .unwrap()
            .trim()
            .to_string()
    }
}
