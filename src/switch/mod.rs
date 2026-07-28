mod apply;
mod errors;
mod model;
mod plan;
mod preflight;
mod record;
mod state;

pub use errors::SwitchError;
pub use model::{
    DeleteSavedWorkResult, QuickSwitchPlan, QuickSwitchRequest, QuickSwitchResult,
    RestoreSavedWorkResult, SavedWork,
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
