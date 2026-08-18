use crate::git::GitRunner;

use super::errors::SwitchError;
use super::model::SavedWork;
use super::{state, stash};

pub(super) struct TrackedPrep {
    pub saved_work: Option<SavedWork>,
    pub carry_pushed: bool,
}

pub(super) struct TrackedSpec<'a> {
    pub source_branch: &'a str,
    pub saved_work_reference: &'a str,
    pub has_tracked_changes: bool,
    pub carry_changes: bool,
}

pub(super) fn prepare_tracked(
    runner: &GitRunner,
    spec: TrackedSpec<'_>,
) -> Result<TrackedPrep, SwitchError> {
    if !spec.has_tracked_changes {
        return Ok(TrackedPrep {
            saved_work: None,
            carry_pushed: false,
        });
    }
    if spec.carry_changes {
        stash::push_tracked(runner)?;
        return Ok(TrackedPrep {
            saved_work: None,
            carry_pushed: true,
        });
    }
    park_as_saved_work(runner, spec)
}

fn park_as_saved_work(
    runner: &GitRunner,
    spec: TrackedSpec<'_>,
) -> Result<TrackedPrep, SwitchError> {
    let snapshot = stash::snapshot(runner)?;
    stash::reset_tracked(runner)?;
    state::update_ref(runner, spec.saved_work_reference, &snapshot, "")?;
    Ok(TrackedPrep {
        saved_work: Some(SavedWork {
            branch: spec.source_branch.to_string(),
            reference: spec.saved_work_reference.to_string(),
            snapshot,
        }),
        carry_pushed: false,
    })
}
