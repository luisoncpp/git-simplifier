use std::collections::BTreeMap;

use crate::git::GitRunner;
use crate::rewrite::{ObjectId, RefName};

use super::super::errors::SyncError;
use super::super::model::{SyncResult, SyncSnapshot};
use super::super::{record, state};
use super::common::Journal;

struct Completion<'a> {
    branch: &'a str,
    base: &'a RefName,
    new_head: &'a ObjectId,
    saved_work: Option<&'a SyncSnapshot>,
    saved_work_warning: Option<String>,
}

impl<'a> Completion<'a> {
    fn refs_after(&self, runner: &GitRunner) -> Result<BTreeMap<String, String>, SyncError> {
        let mut refs = BTreeMap::from([
            ("HEAD".to_string(), self.new_head.to_string()),
            (state::branch_ref(self.branch), self.new_head.to_string()),
            (
                self.base.to_string(),
                state::read_id(runner, self.base.as_str())?.to_string(),
            ),
        ]);
        if let Some(saved_work) = self.saved_work {
            refs.insert(
                saved_work.reference.clone(),
                saved_work.snapshot.to_string(),
            );
        }
        Ok(refs)
    }

    fn result(&self, old_head: ObjectId, applied_index: bool) -> Result<SyncResult, SyncError> {
        Ok(SyncResult {
            branch: RefName::new(state::branch_ref(self.branch))
                .map_err(SyncError::InvalidState)?,
            base: self.base.clone(),
            old_head,
            new_head: self.new_head.clone(),
            saved_work: self.saved_work.cloned(),
            applied_index,
            saved_work_warning: self.saved_work_warning.clone(),
        })
    }
}

/// A snapshot is only ever written when tracked changes were set aside, so a
/// worktree with nothing tracked means none of that work came back. Resuming
/// after a conflicted reapply used to finish silently in exactly that case,
/// reporting Saved work as restored while the tree held none of it.
fn unrestored_warning(
    runner: &GitRunner,
    saved_work: Option<&SyncSnapshot>,
    applied_index: bool,
) -> Result<Option<String>, SyncError> {
    let Some(saved_work) = saved_work else {
        return Ok(None);
    };
    if applied_index || state::has_tracked_changes(runner)? {
        return Ok(None);
    }
    Ok(Some(format!(
        "Saved work did not reach the working tree. It is kept at {} — apply it there, \
         or delete it once you are sure the work is no longer needed.",
        saved_work.reference
    )))
}

pub(super) fn finish(
    runner: &GitRunner,
    journal: &Journal<'_>,
    applied_index: bool,
) -> Result<SyncResult, SyncError> {
    let context = record::active(journal.oplog)?.ok_or_else(|| {
        SyncError::InvalidState("active sync disappeared before completion".to_string())
    })?;
    let branch = state::read_branch(runner)?;
    let new_head = state::read_id(runner, "HEAD")?;
    let completion = Completion {
        branch: &branch,
        base: &context.base,
        new_head: &new_head,
        saved_work: journal.saved_work,
        saved_work_warning: unrestored_warning(runner, journal.saved_work, applied_index)?,
    };
    let after = completion.refs_after(runner)?;
    record::finish(journal.oplog, journal.operation_id, after)?;
    completion.result(context.source_head, applied_index)
}
