use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RepoPath};

use super::errors::ExclusionError;
use super::hook;
use super::model::{ExcludeSubmodulePlan, ExcludeSubmoduleRequest};

pub(crate) fn create(
    runner: &GitRunner,
    request: ExcludeSubmoduleRequest,
) -> Result<ExcludeSubmodulePlan, ExclusionError> {
    let source_head = read_id(runner)?;
    ensure_gitlink(runner, &request.path)?;
    let current_ignore = read_config(runner, &ignore_key(&request.path))?;
    let current_recurse = read_config(runner, "submodule.recurse")?;
    let hook_path = git_path(runner, "hooks")?.join("pre-commit");
    let hook_exists = hook_path.exists();
    let current_hook = read_hook(&hook_path)?;
    let hook_preview = hook::block(&request.path);
    let hook_will_change =
        request.install_hook && !contains(&current_hook, hook_preview.as_bytes());
    Ok(ExcludeSubmodulePlan {
        path: request.path.clone(),
        install_hook: request.install_hook,
        disable_recurse: request.disable_recurse,
        config_lines: config_lines(&request.path, request.disable_recurse),
        staging_command: staging_command(&request.path),
        hook_path,
        hook_preview,
        hook_exists,
        hook_will_change,
        current_ignore,
        current_recurse,
        current_hook,
        source_head,
    })
}

pub(crate) fn verify_current(
    runner: &GitRunner,
    plan: &ExcludeSubmodulePlan,
) -> Result<(), ExclusionError> {
    if read_id(runner)? != plan.source_head {
        return Err(ExclusionError::StalePlan);
    }
    ensure_gitlink(runner, &plan.path)?;
    if read_config(runner, &ignore_key(&plan.path))? != plan.current_ignore {
        return Err(ExclusionError::StalePlan);
    }
    if plan.disable_recurse && read_config(runner, "submodule.recurse")? != plan.current_recurse {
        return Err(ExclusionError::StalePlan);
    }
    if plan.hook_will_change && read_hook(&plan.hook_path)? != plan.current_hook {
        return Err(ExclusionError::StalePlan);
    }
    Ok(())
}

pub(crate) fn ignore_key(path: &RepoPath) -> String {
    format!("submodule.{}.ignore", path.as_str())
}

pub(crate) fn config_lines(path: &RepoPath, disable_recurse: bool) -> Vec<String> {
    let mut lines = vec![format!(
        "git config --local --replace-all {} all",
        hook::shell_quote(&ignore_key(path))
    )];
    if disable_recurse {
        lines.push("git config --local --replace-all submodule.recurse false".to_string());
    }
    lines
}

fn read_id(runner: &GitRunner) -> Result<ObjectId, ExclusionError> {
    let output = runner.run(GitCommand::read(args(&[
        "rev-parse",
        "--verify",
        "HEAD^{commit}",
    ])))?;
    let value = text(&output.stdout)?;
    ObjectId::new(value.trim().to_string()).map_err(ExclusionError::InvalidState)
}

fn ensure_gitlink(runner: &GitRunner, path: &RepoPath) -> Result<(), ExclusionError> {
    let pathspec = format!(":(literal){}", path.as_str());
    let output = runner.run(GitCommand::read(args_with_path(
        &["ls-tree", "-r", "-z", "--full-tree", "HEAD", "--"],
        &pathspec,
    )))?;
    let found = output.stdout.split(|byte| *byte == 0).any(|record| {
        record.starts_with(b"160000 commit ")
            && record
                .iter()
                .position(|byte| *byte == b'\t')
                .map(|tab| record[tab + 1..] == *path.as_str().as_bytes())
                .unwrap_or(false)
    });
    if found {
        return Ok(());
    }
    Err(ExclusionError::InvalidState(format!(
        "path is not a submodule gitlink: {}",
        path
    )))
}

fn read_config(runner: &GitRunner, key: &str) -> Result<Option<String>, ExclusionError> {
    let output = runner.run(GitCommand::read(args_with_path(
        &["config", "--local", "--default", "", "--get"],
        key,
    )))?;
    let value = text(&output.stdout)?.trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

fn git_path(runner: &GitRunner, name: &str) -> Result<PathBuf, ExclusionError> {
    let output = runner.run(GitCommand::read(args_with_path(
        &["rev-parse", "--git-path"],
        name,
    )))?;
    let value = PathBuf::from(text(&output.stdout)?.trim());
    if value.is_absolute() {
        return Ok(value);
    }
    Ok(runner.repo_path().join(value))
}

fn read_hook(path: &PathBuf) -> Result<Vec<u8>, ExclusionError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(fs::read(path)?)
}

fn staging_command(path: &RepoPath) -> String {
    format!(
        "git add -u -- {}",
        hook::shell_quote(&format!(":!{}", path.as_str()))
    )
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn text(bytes: &[u8]) -> Result<String, ExclusionError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ExclusionError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}

fn args_with_path(values: &[&str], path: &str) -> Vec<OsString> {
    let mut result = args(values);
    result.push(OsString::from(path));
    result
}
