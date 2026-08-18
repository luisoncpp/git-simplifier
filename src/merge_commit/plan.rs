use crate::git::GitRunner;

use super::errors::CommitMergeError;
use super::model::{CommitMergePlan, MergeParents};
use super::paths::{excluded_paths, literal, pr_paths_before, triple_dot_paths};
use super::preflight;
use super::state::{merge_base, optional_base, read_branch, read_id};
use super::tree::{build, stage_zero_for_commands};

pub(crate) fn create(runner: &GitRunner) -> Result<CommitMergePlan, CommitMergeError> {
    preflight::check(runner)?;
    let branch = read_branch(runner)?;
    let source_head = read_id(runner, "HEAD")?;
    let merge_head = read_id(runner, "MERGE_HEAD")?;
    let parents = MergeParents {
        base: merge_base(runner, &source_head, &merge_head)?,
        ours: source_head.clone(),
        theirs: merge_head.clone(),
    };
    let built = build(runner, &parents)?;
    let base = optional_base(runner)?;
    let pr_paths_before = match base.as_ref() {
        Some(base_ref) => pr_paths_before(runner, base_ref, &merge_head)?,
        None => Vec::new(),
    };
    let excluded = excluded_paths(runner, &merge_head, &built.tree)?;
    let commands = derive_commands(runner, &parents, &built)?;
    Ok(CommitMergePlan {
        branch,
        source_head,
        merge_head,
        merge_base: parents.base,
        tree: built.tree,
        base,
        conflicted_paths: built.conflicted_paths,
        excluded_paths: excluded,
        pr_paths_before,
        commands,
    })
}

fn derive_commands(
    runner: &GitRunner,
    parents: &MergeParents,
    built: &super::tree::BuiltTree,
) -> Result<Vec<String>, CommitMergeError> {
    let mut commands = vec![
        "git read-tree --empty".to_string(),
        format!("git read-tree -m {} HEAD MERGE_HEAD", parents.base),
    ];
    push_resolution_commands(runner, built, &mut commands)?;
    commands.push(format!("git write-tree  # -> {}", built.tree));
    commands.push(format!("git read-tree {}", built.tree));
    commands.push("git -c submodule.recurse=false commit --no-edit --no-verify".to_string());
    Ok(commands)
}

fn push_resolution_commands(
    runner: &GitRunner,
    built: &super::tree::BuiltTree,
    commands: &mut Vec<String>,
) -> Result<(), CommitMergeError> {
    for path in &built.conflicted_paths {
        let line = match stage_zero_for_commands(runner, path)? {
            Some(entry) => format!(
                "git update-index --cacheinfo {} {} {}",
                entry.mode,
                entry.object,
                literal(path.as_str())
            ),
            None => format!("git update-index --force-remove {}", literal(path.as_str())),
        };
        commands.push(line);
    }
    Ok(())
}

pub(crate) fn verify_current(
    runner: &GitRunner,
    plan: &CommitMergePlan,
) -> Result<(), CommitMergeError> {
    if read_id(runner, "HEAD")? != plan.source_head {
        return Err(CommitMergeError::StalePlan);
    }
    if read_id(runner, "MERGE_HEAD")? != plan.merge_head {
        return Err(CommitMergeError::StalePlan);
    }
    if read_branch(runner)? != plan.branch {
        return Err(CommitMergeError::StalePlan);
    }
    preflight::check(runner)?;
    let parents = MergeParents {
        base: plan.merge_base.clone(),
        ours: plan.source_head.clone(),
        theirs: plan.merge_head.clone(),
    };
    if build(runner, &parents)?.tree != plan.tree {
        return Err(CommitMergeError::StalePlan);
    }
    Ok(())
}

pub(crate) fn check_pr_subset(
    runner: &GitRunner,
    plan: &CommitMergePlan,
) -> Result<(), CommitMergeError> {
    if plan.pr_paths_before.is_empty() {
        return Ok(());
    }
    let base = plan
        .base
        .as_ref()
        .ok_or_else(|| CommitMergeError::InvalidState("Base vanished".to_string()))?;
    let after = triple_dot_paths(runner, base)?;
    let extra = super::paths::extras_vs_pr(&plan.pr_paths_before, &after);
    if extra.is_empty() {
        return Ok(());
    }
    let names = extra
        .iter()
        .map(|path| path.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(CommitMergeError::InvalidState(format!(
        "merge would add to Base…HEAD: {names}"
    )))
}
