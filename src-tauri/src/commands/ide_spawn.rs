use std::path::{Path, PathBuf};
use std::process::Command;

use super::project_settings::IdeChoice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnSpec {
    pub program: String,
    pub args: Vec<String>,
}

pub fn ide_folder_spawn_spec(choice: &IdeChoice) -> Result<SpawnSpec, String> {
    let program = ide_program(choice)?;
    Ok(SpawnSpec {
        program,
        args: Vec::new(),
    })
}

pub fn ide_file_spawn_spec(choice: &IdeChoice, file_path: &str) -> Result<SpawnSpec, String> {
    let program = ide_program(choice)?;
    let args = match choice {
        IdeChoice::Vscode | IdeChoice::Cursor => vec!["--reuse-window".into(), file_path.into()],
        IdeChoice::VisualStudio => vec!["/edit".into(), file_path.into()],
        IdeChoice::Rider | IdeChoice::Custom { .. } => vec![file_path.into()],
    };
    Ok(SpawnSpec { program, args })
}

fn ide_program(choice: &IdeChoice) -> Result<String, String> {
    match choice {
        IdeChoice::Vscode => Ok("code".into()),
        IdeChoice::Cursor => Ok("cursor".into()),
        IdeChoice::VisualStudio => Ok("devenv".into()),
        IdeChoice::Rider => Ok("rider".into()),
        IdeChoice::Custom { command } => {
            let trimmed = command.trim();
            if trimmed.is_empty() {
                return Err("Custom IDE command is empty".into());
            }
            Ok(trimmed.to_string())
        }
    }
}

pub fn resolve_repo_file(worktree_root: &Path, file_path: &str) -> Result<PathBuf, String> {
    if file_path.is_empty() {
        return Err("File path is empty".into());
    }
    let joined = worktree_root.join(file_path);
    Ok(joined.canonicalize().unwrap_or(joined))
}

pub fn spawn_ide(spec: &SpawnSpec, folder: Option<&str>) -> Result<(), String> {
    #[cfg(windows)]
    if windows_use_cmd_shim(&spec.program) {
        return spawn_windows_shim(spec, folder);
    }
    let mut process = Command::new(&spec.program);
    process.args(&spec.args);
    if let Some(folder) = folder {
        process.arg(folder);
    }
    hide_console(&mut process);
    process
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open IDE: {error}"))
}

#[cfg(windows)]
fn spawn_windows_shim(spec: &SpawnSpec, folder: Option<&str>) -> Result<(), String> {
    let mut process = Command::new("cmd");
    process.arg("/C").arg(&spec.program);
    process.args(&spec.args);
    if let Some(folder) = folder {
        process.arg(folder);
    }
    hide_console(&mut process);
    process
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open IDE: {error}"))
}

/// Bare PATH names (`code`, `cursor`) and `.cmd`/`.bat` shims are not
/// executable directly on Windows — `cmd /C` resolves them like the shell.
#[cfg(windows)]
fn windows_use_cmd_shim(program: &str) -> bool {
    let path = Path::new(program);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("exe") => false,
        Some("cmd" | "bat") => true,
        _ => !path.has_root(),
    }
}

#[cfg(not(windows))]
fn windows_use_cmd_shim(_program: &str) -> bool {
    false
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
fn hide_console(process: &mut Command) {
    use std::os::windows::process::CommandExt;
    process.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_process: &mut Command) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_spawn_specs() {
        assert_eq!(
            ide_folder_spawn_spec(&IdeChoice::Vscode).unwrap(),
            SpawnSpec {
                program: "code".into(),
                args: vec![],
            }
        );
        assert_eq!(
            ide_folder_spawn_spec(&IdeChoice::Cursor).unwrap(),
            SpawnSpec {
                program: "cursor".into(),
                args: vec![],
            }
        );
    }

    #[test]
    fn file_spawn_specs_reuse_open_windows() {
        let file = r"C:\work\alpha\src\app.ts";
        assert_eq!(
            ide_file_spawn_spec(&IdeChoice::Vscode, file).unwrap(),
            SpawnSpec {
                program: "code".into(),
                args: vec!["--reuse-window".into(), file.into()],
            }
        );
        assert_eq!(
            ide_file_spawn_spec(&IdeChoice::Cursor, file).unwrap(),
            SpawnSpec {
                program: "cursor".into(),
                args: vec!["--reuse-window".into(), file.into()],
            }
        );
        assert_eq!(
            ide_file_spawn_spec(&IdeChoice::VisualStudio, file).unwrap(),
            SpawnSpec {
                program: "devenv".into(),
                args: vec!["/edit".into(), file.into()],
            }
        );
        assert_eq!(
            ide_file_spawn_spec(&IdeChoice::Rider, file).unwrap(),
            SpawnSpec {
                program: "rider".into(),
                args: vec![file.into()],
            }
        );
    }

    #[test]
    fn resolve_repo_file_joins_from_worktree_root() {
        let root = Path::new(r"C:\work\monorepo");
        let resolved = resolve_repo_file(root, "packages/app/src/foo.ts").unwrap();
        assert_eq!(
            resolved,
            PathBuf::from(r"C:\work\monorepo\packages\app\src\foo.ts")
        );
    }

    #[test]
    fn custom_spawn_spec_rejects_empty_command() {
        assert!(ide_folder_spawn_spec(&IdeChoice::Custom {
            command: "   ".into(),
        })
        .is_err());
    }

    #[cfg(windows)]
    mod windows {
        use super::super::windows_use_cmd_shim;

        #[test]
        fn bare_names_use_cmd_shim() {
            assert!(windows_use_cmd_shim("code"));
            assert!(windows_use_cmd_shim("cursor"));
        }

        #[test]
        fn exe_paths_spawn_directly() {
            assert!(!windows_use_cmd_shim(r"C:\tools\code.exe"));
        }
    }
}
