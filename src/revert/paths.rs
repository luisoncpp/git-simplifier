use std::collections::BTreeMap;
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::inspection::{self, ChangedPath, InspectionError};
use crate::rewrite::{RefName, RepoPath};

use super::errors::RevertError;

/// Union of tracked local dirt and `Base...HEAD` committed diffs.
pub(super) fn revertible_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<ChangedPath>, RevertError> {
    let mut by_path = BTreeMap::new();
    for entry in inspection::changed_paths(runner, base).map_err(map_inspection)? {
        by_path.insert(entry.path.clone(), entry);
    }
    for entry in dirty_paths(runner)? {
        by_path.insert(entry.path.clone(), entry);
    }
    Ok(by_path.into_values().collect())
}

fn dirty_paths(runner: &GitRunner) -> Result<Vec<ChangedPath>, RevertError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=no",
        "--ignore-submodules=all",
    ])))?;
    parse_dirty(&output.stdout)
}

fn parse_dirty(output: &[u8]) -> Result<Vec<ChangedPath>, RevertError> {
    let mut fields = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .peekable();
    let mut result = Vec::new();
    while let Some(record) = fields.next() {
        if record.starts_with(b"u ") || record.starts_with(b"? ") {
            continue;
        }
        if record.starts_with(b"1 ") {
            result.push(ordinary_entry(record)?);
            continue;
        }
        if record.starts_with(b"2 ") {
            let previous = fields.next().ok_or_else(|| {
                RevertError::InvalidState("rename status had no previous path".to_string())
            })?;
            result.push(rename_entry(record, previous)?);
            continue;
        }
    }
    Ok(result)
}

fn ordinary_entry(record: &[u8]) -> Result<ChangedPath, RevertError> {
    let (status, path) = xy_and_path(record, /*field_count=*/ 8)?;
    Ok(ChangedPath {
        path: RepoPath::from_bytes(path).map_err(RevertError::InvalidState)?,
        previous_path: None,
        status,
    })
}

fn rename_entry(record: &[u8], previous: &[u8]) -> Result<ChangedPath, RevertError> {
    let (status, path) = xy_and_path(record, /*field_count=*/ 9)?;
    Ok(ChangedPath {
        path: RepoPath::from_bytes(path).map_err(RevertError::InvalidState)?,
        previous_path: Some(RepoPath::from_bytes(previous).map_err(RevertError::InvalidState)?),
        status,
    })
}

/// Porcelain v2 keeps a fixed field count before the path; the path may contain
/// spaces, so everything after that count is the path bytes.
fn xy_and_path(record: &[u8], field_count: usize) -> Result<(String, &[u8]), RevertError> {
    let xy = record.get(2..4).ok_or_else(|| {
        RevertError::InvalidState("status record was too short".to_string())
    })?;
    let mut seen = 0usize;
    let mut start = 0usize;
    for (index, byte) in record.iter().enumerate() {
        if *byte != b' ' {
            continue;
        }
        seen += 1;
        if seen == field_count {
            start = index + 1;
            break;
        }
    }
    if start == 0 || start >= record.len() {
        return Err(RevertError::InvalidState(
            "status record had no path".to_string(),
        ));
    }
    Ok((dirt_status(xy), &record[start..]))
}

fn dirt_status(xy: &[u8]) -> String {
    let letter = if xy[1] != b'.' { xy[1] } else { xy[0] };
    String::from_utf8_lossy(&[letter]).into_owned()
}

pub(crate) fn literal(path: &str) -> String {
    format!(":(top,literal){path}")
}

fn map_inspection(error: InspectionError) -> RevertError {
    match error {
        InspectionError::Git(error) => RevertError::Git(error),
        other => RevertError::InvalidState(other.to_string()),
    }
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
