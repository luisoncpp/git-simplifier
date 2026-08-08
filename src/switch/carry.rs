//! Carrying tracked changes onto the target branch, and rescuing them when the
//! stash stack refuses to give them back.

use crate::git::GitCommand;

use super::errors::SwitchError;
use super::model::{QuickSwitchPlan, SavedWork};
use super::{state, stash};

#[derive(Default)]
pub(super) struct CarryOutcome {
    pub carried_index: Option<bool>,
    /// Set only when a failed pop was rescued onto the source branch's Saved
    /// work ref instead of being abandoned on the shared stash stack.
    pub saved_work: Option<SavedWork>,
    pub warning: Option<String>,
}

pub(super) fn pop(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
    carry_pushed: bool,
) -> Result<CarryOutcome, SwitchError> {
    if !carry_pushed {
        return Ok(CarryOutcome::default());
    }
    let outcome = stash::pop_carry(runner)?;
    let Some(warning) = outcome.warning else {
        return Ok(CarryOutcome {
            carried_index: Some(outcome.applied_index),
            saved_work: None,
            warning: None,
        });
    };
    let saved_work = anchor_left_behind(runner, switch_plan)?;
    Ok(CarryOutcome {
        carried_index: Some(outcome.applied_index),
        warning: Some(left_behind_message(warning, saved_work.as_ref())),
        saved_work,
    })
}

fn left_behind_message(warning: String, saved_work: Option<&SavedWork>) -> String {
    let Some(saved_work) = saved_work else {
        return warning;
    };
    format!(
        "Carried changes could not be applied cleanly. They are kept as Saved work for {} \
         at {}, so nothing was lost.",
        saved_work.branch, saved_work.reference
    )
}

/// Rescues a carry the stack could not pop onto the source branch's Saved work
/// ref, then drops the entry. Without this the only copy lives in `refs/stash`,
/// which the app never lists — the panel reports "No Saved work" while the
/// changes sit there unreachable.
///
/// Returns `None` when there is nothing to rescue or the branch already owns
/// Saved work; overwriting that would trade one lost snapshot for another.
fn anchor_left_behind(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<Option<SavedWork>, SwitchError> {
    let Some(snapshot) = state::optional_id(runner, "refs/stash")? else {
        return Ok(None);
    };
    let reference = &switch_plan.saved_work_reference;
    if state::optional_id(runner, reference)?.is_some() {
        return Ok(None);
    }
    runner.run_unlocked(GitCommand::write(state::args(&[
        "update-ref",
        "-m",
        "git-helper rescue-carry",
        reference,
        snapshot.as_str(),
        "",
    ])))?;
    stash::drop_top(runner)?;
    Ok(Some(SavedWork {
        branch: switch_plan.source_branch.clone(),
        reference: reference.clone(),
        snapshot,
    }))
}
