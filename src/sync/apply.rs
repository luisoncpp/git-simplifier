#[path = "common.rs"]
mod common;
#[path = "complete.rs"]
mod complete;
#[path = "resume.rs"]
mod resume;
#[path = "start.rs"]
mod start;
#[path = "work.rs"]
mod work;

use crate::git::GitRunner;

use super::errors::SyncError;
use super::model::{SyncRequest, SyncResult, SyncStatus};

pub(crate) fn sync(runner: &GitRunner, request: SyncRequest) -> Result<SyncResult, SyncError> {
    start::run(runner, request)
}

pub(crate) fn resume(runner: &GitRunner) -> Result<SyncResult, SyncError> {
    resume::run(runner)
}

pub(crate) fn status(runner: &GitRunner) -> Result<Option<SyncStatus>, SyncError> {
    resume::status(runner)
}
