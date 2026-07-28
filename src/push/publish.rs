use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};
use crate::rewrite::{ObjectId, RefName};

use super::errors::PublishError;
use super::model::PublishBranchPlan;

pub(crate) fn create(
    runner: &GitRunner,
    branch_name: String,
) -> Result<PublishBranchPlan, PublishError> {
    let branch = local_ref(runner, &branch_name)?;
    let source_head = read_id(runner, branch.as_str())?;
    let remote = resolve_remote(runner)?;
    let remote_branch =
        RefName::new(format!("refs/heads/{branch_name}")).map_err(PublishError::InvalidState)?;
    let upstream = RefName::new(format!("refs/remotes/{remote}/{branch_name}"))
        .map_err(PublishError::InvalidState)?;
    if exists(runner, upstream.as_str()) {
        return Err(PublishError::ExistingRemoteBranch(upstream.to_string()));
    }
    Ok(PublishBranchPlan {
        command: command(&remote, &remote_branch),
        branch,
        branch_name,
        remote,
        remote_branch,
        upstream,
        source_head,
    })
}

pub(crate) fn verify_current(
    runner: &GitRunner,
    plan: &PublishBranchPlan,
) -> Result<(), PublishError> {
    if create(runner, plan.branch_name.clone())? != *plan {
        return Err(PublishError::StalePlan);
    }
    Ok(())
}

/// A brand-new branch has no `branch.<name>.remote` of its own, so the remote is
/// taken from the checked-out branch that the work came from, then from
/// `remote.pushDefault`. Nothing is guessed: with neither set, publishing fails.
fn resolve_remote(runner: &GitRunner) -> Result<String, PublishError> {
    let remote = current_branch_remote(runner)?
        .or(read_config(runner, "remote.pushDefault")?)
        .ok_or(PublishError::NoRemote)?;
    if remote == "." {
        return Err(PublishError::LocalRemote);
    }
    Ok(remote)
}

fn current_branch_remote(runner: &GitRunner) -> Result<Option<String>, PublishError> {
    let Ok(output) = runner.run(GitCommand::read(args(&[
        "symbolic-ref",
        "--quiet",
        "--short",
        "HEAD",
    ]))) else {
        return Ok(None);
    };
    let current = text(&output.stdout)?.trim().to_string();
    if current.is_empty() {
        return Ok(None);
    }
    read_config(runner, &format!("branch.{current}.remote"))
}

fn local_ref(runner: &GitRunner, branch_name: &str) -> Result<RefName, PublishError> {
    if branch_name.is_empty() || branch_name.starts_with('-') || branch_name.contains('\0') {
        return Err(PublishError::InvalidState(format!(
            "invalid branch name: {branch_name}"
        )));
    }
    let reference = format!("refs/heads/{branch_name}");
    if !exists(runner, &reference) {
        return Err(PublishError::MissingBranch(branch_name.to_string()));
    }
    RefName::new(reference).map_err(PublishError::InvalidState)
}

fn exists(runner: &GitRunner, reference: &str) -> bool {
    runner
        .run(GitCommand::read(args(&[
            "show-ref", "--verify", "--quiet", reference,
        ])))
        .is_ok()
}

fn read_config(runner: &GitRunner, key: &str) -> Result<Option<String>, PublishError> {
    let output = runner.run(GitCommand::read(args(&[
        "config",
        "--local",
        "--default",
        "",
        "--get",
        key,
    ])))?;
    let value = text(&output.stdout)?.trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

fn read_id(runner: &GitRunner, name: &str) -> Result<ObjectId, PublishError> {
    let spec = format!("{name}^{{commit}}");
    let output = runner.run(GitCommand::read(args(&["rev-parse", "--verify", &spec])))?;
    ObjectId::new(text(&output.stdout)?.trim().to_string()).map_err(PublishError::InvalidState)
}

/// The empty lease value requires the remote ref to be absent, so a branch
/// someone else pushed in the meantime fails the push instead of being replaced.
fn command(remote: &str, remote_branch: &RefName) -> String {
    format!(
        "git push --force-with-lease={remote_branch}: --set-upstream {remote} {remote_branch}:{remote_branch}"
    )
}

fn text(bytes: &[u8]) -> Result<String, PublishError> {
    String::from_utf8(bytes.to_vec())
        .map_err(|_| PublishError::InvalidState("Git output is not UTF-8".to_string()))
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
