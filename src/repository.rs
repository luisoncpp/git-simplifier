use crate::exclusion::{
    self, ExcludeSubmodulePlan, ExcludeSubmoduleRequest, ExcludeSubmoduleResult, ExclusionError,
};
use crate::git::{GitCommand, GitError, GitOutput, GitRunner, RepositoryConfig};
use crate::inspection::{
    self, ChangedPath, EditableCommit, InspectionError, LocalBranchChoice, RemoteBaseChoice,
    RepositoryOverview, SubmoduleChoice,
};
use crate::push::{
    self, ForcePushError, ForcePushPlan, ForcePushResult, PublishBranchPlan, PublishBranchResult,
    PublishError,
};
use crate::recording::{self, RecoveryEntry, RecoveryError};
use crate::rewrite::{
    self, ApplyError, ApplyResult, EditMessageRequest, RefName, RewriteError, RewritePlan,
    UncommitRequest,
};
use crate::split::{self, SplitBranchPlan, SplitBranchRequest, SplitBranchResult, SplitError};
use crate::switch::{
    self, DeleteSavedWorkResult, QuickSwitchPlan, QuickSwitchRequest, QuickSwitchResult,
    RestoreSavedWorkResult, SavedWork, SwitchError,
};
use crate::sync::{self, SyncError, SyncRequest, SyncResult, SyncStatus};

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

    pub fn overview(&self) -> Result<RepositoryOverview, InspectionError> {
        Ok(self.load_state()?.overview)
    }

    pub fn load_state(&self) -> Result<RepositoryState, InspectionError> {
        let mut overview = inspection::overview(&self.runner)?;
        let saved_work = self
            .list_saved_work()
            .map_err(|error| InspectionError::Parse(error.to_string()))?;
        let operations = self
            .list_operations()
            .map_err(|error| InspectionError::Parse(error.to_string()))?;
        overview.sync_status = self
            .sync_status()
            .map_err(|error| InspectionError::Parse(error.to_string()))?
            .map(|status| status.phase.as_str().to_string());
        overview.saved_work_count = saved_work.len();
        overview.recovery_count = operations.len();
        Ok(RepositoryState {
            overview,
            saved_work,
            operations,
        })
    }

    pub fn list_base_choices(&self) -> Result<Vec<RemoteBaseChoice>, InspectionError> {
        inspection::base_choices(&self.runner)
    }

    pub fn list_changed_paths(&self, base: RefName) -> Result<Vec<ChangedPath>, InspectionError> {
        inspection::changed_paths(&self.runner, &base)
    }

    pub fn list_editable_commits(
        &self,
        base: RefName,
    ) -> Result<Vec<EditableCommit>, InspectionError> {
        inspection::editable_commits(&self.runner, &base)
    }

    pub fn list_local_branches(&self) -> Result<Vec<LocalBranchChoice>, InspectionError> {
        inspection::local_branches(&self.runner)
    }

    pub fn list_submodules(&self) -> Result<Vec<SubmoduleChoice>, InspectionError> {
        inspection::submodules(&self.runner)
    }

    pub fn set_base(&self, base: RefName) -> Result<(), InspectionError> {
        inspection::set_base(&self.runner, base)
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
