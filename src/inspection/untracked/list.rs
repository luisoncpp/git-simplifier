use std::ffi::OsString;

use crate::git::{GitCommand, GitRunner};

use super::super::errors::InspectionError;

pub(super) fn all(runner: &GitRunner) -> Result<Vec<String>, InspectionError> {
    let mut paths = with_flags(runner, &["--exclude-standard"])?;
    let ignored = with_flags(runner, &["-i", "--exclude-standard"])?;
    for path in ignored {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    Ok(paths)
}

pub(super) fn not_ignored(runner: &GitRunner) -> Result<Vec<String>, InspectionError> {
    with_flags(runner, &["--exclude-standard"])
}

fn with_flags(runner: &GitRunner, flags: &[&str]) -> Result<Vec<String>, InspectionError> {
    let mut args = vec![
        OsString::from("ls-files"),
        OsString::from("-z"),
        OsString::from("--others"),
    ];
    for flag in flags {
        args.push(OsString::from(*flag));
    }
    let output = runner.run(GitCommand::read(args))?;
    Ok(parse_null_paths(&output.stdout))
}

fn parse_null_paths(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}
