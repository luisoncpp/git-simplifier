mod support;

use std::ffi::OsString;
use std::fs;

use git_helper_core::{
    CleanupDiscovery, CleanupError, CleanupKind, CleanupRequest, ExclusionReason, GitCommand,
    KeptReason, RecoveryEntry, RefName,
};
use support::fixture_repo::FixtureRepo;
use tempfile::TempDir;

#[test]
fn cleanup_offers_a_branch_merged_into_base() {
    let fixture = fixture_with_merged_branch();

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    let spike = choice(&discovery, "spike").unwrap();
    assert_eq!(spike.reference, "refs/heads/spike");
    assert_eq!(spike.kind, CleanupKind::Local);
    assert_eq!(spike.author_email, "fixture@example.test");
    assert!(spike.mine);
    assert!(!spike.protected);
}

#[test]
fn cleanup_does_not_offer_a_branch_that_is_not_merged_into_base() {
    let fixture = fixture_with_merged_branch();
    fixture.checkout("base");
    fixture.branch("ongoing");
    fixture.commit_file("ongoing.txt", "wip\n", "ongoing work");
    fixture.checkout("feature");

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    assert!(choice(&discovery, "ongoing").is_none());
    assert!(!excluded(&discovery, "ongoing"));
}

#[test]
fn cleanup_never_offers_the_current_branch() {
    let fixture = fixture_with_merged_branch();

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    // `feature` sits on the commit Base grew from, so it is genuinely merged:
    // this proves the exclusion fires rather than the branch being ineligible.
    assert!(choice(&discovery, "feature").is_none());
    assert_eq!(reason(&discovery, "feature"), Some(ExclusionReason::CurrentBranch));
}

#[test]
fn cleanup_never_offers_the_local_branch_the_base_tracks() {
    let fixture = fixture_with_merged_branch();

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    assert!(choice(&discovery, "base").is_none());
    assert_eq!(reason(&discovery, "base"), Some(ExclusionReason::BaseBranch));
}

#[test]
fn cleanup_never_offers_a_branch_with_saved_work() {
    let fixture = fixture_with_merged_branch();
    let head = read(&fixture, &["rev-parse", "refs/heads/spike"]);
    run(&fixture, &["update-ref", "refs/githelper/wip/spike", &head]);

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    assert!(choice(&discovery, "spike").is_none());
    assert_eq!(reason(&discovery, "spike"), Some(ExclusionReason::SavedWork));
}

#[test]
fn cleanup_never_offers_a_branch_checked_out_in_another_worktree() {
    let fixture = fixture_with_merged_branch();
    let _holder = fixture.add_worktree("spike");

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    assert!(choice(&discovery, "spike").is_none());
    assert_eq!(
        reason(&discovery, "spike"),
        Some(ExclusionReason::CheckedOutInWorktree)
    );
}

#[test]
fn cleanup_reports_a_branch_authored_by_someone_else_as_not_mine() {
    let fixture = fixture_with_merged_branch();
    fixture.checkout("base");
    fixture.branch("theirs");
    commit_as(&fixture, "other@example.test", "their work");
    fixture.checkout("base");
    fixture.merge("theirs", "merge theirs");
    fixture.set_base_ref();
    fixture.checkout("feature");

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    let theirs = choice(&discovery, "theirs").unwrap();
    assert_eq!(theirs.author_email, "other@example.test");
    assert!(!theirs.mine);
    assert!(choice(&discovery, "spike").unwrap().mine);
}

#[test]
fn cleanup_offers_a_shared_name_without_pre_ticking_it() {
    let fixture = fixture_with_merged_branch();
    fixture.checkout("base");
    fixture.branch("develop");
    fixture.commit_file("develop.txt", "shared\n", "shared work");
    fixture.checkout("base");
    fixture.merge("develop", "merge develop");
    fixture.set_base_ref();
    fixture.checkout("feature");

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    assert!(choice(&discovery, "develop").unwrap().protected);
    assert!(!choice(&discovery, "spike").unwrap().protected);
}

#[test]
fn cleanup_offers_a_merged_remote_branch_with_no_local_counterpart() {
    let fixture = fixture_with_merged_branch();
    fixture.configure_origin_to_self();
    let spike = read(&fixture, &["rev-parse", "refs/heads/spike"]);
    run(&fixture, &["update-ref", "refs/remotes/origin/orphan", &spike]);

    let discovery = fixture.repo.discover_cleanup(&base()).unwrap();

    let orphan = choice(&discovery, "orphan").unwrap();
    assert_eq!(orphan.kind, CleanupKind::RemoteOnly);
    assert_eq!(orphan.reference, "refs/remotes/origin/orphan");
    assert_eq!(
        orphan.remote.as_ref().unwrap().remote_ref,
        "refs/heads/orphan"
    );
}

