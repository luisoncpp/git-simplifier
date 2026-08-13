use crate::git::GitRunner;

use super::errors::CommitMergeError;
use super::state;

pub(crate) fn check(runner: &GitRunner) -> Result<(), CommitMergeError> {
    state::read_branch(runner)?;
    state::refuse_other_operations(runner)?;
    if !state::merge_in_progress(runner)? {
        return Err(CommitMergeError::InvalidState(
            "no merge in progress".to_string(),
        ));
    }
    if state::has_unmerged_entries(runner)? {
        return Err(CommitMergeError::InvalidState(
            "Resolve merge conflicts first.".to_string(),
        ));
    }
    Ok(())
}
