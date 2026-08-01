use crate::git::GitRunner;
use crate::rewrite::RefName;

use super::eligibility::{self, Inputs};
use super::errors::CleanupError;
use super::model::CleanupDiscovery;
use super::refs;
use super::state;

/// The maximal offerable set: every branch merged into Base that survives every
/// safety rule, annotated with the facts the UI's three toggles need. The
/// toggles are display filters and never reach Git, so flipping one costs no
/// repository work and cannot widen what is safe to delete.
pub(super) fn eligible(
    runner: &GitRunner,
    base: &RefName,
) -> Result<CleanupDiscovery, CleanupError> {
    state::ensure_remote_base(base)?;
    // Base is pinned to a SHA once so discovery, planning, and verification
    // cannot disagree because a fetch landed between them.
    let base_head = state::read_id(runner, base.as_str())?;
    let identity = state::identity(runner)?;
    let classified = eligibility::classify(Inputs {
        base: base.clone(),
        identity: identity.clone(),
        remote_names: refs::remote_names(runner)?,
        locals: refs::merged_locals(runner, &base_head)?,
        remotes: refs::remotes(runner)?,
        merged_remotes: refs::merged_remote_names(runner, &base_head)?,
        saved_work: refs::saved_work_branches(runner)?,
    });
    Ok(CleanupDiscovery {
        base: base.clone(),
        base_head,
        identity,
        choices: classified.choices,
        excluded: classified.excluded,
    })
}
