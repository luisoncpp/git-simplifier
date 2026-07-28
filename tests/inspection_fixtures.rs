mod support;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use git_helper_core::RefName;
use support::fixture_repo::FixtureRepo;

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
