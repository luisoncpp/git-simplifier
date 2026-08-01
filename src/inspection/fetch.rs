use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};

use super::errors::InspectionError;

pub(super) fn fetch_remotes(runner: &GitRunner) -> Result<(), InspectionError> {
    runner.run_unlocked(GitCommand::write(args(&[
        "fetch",
        "--all",
        "--no-tags",
        "--no-recurse-submodules",
    ])))?;
    Ok(())
}

fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
