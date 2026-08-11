use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{RefName, RepoPath};

use super::errors::InspectionError;
use super::model::DirtySubmodule;
use super::queries::{ensure_base_resolves, ensure_remote_base, parse_changed_paths, run};

/// Submodule gitlinks that are locally dirty and/or differ from Base in HEAD.
pub(crate) fn dirty_submodules(
    runner: &GitRunner,
    base: Option<&RefName>,
) -> Result<Vec<DirtySubmodule>, InspectionError> {
    let gitlinks = gitlink_paths(runner)?;
    if gitlinks.is_empty() {
        return Ok(Vec::new());
    }
    let mut entries = BTreeMap::<RepoPath, DirtySubmodule>::new();
    for path in local_dirty(runner, &gitlinks)? {
        entries.insert(
            path.clone(),
            DirtySubmodule {
                path,
                local_dirty: true,
                in_editable_range: false,
            },
        );
    }
    if let Some(base) = base {
        ensure_remote_base(base)?;
        ensure_base_resolves(runner, base)?;
        for path in editable_gitlinks(runner, base, &gitlinks)? {
            entries
                .entry(path.clone())
                .and_modify(|entry| entry.in_editable_range = true)
                .or_insert(DirtySubmodule {
                    path,
                    local_dirty: false,
                    in_editable_range: true,
                });
        }
    }
    Ok(entries.into_values().collect())
}

fn gitlink_paths(runner: &GitRunner) -> Result<BTreeSet<RepoPath>, InspectionError> {
    let output = run(runner, &["ls-tree", "-r", "-z", "--full-tree", "HEAD"])?;
    let mut paths = BTreeSet::new();
    for record in output.split(|byte| *byte == 0).filter(|record| !record.is_empty()) {
        if !record.starts_with(b"160000 commit ") {
            continue;
        }
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| InspectionError::Parse("submodule entry had no path".to_string()))?;
        let path = RepoPath::new(
            String::from_utf8(record[tab + 1..].to_vec())
                .map_err(|_| InspectionError::Parse("submodule path was not UTF-8".to_string()))?,
        )
        .map_err(InspectionError::Parse)?;
        paths.insert(path);
    }
    Ok(paths)
}

fn local_dirty(
    runner: &GitRunner,
    gitlinks: &BTreeSet<RepoPath>,
) -> Result<Vec<RepoPath>, InspectionError> {
    let output = runner.run(GitCommand::read(args(&[
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=no",
        "--ignore-submodules=none",
    ])))?;
    Ok(parse_dirty_gitlinks(&output.stdout, gitlinks))
}

fn parse_dirty_gitlinks(output: &[u8], gitlinks: &BTreeSet<RepoPath>) -> Vec<RepoPath> {
    let mut fields = output
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .peekable();
    let mut result = Vec::new();
    while let Some(record) = fields.next() {
        if record.starts_with(b"u ") || record.starts_with(b"? ") {
            continue;
        }
        let path = match record {
            r if r.starts_with(b"1 ") => path_from_v2(r, /*field_count=*/ 8),
            r if r.starts_with(b"2 ") => {
                fields.next();
                path_from_v2(r, /*field_count=*/ 9)
            }
            _ => None,
        };
        let Some(path) = path else { continue };
        let Ok(repo_path) = RepoPath::new(path) else { continue };
        if gitlinks.contains(&repo_path) {
            result.push(repo_path);
        }
    }
    result
}

fn path_from_v2(record: &[u8], field_count: usize) -> Option<String> {
    let xy = record.get(2..4)?;
    if xy == b".." {
        return None;
    }
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
        return None;
    }
    String::from_utf8(record[start..].to_vec()).ok()
}

fn editable_gitlinks(
    runner: &GitRunner,
    base: &RefName,
    gitlinks: &BTreeSet<RepoPath>,
) -> Result<Vec<RepoPath>, InspectionError> {
    let base = base.as_str();
    let output = run(
        runner,
        &[
            "diff",
            "--name-status",
            "-z",
            "--no-relative",
            "--ignore-submodules=none",
            &format!("{base}...HEAD"),
            "--",
        ],
    )?;
    let changed = parse_changed_paths(&output)?;
    Ok(changed
        .into_iter()
        .filter(|entry| gitlinks.contains(&entry.path))
        .map(|entry| entry.path)
        .collect())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
