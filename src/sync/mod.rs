mod apply;
mod errors;
mod model;
mod preflight;
mod record;
mod state;

pub use errors::SyncError;
pub use model::{SyncPhase, SyncRequest, SyncResult, SyncSnapshot, SyncStatus};

use crate::git::GitRunner;

pub(crate) fn sync(runner: &GitRunner, request: SyncRequest) -> Result<SyncResult, SyncError> {
    apply::sync(runner, request)
}

pub(crate) fn resume(runner: &GitRunner) -> Result<SyncResult, SyncError> {
    apply::resume(runner)
}

pub(crate) fn status(runner: &GitRunner) -> Result<Option<SyncStatus>, SyncError> {
    apply::status(runner)
}
