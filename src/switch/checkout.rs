use std::ffi::OsString;

use crate::git::GitCommand;

use super::errors::SwitchError;
use super::model::QuickSwitchPlan;
use super::present;

pub(crate) fn switch_branch(
    runner: &crate::git::GitRunner,
    switch_plan: &QuickSwitchPlan,
) -> Result<(), SwitchError> {
    if let Some(remote) = &switch_plan.create_from_remote {
        let start = remote
            .strip_prefix("refs/remotes/")
            .unwrap_or(remote.as_str());
        runner.run_unlocked(GitCommand::write(vec![
            OsString::from("switch"),
            OsString::from("--no-recurse-submodules"),
            OsString::from("-c"),
            OsString::from(&switch_plan.target_branch),
            OsString::from(remote),
        ]))?;
        return match set_upstream(runner, &switch_plan.target_branch, start) {
            Ok(()) => present::delete(runner),
            Err(error) => Err(error),
        };
    }
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("switch"),
        OsString::from("--no-recurse-submodules"),
        OsString::from("--no-guess"),
        OsString::from("--"),
        OsString::from(&switch_plan.target_branch),
    ]))?;
    present::delete(runner)?;
    Ok(())
}

pub(crate) fn switch_detach(
    runner: &crate::git::GitRunner,
    commit: &str,
) -> Result<(), SwitchError> {
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("switch"),
        OsString::from("--no-recurse-submodules"),
        OsString::from("--detach"),
        OsString::from(commit),
    ]))?;
    Ok(())
}

fn set_upstream(
    runner: &crate::git::GitRunner,
    local: &str,
    remote_short: &str,
) -> Result<(), SwitchError> {
    let (remote, branch) = remote_short.split_once('/').ok_or_else(|| {
        SwitchError::InvalidState(format!("invalid remote-tracking name: {remote_short}"))
    })?;
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("config"),
        OsString::from("--local"),
        OsString::from(format!("branch.{local}.remote")),
        OsString::from(remote),
    ]))?;
    runner.run_unlocked(GitCommand::write(vec![
        OsString::from("config"),
        OsString::from("--local"),
        OsString::from(format!("branch.{local}.merge")),
        OsString::from(format!("refs/heads/{branch}")),
    ]))?;
    Ok(())
}
