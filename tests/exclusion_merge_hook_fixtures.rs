mod support;

use std::ffi::OsString;

use git_helper_core::{ExcludeSubmoduleRequest, GitCommand, RepoPath};
use support::fixture_repo::FixtureRepo;

#[test]
fn guard_allows_a_gitlink_that_matches_merge_head() {
    let fixture = conflicted_wiki_merge();
    fixture
        .repo
        .run(GitCommand::write(args(&["commit", "--no-edit"])))
        .expect("merge commit with matching MERGE_HEAD gitlink");
}

#[test]
fn guard_rejects_a_dirty_gitlink_during_merge() {
    let fixture = conflicted_wiki_merge();
    let dirty = FixtureRepo::new().head();
    fixture
        .repo
        .run(GitCommand::write(args(&[
            "update-index",
            "--add",
            "--cacheinfo",
            "160000",
            &dirty,
            "wiki",
        ])))
        .unwrap();
    let result = fixture.repo.run(GitCommand::write(args(&[
        "commit",
        "--no-edit",
    ])));
    assert!(result.is_err());
}

fn conflicted_wiki_merge() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    let child = FixtureRepo::new();
    let ours = child.head();
    seed_wiki_on_base(&fixture, &ours);
    fixture.commit_file("README.md", "feature\n", "feature change");
    child.commit_file("nested.md", "moved\n", "move wiki");
    move_wiki_on_base(&fixture, &child.head());
    exclude_and_merge(&fixture);
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    fixture
}

fn seed_wiki_on_base(fixture: &FixtureRepo, ours: &str) {
    fixture.switch_to_base();
    fixture.add_gitlink("wiki", ours, "add wiki");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    fixture
        .repo
        .run(GitCommand::write(args(&["merge", "--ff-only", "base"])))
        .unwrap();
}

fn move_wiki_on_base(fixture: &FixtureRepo, theirs: &str) {
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base changed\n", "base change");
    fixture.add_gitlink("wiki", theirs, "base moved wiki");
    fixture.set_base_ref();
    fixture.switch_to_feature();
}

fn exclude_and_merge(fixture: &FixtureRepo) {
    let plan = fixture
        .repo
        .plan_exclude_submodule(ExcludeSubmoduleRequest {
            path: RepoPath::new("wiki".to_string()).unwrap(),
            install_hook: true,
            disable_recurse: true,
        })
        .unwrap();
    fixture.repo.apply_exclude_submodule(&plan).unwrap();
    let merge = fixture.repo.run(GitCommand::write(args(&[
        "merge",
        "--no-edit",
        "base",
    ])));
    assert!(merge.is_err());
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
