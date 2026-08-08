use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::super::errors::SyncError;
use super::super::model::{SyncPhase, SyncResult, SyncSnapshot};
use super::super::state;
use super::common::Journal;

pub(super) fn merge_base(
    runner: &GitRunner,
    journal: &Journal<'_>,
    base: &RefName,
) -> Result<(), SyncError> {
    let result = runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "merge",
        "--no-edit",
        base.as_str(),
    ])));
    if let Err(source) = result {
        journal.update(SyncPhase::BaseMergeConflict)?;
        return Err(SyncError::BaseMergeConflict { source });
    }
    Ok(())
}

pub(super) fn reapply(runner: &GitRunner, journal: &Journal<'_>) -> Result<SyncResult, SyncError> {
    journal.update(SyncPhase::WipReapply)?;
    let applied_index = apply_snapshot(runner, journal)?;
    finish(runner, journal, applied_index)
}

pub(super) fn finish(
    runner: &GitRunner,
    journal: &Journal<'_>,
    applied_index: bool,
) -> Result<SyncResult, SyncError> {
    super::complete::finish(runner, journal, applied_index)
}

pub(super) fn save(
    runner: &GitRunner,
    journal: &Journal<'_>,
) -> Result<Option<SyncSnapshot>, SyncError> {
    if !state::has_tracked_changes(runner)? {
        return Ok(None);
    }
    let output = runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "stash",
        "create",
    ])))?;
    let snapshot =
        ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(SyncError::InvalidState)?;
    let reference = format!(
        "refs/githelper/backup/sync-{}-{}-wip",
        crate::recording::timestamp(),
        std::process::id()
    );
    update_ref(runner, &reference, &snapshot)?;
    let saved_work = SyncSnapshot {
        reference,
        snapshot,
    };
    let journal = Journal::new(journal.oplog, journal.operation_id, Some(&saved_work));
    journal.update(SyncPhase::Snapshot)?;
    Ok(Some(saved_work))
}

pub(super) fn read_snapshot(
    runner: &GitRunner,
    reference: Option<&str>,
) -> Result<Option<SyncSnapshot>, SyncError> {
    let Some(reference) = reference else {
        return Ok(None);
    };
    let snapshot = state::read_id(runner, reference)?;
    Ok(Some(SyncSnapshot {
        reference: reference.to_string(),
        snapshot,
    }))
}

fn apply_snapshot(runner: &GitRunner, journal: &Journal<'_>) -> Result<bool, SyncError> {
    let Some(saved_work) = journal.saved_work else {
        return Ok(false);
    };
    let indexed = runner
        .run_unlocked(GitCommand::write(args(&[
            "-c",
            "submodule.recurse=false",
            "stash",
            "apply",
            "--index",
            &saved_work.reference,
        ])))
        .is_ok();
    if indexed {
        return Ok(true);
    }
    if let Err(source) = runner.run_unlocked(GitCommand::write(args(&[
        "-c",
        "submodule.recurse=false",
        "stash",
        "apply",
        &saved_work.reference,
    ]))) {
        journal.update(SyncPhase::WipReapplyConflict)?;
        return Err(SyncError::WipReapplyConflict { source });
    }
    Ok(false)
}

fn update_ref(runner: &GitRunner, reference: &str, snapshot: &ObjectId) -> Result<(), SyncError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "update-ref",
        "-m",
        "git-helper sync save-work",
        reference,
        snapshot.as_str(),
        "",
    ])))?;
    Ok(())
}

fn text(bytes: &[u8]) -> Result<String, SyncError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SyncError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
