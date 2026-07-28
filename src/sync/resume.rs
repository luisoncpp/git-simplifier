use crate::git::GitRunner;
use crate::recording::Oplog;
use crate::rewrite::RefName;

use super::super::errors::SyncError;
use super::super::model::{SyncPhase, SyncResult, SyncStatus};
use super::super::{record, state};
use super::common::{open_log, Journal};
use super::{start, work};

pub(super) fn run(runner: &GitRunner) -> Result<SyncResult, SyncError> {
    let oplog = open_log(runner)?;
    let Some(context) = record::active(&oplog)? else {
        return Err(SyncError::InvalidState(
            "no interrupted sync exists".to_string(),
        ));
    };
    match context.phase {
        SyncPhase::Fetch => retry_fetch(runner, &oplog, context),
        SyncPhase::BaseMergeConflict => resume_after_merge(runner, &oplog, context),
        SyncPhase::WipReapplyConflict => finish_after_reapply(runner, &oplog, context),
        _ => Err(SyncError::InvalidState(
            "the interrupted sync has not reached a resumable phase".to_string(),
        )),
    }
}

pub(super) fn status(runner: &GitRunner) -> Result<Option<SyncStatus>, SyncError> {
    let oplog = open_log(runner)?;
    let Some(context) = record::active(&oplog)? else {
        return Ok(None);
    };
    let saved_work = work::read_snapshot(runner, context.snapshot_reference.as_deref())?;
    Ok(Some(SyncStatus {
        operation_id: context.id,
        branch: RefName::new(state::branch_ref(&context.branch))
            .map_err(SyncError::InvalidState)?,
        base: context.base,
        source_head: context.source_head,
        phase: context.phase,
        saved_work,
    }))
}

fn retry_fetch(
    runner: &GitRunner,
    oplog: &Oplog,
    context: record::Context,
) -> Result<SyncResult, SyncError> {
    state::ensure_no_operation(runner)?;
    ensure_original_checkout(runner, &context)?;
    let spec = state::base_spec(&context.base)?;
    start::fetch_base(runner, &context.base, &spec)?;
    let journal = Journal::new(oplog, &context.id, None);
    start::continue_after_fetch(runner, &journal, &context.base)
}

fn ensure_original_checkout(
    runner: &GitRunner,
    context: &record::Context,
) -> Result<(), SyncError> {
    if state::read_branch(runner)? != context.branch
        || state::read_id(runner, "HEAD")? != context.source_head
    {
        return Err(SyncError::InvalidState(
            "the branch or HEAD changed after fetch was interrupted".to_string(),
        ));
    }
    Ok(())
}

fn resume_after_merge(
    runner: &GitRunner,
    oplog: &Oplog,
    context: record::Context,
) -> Result<SyncResult, SyncError> {
    state::ensure_no_operation(runner)?;
    if state::read_id(runner, "HEAD")? == context.source_head {
        return Err(SyncError::InvalidState(
            "the base merge was aborted or has not been committed".to_string(),
        ));
    }
    let saved_work = work::read_snapshot(runner, context.snapshot_reference.as_deref())?;
    let journal = Journal::new(oplog, &context.id, saved_work.as_ref());
    work::reapply(runner, &journal)
}

fn finish_after_reapply(
    runner: &GitRunner,
    oplog: &Oplog,
    context: record::Context,
) -> Result<SyncResult, SyncError> {
    state::ensure_no_operation(runner)?;
    if state::has_unmerged_entries(runner)? {
        return Err(SyncError::InvalidState(
            "resolve the Saved work reapply conflict before resuming".to_string(),
        ));
    }
    let saved_work = work::read_snapshot(runner, context.snapshot_reference.as_deref())?;
    let journal = Journal::new(oplog, &context.id, saved_work.as_ref());
    work::finish(runner, &journal, false)
}