#[test]
fn cleanup_deletes_a_local_branch_and_records_a_reversible_operation() {
    let fixture = fixture_with_merged_branch();
    let head = read(&fixture, &["rev-parse", "refs/heads/spike"]);
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();

    let result = fixture.repo.apply_cleanup(&plan).unwrap();

    assert_eq!(result.deleted_local, vec!["refs/heads/spike".to_string()]);
    assert!(result.deleted_remote.is_empty());
    assert!(!reference_exists(&fixture, "refs/heads/spike"));
    let entry = record(&fixture, "cleanup-local-branches");
    assert!(entry.reversible);
    assert!(entry.finished.is_some());
    assert_eq!(entry.refs_before.get("refs/heads/spike"), Some(&head));
    assert_eq!(entry.commands, plan.commands);
}

/// The recovery panel promises a copy-pasteable restore. Running it is the only
/// way that promise is real rather than cosmetic.
#[test]
fn the_recorded_recovery_command_restores_the_deleted_branch() {
    let fixture = fixture_with_merged_branch();
    let head = read(&fixture, &["rev-parse", "refs/heads/spike"]);
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    fixture.repo.apply_cleanup(&plan).unwrap();

    let command = record(&fixture, "cleanup-local-branches")
        .recovery_command
        .unwrap();
    execute(&fixture, &command);

    assert_eq!(read(&fixture, &["rev-parse", "refs/heads/spike"]), head);
}

#[test]
fn cleanup_rejects_an_empty_selection() {
    let fixture = fixture_with_merged_branch();

    let error = fixture.repo.plan_cleanup(request(&[])).unwrap_err();

    assert!(matches!(error, CleanupError::EmptySelection));
}

/// Planning recomputes eligibility instead of trusting the caller, so a safety
/// exclusion cannot be bypassed by sending its reference anyway.
#[test]
fn cleanup_rejects_a_chosen_branch_that_is_not_eligible() {
    let fixture = fixture_with_merged_branch();

    let error = fixture
        .repo
        .plan_cleanup(request(&["refs/heads/base"]))
        .unwrap_err();

    assert!(matches!(error, CleanupError::NotEligible(name) if name == "refs/heads/base"));
    assert!(reference_exists(&fixture, "refs/heads/base"));
}

#[test]
fn cleanup_refuses_a_branch_that_moved_after_planning() {
    let fixture = fixture_with_merged_branch();
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    fixture.checkout("spike");
    fixture.commit_file("late.txt", "late\n", "late work");
    fixture.checkout("feature");

    let error = fixture.repo.apply_cleanup(&plan).unwrap_err();

    assert!(matches!(error, CleanupError::StalePlan));
    assert!(reference_exists(&fixture, "refs/heads/spike"));
}

/// Advancing Base can only add merged branches, never unmerge one, so a fetch
/// between review and apply must not invalidate the plan.
#[test]
fn cleanup_tolerates_a_base_that_moved_forward() {
    let fixture = fixture_with_merged_branch();
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    fixture.checkout("base");
    fixture.commit_file("later.txt", "later\n", "later base work");
    fixture.set_base_ref();
    fixture.checkout("feature");

    fixture.repo.apply_cleanup(&plan).unwrap();

    assert!(!reference_exists(&fixture, "refs/heads/spike"));
}

#[test]
fn cleanup_refuses_while_a_git_operation_is_in_progress() {
    let fixture = fixture_with_merged_branch();
    let head = read(&fixture, &["rev-parse", "HEAD"]);
    fs::write(fixture.root.path().join(".git").join("MERGE_HEAD"), head).unwrap();

    let error = fixture
        .repo
        .plan_cleanup(request(&["refs/heads/spike"]))
        .unwrap_err();

    assert!(matches!(error, CleanupError::InvalidState(_)));
}

