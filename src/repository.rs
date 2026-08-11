use crate::exclusion::{
    self, ExcludeSubmodulePlan, ExcludeSubmoduleRequest, ExcludeSubmoduleResult, ExclusionError,
};
use crate::git::{GitCommand, GitError, GitOutput, GitRunner, RepositoryConfig};
use crate::inspection::RepositoryOverview;
use crate::push::{
    self, ForcePushError, ForcePushPlan, ForcePushResult, PublishBranchPlan, PublishBranchResult,
    PublishError,
};
use crate::recording::{self, RecoveryEntry, RecoveryError};
use crate::revert::{
    self, RevertError, RevertPlan, RevertRequest, RevertResult,
};
use crate::rewrite::{
    self, ApplyError, ApplyResult, EditMessageRequest, RewriteError, RewritePlan, UncommitRequest,
};
use crate::submodule_cleanup::{
    self, SubmoduleCleanupError, SubmoduleCleanupPlan, SubmoduleCleanupRequest,
    SubmoduleCleanupResult,
};
use crate::split::{self, SplitBranchPlan, SplitBranchRequest, SplitBranchResult, SplitError};
use crate::switch::{
    self, DeleteSavedWorkResult, PullResolution, QuickSwitchPlan, QuickSwitchRequest,
    QuickSwitchResult, QuickSwitchStatus, RestoreSavedWorkResult, SavedWork, SwitchError,
};
use crate::sync::{self, SyncError, SyncRequest, SyncResult, SyncStatus};

mod cleanup;
mod read;

pub struct GitRepository {
    runner: GitRunner,
}

pub struct RepositoryState {
    pub overview: RepositoryOverview,
    pub saved_work: Vec<SavedWork>,
    pub operations: Vec<RecoveryEntry>,
}

impl GitRepository {
    pub fn open(config: RepositoryConfig) -> Result<Self, GitError> {
        Ok(Self {
            runner: GitRunner::open(config)?,
        })
    }

    pub fn run(&self, command: GitCommand) -> Result<GitOutput, GitError> {
        self.runner.run(command)
    }

    pub fn list_operations(&self) -> Result<Vec<RecoveryEntry>, RecoveryError> {
        self.runner
            .with_write_lock(|| recording::list(&self.runner))
    }

    pub fn plan_force_push(&self) -> Result<ForcePushPlan, ForcePushError> {
        push::create(&self.runner)
    }

    pub fn apply_force_push(
        &self,
        plan: &ForcePushPlan,
    ) -> Result<ForcePushResult, ForcePushError> {
        self.runner
            .with_write_lock(|| push::apply(&self.runner, plan))
    }

    pub fn plan_publish_branch(&self, branch: String) -> Result<PublishBranchPlan, PublishError> {
        push::create_publish(&self.runner, branch)
    }

    pub fn apply_publish_branch(
        &self,
        plan: &PublishBranchPlan,
    ) -> Result<PublishBranchResult, PublishError> {
        self.runner
            .with_write_lock(|| push::apply_publish(&self.runner, plan))
    }

    pub fn plan_uncommit(&self, request: UncommitRequest) -> Result<RewritePlan, RewriteError> {
        rewrite::plan(&self.runner, request)
    }

    pub fn plan_revert(&self, request: RevertRequest) -> Result<RevertPlan, RevertError> {
        revert::create(&self.runner, request)
    }

    pub fn apply_revert(&self, plan: &RevertPlan) -> Result<RevertResult, RevertError> {
        self.runner
            .with_write_lock(|| revert::apply(&self.runner, plan))
    }

    pub fn plan_exclude_submodule(
        &self,
        request: ExcludeSubmoduleRequest,
    ) -> Result<ExcludeSubmodulePlan, ExclusionError> {
        exclusion::create(&self.runner, request)
    }

    pub fn apply_exclude_submodule(
        &self,
        plan: &ExcludeSubmodulePlan,
    ) -> Result<ExcludeSubmoduleResult, ExclusionError> {
        self.runner
            .with_write_lock(|| exclusion::apply(&self.runner, plan))
    }

    pub fn plan_submodule_cleanup(
        &self,
        request: SubmoduleCleanupRequest,
    ) -> Result<SubmoduleCleanupPlan, SubmoduleCleanupError> {
        submodule_cleanup::create(&self.runner, request)
    }

    pub fn apply_submodule_cleanup(
        &self,
        plan: &SubmoduleCleanupPlan,
    ) -> Result<SubmoduleCleanupResult, SubmoduleCleanupError> {
        self.runner
            .with_write_lock(|| submodule_cleanup::apply(&self.runner, plan))
    }

    pub fn plan_edit_message(
        &self,
        request: EditMessageRequest,
    ) -> Result<RewritePlan, RewriteError> {
        rewrite::plan_edit_message(&self.runner, request)
    }

    pub fn apply_rewrite(&self, plan: &RewritePlan) -> Result<ApplyResult, ApplyError> {
        self.runner
            .with_write_lock(|| rewrite::apply(&self.runner, plan))
    }

    pub fn plan_quick_switch(
        &self,
        request: QuickSwitchRequest,
    ) -> Result<QuickSwitchPlan, SwitchError> {
        switch::create_plan(&self.runner, request)
    }

    pub fn apply_quick_switch(
        &self,
        plan: &QuickSwitchPlan,
    ) -> Result<QuickSwitchResult, SwitchError> {
        self.runner
            .with_write_lock(|| switch::apply_plan(&self.runner, plan))
    }

    pub fn resolve_quick_switch_pull(
        &self,
        resolution: PullResolution,
    ) -> Result<QuickSwitchResult, SwitchError> {
        self.runner
            .with_write_lock(|| switch::resolve_pull(&self.runner, resolution))
    }

    pub fn quick_switch_status(&self) -> Result<Option<QuickSwitchStatus>, SwitchError> {
        switch::status(&self.runner)
    }

    pub fn plan_split_branch(
        &self,
        request: SplitBranchRequest,
    ) -> Result<SplitBranchPlan, SplitError> {
        split::create_plan(&self.runner, request)
    }

    pub fn apply_split_branch(
        &self,
        plan: &SplitBranchPlan,
    ) -> Result<SplitBranchResult, SplitError> {
        self.runner
            .with_write_lock(|| split::apply_plan(&self.runner, plan))
    }

    pub fn list_saved_work(&self) -> Result<Vec<SavedWork>, SwitchError> {
        switch::list(&self.runner)
    }

    pub fn restore_saved_work(&self) -> Result<RestoreSavedWorkResult, SwitchError> {
        self.runner
            .with_write_lock(|| switch::restore(&self.runner))
    }

    pub fn delete_saved_work(&self, branch: String) -> Result<DeleteSavedWorkResult, SwitchError> {
        self.runner
            .with_write_lock(|| switch::delete(&self.runner, &branch))
    }

    pub fn sync(&self, request: SyncRequest) -> Result<SyncResult, SyncError> {
        self.runner
            .with_write_lock(|| sync::sync(&self.runner, request))
    }

    pub fn resume_sync(&self) -> Result<SyncResult, SyncError> {
        self.runner.with_write_lock(|| sync::resume(&self.runner))
    }

    pub fn sync_status(&self) -> Result<Option<SyncStatus>, SyncError> {
        sync::status(&self.runner)
    }
}
