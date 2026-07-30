use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{RefName, RepoPath};

use super::errors::InspectionError;
use super::model::FileDiff;

const LOAD_CONTEXT: &str = "3";
/// Git has no infinite-context flag, and `INT_MAX` is not safe: xdiff computes a
/// hunk's end as `start + change + context` in `int` *before* clamping it to the
/// record count, so a near-`INT_MAX` context overflows negative and the clamp
/// never fires. Ten million lines exceeds any file worth diffing.
const FULL_CONTEXT: &str = "10000000";

pub(crate) fn branch_diff(runner: &GitRunner, base: &RefName) -> Result<String, InspectionError> {
    super::queries::ensure_remote_base(base)?;
    let range = merge_base_range(base);
    patch_text(runner, diff_args(&range, LOAD_CONTEXT, /*pathspec=*/ None))
}

/// Parsed from the very string `branch_diff` returns, so the copyable patch and
/// the structured diff are the same Git output by construction and cannot drift.
pub(crate) fn files_diff(
    runner: &GitRunner,
    base: &RefName,
) -> Result<Vec<FileDiff>, InspectionError> {
    super::patch::parse_patch(&branch_diff(runner, base)?)
}

/// One file at full context, so a viewer can reveal any window of it without
/// another round trip. `None` means the path no longer differs from Base — HEAD
/// may have moved since the diff was loaded — which is a refresh, not an error.
pub(crate) fn full_file_diff(
    runner: &GitRunner,
    base: &RefName,
    path: &RepoPath,
) -> Result<Option<FileDiff>, InspectionError> {
    super::queries::ensure_remote_base(base)?;
    let range = merge_base_range(base);
    // Pinned so the pathspec and the names Git prints agree below the Git root.
    let pathspec = format!(":(top,literal){}", path.as_str());
    let text = patch_text(runner, diff_args(&range, FULL_CONTEXT, Some(&pathspec)))?;
    let Some(mut file) = super::patch::parse_patch(&text)?.pop() else {
        return Ok(None);
    };
    file.complete = true;
    Ok(Some(file))
}

fn merge_base_range(base: &RefName) -> String {
    format!("{}...HEAD", base.as_str())
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
