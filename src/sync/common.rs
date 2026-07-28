use crate::git::GitRunner;
use crate::recording::Oplog;

use super::super::errors::SyncError;
use super::super::model::{SyncPhase, SyncSnapshot};
use super::super::record;

pub(super) struct Journal<'a> {
    pub oplog: &'a Oplog,
    pub operation_id: &'a str,
    pub saved_work: Option<&'a SyncSnapshot>,
}

impl<'a> Journal<'a> {
    pub(super) fn new(
        oplog: &'a Oplog,
        operation_id: &'a str,
        saved_work: Option<&'a SyncSnapshot>,
    ) -> Self {
        Self {
            oplog,
            operation_id,
            saved_work,
        }
    }

    pub(super) fn update(&self, phase: SyncPhase) -> Result<(), SyncError> {
        record::update_phase(
            self.oplog,
            self.operation_id,
            record::PhaseUpdate {
                phase,
                snapshot_reference: self.saved_work.map(|value| value.reference.clone()),
            },
        )
    }
}

pub(super) fn open_log(runner: &GitRunner) -> Result<Oplog, SyncError> {
    Oplog::open(&runner.git_dir()?).map_err(|error| SyncError::Recording(error.to_string()))
}

pub(super) fn reject_active(oplog: &Oplog) -> Result<(), SyncError> {
    if record::active(oplog)?.is_some() {
        return Err(SyncError::InvalidState(
            "a sync is already in progress; resume or inspect it first".to_string(),
        ));
    }
    Ok(())
}
