use std::fs;

use crate::git::{GitCommand, GitRunner};
use crate::recording::{timestamp, OperationRecord, Oplog};

use super::errors::ExclusionError;
use super::model::{ExcludeSubmodulePlan, ExcludeSubmoduleResult};
use super::plan;

pub(crate) fn apply(
    runner: &GitRunner,
    exclusion: &ExcludeSubmodulePlan,
) -> Result<ExcludeSubmoduleResult, ExclusionError> {
    plan::verify_current(runner, exclusion)?;
    let oplog = Oplog::open(&runner.git_dir()?)
        .map_err(|error| ExclusionError::Recording(error.to_string()))?;
    let operation_id = begin_record(&oplog, exclusion)?;
    let config_changed = write_config(runner, exclusion)?;
    let hook_changed = write_hook(exclusion)?;
    oplog
        .finish(&operation_id, Default::default())
        .map_err(|error| ExclusionError::Recording(error.to_string()))?;
    Ok(ExcludeSubmoduleResult {
        path: exclusion.path.clone(),
        config_changed,
        hook_changed,
    })
}

fn write_config(
    runner: &GitRunner,
    exclusion: &ExcludeSubmodulePlan,
) -> Result<bool, ExclusionError> {
    let mut changed = exclusion.current_ignore.as_deref() != Some("all");
    if changed {
        set_config(runner, &plan::ignore_key(&exclusion.path), "all")?;
    }
    if exclusion.disable_recurse && exclusion.current_recurse.as_deref() != Some("false") {
        set_config(runner, "submodule.recurse", "false")?;
        changed = true;
    }
    Ok(changed)
}

fn set_config(runner: &GitRunner, key: &str, value: &str) -> Result<(), ExclusionError> {
    let values = vec!["config", "--local", "--replace-all", key, value];
    runner.run_unlocked(GitCommand::write(GitRunner::command_args(&values)))?;
    Ok(())
}

fn write_hook(exclusion: &ExcludeSubmodulePlan) -> Result<bool, ExclusionError> {
    if !exclusion.hook_will_change {
        return Ok(false);
    }
    let mut content = exclusion.current_hook.clone();
    if content.is_empty() {
        content.extend_from_slice(b"#!/bin/sh\n");
    } else if !content.ends_with(b"\n") {
        content.push(b'\n');
    }
    content.extend_from_slice(exclusion.hook_preview.as_bytes());
    if let Some(parent) = exclusion.hook_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&exclusion.hook_path, content)?;
    make_executable(&exclusion.hook_path)?;
    Ok(true)
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> Result<(), ExclusionError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> Result<(), ExclusionError> {
    Ok(())
}

fn begin_record(oplog: &Oplog, exclusion: &ExcludeSubmodulePlan) -> Result<String, ExclusionError> {
    let started = timestamp();
    let id = format!("exclude-submodule-{started}-{}", std::process::id());
    let mut commands = exclusion.config_lines.clone();
    if exclusion.hook_will_change {
        commands.push(format!(
            "append excluded-submodule guard to {}",
            exclusion.hook_path.display()
        ));
    }
    let record = OperationRecord {
        id: id.clone(),
        operation: "exclude-submodule".to_string(),
        started,
        finished: None,
        refs_before: Default::default(),
        refs_after: Default::default(),
        snapshots: Default::default(),
        details: Default::default(),
        phase: None,
        commands,
        reversible: false,
    };
    oplog
        .begin(record)
        .map_err(|error| ExclusionError::Recording(error.to_string()))?;
    Ok(id)
}
