mod support;

use git_helper_core::{RecoveryEntry, RefName, RepoPath, SyncRequest, UncommitRequest};
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

/// A finished operation kept whatever in-flight phase it last recorded, so the
/// history showed a completed sync as still stopped at its conflict.
#[test]
fn a_finished_operation_records_no_in_flight_phase() {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base update\n", "base change");
    fixture.switch_to_feature();
    fixture.write_worktree_file("README.md", "local\n");

    let _ = fixture.repo.sync(SyncRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
    });
    fixture.reset_hard();
    fixture.repo.resume_sync().unwrap();

    let entry = latest(&fixture);

    assert!(entry.finished.is_some());
    assert_eq!(entry.phase, None);
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
