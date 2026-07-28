use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use super::command::{AccessMode, GitCommand, GitOutput};
use super::error::GitError;

pub(crate) fn execute(git: &Path, repo: &Path, command: GitCommand) -> Result<GitOutput, GitError> {
    let mut process = Command::new(git);
    process.current_dir(repo).args(&command.args);
    set_environment(&mut process, &command);
    let output = match command.stdin {
        Some(input) => run_with_input(&mut process, &input),
        None => process.output(),
    }
    .map_err(|source| GitError::Spawn { source })?;
    let result = GitOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code(),
    };
    if !output.status.success() {
        return Err(GitError::Command {
            args: command.args,
            exit_code: result.exit_code,
            stderr: result.stderr,
        });
    }
    Ok(result)
}

fn set_environment(process: &mut Command, command: &GitCommand) {
    process.env("GIT_TERMINAL_PROMPT", "0");
    process.env("GIT_EDITOR", "true");
    process.env("GIT_PAGER", "cat");
    process.env("LC_ALL", "C");
    if command.access == AccessMode::ReadOnly {
        process.env("GIT_OPTIONAL_LOCKS", "0");
    }
    for (key, value) in &command.environment {
        process.env(key, value);
    }
}

fn run_with_input(process: &mut Command, input: &[u8]) -> std::io::Result<Output> {
    process.stdin(Stdio::piped());
    process.stdout(Stdio::piped());
    process.stderr(Stdio::piped());
    let mut child = process.spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input)?;
    }
    child.wait_with_output()
}

pub(crate) fn args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(|value| OsString::from(*value)).collect()
}