#[test]
fn cleanup_deletes_the_remote_counterpart_and_records_it_as_irreversible() {
    let (fixture, _origin) = fixture_with_published_branch();
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    assert_eq!(plan.remote_count, 1);

    let result = fixture.repo.apply_cleanup(&plan).unwrap();

    assert_eq!(
        result.deleted_remote,
        vec!["refs/remotes/origin/spike".to_string()]
    );
    assert!(read(&fixture, &["ls-remote", "origin", "refs/heads/spike"]).is_empty());
    assert!(!reference_exists(&fixture, "refs/heads/spike"));
    let remote = record(&fixture, "cleanup-remote-branches");
    assert!(!remote.reversible);
    assert_eq!(remote.recovery_command, None);
    assert!(record(&fixture, "cleanup-local-branches").reversible);
}

#[test]
fn cleanup_keeps_a_remote_branch_that_is_ahead_of_base() {
    let (fixture, _origin) = fixture_with_published_branch();
    run(&fixture, &["switch", "-c", "ahead", "spike"]);
    fixture.commit_file("ahead.txt", "ahead\n", "ahead work");
    run(&fixture, &["push", "origin", "ahead:refs/heads/spike"]);
    run(&fixture, &["switch", "feature"]);
    run(&fixture, &["branch", "-D", "ahead"]);
    run(&fixture, &["fetch", "origin"]);

    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    fixture.repo.apply_cleanup(&plan).unwrap();

    assert_eq!(plan.remote_count, 0);
    assert_eq!(plan.kept_remotes[0].reason, KeptReason::NotMerged);
    assert!(!read(&fixture, &["ls-remote", "origin", "refs/heads/spike"]).is_empty());
    assert!(!reference_exists(&fixture, "refs/heads/spike"));
}

/// The lease is the only guard against deleting work pushed since the last
/// fetch, and remotes run first so a rejection destroys nothing at all.
#[test]
fn cleanup_refuses_to_delete_a_remote_branch_that_moved_since_the_fetch() {
    let (fixture, _origin) = fixture_with_published_branch();
    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();
    let _other = push_from_elsewhere(&fixture, "spike");

    let error = fixture.repo.apply_cleanup(&plan).unwrap_err();

    assert!(matches!(error, CleanupError::RemoteRejected { .. }));
    assert!(!read(&fixture, &["ls-remote", "origin", "refs/heads/spike"]).is_empty());
    assert!(reference_exists(&fixture, "refs/heads/spike"));
}

#[test]
fn cleanup_does_not_touch_the_remote_when_counterparts_are_disabled() {
    let (fixture, _origin) = fixture_with_published_branch();
    let plan = fixture
        .repo
        .plan_cleanup(CleanupRequest {
            base: base(),
            chosen: vec!["refs/heads/spike".to_string()],
            include_remote_counterparts: false,
        })
        .unwrap();

    fixture.repo.apply_cleanup(&plan).unwrap();

    assert_eq!(plan.remote_count, 0);
    assert_eq!(plan.kept_remotes[0].reason, KeptReason::Disabled);
    assert!(!plan.commands.iter().any(|command| command.contains("push")));
    assert!(!read(&fixture, &["ls-remote", "origin", "refs/heads/spike"]).is_empty());
    assert!(!reference_exists(&fixture, "refs/heads/spike"));
}

/// A remote is resolved from the configured upstream or not at all. Guessing
/// `origin/<name>` is fine for choosing what to pull; it is not fine for
/// choosing what to destroy on a server.
#[test]
fn cleanup_does_not_guess_a_remote_for_a_branch_with_no_upstream() {
    let (fixture, _origin) = fixture_with_published_branch();
    run(&fixture, &["branch", "--unset-upstream", "spike"]);

    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();

    assert_eq!(plan.remote_count, 0);
    assert_eq!(plan.kept_remotes[0].reason, KeptReason::NoUpstream);
    assert!(!plan.commands.iter().any(|command| command.contains("push")));
}

#[test]
fn cleanup_lists_remote_deletions_before_local_ones() {
    let (fixture, _origin) = fixture_with_published_branch();

    let plan = fixture.repo.plan_cleanup(request(&["refs/heads/spike"])).unwrap();

    assert!(plan.commands[0].starts_with("git push --atomic"));
    assert!(plan.commands[0].contains("--force-with-lease=refs/heads/spike:"));
    assert!(plan.commands[1].starts_with("git update-ref -d"));
    assert_eq!(plan.commands.len(), 2);
}

