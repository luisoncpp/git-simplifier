mod support;

use std::ffi::OsString;
use std::net::TcpListener;
use std::sync::mpsc;
use std::time::Duration;

use git_helper_core::{FetchControl, FetchStatus, GitCommand};
use support::fixture_repo::FixtureRepo;

#[test]
fn fetch_reports_progress_and_advances_remote_tracking_refs() {
    let fixture = FixtureRepo::new();
    let remote = fixture.add_bare_origin();
    let bare = remote.path().to_str().unwrap().to_string();
    // A donor clone pushes a commit the fixture lacks, so fetch must receive objects.
    let donor_root = tempfile::tempdir().unwrap();
    run_git_at(donor_root.path(), &["clone", &bare, "repo"]);
    let donor_repo = donor_root.path().join("repo");
    run_git_at(&donor_repo, &["config", "user.name", "Donor"]);
    run_git_at(&donor_repo, &["config", "user.email", "donor@test"]);
    run_git_at(&donor_repo, &["switch", "base"]);
    std::fs::write(donor_repo.join("base.txt"), "base update\n").unwrap();
    run_git_at(&donor_repo, &["add", "base.txt"]);
    run_git_at(&donor_repo, &["commit", "-m", "base change"]);
    let base_head = head_at(&donor_repo);
    run_git_at(&donor_repo, &["push", "origin", "base"]);
    fixture.switch_to_feature();

    let mut events = Vec::new();
    let status = fixture
        .repo
        .fetch_remotes_with_progress(&FetchControl::new(), |event| events.push(event))
        .unwrap();

    assert_eq!(status, FetchStatus::Completed);
    assert!(!events.is_empty());
    assert!(events.iter().all(|event| event.done <= event.total));
    assert!(events.iter().all(|event| !event.phase.is_empty()));
    assert_eq!(remote_base(&fixture), base_head);
}

#[test]
fn fetch_cancelled_before_it_starts_spawns_nothing() {
    let fixture = FixtureRepo::new();
    let control = FetchControl::new();
    control.cancel();

    let status = fixture
        .repo
        .fetch_remotes_with_progress(&control, |_| {})
        .unwrap();

    assert_eq!(status, FetchStatus::Cancelled);
    assert!(!control.is_running());
}

#[test]
fn fetch_can_be_cancelled_mid_flight() {
    let fixture = FixtureRepo::new();
    let port = silent_http_port();
    run_git(
        &fixture,
        &["remote", "add", "origin", &format!("http://127.0.0.1:{port}/repo.git")],
    );
    let control = FetchControl::new();
    let (tx, rx) = mpsc::channel();
    {
        let control = control.clone();
        std::thread::spawn(move || {
            let result = fixture
                .repo
                .fetch_remotes_with_progress(&control, |_| {});
            let _ = tx.send(result);
        });
    }
    wait_until_running(&control);

    control.cancel();

    let status = rx
        .recv_timeout(Duration::from_secs(30))
        .expect("a cancelled fetch must not hang")
        .unwrap();
    assert_eq!(status, FetchStatus::Cancelled);
}

#[test]
fn fetch_failure_reports_git_stderr_without_progress_noise() {
    let fixture = FixtureRepo::new();
    let port = unused_port();
    run_git(
        &fixture,
        &["remote", "add", "origin", &format!("http://127.0.0.1:{port}/repo.git")],
    );

    let error = fixture
        .repo
        .fetch_remotes_with_progress(&FetchControl::new(), |_| {})
        .unwrap_err();

    let text = error.to_string();
    assert!(text.contains("unable to access"), "unexpected error: {text}");
}

/// A listener that accepts connections and never answers keeps Git's HTTP
/// transport waiting forever — until the cancel kills it. Leaking the accepted
/// socket keeps the connection open for the rest of the test process.
fn silent_http_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            std::mem::forget(stream);
        }
    });
    port
}

/// Binding then dropping leaves the port closed, so the connection is refused
/// immediately instead of hanging.
fn unused_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_until_running(control: &FetchControl) {
    for _ in 0..500 {
        if control.is_running() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("fetch child was never registered");
}

fn remote_base(fixture: &FixtureRepo) -> String {
    let output = fixture
        .repo
        .run(GitCommand::read(args(&["rev-parse", "refs/remotes/origin/base"])))
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn run_git(fixture: &FixtureRepo, values: &[&str]) {
    fixture.repo.run(GitCommand::write(args(values))).unwrap();
}

fn run_git_at(repo: &std::path::Path, values: &[&str]) {
    std::process::Command::new("git")
        .current_dir(repo)
        .args(values)
        .status()
        .expect("git command failed");
}

fn head_at(repo: &std::path::Path) -> String {
    let output = std::process::Command::new("git")
        .current_dir(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}
