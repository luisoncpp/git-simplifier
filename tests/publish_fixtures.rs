mod support;

use std::ffi::OsString;

use git_helper_core::{GitCommand, PublishError, RefName, RepoPath, SplitBranchRequest};
use support::fixture_repo::FixtureRepo;
use tempfile::TempDir;

#[test]
fn publishing_a_split_branch_creates_the_remote_branch_and_sets_upstream() {
    let (fixture, _remote) = fixture_with_remote();
    let split = split_out(&fixture, "carved");

    let plan = fixture
        .repo
        .plan_publish_branch("carved".to_string())
        .unwrap();
    let result = fixture.repo.apply_publish_branch(&plan).unwrap();

    assert_eq!(result.remote, "origin");
    assert_eq!(result.head, split);
    assert_eq!(
        result.upstream,
        RefName::new("refs/remotes/origin/carved".to_string()).unwrap()
    );
    assert_eq!(remote_head(&fixture, "carved"), split.to_string());
    assert_eq!(config(&fixture, "branch.carved.remote"), "origin");
    assert_eq!(config(&fixture, "branch.carved.merge"), "refs/heads/carved");
}

/// The plan is a first publish, not a force push, so an existing remote branch
/// must stop it at planning rather than be quietly replaced.
#[test]
fn publishing_refuses_a_branch_the_remote_already_has() {
    let (fixture, _remote) = fixture_with_remote();
    split_out(&fixture, "carved");
    run(
        &fixture,
        &["update-ref", "refs/remotes/origin/carved", "carved"],
    );

    let result = fixture.repo.plan_publish_branch("carved".to_string());

    assert!(matches!(
        result,
        Err(PublishError::ExistingRemoteBranch(name)) if name == "refs/remotes/origin/carved"
    ));
}

/// The remote-tracking ref can be absent while the remote itself already has the
/// branch. The empty lease is what catches that, so it has to be exercised.
#[test]
fn the_empty_lease_refuses_a_branch_that_appeared_on_the_remote() {
    let (fixture, _remote) = fixture_with_remote();
    split_out(&fixture, "carved");
    let plan = fixture
        .repo
        .plan_publish_branch("carved".to_string())
        .unwrap();
    run(&fixture, &["update-ref", "refs/heads/carved-rival", "base"]);
    run(
        &fixture,
        &[
            "push",
            "origin",
            "refs/heads/carved-rival:refs/heads/carved",
        ],
    );
    run(
        &fixture,
        &["update-ref", "-d", "refs/remotes/origin/carved"],
    );

    let result = fixture.repo.apply_publish_branch(&plan);

    assert!(result.is_err(), "the push overwrote a branch it never saw");
}

#[test]
fn publishing_reports_a_missing_branch_instead_of_pushing_nothing() {
    let (fixture, _remote) = fixture_with_remote();

    let result = fixture.repo.plan_publish_branch("absent".to_string());

    assert!(matches!(
        result,
        Err(PublishError::MissingBranch(name)) if name == "absent"
    ));
}

#[test]
fn publishing_needs_a_configured_remote_instead_of_assuming_origin() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("Assets/kept.txt", "kept\n", "kept");
    split_out(&fixture, "carved");

    let result = fixture.repo.plan_publish_branch("carved".to_string());

    assert!(matches!(result, Err(PublishError::NoRemote)));
}

#[test]
fn publishing_records_an_irreversible_operation_naming_the_remote() {
    let (fixture, _remote) = fixture_with_remote();
    split_out(&fixture, "carved");
    let plan = fixture
        .repo
        .plan_publish_branch("carved".to_string())
        .unwrap();

    fixture.repo.apply_publish_branch(&plan).unwrap();

    let entry = fixture
        .repo
        .list_operations()
        .unwrap()
        .into_iter()
        .find(|entry| entry.operation == "publish-branch")
        .unwrap();
    assert!(entry.finished.is_some());
    assert!(!entry.reversible);
    assert_eq!(entry.commands, vec![plan.command.clone()]);
    assert_eq!(
        entry.details.get("remote").map(String::as_str),
        Some("origin")
    );
}

/// The remote directory is returned so the caller keeps it alive for the test.
fn fixture_with_remote() -> (FixtureRepo, TempDir) {
    let fixture = FixtureRepo::new();
    fixture.commit_file("Assets/kept.txt", "kept\n", "kept");
    let remote = fixture.add_bare_origin();
    run(&fixture, &["config", "branch.feature.remote", "origin"]);
    run(
        &fixture,
        &["config", "branch.feature.merge", "refs/heads/feature"],
    );
    (fixture, remote)
}

fn split_out(fixture: &FixtureRepo, branch: &str) -> git_helper_core::ObjectId {
    let plan = fixture
        .repo
        .plan_split_branch(SplitBranchRequest {
            base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
            new_branch: branch.to_string(),
            paths: vec![RepoPath::new("Assets/kept.txt".to_string()).unwrap()],
            message: None,
        })
        .unwrap();
    fixture.repo.apply_split_branch(&plan).unwrap().commit
}

fn remote_head(fixture: &FixtureRepo, branch: &str) -> String {
    let spec = format!("refs/heads/{branch}");
    let output = read(fixture, &["ls-remote", "origin", &spec]);
    String::from_utf8(output)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string()
}

fn config(fixture: &FixtureRepo, key: &str) -> String {
    String::from_utf8(read(fixture, &["config", "--local", "--get", key]))
        .unwrap()
        .trim()
        .to_string()
}

fn run(fixture: &FixtureRepo, values: &[&str]) {
    let args: Vec<OsString> = values.iter().map(|value| OsString::from(*value)).collect();
    fixture.repo.run(GitCommand::write(args)).unwrap();
}

fn read(fixture: &FixtureRepo, values: &[&str]) -> Vec<u8> {
    let args: Vec<OsString> = values.iter().map(|value| OsString::from(*value)).collect();
    fixture.repo.run(GitCommand::read(args)).unwrap().stdout
}
