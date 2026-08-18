mod apply;
mod carry;
mod checkout;
mod errors;
mod history_apply;
mod history_model;
mod history_plan;
mod history_record;
mod model;
mod plan;
mod prep;
mod present;
mod preflight;
mod preview;
mod pull;
mod record;
mod record_commands;
mod resolve;
mod restore_apply;
mod saved;
mod source;
mod state;
mod stash;
mod untracked;

pub use errors::SwitchError;
pub use history_model::{HistorySwitchPlan, HistorySwitchRequest, HistorySwitchResult};
pub use model::{
    DeleteSavedWorkResult, PullResolution, QuickSwitchPhase, QuickSwitchPlan, QuickSwitchRequest,
    QuickSwitchResult, QuickSwitchStatus, RestoreSavedWorkResult, SavedWork,
    SavedWorkApplyPreview,
};

use crate::git::GitRunner;

pub(crate) fn create_plan(
    runner: &GitRunner,
    request: QuickSwitchRequest,
) -> Result<QuickSwitchPlan, SwitchError> {
    plan::create(runner, request)
}

pub(crate) fn apply_plan(
    runner: &GitRunner,
    plan: &QuickSwitchPlan,
) -> Result<QuickSwitchResult, SwitchError> {
    apply::switch(runner, plan)
}

pub(crate) fn create_history_plan(
    runner: &GitRunner,
    request: HistorySwitchRequest,
) -> Result<HistorySwitchPlan, SwitchError> {
    history_plan::create(runner, request)
}

pub(crate) fn apply_history_plan(
    runner: &GitRunner,
    plan: &HistorySwitchPlan,
) -> Result<HistorySwitchResult, SwitchError> {
    history_apply::apply(runner, plan)
}

pub(crate) fn present_branch(runner: &GitRunner) -> Result<Option<String>, SwitchError> {
    present::read(runner)
}

pub(crate) fn resolve_pull(
    runner: &GitRunner,
    resolution: PullResolution,
) -> Result<QuickSwitchResult, SwitchError> {
    resolve::resolve(runner, resolution)
}

pub(crate) fn status(runner: &GitRunner) -> Result<Option<QuickSwitchStatus>, SwitchError> {
    resolve::status(runner)
}

pub(crate) fn list(runner: &GitRunner) -> Result<Vec<SavedWork>, SwitchError> {
    plan::list_saved_work(runner)
}

pub(crate) fn restore(runner: &GitRunner) -> Result<RestoreSavedWorkResult, SwitchError> {
    apply::restore(runner)
}

pub(crate) fn delete(
    runner: &GitRunner,
    branch: &str,
) -> Result<DeleteSavedWorkResult, SwitchError> {
    apply::delete(runner, branch)
}

pub(crate) fn preview_apply(
    runner: &GitRunner,
    branch: &str,
) -> Result<SavedWorkApplyPreview, SwitchError> {
    preview::preview(runner, branch)
}
