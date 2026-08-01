use super::data::RepositorySnapshot;
use super::state::AppState;
use git_helper_core::GitRepository;

pub(super) fn snapshot(state: &AppState) -> Result<RepositorySnapshot, String> {
    with_repository(state, |repository| {
        let state = repository.load_state().map_err(|error| error.to_string())?;
        Ok(RepositorySnapshot {
            overview: state.overview,
            saved_work: state.saved_work,
            operations: state.operations,
        })
    })
}

pub(crate) fn with_repository<T>(
    state: &AppState,
    action: impl FnOnce(&GitRepository) -> Result<T, String>,
) -> Result<T, String> {
    let repository = state
        .repository
        .lock()
        .map_err(|_| "repository state lock was poisoned".to_string())?;
    let Some(repository) = repository.as_ref() else {
        let error = state
            .init_error
            .lock()
            .map_err(|_| "repository initialization lock was poisoned".to_string())?;
        return Err(error
            .clone()
            .unwrap_or_else(|| "repository is unavailable".to_string()));
    };
    action(repository)
}
