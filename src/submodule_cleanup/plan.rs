use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::inspection::{self, DirtySubmodule};
use crate::rewrite::{ObjectId, RefName, RepoPath, UncommitRequest};

use super::errors::SubmoduleCleanupError;
use super::model::{SubmoduleCleanupPlan, SubmoduleCleanupRequest};

pub(super) fn create(
    runner: &GitRunner,
    request: SubmoduleCleanupRequest,
) -> Result<SubmoduleCleanupPlan, SubmoduleCleanupError> {
    if request.paths.is_empty() {
        return Err(SubmoduleCleanupError::InvalidState(
            "select at least one submodule".to_string(),
        ));
    }
    if !request.uncommit && !request.revert {
        return Err(SubmoduleCleanupError::InvalidState(
            "select at least one cleanup action".to_string(),
        ));
    }
    let eligible = eligible_paths(runner, &request.base)?;
    for path in &request.paths {
        if !eligible.contains(path) {
            return Err(SubmoduleCleanupError::InvalidState(format!(
                "submodule is not dirty: {path}"
            )));
        }
    }
    let flags = flags_for(runner, &request.base)?;
    let uncommit_paths = if request.uncommit {
        request
            .paths
            .iter()
            .filter(|path| {
                flags
                    .get(*path)
                    .is_some_and(|entry| entry.in_editable_range)
            })
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    let revert_paths = if request.revert {
        request.paths.clone()
    } else {
        Vec::new()
    };
    let source_head = read_id(runner, "HEAD")?;
    let uncommit_plan = if uncommit_paths.is_empty() {
        None
    } else {
        Some(crate::rewrite::plan(
            runner,
            UncommitRequest {
                base: request.base.clone(),
                paths: uncommit_paths.clone(),
            },
        )?)
    };
    let mut commands = Vec::new();
    if !revert_paths.is_empty() {
        for path in &revert_paths {
            commands.push(restore_command(path.as_str()));
            commands.push(sync_command(&[path.clone()]));
            commands.push(nested_checkout_command(path.as_str()));
            commands.push(nested_clean_command(path.as_str()));
        }
    }
    Ok(SubmoduleCleanupPlan {
        paths: request.paths,
        uncommit: request.uncommit,
        revert: request.revert,
        uncommit_paths,
        revert_paths,
        base_ref: request.base,
        commands,
        uncommit_plan,
        source_head,
    })
}

pub(super) fn verify_current(
    runner: &GitRunner,
    plan: &SubmoduleCleanupPlan,
) -> Result<(), SubmoduleCleanupError> {
    if read_id(runner, "HEAD")? != plan.source_head {
        return Err(SubmoduleCleanupError::StalePlan);
    }
    let eligible = eligible_paths(runner, &plan.base_ref)?;
    for path in &plan.paths {
        if !eligible.contains(path) {
            return Err(SubmoduleCleanupError::StalePlan);
        }
    }
    Ok(())
}

fn eligible_paths(
    runner: &GitRunner,
    base: &RefName,
) -> Result<BTreeSet<RepoPath>, SubmoduleCleanupError> {
    Ok(inspection::dirty_submodules(runner, Some(base))
        .map_err(|error| SubmoduleCleanupError::InvalidState(error.to_string()))?
        .into_iter()
        .map(|entry| entry.path)
        .collect())
}

fn flags_for(
    runner: &GitRunner,
    base: &RefName,
) -> Result<BTreeMap<RepoPath, DirtySubmodule>, SubmoduleCleanupError> {
    Ok(inspection::dirty_submodules(runner, Some(base))
        .map_err(|error| SubmoduleCleanupError::InvalidState(error.to_string()))?
        .into_iter()
        .map(|entry| (entry.path.clone(), entry))
        .collect())
}

fn sync_command(paths: &[RepoPath]) -> String {
    let mut parts = vec![
        "git".to_string(),
        "-c".to_string(),
        "submodule.recurse=false".to_string(),
        "submodule".to_string(),
        "update".to_string(),
        "--force".to_string(),
        "--".to_string(),
    ];
    for path in paths {
        parts.push(path.as_str().to_string());
    }
    parts.join(" ")
}

fn restore_command(path: &str) -> String {
    format!(
        "git -c submodule.recurse=false restore --source=HEAD --staged --worktree -- :(top,literal){path}"
    )
}

fn nested_checkout_command(path: &str) -> String {
    format!("git -C {path} checkout --force HEAD")
}

fn nested_clean_command(path: &str) -> String {
    format!("git -C {path} clean -fd")
}

fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, SubmoduleCleanupError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string())
        .map_err(SubmoduleCleanupError::InvalidState)
}

fn text(bytes: &[u8]) -> Result<String, SubmoduleCleanupError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| SubmoduleCleanupError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
