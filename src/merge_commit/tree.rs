use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RepoPath};

use super::errors::CommitMergeError;
use super::model::{IndexEntry, MergeParents};
use super::paths::literal;
use super::state::{index_command, temp_index_path, text, TempIndex};

pub(crate) struct BuiltTree {
    pub tree: ObjectId,
    pub conflicted_paths: Vec<RepoPath>,
}

pub(crate) fn build(
    runner: &GitRunner,
    parents: &MergeParents,
) -> Result<BuiltTree, CommitMergeError> {
    let index = temp_index_path(runner)?;
    let _guard = TempIndex::new(index.clone());
    read_merge_tree(runner, &index, parents)?;
    let conflicted = conflicted_paths(runner, &index)?;
    overlay_resolutions(runner, &index, &conflicted)?;
    let tree = write_tree(runner, &index)?;
    Ok(BuiltTree {
        tree,
        conflicted_paths: conflicted,
    })
}

fn read_merge_tree(
    runner: &GitRunner,
    index: &Path,
    parents: &MergeParents,
) -> Result<(), CommitMergeError> {
    runner.run_unlocked(index_command(index, &["read-tree", "--empty"]))?;
    runner.run_unlocked(index_command(
        index,
        &[
            "read-tree",
            "-m",
            parents.base.as_str(),
            parents.ours.as_str(),
            parents.theirs.as_str(),
        ],
    ))?;
    Ok(())
}

fn overlay_resolutions(
    runner: &GitRunner,
    index: &Path,
    paths: &[RepoPath],
) -> Result<(), CommitMergeError> {
    for path in paths {
        match stage_zero(runner, path)? {
            Some(entry) => copy_entry(runner, index, &entry)?,
            None => remove_entry(runner, index, path)?,
        }
    }
    Ok(())
}

fn conflicted_paths(runner: &GitRunner, index: &Path) -> Result<Vec<RepoPath>, CommitMergeError> {
    let output = runner.run_unlocked(index_command(index, &["ls-files", "-u", "-z"]))?;
    let mut paths = BTreeSet::new();
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
    {
        paths.insert(parse_unmerged_path(record)?);
    }
    Ok(paths.into_iter().collect())
}

fn parse_unmerged_path(record: &[u8]) -> Result<RepoPath, CommitMergeError> {
    let tab = record
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or_else(|| CommitMergeError::InvalidState("ls-files record had no path".to_string()))?;
    let path = String::from_utf8(record[tab + 1..].to_vec())
        .map_err(|_| CommitMergeError::InvalidState("path is not UTF-8".to_string()))?;
    RepoPath::new(path).map_err(CommitMergeError::InvalidState)
}

fn stage_zero(runner: &GitRunner, path: &RepoPath) -> Result<Option<IndexEntry>, CommitMergeError> {
    let spec = literal(path.as_str());
    let output = runner.run_unlocked(GitCommand::read(vec![
        OsString::from("ls-files"),
        OsString::from("--stage"),
        OsString::from("-z"),
        OsString::from("--"),
        OsString::from(spec),
    ]))?;
    for record in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
    {
        let entry = parse_stage_record(record, path)?;
        if entry.stage == 0 {
            return Ok(Some(entry.into()));
        }
    }
    Ok(None)
}

struct StageEntry {
    mode: String,
    object: ObjectId,
    stage: u8,
    path: RepoPath,
}

fn parse_stage_record(record: &[u8], path: &RepoPath) -> Result<StageEntry, CommitMergeError> {
    let tab = record
        .iter()
        .position(|byte| *byte == b'\t')
        .ok_or_else(|| CommitMergeError::InvalidState("ls-files record had no path".to_string()))?;
    let head = text(&record[..tab])?;
    let parts: Vec<&str> = head.split(' ').collect();
    if parts.len() != 3 {
        return Err(CommitMergeError::InvalidState(
            "ls-files stage record was malformed".to_string(),
        ));
    }
    Ok(StageEntry {
        mode: parts[0].to_string(),
        object: ObjectId::new(parts[1].to_string()).map_err(CommitMergeError::InvalidState)?,
        stage: parts[2].parse().map_err(|_| {
            CommitMergeError::InvalidState("ls-files stage was not a number".to_string())
        })?,
        path: path.clone(),
    })
}

impl From<StageEntry> for IndexEntry {
    fn from(value: StageEntry) -> Self {
        Self {
            mode: value.mode,
            object: value.object,
            path: value.path,
        }
    }
}

fn copy_entry(
    runner: &GitRunner,
    index: &Path,
    entry: &IndexEntry,
) -> Result<(), CommitMergeError> {
    runner.run_unlocked(index_command(
        index,
        &[
            "update-index",
            "--cacheinfo",
            &entry.mode,
            entry.object.as_str(),
            entry.path.as_str(),
        ],
    ))?;
    Ok(())
}

fn remove_entry(runner: &GitRunner, index: &Path, path: &RepoPath) -> Result<(), CommitMergeError> {
    runner.run_unlocked(index_command(
        index,
        &["update-index", "--force-remove", path.as_str()],
    ))?;
    Ok(())
}

fn write_tree(runner: &GitRunner, index: &Path) -> Result<ObjectId, CommitMergeError> {
    let output = runner.run_unlocked(index_command(index, &["write-tree"]))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(CommitMergeError::InvalidState)
}

pub(crate) fn stage_zero_for_commands(
    runner: &GitRunner,
    path: &RepoPath,
) -> Result<Option<IndexEntry>, CommitMergeError> {
    stage_zero(runner, path)
}
