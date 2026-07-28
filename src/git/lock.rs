use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use super::error::GitError;

pub(crate) type RepoLock = Arc<Mutex<()>>;

static LOCKS: OnceLock<Mutex<HashMap<String, RepoLock>>> = OnceLock::new();

pub(crate) fn lock_for(path: &Path) -> RepoLock {
    let locks = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let key = path.to_string_lossy().to_string();
    let mut entries = locks.lock().expect("repository lock registry poisoned");
    entries
        .entry(key)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

pub(crate) fn with_lock<T, E>(
    lock: &RepoLock,
    action: impl FnOnce() -> Result<T, E>,
) -> Result<T, E>
where
    E: From<GitError>,
{
    let _guard = lock.lock().map_err(|_| E::from(GitError::LockPoisoned))?;
    action()
}
