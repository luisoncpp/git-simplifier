mod support;

use std::fs;

use git_helper_core::{
    ExcludeSubmoduleRequest, RefName, RepoPath, SubmoduleCleanupRequest,
};
use support::fixture_repo::FixtureRepo;
use support::submodule::{add_submodule, head, run};

#[test]
fn dirty_submodules_lists_local_dirt() {
    let fixture = FixtureRepo::new();
    let child = add_submodule_on_base(&fixture);
    fixture.switch_to_feature();
    run(&fixture.repo, &["merge", "--ff-only", "base"]);
    fs::write(
        fixture.root.path().join("Modules/Engine/README.md"),
        b"dirty\n",
    )
    .unwrap();
    let _child = child;

    let dirty = fixture
        .repo
        .list_dirty_submodules(Some(base()))
        .unwrap();
    let engine = dirty
        .iter()
        .find(|entry| entry.path.as_str() == "Modules/Engine")
        .expect("dirty submodule");
    assert!(engine.local_dirty);
    assert!(!engine.in_editable_range);
}

#[test]
fn dirty_submodules_lists_committed_pointer_even_when_excluded() {
    let fixture = FixtureRepo::new();
    let child = add_submodule(&fixture);
    bump_submodule(&fixture, &child);
    let plan = fixture
        .repo
        .plan_exclude_submodule(ExcludeSubmoduleRequest {
            path: RepoPath::new("Modules/Engine".to_string()).unwrap(),
            install_hook: false,
            disable_recurse: false,
        })
        .unwrap();
    fixture.repo.apply_exclude_submodule(&plan).unwrap();

    let dirty = fixture
        .repo
        .list_dirty_submodules(Some(base()))
        .unwrap();
    let engine = dirty
        .iter()
        .find(|entry| entry.path.as_str() == "Modules/Engine")
        .expect("committed submodule diff");
    assert!(engine.in_editable_range);
}

#[test]
fn cleanup_uncommits_and_reverts_dirty_submodule() {
    let fixture = FixtureRepo::new();
    let child = add_submodule(&fixture);
    bump_submodule(&fixture, &child);
    fs::write(
        fixture.root.path().join("Modules/Engine/README.md"),
        b"dirty\n",
    )
    .unwrap();

    let plan = fixture
        .repo
        .plan_submodule_cleanup(request(
            vec!["Modules/Engine"],
            /*uncommit=*/ true,
            /*revert=*/ true,
        ))
        .unwrap();
    fixture.repo.apply_submodule_cleanup(&plan).unwrap();

    assert!(!fixture.tree_has_path("Modules/Engine"));
    let status = String::from_utf8(fixture.status()).unwrap();
    assert!(!status.contains("Modules/Engine"));
}

#[test]
fn cleanup_revert_only_clears_untracked_inside_submodule() {
    let fixture = FixtureRepo::new();
    add_submodule_on_base(&fixture);
    fixture.switch_to_feature();
    run(&fixture.repo, &["merge", "--ff-only", "base"]);
    let head_before = fixture.head();
    fs::write(
        fixture.root.path().join("Modules/Engine/extra.txt"),
        b"untracked\n",
    )
    .unwrap();

    let plan = fixture
        .repo
        .plan_submodule_cleanup(request(
            vec!["Modules/Engine"],
            /*uncommit=*/ false,
            /*revert=*/ true,
        ))
        .unwrap();
    fixture.repo.apply_submodule_cleanup(&plan).unwrap();

    assert_eq!(fixture.head(), head_before);
    assert!(!fixture
        .root
        .path()
        .join("Modules/Engine/extra.txt")
        .exists());
    let status = String::from_utf8(fixture.status()).unwrap();
    assert!(!status.contains("Modules/Engine"));
}

#[test]
fn cleanup_revert_only_clears_local_dirt_without_rewriting() {
    let fixture = FixtureRepo::new();
    add_submodule_on_base(&fixture);
    fixture.switch_to_feature();
    run(&fixture.repo, &["merge", "--ff-only", "base"]);
    let head_before = fixture.head();
    fs::write(
        fixture.root.path().join("Modules/Engine/README.md"),
        b"dirty\n",
    )
    .unwrap();

    let plan = fixture
        .repo
        .plan_submodule_cleanup(request(
            vec!["Modules/Engine"],
            /*uncommit=*/ false,
            /*revert=*/ true,
        ))
        .unwrap();
    fixture.repo.apply_submodule_cleanup(&plan).unwrap();

    assert_eq!(fixture.head(), head_before);
    let status = String::from_utf8(fixture.status()).unwrap();
    assert!(!status.contains("Modules/Engine"));
}

fn add_submodule_on_base(fixture: &FixtureRepo) -> git_helper_core::GitRepository {
    fixture.switch_to_base();
    let child = add_submodule(fixture);
    fixture.set_base_ref();
    child
}

fn bump_submodule(fixture: &FixtureRepo, child: &git_helper_core::GitRepository) {
    fs::write(
        fixture.root.path().join("Modules/Engine/README.md"),
        b"v2\n",
    )
    .unwrap();
    run(child, &["add", "README.md"]);
    run(child, &["commit", "-m", "v2"]);
    let new_head = head(child);
    fixture.add_gitlink("Modules/Engine", &new_head, "bump submodule");
}

fn request(paths: Vec<&str>, uncommit: bool, revert: bool) -> SubmoduleCleanupRequest {
    SubmoduleCleanupRequest {
        base: base(),
        paths: paths
            .into_iter()
            .map(|path| RepoPath::new(path.to_string()).unwrap())
            .collect(),
        uncommit,
        revert,
    }
}

fn base() -> RefName {
    RefName::new("refs/remotes/origin/base".to_string()).unwrap()
}
