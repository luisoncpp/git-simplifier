mod support;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use git_helper_core::RefName;
use support::fixture_repo::FixtureRepo;

#[test]
fn editable_commit_discovery_ignores_git_log_record_whitespace() {
    let fixture = FixtureRepo::new();
    fixture.commit_file("feature.txt", "feature\n", "editable message");
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();

    let commits = fixture.repo.list_editable_commits(base).unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].subject, "editable message");
    assert_eq!(commits[0].message, "editable message");
}

#[test]
fn setting_base_completes_without_relocking_the_repository() {
    let fixture = FixtureRepo::new();
    let base = RefName::new("refs/remotes/origin/base".to_string()).unwrap();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        sender.send(fixture.repo.set_base(base)).unwrap();
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("set_base deadlocked while acquiring the repository write lock");
    result.unwrap();
}
