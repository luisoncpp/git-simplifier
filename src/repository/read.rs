use crate::inspection::{
    self, ChangedPath, DiffCompare, EditableCommit, FileDiff, InspectionError, LocalBranchChoice,
    RemoteBaseChoice, RepositoryOverview, SubmoduleChoice,
};
use crate::rewrite::{ObjectId, RefName, RepoPath};
use crate::switch;

use super::{GitRepository, RepositoryState};

impl GitRepository {
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
        overview.quick_switch_status = self
            .quick_switch_status()
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

    pub fn list_revert_paths(
        &self,
        base: RefName,
    ) -> Result<Vec<ChangedPath>, crate::revert::RevertError> {
        crate::revert::list_paths(&self.runner, &base)
    }

    pub fn branch_diff(
        &self,
        base: RefName,
        compare: DiffCompare,
    ) -> Result<String, InspectionError> {
        inspection::branch_diff(&self.runner, &base, compare)
    }

    pub fn files_diff(
        &self,
        base: RefName,
        compare: DiffCompare,
    ) -> Result<Vec<FileDiff>, InspectionError> {
        inspection::files_diff(&self.runner, &base, compare)
    }

    pub fn full_file_diff(
        &self,
        base: RefName,
        path: RepoPath,
        compare: DiffCompare,
    ) -> Result<Option<FileDiff>, InspectionError> {
        inspection::full_file_diff(&self.runner, &base, &path, compare)
    }

    pub fn preview_saved_work_apply(
        &self,
        branch: String,
    ) -> Result<crate::switch::SavedWorkApplyPreview, crate::switch::SwitchError> {
        switch::preview_apply(&self.runner, &branch)
    }

    pub fn saved_work_apply_files_diff(
        &self,
        before: ObjectId,
        after: ObjectId,
    ) -> Result<Vec<FileDiff>, InspectionError> {
        inspection::tree_files_diff(&self.runner, &before, &after)
    }

    pub fn saved_work_apply_full_file_diff(
        &self,
        before: ObjectId,
        after: ObjectId,
        path: RepoPath,
    ) -> Result<Option<FileDiff>, InspectionError> {
        inspection::tree_full_file_diff(&self.runner, &before, &after, &path)
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

    pub fn fetch_remotes(&self) -> Result<(), InspectionError> {
        self.runner
            .with_write_lock(|| inspection::fetch_remotes(&self.runner))
    }
}
