mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{
    ExcludeSubmoduleRequest, GitCommand, GitRepository, RefName, RepoPath, RepositoryConfig,
    SyncError, SyncPhase, SyncRequest,
};
use support::fixture_repo::FixtureRepo;

#[test]
fn sync_fetches_base_and_reapplies_staged_work() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    let old_head = fixture.head();
    fixture.switch_to_base();
    fixture.commit_file("base.txt", "base update\n", "base change");
    fixture.switch_to_feature();
    fixture.write_worktree_file("README.md", "staged\n");
    fixture.stage_file("README.md");
    fixture.write_worktree_file("README.md", "unstaged\n");

    let result = fixture.repo.sync(request()).unwrap();

    assert_eq!(result.old_head.as_str(), old_head);
    assert_ne!(result.old_head, result.new_head);
    assert!(result.applied_index);
    assert_eq!(read_worktree(&fixture, "README.md"), "unstaged\n");
    assert_eq!(fixture.cached_paths(), b"README.md\0");
    assert!(fixture.tree_has_path("base.txt"));
    assert!(result.saved_work.is_some());
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn commit_merge_then_resume_sync_completes_a_base_conflict() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("README.md", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base changed\n", "base change");
    fixture.switch_to_feature();

    let result = fixture.repo.sync(request());
    assert!(matches!(result, Err(SyncError::BaseMergeConflict { .. })));

    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    let plan = fixture.repo.plan_commit_merge().unwrap();
    fixture.repo.apply_commit_merge(&plan).unwrap();

    let resumed = fixture.repo.resume_sync().unwrap();
    assert_eq!(read_worktree(&fixture, "README.md"), "resolved\n");
    assert!(resumed.saved_work.is_none());
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn sync_labels_a_base_merge_conflict_and_can_resume_after_commit() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("README.md", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base changed\n", "base change");
    fixture.switch_to_feature();

    let result = fixture.repo.sync(request());

    assert!(matches!(result, Err(SyncError::BaseMergeConflict { .. })));
    assert_eq!(
        fixture.repo.sync_status().unwrap().unwrap().phase,
        SyncPhase::BaseMergeConflict
    );
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    fixture.commit_file("README.md", "resolved\n", "resolve base merge");

    let resumed = fixture.repo.resume_sync().unwrap();

    assert_eq!(read_worktree(&fixture, "README.md"), "resolved\n");
    assert!(resumed.saved_work.is_none());
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn sync_labels_a_saved_work_conflict_and_resumes_after_resolution() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base update\n", "base change");
    fixture.switch_to_feature();
    fixture.write_worktree_file("README.md", "local\n");

    let result = fixture.repo.sync(request());

    assert!(matches!(result, Err(SyncError::WipReapplyConflict { .. })));
    assert_eq!(
        fixture.repo.sync_status().unwrap().unwrap().phase,
        SyncPhase::WipReapplyConflict
    );
    fixture.write_worktree_file("README.md", "resolved local\n");
    fixture.stage_file("README.md");

    let resumed = fixture.repo.resume_sync().unwrap();

    assert!(!resumed.applied_index);
    assert_eq!(read_worktree(&fixture, "README.md"), "resolved local\n");
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn sync_rejects_an_untracked_path_that_base_would_write() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.switch_to_base();
    fixture.commit_file("collision.txt", "base\n", "base collision");
    fixture.switch_to_feature();
    fixture.write_worktree_file("collision.txt", "local\n");

    let result = fixture.repo.sync(request());

    assert!(
        matches!(result, Err(SyncError::UntrackedConflict(paths)) if paths.contains("collision.txt"))
    );
    assert_eq!(read_worktree(&fixture, "collision.txt"), "local\n");
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn sync_preserves_excluded_submodule_worktree_state() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.switch_to_base();
    fixture.commit_file("base.txt", "base update\n", "base change");
    fixture.switch_to_feature();
    let child = add_excluded_submodule(&fixture);
    run(&fixture.repo, &["config", "submodule.recurse", "true"]);
    let child_head = repo_head(&child);
    fixture.write_worktree_file("Modules/Engine/README.md", "local change\n");
    fixture.write_worktree_file("Modules/Engine/scratch.txt", "untracked\n");
    fixture.write_worktree_file("README.md", "outer local\n");

    let result = fixture.repo.sync(request()).unwrap();

    assert_eq!(repo_head(&child), child_head);
    assert_eq!(
        read_worktree(&fixture, "Modules/Engine/README.md"),
        "local change\n"
    );
    assert_eq!(
        read_worktree(&fixture, "Modules/Engine/scratch.txt"),
        "untracked\n"
    );
    assert_eq!(read_worktree(&fixture, "README.md"), "outer local\n");
    assert!(fixture.tree_has_path("base.txt"));
    assert!(result.saved_work.is_some());
}

fn request() -> SyncRequest {
    SyncRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
    }
}

fn read_worktree(fixture: &FixtureRepo, path: &str) -> String {
    fs::read_to_string(fixture.root.path().join(path))
        .unwrap()
        .replace("\r\n", "\n")
}

fn add_excluded_submodule(fixture: &FixtureRepo) -> GitRepository {
    let path = fixture.root.path().join("Modules/Engine");
    fs::create_dir_all(&path).unwrap();
    let child = GitRepository::open(RepositoryConfig {
        path: path.clone(),
        git_executable: "git".into(),
    })
    .unwrap();
    run(&child, &["init", "-b", "main"]);
    run(&child, &["config", "user.name", "Fixture User"]);
    run(&child, &["config", "user.email", "fixture@example.test"]);
    fs::write(path.join("README.md"), b"child\n").unwrap();
    run(&child, &["add", "--", "README.md"]);
    run(&child, &["commit", "-m", "child"]);
    fixture.add_gitlink("Modules/Engine", &repo_head(&child), "add submodule");
    let plan = fixture
        .repo
        .plan_exclude_submodule(ExcludeSubmoduleRequest {
            path: RepoPath::new("Modules/Engine".to_string()).unwrap(),
            install_hook: false,
            disable_recurse: false,
        })
        .unwrap();
    fixture.repo.apply_exclude_submodule(&plan).unwrap();
    child
}

fn repo_head(repo: &GitRepository) -> String {
    let output = repo
        .run(GitCommand::read(args(&["rev-parse", "HEAD"])))
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn run(repo: &GitRepository, values: &[&str]) {
    repo.run(GitCommand::write(args(values))).unwrap();
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
