mod annotate;
mod body;
mod list;
mod types;

use std::collections::BTreeSet;

use crate::git::GitRunner;
use crate::rewrite::RepoPath;

use super::errors::InspectionError;
use super::model::{FileDiff, UntrackedAnnotations};
use super::query::UntrackedFilters;

pub(crate) fn append_untracked(
    runner: &GitRunner,
    files: &mut Vec<FileDiff>,
    filters: UntrackedFilters,
) -> Result<(), InspectionError> {
    let paths = list::collect(runner, filters)?;
    if paths.is_empty() {
        return Ok(());
    }
    let head_time = annotate::head_commit_time(runner)?;
    let tracked: BTreeSet<String> = files.iter().map(|file| file.path.as_str().to_string()).collect();
    let mut ctx = PushCtx {
        runner,
        files,
        tracked: &tracked,
        filters,
        head_time,
    };
    for listed in paths {
        push_one(&mut ctx, &listed)?;
    }
    Ok(())
}

pub(crate) fn synthesized_if_untracked(
    runner: &GitRunner,
    path: &RepoPath,
) -> Result<Option<FileDiff>, InspectionError> {
    let Some(not_ignored) = list::classify_one(runner, path.as_str())? else {
        return Ok(None);
    };
    let head_time = annotate::head_commit_time(runner)?;
    let annotations = annotate::for_path(runner, path.as_str(), not_ignored, head_time)?;
    Ok(Some(body::synthesize(runner, path.clone(), annotations)?))
}

struct PushCtx<'a> {
    runner: &'a GitRunner,
    files: &'a mut Vec<FileDiff>,
    tracked: &'a BTreeSet<String>,
    filters: UntrackedFilters,
    head_time: u64,
}

fn push_one(ctx: &mut PushCtx<'_>, listed: &list::ListedPath) -> Result<(), InspectionError> {
    if ctx.tracked.contains(&listed.path) {
        return Ok(());
    }
    if ctx.filters.exclude_unknown_types && !types::is_known_type(&listed.path) {
        return Ok(());
    }
    let repo_path = RepoPath::new(listed.path.clone()).map_err(InspectionError::Parse)?;
    let annotations =
        annotate::for_path(ctx.runner, &listed.path, listed.not_ignored, ctx.head_time)?;
    if hidden_by_filters(ctx.filters, &annotations) {
        return Ok(());
    }
    // Gitignored / node_modules trees can be huge; listing must not read bodies.
    // Expansion loads content through `synthesized_if_untracked` / full_file_diff.
    if body::should_stub(&annotations) {
        ctx.files.push(body::stub(repo_path, annotations));
        return Ok(());
    }
    ctx.files
        .push(body::synthesize(ctx.runner, repo_path, annotations)?);
    Ok(())
}

fn hidden_by_filters(filters: UntrackedFilters, annotations: &UntrackedAnnotations) -> bool {
    (filters.exclude_older_than_head && annotations.older_than_or_at_head)
        || (filters.exclude_root_dot && annotations.root_dot)
        || (filters.exclude_node_modules && annotations.in_node_modules)
        || (filters.respect_gitignore && annotations.gitignored)
}
