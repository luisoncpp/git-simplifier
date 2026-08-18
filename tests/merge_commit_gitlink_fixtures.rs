mod support;

use std::ffi::OsString;

use git_helper_core::{ExcludeSubmoduleRequest, GitCommand, RepoPath};
use support::fixture_repo::FixtureRepo;

#[test]
fn commit_merge_keeps_base_gitlink_out_of_the_pr() {
    let fixture = FixtureRepo::new();
    fixture.set_config("githelper.base", "refs/remotes/origin/base");
    let child = FixtureRepo::new();
    let ours = child.head();
    fixture.switch_to_base();
    fixture.add_gitlink("wiki", &ours, "add wiki");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    fixture
        .repo
        .run(GitCommand::write(args(&["merge", "--ff-only", "base"])))
        .unwrap();
    fixture.commit_file("README.md", "feature\n", "feature change");
    child.commit_file("nested.md", "moved\n", "move wiki");
    let theirs = child.head();
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base changed\n", "base change");
    fixture.add_gitlink("wiki", &theirs, "base moved wiki");
    fixture.set_base_ref();
    fixture.switch_to_feature();
    exclude_wiki(&fixture);
    start_merge(&fixture);
    fixture.write_worktree_file("README.md", "resolved\n");
    fixture.stage_file("README.md");
    let before = pr_paths(&fixture);
    assert!(!bytes_has_path(&before, "wiki"), "{before:?}");

    let plan = fixture.repo.plan_commit_merge().unwrap();
    fixture.repo.apply_commit_merge(&plan).unwrap();

    assert_eq!(gitlink_at(&fixture, "HEAD"), theirs);
    let after = pr_paths(&fixture);
    assert!(!bytes_has_path(&after, "wiki"), "{after:?}");
}

fn exclude_wiki(fixture: &FixtureRepo) {
    let plan = fixture
        .repo
        .plan_exclude_submodule(ExcludeSubmoduleRequest {
            path: RepoPath::new("wiki".to_string()).unwrap(),
            install_hook: true,
            disable_recurse: true,
        })
        .unwrap();
    fixture.repo.apply_exclude_submodule(&plan).unwrap();
}

fn start_merge(fixture: &FixtureRepo) {
    let result = fixture.repo.run(GitCommand::write(args(&[
        "merge",
        "--no-edit",
        "base",
    ])));
    assert!(result.is_err());
}

fn pr_paths(fixture: &FixtureRepo) -> Vec<u8> {
    fixture
        .repo
        .run(GitCommand::read(args(&[
            "diff",
            "--name-only",
            "--no-relative",
            "--ignore-submodules=none",
            "-z",
            "refs/remotes/origin/base...HEAD",
        ])))
        .unwrap()
        .stdout
}

fn gitlink_at(fixture: &FixtureRepo, treeish: &str) -> String {
    let output = fixture
        .repo
        .run(GitCommand::read(args(&["ls-tree", treeish, "--", "wiki"])))
        .unwrap();
    let line = String::from_utf8(output.stdout).unwrap();
    let object = line.split_whitespace().nth(2).expect("gitlink object");
    object.to_string()
}

fn bytes_has_path(bytes: &[u8], path: &str) -> bool {
    bytes
        .split(|byte| *byte == 0)
        .any(|entry| entry == path.as_bytes())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
