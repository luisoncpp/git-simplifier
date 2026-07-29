use std::path::PathBuf;
use std::sync::Mutex;

use git_helper_core::{GitRepository, RepositoryConfig};

use super::data::PendingOperation;

pub struct AppState {
    pub(super) repository: Mutex<Option<GitRepository>>,
    pub(super) init_error: Mutex<Option<String>>,
    pub(super) path: Mutex<PathBuf>,
    pub(super) pending: Mutex<Option<PendingOperation>>,
}

impl AppState {
    pub fn new() -> Self {
        let path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let state = Self {
            repository: Mutex::new(None),
            init_error: Mutex::new(None),
            path: Mutex::new(path.clone()),
            pending: Mutex::new(None),
        };
        if let Err(error) = state.open_path(path) {
            state.set_error(error);
        }
        state
    }

    /// A failed open leaves the previous repository in place so a bad picker
    /// choice cannot wipe a working session; the caller surfaces the error.
    pub fn open_path(&self, path: PathBuf) -> Result<(), String> {
        let config = RepositoryConfig {
            path: path.clone(),
            git_executable: PathBuf::from("git"),
        };
        let repository = GitRepository::open(config).map_err(|error| error.to_string())?;
        if let Ok(mut current_path) = self.path.lock() {
            *current_path = path;
        }
        self.set_repository(repository);
        Ok(())
    }

    fn set_repository(&self, repository: GitRepository) {
        if let Ok(mut current) = self.repository.lock() {
            *current = Some(repository);
        }
        if let Ok(mut error) = self.init_error.lock() {
            *error = None;
        }
        if let Ok(mut pending) = self.pending.lock() {
            *pending = None;
        }
    }

    fn set_error(&self, message: String) {
        if let Ok(mut error) = self.init_error.lock() {
            *error = Some(message);
        }
    }

    pub fn set_pending(&self, operation: PendingOperation) -> Result<(), String> {
        self.pending
            .lock()
            .map_err(|_| "pending operation lock was poisoned".to_string())
            .map(|mut value| *value = Some(operation))
    }

    pub fn take_pending(&self, id: &str) -> Result<PendingOperation, String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "pending operation lock was poisoned".to_string())?;
        let Some(operation) = pending.take() else {
            return Err("no operation review is pending".to_string());
        };
        if operation.id() != id {
            *pending = Some(operation);
            return Err("operation review is stale".to_string());
        }
        Ok(operation)
    }

    pub fn cancel_pending(&self, id: &str) -> Result<(), String> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|_| "pending operation lock was poisoned".to_string())?;
        if pending.as_ref().map(|operation| operation.id()) == Some(id) {
            *pending = None;
            return Ok(());
        }
        Err("operation review is stale".to_string())
    }
}