/// Leaves HEAD on `feature`, `spike` merged into `base`, and Base advanced past
/// both. `feature` is deliberately left merged so exclusion tests are not vacuous.
fn fixture_with_merged_branch() -> FixtureRepo {
    let fixture = FixtureRepo::new();
    fixture.checkout("base");
    fixture.branch("spike");
    fixture.commit_file("spike.txt", "work\n", "spike work");
    fixture.checkout("base");
    fixture.merge("spike", "merge spike");
    fixture.set_base_ref();
    fixture.checkout("feature");
    fixture
}

/// The same shape against a real bare origin, with `spike` tracking
/// `origin/spike`. The returned `TempDir` owns the remote and must stay alive.
fn fixture_with_published_branch() -> (FixtureRepo, TempDir) {
    let fixture = FixtureRepo::new();
    let remote = fixture.add_bare_origin();
    fixture.checkout("base");
    fixture.branch("spike");
    fixture.commit_file("spike.txt", "work\n", "spike work");
    run(&fixture, &["push", "-u", "origin", "spike"]);
    fixture.checkout("base");
    fixture.merge("spike", "merge spike");
    run(&fixture, &["push", "origin", "base"]);
    run(&fixture, &["fetch", "origin"]);
    fixture.checkout("feature");
    (fixture, remote)
}

/// A second clone pushing to the same origin. Pushing from `fixture` itself
/// would also advance its own tracking ref, which the local staleness check
/// would catch first — that is not the race the lease exists for.
fn push_from_elsewhere(fixture: &FixtureRepo, branch: &str) -> TempDir {
    let origin = read(fixture, &["remote", "get-url", "origin"]);
    let holder = tempfile::tempdir().unwrap();
    let clone = holder.path().join("clone");
    let clone = clone.to_str().unwrap().to_string();
    run(fixture, &["clone", &origin, &clone]);
    run(fixture, &["-C", &clone, "config", "user.email", "other@example.test"]);
    run(fixture, &["-C", &clone, "config", "user.name", "Other"]);
    run(fixture, &["-C", &clone, "switch", branch]);
    run(
        fixture,
        &["-C", &clone, "commit", "--allow-empty", "-m", "their work"],
    );
    run(fixture, &["-C", &clone, "push", "origin", branch]);
    holder
}

fn base() -> RefName {
    RefName::new("refs/remotes/origin/base".to_string()).unwrap()
}

fn request(chosen: &[&str]) -> CleanupRequest {
    CleanupRequest {
        base: base(),
        chosen: chosen.iter().map(|value| value.to_string()).collect(),
        include_remote_counterparts: true,
    }
}

fn choice<'a>(
    discovery: &'a CleanupDiscovery,
    branch: &str,
) -> Option<&'a git_helper_core::CleanupChoice> {
    discovery
        .choices
        .iter()
        .find(|choice| choice.branch == branch)
}

fn reason(discovery: &CleanupDiscovery, branch: &str) -> Option<ExclusionReason> {
    discovery
        .excluded
        .iter()
        .find(|entry| entry.branch == branch)
        .map(|entry| entry.reason)
}

fn excluded(discovery: &CleanupDiscovery, branch: &str) -> bool {
    reason(discovery, branch).is_some()
}

fn record(fixture: &FixtureRepo, operation: &str) -> RecoveryEntry {
    fixture
        .repo
        .list_operations()
        .unwrap()
        .into_iter()
        .find(|entry| entry.operation == operation)
        .unwrap_or_else(|| panic!("no {operation} record"))
}

fn commit_as(fixture: &FixtureRepo, email: &str, message: &str) {
    let identity = format!("user.email={email}");
    run(
        fixture,
        &[
            "-c",
            &identity,
            "-c",
            "user.name=Other",
            "commit",
            "--allow-empty",
            "-m",
            message,
        ],
    );
}

fn execute(fixture: &FixtureRepo, command: &str) {
    for step in command.split(" && ") {
        let values = step
            .strip_prefix("git ")
            .unwrap_or(step)
            .split_whitespace()
            .collect::<Vec<_>>();
        run(fixture, &values);
    }
}

fn reference_exists(fixture: &FixtureRepo, reference: &str) -> bool {
    fixture
        .repo
        .run(GitCommand::read(osargs(&[
            "show-ref",
            "--verify",
            "--quiet",
            reference,
        ])))
        .is_ok()
}

fn run(fixture: &FixtureRepo, values: &[&str]) {
    fixture.repo.run(GitCommand::write(osargs(values))).unwrap();
}

fn read(fixture: &FixtureRepo, values: &[&str]) -> String {
    let output = fixture.repo.run(GitCommand::read(osargs(values))).unwrap();
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

fn osargs(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
