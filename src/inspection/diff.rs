use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName, RepoPath};

use super::errors::InspectionError;
use super::model::{DiffCompare, FileDiff};
use super::query::FilesDiffQuery;

const LOAD_CONTEXT: &str = "3";
/// Git has no infinite-context flag, and `INT_MAX` is not safe: xdiff computes a
/// hunk's end as `start + change + context` in `int` *before* clamping it to the
/// record count, so a near-`INT_MAX` context overflows negative and the clamp
/// never fires. Ten million lines exceeds any file worth diffing.
const FULL_CONTEXT: &str = "10000000";

pub(crate) fn branch_diff(
    runner: &GitRunner,
    base: &RefName,
    compare: DiffCompare,
) -> Result<String, InspectionError> {
    super::queries::ensure_remote_base(base)?;
    let range = diff_tip(runner, base, compare)?;
    patch_text(runner, diff_args(&range, LOAD_CONTEXT, /*pathspec=*/ None))
}

/// Parsed from the very string `branch_diff` returns, so the copyable patch and
/// the structured diff are the same Git output by construction and cannot drift.
pub(crate) fn files_diff(
    runner: &GitRunner,
    base: &RefName,
    query: FilesDiffQuery,
) -> Result<Vec<FileDiff>, InspectionError> {
    let mut files = super::patch::parse_patch(&branch_diff(runner, base, query.compare)?)?;
    if query.compare == DiffCompare::Local {
        super::untracked::append_untracked(runner, &mut files, query.untracked)?;
    }
    Ok(files)
}

/// One file at full context, so a viewer can reveal any window of it without
/// another round trip. `None` means the path no longer differs from Base — HEAD
/// may have moved since the diff was loaded — which is a refresh, not an error.
pub(crate) fn full_file_diff(
    runner: &GitRunner,
    base: &RefName,
    path: &RepoPath,
    compare: DiffCompare,
) -> Result<Option<FileDiff>, InspectionError> {
    super::queries::ensure_remote_base(base)?;
    let range = diff_tip(runner, base, compare)?;
    // Pinned so the pathspec and the names Git prints agree below the Git root.
    let pathspec = format!(":(top,literal){}", path.as_str());
    let text = patch_text(runner, diff_args(&range, FULL_CONTEXT, Some(&pathspec)))?;
    if let Some(mut file) = super::patch::parse_patch(&text)?.pop() {
        file.complete = true;
        return Ok(Some(file));
    }
    if compare == DiffCompare::Local {
        return super::untracked::synthesized_if_untracked(runner, path);
    }
    Ok(None)
}

pub(crate) fn tree_files_diff(
    runner: &GitRunner,
    before: &ObjectId,
    after: &ObjectId,
) -> Result<Vec<FileDiff>, InspectionError> {
    super::patch::parse_patch(&tree_patch_text(
        runner,
        before,
        after,
        LOAD_CONTEXT,
        /*pathspec=*/ None,
    )?)
}

pub(crate) fn tree_full_file_diff(
    runner: &GitRunner,
    before: &ObjectId,
    after: &ObjectId,
    path: &RepoPath,
) -> Result<Option<FileDiff>, InspectionError> {
    let pathspec = format!(":(top,literal){}", path.as_str());
    let text = tree_patch_text(
        runner,
        before,
        after,
        FULL_CONTEXT,
        Some(&pathspec),
    )?;
    let Some(mut file) = super::patch::parse_patch(&text)?.pop() else {
        return Ok(None);
    };
    file.complete = true;
    Ok(Some(file))
}

fn tree_patch_text(
    runner: &GitRunner,
    before: &ObjectId,
    after: &ObjectId,
    context: &str,
    pathspec: Option<&str>,
) -> Result<String, InspectionError> {
    patch_text(
        runner,
        tree_diff_args(before.as_str(), after.as_str(), context, pathspec),
    )
}

fn tree_diff_args(
    before: &str,
    after: &str,
    context: &str,
    pathspec: Option<&str>,
) -> Vec<OsString> {
    let unified = format!("--unified={context}");
    let mut args = vec![
        "-c",
        "diff.noprefix=false",
        "diff",
        "--binary",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-relative",
        "--no-renames",
        "--ignore-submodules=none",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        unified.as_str(),
        before,
        after,
        "--",
    ];
    if let Some(pathspec) = pathspec {
        args.push(pathspec);
    }
    args.into_iter().map(Into::into).collect()
}

fn diff_tip(
    runner: &GitRunner,
    base: &RefName,
    compare: DiffCompare,
) -> Result<String, InspectionError> {
    match compare {
        DiffCompare::Head => Ok(format!("{}...HEAD", base.as_str())),
        DiffCompare::Local => merge_base_oid(runner, base).map(|oid| oid.as_str().to_string()),
    }
}

fn merge_base_oid(runner: &GitRunner, base: &RefName) -> Result<ObjectId, InspectionError> {
    let output = runner.run(GitCommand::read(vec![
        "merge-base".into(),
        base.as_str().into(),
        "HEAD".into(),
    ]))?;
    ObjectId::new(
        String::from_utf8(output.stdout)
            .map_err(|_| InspectionError::Parse("merge-base was not UTF-8".to_string()))?
            .trim()
            .to_string(),
    )
    .map_err(InspectionError::Parse)
}

/// The single source of truth for the stable patch flags. Both Inspection
/// surfaces are built from this argv, so neither can acquire options the other
/// lacks. Every flag neutralizes a repository or user setting: color, external
/// diff and textconv drivers, relative paths, rename collapsing, submodule
/// hiding, and configurable prefixes.
fn diff_args(range: &str, context: &str, pathspec: Option<&str>) -> Vec<OsString> {
    let unified = format!("--unified={context}");
    let mut args = vec![
        "-c",
        "diff.noprefix=false",
        "diff",
        "--binary",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-relative",
        "--no-renames",
        "--ignore-submodules=none",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        unified.as_str(),
        range,
        "--",
    ];
    if let Some(pathspec) = pathspec {
        args.push(pathspec);
    }
    args.into_iter().map(Into::into).collect()
}

fn patch_text(runner: &GitRunner, args: Vec<OsString>) -> Result<String, InspectionError> {
    let output = runner.run(GitCommand::read(args))?;
    String::from_utf8(output.stdout)
        .map_err(|_| InspectionError::Parse("Branch diff was not UTF-8".to_string()))
}
