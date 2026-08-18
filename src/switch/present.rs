use crate::git::{GitCommand, GitRunner};

use super::errors::SwitchError;
use super::state;

pub(super) const PRESENT_REF: &str = "refs/githelper/present";

pub(super) fn read(runner: &GitRunner) -> Result<Option<String>, SwitchError> {
    let Ok(output) = runner.run(GitCommand::read(state::args(&[
        "symbolic-ref",
        "--quiet",
        PRESENT_REF,
    ]))) else {
        return Ok(None);
    };
    let value = state::text(&output.stdout)?.trim().to_string();
    Ok(value
        .strip_prefix("refs/heads/")
        .map(|name| name.to_string())
        .filter(|name| !name.is_empty()))
}

pub(super) fn write(runner: &GitRunner, branch: &str) -> Result<(), SwitchError> {
    let target = state::branch_ref(branch);
    runner.run_unlocked(GitCommand::write(state::args(&[
        "symbolic-ref",
        PRESENT_REF,
        &target,
    ])))?;
    Ok(())
}

pub(super) fn delete(runner: &GitRunner) -> Result<(), SwitchError> {
    let _ = runner.run_unlocked(GitCommand::write(state::args(&[
        "symbolic-ref",
        "--delete",
        PRESENT_REF,
    ])));
    Ok(())
}
