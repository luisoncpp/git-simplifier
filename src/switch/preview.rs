use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::ObjectId;

use super::errors::SwitchError;
use super::model::SavedWorkApplyPreview;
use super::plan;
use super::state;

const MERGE_CONFLICT_EXIT: i32 = 1;

pub(crate) fn preview(runner: &GitRunner, branch: &str) -> Result<SavedWorkApplyPreview, SwitchError> {
    state::ensure_no_operation(runner)?;
    state::validate_branch_name(runner, branch)?;
    let saved = plan::read_saved_work(runner, branch)?
        .ok_or_else(|| SwitchError::MissingSavedWork(branch.to_string()))?;
    let on_current = state::read_branch(runner)? == branch;
    let stash = saved.snapshot.as_str();
    let base = tree_of(runner, &format!("{stash}^1"))?;
    let stash_tree = tree_of(runner, stash)?;
    let ours = if on_current {
        worktree_tree(runner)?
    } else {
        tree_of(runner, &state::branch_ref(branch))?
    };
    let (after, worktree_conflicts) = merge_apply(runner, &base, &ours, &stash_tree)?;
    let index_conflicts = index_conflicts(runner, stash, &base, on_current, branch)?;
    Ok(SavedWorkApplyPreview {
        branch: branch.to_string(),
        on_current_branch: on_current,
        before_tree: ObjectId::new(ours).map_err(SwitchError::InvalidState)?,
        after_tree: ObjectId::new(after).map_err(SwitchError::InvalidState)?,
        worktree_conflicts,
        index_conflicts,
    })
}

fn index_conflicts(
    runner: &GitRunner,
    stash: &str,
    base: &str,
    on_current: bool,
    branch: &str,
) -> Result<bool, SwitchError> {
    let index_parent = format!("{stash}^2");
    if state::optional_id(runner, &index_parent)?.is_none() {
        return Ok(false);
    }
    let theirs = tree_of(runner, &index_parent)?;
    let ours = if on_current {
        index_tree(runner)?
    } else {
        tree_of(runner, &state::branch_ref(branch))?
    };
    Ok(merge_apply(runner, base, &ours, &theirs)?.1)
}

fn merge_apply(
    runner: &GitRunner,
    base: &str,
    ours: &str,
    theirs: &str,
) -> Result<(String, bool), SwitchError> {
    let merge_base = format!("--merge-base={base}");
    let output = runner.run_unlocked_allowing_exit(
        GitCommand::read(state::args(&[
            "merge-tree",
            "--write-tree",
            &merge_base,
            "--messages",
            ours,
            theirs,
        ])),
        &[MERGE_CONFLICT_EXIT],
    )?;
    let conflicts = output.exit_code == Some(MERGE_CONFLICT_EXIT);
    let merged = first_line(&output.stdout)?;
    Ok((merged, conflicts))
}

fn worktree_tree(runner: &GitRunner) -> Result<String, SwitchError> {
    let index = preview_index_path(runner)?;
    let _guard = TempIndex::new(index.clone());
    let env = |cmd: &[&str]| {
        GitCommand::write(state::args(cmd))
            .with_environment(OsString::from("GIT_INDEX_FILE"), index.clone().into_os_string())
    };
    runner.run_unlocked(env(&["read-tree", "HEAD"]))?;
    runner.run_unlocked(env(&["add", "-u"]))?;
    let output = runner.run_unlocked(env(&["write-tree"]))?;
    first_line(&output.stdout)
}

fn index_tree(runner: &GitRunner) -> Result<String, SwitchError> {
    let output = runner.run_unlocked(GitCommand::write(state::args(&["write-tree"])))?;
    first_line(&output.stdout)
}

fn tree_of(runner: &GitRunner, spec: &str) -> Result<String, SwitchError> {
    let output = runner.run_unlocked(GitCommand::read(state::args(&[
        "rev-parse",
        &format!("{spec}^{{tree}}"),
    ])))?;
    first_line(&output.stdout)
}

fn preview_index_path(runner: &GitRunner) -> Result<PathBuf, SwitchError> {
    let dir = runner.git_dir()?.join("githelper");
    fs::create_dir_all(&dir).map_err(|error| SwitchError::InvalidState(error.to_string()))?;
    Ok(dir.join(format!("preview-index-{}", std::process::id())))
}

fn first_line(bytes: &[u8]) -> Result<String, SwitchError> {
    let text = state::text(bytes)?;
    text.lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .ok_or_else(|| SwitchError::InvalidState("Git output was empty".to_string()))
}

struct TempIndex(PathBuf);

impl TempIndex {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Drop for TempIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
