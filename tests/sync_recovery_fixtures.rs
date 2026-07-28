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
