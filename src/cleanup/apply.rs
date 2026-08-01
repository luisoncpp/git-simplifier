use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::recording::Oplog;

use super::errors::CleanupError;
use super::model::{CleanupPlan, CleanupResult, LocalDeletion};
use super::remote::{self, RemotePush};
use super::{plan, record};

/// Remote deletions run **before** local ones. The local branch is the backup
/// for an irreversible server deletion — while it still exists, a lost remote
/// is one re-push away — and a rejected push then fails before anything at all
/// has been destroyed.
pub(crate) fn cleanup(
    runner: &GitRunner,
    cleanup_plan: &CleanupPlan,
) -> Result<CleanupResult, CleanupError> {
    plan::verify_current(runner, cleanup_plan)?;
    let oplog =
        Oplog::open(&runner.git_dir()?).map_err(|error| CleanupError::Recording(error.to_string()))?;
    let session = Session {
        oplog: &oplog,
        group: record::group_id(),
    };
    let deleted_remote = delete_remotes(runner, cleanup_plan, &session)?;
    let deleted_local = delete_locals(runner, cleanup_plan, &session)?;
    Ok(CleanupResult {
        deleted_local,
        deleted_remote,
        kept_remotes: cleanup_plan
            .kept_remotes
            .iter()
            .map(|kept| kept.branch.clone())
            .collect(),
    })
}

struct Session<'a> {
    oplog: &'a Oplog,
    group: String,
}

/// One record per remote, not one for the whole phase: with `--atomic` each
/// push is all-or-nothing, so a per-remote record is exactly true even when a
/// later remote rejects the deletion.
fn delete_remotes(
    runner: &GitRunner,
    cleanup_plan: &CleanupPlan,
    session: &Session,
) -> Result<Vec<String>, CleanupError> {
    let mut deleted = Vec::new();
    for push in remote::group(cleanup_plan) {
        let id = record::begin_remote(session.oplog, &push, &session.group)?;
        run_push(runner, &push)?;
        record::finish(session.oplog, &id)?;
        deleted.extend(push.deletions.iter().map(|entry| entry.tracking_ref.clone()));
    }
    Ok(deleted)
}

fn run_push(runner: &GitRunner, push: &RemotePush) -> Result<(), CleanupError> {
    runner
        .run_unlocked(GitCommand::write(remote::push_args(push)))
        .map_err(|error| CleanupError::RemoteRejected {
            remote: push.remote.clone(),
            stderr: stderr_of(&error),
        })?;
    Ok(())
}

fn stderr_of(error: &crate::git::GitError) -> String {
    error
        .raw_stderr()
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
        .unwrap_or_else(|| error.to_string())
}

fn delete_locals(
    runner: &GitRunner,
    cleanup_plan: &CleanupPlan,
    session: &Session,
) -> Result<Vec<String>, CleanupError> {
    let deletions = cleanup_plan
        .branches
        .iter()
        .filter_map(|entry| entry.local.as_ref())
        .collect::<Vec<_>>();
    if deletions.is_empty() {
        return Ok(Vec::new());
    }
    let id = record::begin_local(session.oplog, cleanup_plan, &session.group)?;
    let mut deleted = Vec::new();
    for deletion in deletions {
        delete_ref(runner, deletion)?;
        deleted.push(deletion.reference.clone());
    }
    record::finish(session.oplog, &id)?;
    Ok(deleted)
}

/// The expected old SHA makes Git refuse if the branch moved. `git branch -d`
/// is the wrong tool here: it measures merged-ness against HEAD, not Base.
fn delete_ref(runner: &GitRunner, deletion: &LocalDeletion) -> Result<(), CleanupError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("update-ref"),
        OsString::from("-d"),
        OsString::from("-m"),
        OsString::from("git-helper cleanup"),
        OsString::from(&deletion.reference),
        OsString::from(deletion.head.as_str()),
    ]))?;
    Ok(())
}
