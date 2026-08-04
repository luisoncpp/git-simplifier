mod annotate;
mod body;
mod list;

use std::collections::BTreeSet;

use crate::git::GitRunner;
use crate::rewrite::RepoPath;

use super::errors::InspectionError;
use super::model::FileDiff;

pub(crate) fn append_untracked(
    runner: &GitRunner,
    files: &mut Vec<FileDiff>,
) -> Result<(), InspectionError> {
    let paths = list::all(runner)?;
    if paths.is_empty() {
        return Ok(());
    }
    let visible = list::not_ignored(runner)?;
    let not_ignored: BTreeSet<&str> = visible.iter().map(String::as_str).collect();
    let head_time = annotate::head_commit_time(runner)?;
    let tracked: BTreeSet<String> = files.iter().map(|file| file.path.as_str().to_string()).collect();
    for path in paths {
        if tracked.contains(&path) {
            continue;
        }
        let repo_path = RepoPath::new(path.clone()).map_err(InspectionError::Parse)?;
        let annotations = annotate::for_path(
            runner,
            &path,
            not_ignored.contains(path.as_str()),
            head_time,
        )?;
        // Gitignored / node_modules trees can be huge; listing must not read bodies.
        // Expansion loads content through `synthesized_if_untracked` / full_file_diff.
        if body::should_stub(&annotations) {
            files.push(body::stub(repo_path, annotations));
            continue;
        }
        files.push(body::synthesize(runner, repo_path, annotations)?);
    }
    Ok(())
}

pub(crate) fn synthesized_if_untracked(
    runner: &GitRunner,
    path: &RepoPath,
) -> Result<Option<FileDiff>, InspectionError> {
    let all = list::all(runner)?;
    if !all.iter().any(|entry| entry == path.as_str()) {
        return Ok(None);
    }
    let visible = list::not_ignored(runner)?;
    let not_ignored = visible.iter().any(|entry| entry == path.as_str());
    let head_time = annotate::head_commit_time(runner)?;
    let annotations = annotate::for_path(runner, path.as_str(), not_ignored, head_time)?;
    Ok(Some(body::synthesize(runner, path.clone(), annotations)?))
}
