use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::RefName;

use super::super::errors::SyncError;
use super::super::model::{SyncPhase, SyncRequest, SyncResult};
use super::super::{preflight, record, state};
use super::common::{open_log, reject_active, Journal};
use super::work;

pub(super) fn run(runner: &GitRunner, request: SyncRequest) -> Result<SyncResult, SyncError> {
    let oplog = open_log(runner)?;
    reject_active(&oplog)?;
    state::ensure_no_operation(runner)?;
    let spec = state::base_spec(&request.base)?;
    let branch = state::read_branch(runner)?;
    let old_head = state::read_id(runner, "HEAD")?;
    let base_before = state::optional_id(runner, request.base.as_str())?;
    let input = record::BeginInput {
        branch: branch.clone(),
        base: request.base.clone(),
        source_head: old_head,
        base_before,
    };
    let operation_id = record::begin(&oplog, input)?;
    fetch_base(runner, &request.base, &spec)?;
    let journal = Journal::new(&oplog, &operation_id, None);
    continue_after_fetch(runner, &journal, &request.base)
}

pub(super) fn continue_after_fetch(
    runner: &GitRunner,
    journal: &Journal<'_>,
    base: &RefName,
) -> Result<SyncResult, SyncError> {
    ensure_untracked_safe(runner, journal, base)?;
    let saved_work = work::save(runner, journal)?;
    let journal = Journal::new(journal.oplog, journal.operation_id, saved_work.as_ref());
    journal.update(SyncPhase::BaseMerge)?;
    reset_tracked_work(runner)?;
    work::merge_base(runner, &journal, base)?;
    work::reapply(runner, &journal)
}

fn ensure_untracked_safe(
    runner: &GitRunner,
    journal: &Journal<'_>,
    base: &RefName,
) -> Result<(), SyncError> {
    let conflicts = preflight::untracked_conflicts(runner, base)?;
    if conflicts.is_empty() {
        return Ok(());
    }
    journal.update(SyncPhase::Fetch)?;
    let base_head = state::read_id(runner, base.as_str())?;
    record::finish(
        journal.oplog,
        journal.operation_id,
        BTreeMap::from([(base.to_string(), base_head.to_string())]),
    )?;
    Err(SyncError::UntrackedConflict(conflicts.join(", ")))
}

pub(super) fn fetch_base(
    runner: &GitRunner,
    base: &RefName,
    spec: &state::BaseSpec,
) -> Result<(), SyncError> {
    let refspec = format!("+{}:{}", spec.branch, base.as_str());
    runner.run_unlocked(GitCommand::write(args(&[
        "fetch",
        "--no-tags",
        "--no-recurse-submodules",
        &spec.remote,
        &refspec,
    ])))?;
    Ok(())
}

fn reset_tracked_work(runner: &GitRunner) -> Result<(), SyncError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "reset",
        "--hard",
        "--no-recurse-submodules",
        "HEAD",
    ])))?;
    Ok(())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
