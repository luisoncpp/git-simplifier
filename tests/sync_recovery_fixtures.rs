mod support;

use std::ffi::OsString;

use git_helper_core::{GitCommand, RefName, SyncError, SyncPhase, SyncRequest};
use support::fixture_repo::FixtureRepo;

#[test]
fn sync_retries_a_recorded_fetch_after_the_remote_recovers() {
    let fixture = fixture_with_interrupted_fetch();
    fixture.configure_origin_to_self();

    let resumed = fixture.repo.resume_sync().unwrap();

    assert!(fixture.tree_has_path("base.txt"));
    assert_ne!(resumed.old_head, resumed.new_head);
    assert!(fixture.repo.sync_status().unwrap().is_none());
}

#[test]
fn fetch_retry_refuses_a_changed_head() {
    let fixture = fixture_with_interrupted_fetch();
    fixture.configure_origin_to_self();
    fixture.commit_file("later.txt", "later\n", "later change");

    let result = fixture.repo.resume_sync();

    assert!(matches!(
        result,
        Err(SyncError::InvalidState(message)) if message.contains("branch or HEAD changed")
    ));
    assert_eq!(
        fixture.repo.sync_status().unwrap().unwrap().phase,
        SyncPhase::Fetch
    );
}

/// Resolving a reapply conflict by discarding leaves the worktree clean, which
/// used to read as "the Saved work is in the tree" and finished the sync silently.
#[test]
fn resume_reports_saved_work_that_never_reached_the_worktree() {
    let fixture = fixture_with_conflicted_reapply();

    fixture.reset_hard();
    let resumed = fixture.repo.resume_sync().unwrap();

    let saved_work = resumed.saved_work.expect("the snapshot stays recorded");
    assert!(resumed.saved_work_warning.is_some());
    assert!(fixture.ref_exists(&saved_work.reference));
}

#[test]
fn resume_stays_silent_when_the_resolution_kept_the_work() {
    let fixture = fixture_with_conflicted_reapply();

    fixture.write_worktree_file("README.md", "resolved local\n");
    fixture.stage_file("README.md");
    let resumed = fixture.repo.resume_sync().unwrap();

    assert!(resumed.saved_work_warning.is_none());
}

/// Leaves the sync parked at `wip-reapply-conflict` with a recorded snapshot.
fn fixture_with_conflicted_reapply() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("README.md", "base update\n", "base change");
    fixture.switch_to_feature();
    fixture.write_worktree_file("README.md", "local\n");

    let result = fixture.repo.sync(SyncRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
    });

    assert!(matches!(result, Err(SyncError::WipReapplyConflict { .. })));
    fixture
}

fn fixture_with_interrupted_fetch() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.configure_origin_to_self();
    fixture.commit_file("feature.txt", "feature\n", "feature change");
    fixture.switch_to_base();
    fixture.commit_file("base.txt", "base update\n", "base change");
    fixture.switch_to_feature();
    let missing = fixture.root.path().join("missing-remote");
    set_remote_url(&fixture, missing.to_string_lossy().as_ref());
    let result = fixture.repo.sync(request());
    assert!(matches!(result, Err(SyncError::Git(_))));
    assert_eq!(
        fixture.repo.sync_status().unwrap().unwrap().phase,
        SyncPhase::Fetch
    );
    fixture
}

fn set_remote_url(fixture: &FixtureRepo, url: &str) {
    let args = ["config", "remote.origin.url", url]
        .into_iter()
        .map(OsString::from)
        .collect();
    fixture.repo.run(GitCommand::write(args)).unwrap();
}

fn request() -> SyncRequest {
    SyncRequest {
        base: RefName::new("refs/remotes/origin/base".to_string()).unwrap(),
    }
}
