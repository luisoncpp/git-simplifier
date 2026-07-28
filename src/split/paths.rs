use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RepoPath};

use super::errors::SplitError;
use super::state::args;

const META_SUFFIX: &str = ".meta";

pub(super) struct Range<'a> {
    pub from: &'a ObjectId,
    pub to: &'a ObjectId,
}

pub(super) struct Selection {
    pub selected: Vec<RepoPath>,
    /// Selection plus companions: the exact files the split will carry.
    pub carried: Vec<RepoPath>,
    pub companions: Vec<RepoPath>,
}

pub(super) fn select(
    runner: &GitRunner,
    requested: Vec<RepoPath>,
    range: Range<'_>,
) -> Result<Selection, SplitError> {
    let selected = normalize(requested)?;
    let changed = changed_between(runner, range.from, range.to)?;
    let matched = matched(&selected, &changed);
    if matched.is_empty() {
        return Err(SplitError::NoChanges);
    }
    let companions = companions(&matched, &changed);
    Ok(Selection {
        selected,
        carried: combine(matched, &companions),
        companions,
    })
}

fn combine(matched: Vec<RepoPath>, companions: &[RepoPath]) -> Vec<RepoPath> {
    let mut values = matched;
    values.extend(companions.iter().cloned());
    values.sort();
    values.dedup();
    values
}

fn normalize(paths: Vec<RepoPath>) -> Result<Vec<RepoPath>, SplitError> {
    let mut values: Vec<RepoPath> = paths
        .into_iter()
        .map(|path| RepoPath::new(path.as_str().trim_end_matches('/').to_string()))
        .collect::<Result<_, _>>()
        .map_err(SplitError::InvalidState)?;
    values.sort();
    values.dedup();
    if values.is_empty() {
        return Err(SplitError::EmptySelection);
    }
    Ok(values)
}

fn changed_between(
    runner: &GitRunner,
    from: &ObjectId,
    to: &ObjectId,
) -> Result<Vec<RepoPath>, SplitError> {
    let output = runner.run(GitCommand::read(args(&[
        "diff",
        "--name-only",
        "-z",
        "--no-relative",
        "--no-renames",
        "--no-ext-diff",
        from.as_str(),
        to.as_str(),
    ])))?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .map(|record| RepoPath::from_bytes(record).map_err(SplitError::InvalidState))
        .collect()
}

/// Changed files covered by the selection, either exactly or as a directory prefix.
fn matched(selected: &[RepoPath], changed: &[RepoPath]) -> Vec<RepoPath> {
    changed
        .iter()
        .filter(|path| selected.iter().any(|choice| covers(choice, path)))
        .cloned()
        .collect()
}

/// A Unity `.meta` file is meaningless without its asset and vice versa, so a
/// changed partner of a matched path is always carried along.
fn companions(matched: &[RepoPath], changed: &[RepoPath]) -> Vec<RepoPath> {
    changed
        .iter()
        .filter(|path| !matched.contains(path))
        .filter(|path| {
            partner(path)
                .map(|name| matched.iter().any(|other| other.as_str() == name))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

fn covers(selected: &RepoPath, path: &RepoPath) -> bool {
    let choice = selected.as_str();
    let value = path.as_str();
    value == choice || value.starts_with(&format!("{choice}/"))
}

fn partner(path: &RepoPath) -> Option<String> {
    let value = path.as_str();
    match value.strip_suffix(META_SUFFIX) {
        Some(asset) if !asset.is_empty() => Some(asset.to_string()),
        _ => Some(format!("{value}{META_SUFFIX}")),
    }
}
