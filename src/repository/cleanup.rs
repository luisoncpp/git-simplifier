use crate::cleanup::{
    self, CleanupDiscovery, CleanupError, CleanupPlan, CleanupRequest, CleanupResult,
};
use crate::rewrite::RefName;

use super::GitRepository;

impl GitRepository {
    /// Read-only, so it takes no write lock: the toggles the UI offers filter
    /// this one result rather than each triggering another scan.
    pub fn discover_cleanup(&self, base: &RefName) -> Result<CleanupDiscovery, CleanupError> {
        cleanup::discover_branches(&self.runner, base)
    }

    pub fn plan_cleanup(&self, request: CleanupRequest) -> Result<CleanupPlan, CleanupError> {
        cleanup::create_plan(&self.runner, request)
    }

    /// A multi-command transaction: the lock is taken here, so every write
    /// inside the module must use `run_unlocked` or it would deadlock.
    pub fn apply_cleanup(&self, plan: &CleanupPlan) -> Result<CleanupResult, CleanupError> {
        self.runner
            .with_write_lock(|| cleanup::apply_plan(&self.runner, plan))
    }
}
