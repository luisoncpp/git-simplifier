mod support;

use git_helper_core::{RecoveryEntry, RefName, RepoPath, UncommitRequest};
use support::fixture_repo::FixtureRepo;

#[test]
fn operation_history_exposes_ref_recovery_for_a_completed_rewrite() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("accidental.txt", "keep locally\n", "accidental file");
    fixture.set_base_ref();
    let old_head = fixture.head();

    let plan = fixture
        .repo
        .plan_uncommit(UncommitRequest {
            base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
            paths: vec![RepoPath::new("accidental.txt".to_string()).unwrap()],
        })
        .unwrap();
    fixture.repo.apply_rewrite(&plan).unwrap();

    let entry = latest(&fixture);

    assert_eq!(entry.operation, "uncommit");
    assert_eq!(entry.refs_before["refs/heads/feature"], old_head);
    assert!(entry.finished.is_some());
    assert_eq!(
        entry.recovery_command,
        Some(format!("git update-ref refs/heads/feature {old_head}"))
    );
}

#[test]
fn operation_history_is_empty_before_the_first_operation() {
    let fixture = FixtureRepo::new();

    assert!(fixture.repo.list_operations().unwrap().is_empty());
}

fn latest(fixture: &FixtureRepo) -> RecoveryEntry {
    fixture
        .repo
        .list_operations()
        .unwrap()
        .into_iter()
        .last()
        .unwrap()
}
