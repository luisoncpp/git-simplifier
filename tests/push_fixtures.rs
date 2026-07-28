mod support;

use std::ffi::OsString;
use std::path::Path;

use git_helper_core::{ForcePushError, GitCommand};
use support::fixture_repo::FixtureRepo;
use tempfile::TempDir;

#[test]
fn force_push_uses_the_observed_remote_sha_as_an_explicit_lease() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    let remote = configure_remote(&fixture);
    fixture.commit_file("local.txt", "local\n", "local change");

    let plan = fixture.repo.plan_force_push().unwrap();
    assert_eq!(plan.remote, "origin");
    assert!(plan.command.contains(&format!(
        "--force-with-lease={}:{}",
        plan.remote_branch, plan.expected_remote
    )));

    let result = fixture.repo.apply_force_push(&plan).unwrap();

    assert_eq!(result.new_head, plan.source_head);
    assert_eq!(
        remote_head(&fixture, remote.path()),
        result.new_head.to_string()
    );
    let log =
        std::fs::read_to_string(fixture.root.path().join(".git/githelper/oplog.json")).unwrap();
    assert!(log.contains("\"operation\": \"force-push\""));
}

#[test]
fn force_push_rejects_a_plan_when_the_observed_remote_ref_changes() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    let _remote = configure_remote(&fixture);
    fixture.commit_file("local.txt", "local\n", "local change");
    let plan = fixture.repo.plan_force_push().unwrap();

    run(
        &fixture,
        vec!["update-ref", "refs/remotes/origin/feature", "HEAD~2"],
    );

    assert!(matches!(
        fixture.repo.apply_force_push(&plan),
        Err(ForcePushError::StalePlan)
    ));
}

#[test]
fn force_push_lease_rejects_a_remote_advance_hidden_by_a_stale_tracking_ref() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    let _remote = configure_remote(&fixture);
    fixture.commit_file("local.txt", "local\n", "local change");
    let plan = fixture.repo.plan_force_push().unwrap();

    run(
        &fixture,
        vec!["push", "--force", "origin", "base:refs/heads/feature"],
    );
    run(
        &fixture,
        vec![
            "update-ref",
            "refs/remotes/origin/feature",
            plan.expected_remote.as_str(),
        ],
    );

    assert!(matches!(
        fixture.repo.apply_force_push(&plan),
        Err(ForcePushError::Git(_))
    ));
}

#[test]
fn force_push_requires_a_remote_upstream() {
    let fixture = FixtureRepo::new();

    assert!(matches!(
        fixture.repo.plan_force_push(),
        Err(ForcePushError::NoUpstream)
    ));
}

fn configure_remote(fixture: &FixtureRepo) -> TempDir {
    let remote = tempfile::tempdir().unwrap();
    let remote_path = remote.path().to_string_lossy().to_string();
    run(fixture, vec!["init", "--bare", remote_path.as_str()]);
    run(
        fixture,
        vec!["remote", "add", "origin", remote_path.as_str()],
    );
    run(fixture, vec!["push", "--set-upstream", "origin", "feature"]);
    remote
}

fn remote_head(fixture: &FixtureRepo, remote: &Path) -> String {
    let remote_path = remote.to_string_lossy().to_string();
    let values = vec![
        OsString::from("--git-dir"),
        OsString::from(remote_path),
        OsString::from("rev-parse"),
        OsString::from("refs/heads/feature"),
    ];
    String::from_utf8(fixture.repo.run(GitCommand::read(values)).unwrap().stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn run(fixture: &FixtureRepo, values: Vec<&str>) {
    let args = values.into_iter().map(OsString::from).collect();
    fixture.repo.run(GitCommand::write(args)).unwrap();
}
