use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};

use super::super::errors::InspectionError;
use super::super::query::UntrackedFilters;

pub(super) struct ListedPath {
    pub path: String,
    pub not_ignored: bool,
}

/// Discover untracked paths with filter constraints applied to the Git search
/// itself. Ignored trees are not walked when `respect_gitignore` is on.
pub(super) fn collect(
    runner: &GitRunner,
    filters: UntrackedFilters,
) -> Result<Vec<ListedPath>, InspectionError> {
    let excludes = pathspec_excludes(filters);
    let mut paths = as_listed(
        with_flags(runner, &["--exclude-standard"], &excludes)?,
        /*not_ignored=*/ true,
    );
    if filters.respect_gitignore {
        return Ok(paths);
    }
    for path in with_flags(runner, &["-i", "--exclude-standard"], &excludes)? {
        if paths.iter().any(|entry| entry.path == path) {
            continue;
        }
        paths.push(ListedPath {
            path,
            not_ignored: false,
        });
    }
    Ok(paths)
}

/// Whether `path` is currently an untracked worktree path, and whether Git's
/// exclude rules hide it. One pathspec — never a full-tree scan.
pub(super) fn classify_one(
    runner: &GitRunner,
    path: &str,
) -> Result<Option<bool>, InspectionError> {
    let pathspec = [OsString::from(format!(":(top,literal){path}"))];
    if !with_flags(runner, &["--exclude-standard"], &pathspec)?.is_empty() {
        return Ok(Some(true));
    }
    if !with_flags(runner, &["-i", "--exclude-standard"], &pathspec)?.is_empty() {
        return Ok(Some(false));
    }
    Ok(None)
}

fn as_listed(paths: Vec<String>, not_ignored: bool) -> Vec<ListedPath> {
    paths
        .into_iter()
        .map(|path| ListedPath { path, not_ignored })
        .collect()
}

fn pathspec_excludes(filters: UntrackedFilters) -> Vec<OsString> {
    let mut specs = vec![OsString::from(".")];
    if filters.exclude_node_modules {
        specs.push(OsString::from(":(exclude,glob)node_modules/**"));
        specs.push(OsString::from(":(exclude,glob)**/node_modules/**"));
    }
    if filters.exclude_root_dot {
        specs.push(OsString::from(":(exclude,glob).*"));
        specs.push(OsString::from(":(exclude,glob).*/**"));
    }
    specs
}

fn with_flags(
    runner: &GitRunner,
    flags: &[&str],
    pathspecs: &[OsString],
) -> Result<Vec<String>, InspectionError> {
    let mut args = vec![
        OsString::from("ls-files"),
        OsString::from("-z"),
        OsString::from("--others"),
    ];
    for flag in flags {
        args.push(OsString::from(*flag));
    }
    args.push(OsString::from("--"));
    args.extend(pathspecs.iter().cloned());
    let output = runner.run(GitCommand::read(args))?;
    Ok(parse_null_paths(&output.stdout))
}

fn parse_null_paths(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}
