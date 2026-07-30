mod support;

use git_helper_core::{RefName, RepoPath, RevertRequest, RevertTarget};
use support::fixture_repo::FixtureRepo;

#[test]
fn revert_paths_are_the_union_of_dirt_and_base_diffs() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("committed.txt", "on feature\n", "add committed");
    fixture.write_worktree_file("README.md", "dirty\n");
    fixture.write_worktree_file("untracked.txt", "leave me\n");

    let paths = fixture
        .repo
        .list_revert_paths(base())
        .unwrap()
        .into_iter()
        .map(|entry| entry.path.as_str().to_string())
        .collect::<Vec<_>>();

    assert!(paths.contains(&"committed.txt".to_string()));
    assert!(paths.contains(&"README.md".to_string()));
    assert!(!paths.iter().any(|path| path == "untracked.txt"));
}

#[test]
fn revert_to_head_restores_index_and_worktree_without_rewriting() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("keep.txt", "feature\n", "add keep");
    let head_before = fixture.head();
    fixture.write_worktree_file("keep.txt", "dirty\n");
    fixture.stage_file("keep.txt");
    fixture.write_worktree_file("other.txt", "untouched dirt\n");

    let plan = fixture
        .repo
        .plan_revert(request(vec!["keep.txt"], RevertTarget::Head))
        .unwrap();
    assert!(plan.commands[0].contains("--source=HEAD"));
    assert!(plan.commands[0].contains(":(top,literal)keep.txt"));

    fixture.repo.apply_revert(&plan).unwrap();

    assert_eq!(fixture.head(), head_before);
    assert_eq!(normalize(&fixture.head_path("keep.txt")), "feature\n");
    assert_eq!(
        normalize(std::fs::read_to_string(fixture.root.path().join("keep.txt")).unwrap().as_bytes()),
        "feature\n"
    );
    assert_eq!(
        normalize(std::fs::read_to_string(fixture.root.path().join("other.txt")).unwrap().as_bytes()),
        "untouched dirt\n"
    );
    let log = std::fs::read_to_string(fixture.root.path().join(".git/githelper/oplog.json")).unwrap();
    assert!(log.contains("\"operation\": \"revert\""));
}

#[test]
fn revert_to_base_leaves_a_local_diff_from_head() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("tracked.txt", "feature version\n", "add tracked");
    let head_before = fixture.head();

    let plan = fixture
        .repo
        .plan_revert(request(vec!["tracked.txt"], RevertTarget::Base))
        .unwrap();
    assert!(plan.commands[0].contains("--source=refs/remotes/origin/base"));

    fixture.repo.apply_revert(&plan).unwrap();

    assert_eq!(fixture.head(), head_before);
    assert!(fixture.tree_has_path("tracked.txt"));
    assert!(!fixture.root.path().join("tracked.txt").exists());
    let status = String::from_utf8(fixture.status()).unwrap();
    assert!(status.contains("tracked.txt"));
}

#[test]
fn revert_pathspecs_work_from_a_subdirectory() {
    let fixture = FixtureRepo::new();
    std::fs::create_dir_all(fixture.root.path().join("dir")).unwrap();
    fixture.commit_file("dir/nested.txt", "feature\n", "add nested");
    fixture.write_worktree_file("dir/nested.txt", "dirty\n");
    let nested = fixture.reopen_at("dir");

    let plan = nested
        .plan_revert(request(vec!["dir/nested.txt"], RevertTarget::Head))
        .unwrap();
    nested.apply_revert(&plan).unwrap();

    assert_eq!(
        normalize(std::fs::read_to_string(fixture.root.path().join("dir/nested.txt")).unwrap().as_bytes()),
        "feature\n"
    );
}

fn request(paths: Vec<&str>, target: RevertTarget) -> RevertRequest {
    RevertRequest {
        base: base(),
        paths: paths
            .into_iter()
            .map(|path| RepoPath::new(path.to_string()).unwrap())
            .collect(),
        target,
    }
}

fn base() -> RefName {
    RefName::new("refs/remotes/origin/base".to_string()).unwrap()
}

fn normalize(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}
