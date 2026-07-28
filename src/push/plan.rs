use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::errors::ForcePushError;
use super::model::ForcePushPlan;

pub(crate) fn create(runner: &GitRunner) -> Result<ForcePushPlan, ForcePushError> {
    let branch = read_branch(runner)?;
    let branch_name = branch_name(&branch)?;
    let remote = read_config(runner, &format!("branch.{branch_name}.remote"))?
        .ok_or(ForcePushError::NoUpstream)?;
    if remote == "." {
        return Err(ForcePushError::LocalUpstream);
    }
    let remote_branch = read_config(runner, &format!("branch.{branch_name}.merge"))?
        .ok_or(ForcePushError::NoUpstream)?;
    let remote_branch = validate_remote_branch(remote_branch)?;
    let branch_path = remote_branch
        .as_str()
        .strip_prefix("refs/heads/")
        .ok_or_else(|| ForcePushError::InvalidState(remote_branch.to_string()))?;
    let upstream = RefName::new(format!("refs/remotes/{remote}/{branch_path}"))
        .map_err(ForcePushError::InvalidState)?;
    let expected_remote = read_id(runner, upstream.as_str())?;
    let source_head = read_id(runner, "HEAD")?;
    let command = command(&remote, &remote_branch, &expected_remote);
    Ok(ForcePushPlan {
        branch,
        upstream,
        remote,
        remote_branch,
        expected_remote,
        source_head,
        command,
    })
}

pub(crate) fn verify_current(
    runner: &GitRunner,
    plan: &ForcePushPlan,
) -> Result<(), ForcePushError> {
    if create(runner)? != *plan {
        return Err(ForcePushError::StalePlan);
    }
    Ok(())
}

fn read_branch(runner: &GitRunner) -> Result<RefName, ForcePushError> {
    let output = runner
        .run(GitCommand::read(args(&["symbolic-ref", "--quiet", "HEAD"])))
        .map_err(|_| ForcePushError::DetachedHead)?;
    let value = text(&output.stdout)?;
    RefName::new(value.trim().to_string()).map_err(ForcePushError::InvalidState)
}

fn branch_name(branch: &RefName) -> Result<String, ForcePushError> {
    branch
        .as_str()
        .strip_prefix("refs/heads/")
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ForcePushError::InvalidState("HEAD is not a local branch".to_string()))
}

fn read_config(runner: &GitRunner, key: &str) -> Result<Option<String>, ForcePushError> {
    let values = ["config", "--local", "--default", "", "--get", key];
    let output = runner.run(GitCommand::read(args(&values)))?;
    let value = text(&output.stdout)?.trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

fn validate_remote_branch(value: String) -> Result<RefName, ForcePushError> {
    if !value.starts_with("refs/heads/") || value == "refs/heads/" {
        return Err(ForcePushError::InvalidState(value));
    }
    RefName::new(value).map_err(ForcePushError::InvalidState)
}

fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, ForcePushError> {
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", name])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(ForcePushError::InvalidState)
}

fn command(remote: &str, remote_branch: &RefName, expected: &ObjectId) -> String {
    format!(
        "git push --force-with-lease={}:{} {} HEAD:{}",
        remote_branch, expected, remote, remote_branch
    )
}

fn text(bytes: &[u8]) -> Result<String, ForcePushError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| ForcePushError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
