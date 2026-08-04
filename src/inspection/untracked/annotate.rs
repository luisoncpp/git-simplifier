use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::git::{GitCommand, GitRunner};

use super::super::errors::InspectionError;
use super::super::model::UntrackedAnnotations;

pub(super) fn head_commit_time(runner: &GitRunner) -> Result<u64, InspectionError> {
    let output = runner.run(GitCommand::read(vec![
        "log".into(),
        "-1".into(),
        "--format=%ct".into(),
        "HEAD".into(),
    ]))?;
    let text = String::from_utf8(output.stdout)
        .map_err(|_| InspectionError::Parse("HEAD commit time was not UTF-8".to_string()))?;
    text.trim()
        .parse()
        .map_err(|_| InspectionError::Parse("HEAD commit time was not a number".to_string()))
}

pub(super) fn for_path(
    runner: &GitRunner,
    path: &str,
    not_ignored: bool,
    head_secs: u64,
) -> Result<UntrackedAnnotations, InspectionError> {
    let worktree = runner.repo_path().join(path);
    Ok(UntrackedAnnotations {
        older_than_or_at_head: is_older_than_or_at_head(&worktree, head_secs)?,
        root_dot: first_segment_dot(path),
        in_node_modules: has_node_modules(path),
        gitignored: !not_ignored,
    })
}

fn first_segment_dot(path: &str) -> bool {
    path.split('/')
        .next()
        .is_some_and(|segment| segment.starts_with('.'))
}

fn has_node_modules(path: &str) -> bool {
    path.split('/').any(|segment| segment == "node_modules")
}

fn is_older_than_or_at_head(path: &Path, head_secs: u64) -> Result<bool, InspectionError> {
    let metadata = fs::metadata(path).map_err(|error| InspectionError::Parse(error.to_string()))?;
    let file_secs = birth_or_modified(&metadata);
    Ok(file_secs < head_secs)
}

fn birth_or_modified(metadata: &fs::Metadata) -> u64 {
    let time = metadata
        .created()
        .or_else(|_| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
