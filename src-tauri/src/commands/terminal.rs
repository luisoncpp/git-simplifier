use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

use super::prefs::PrefsStore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSpawnSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
}

pub fn default_terminal_name() -> String {
    if is_windows_terminal_available() {
        "Windows Terminal (PowerShell)".into()
    } else {
        "Windows PowerShell".into()
    }
}

pub fn is_windows_terminal_available() -> bool {
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let app = PathBuf::from(local)
                .join("Microsoft")
                .join("WindowsApps")
                .join("wt.exe");
            if app.exists() {
                return true;
            }
        }
        find_in_path("wt.exe") || find_in_path("wt")
    }
    #[cfg(not(windows))]
    {
        find_in_path("wt")
    }
}

pub fn terminal_spawn_spec(
    custom: &str,
    folder: &str,
    wt_available: bool,
) -> Result<TerminalSpawnSpec, String> {
    let trimmed_folder = folder.trim();
    if trimmed_folder.is_empty() {
        return Err("Repository path is empty".into());
    }
    let trimmed_custom = custom.trim();
    if !trimmed_custom.is_empty() {
        return Ok(TerminalSpawnSpec {
            program: trimmed_custom.to_string(),
            args: Vec::new(),
            cwd: Some(trimmed_folder.to_string()),
        });
    }
    if wt_available {
        return Ok(TerminalSpawnSpec {
            program: "wt".into(),
            args: vec!["-d".into(), trimmed_folder.into(), "powershell.exe".into()],
            cwd: None,
        });
    }
    Ok(TerminalSpawnSpec {
        program: "powershell.exe".into(),
        args: Vec::new(),
        cwd: Some(trimmed_folder.to_string()),
    })
}

pub fn spawn_terminal(custom: &str, folder: &str) -> Result<(), String> {
    let spec = terminal_spawn_spec(custom, folder, is_windows_terminal_available())?;
    spawn_spec(&spec)
}

pub(crate) fn spawn_spec(spec: &TerminalSpawnSpec) -> Result<(), String> {
    let mut process = Command::new(&spec.program);
    process.args(&spec.args);
    if let Some(ref cwd) = spec.cwd {
        process.current_dir(cwd);
    }
    #[cfg(windows)]
    configure_console(&mut process, &spec.program);
    process
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open terminal: {error}"))
}

#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

#[cfg(windows)]
fn configure_console(process: &mut Command, _program: &str) {
    use std::os::windows::process::CommandExt;
    process.creation_flags(CREATE_NEW_CONSOLE);
}

pub(crate) fn find_in_path(name: &str) -> bool {
    let path_var = match std::env::var_os("PATH") {
        Some(val) => val,
        None => return false,
    };
    for dir in std::env::split_paths(&path_var) {
        if dir.join(name).exists() {
            return true;
        }
    }
    false
}

#[tauri::command(async)]
pub fn set_terminal_path(
    app: AppHandle,
    terminal_path: String,
) -> Result<super::prefs::UiPreferences, String> {
    PrefsStore::from_app(&app)?.set_terminal_path(terminal_path)
}

#[tauri::command(async)]
pub fn open_in_terminal(app: AppHandle, path: String) -> Result<(), String> {
    let prefs = PrefsStore::from_app(&app)?.load()?;
    spawn_terminal(&prefs.terminal_path, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_terminal_uses_custom_program_and_cwd() {
        let spec = terminal_spawn_spec(
            "C:\\tools\\alacritty.exe",
            "C:\\work\\repo",
            /*wt_available=*/ true,
        )
        .unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "C:\\tools\\alacritty.exe".into(),
                args: vec![],
                cwd: Some("C:\\work\\repo".into()),
            }
        );
    }

    #[test]
    fn default_with_wt_uses_wt_and_powershell_arg() {
        let spec = terminal_spawn_spec("", "C:\\work\\repo", /*wt_available=*/ true).unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "wt".into(),
                args: vec!["-d".into(), "C:\\work\\repo".into(), "powershell.exe".into()],
                cwd: None,
            }
        );
    }

    #[test]
    fn default_without_wt_falls_back_to_powershell() {
        let spec = terminal_spawn_spec("", "C:\\work\\repo", /*wt_available=*/ false).unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "powershell.exe".into(),
                args: vec![],
                cwd: Some("C:\\work\\repo".into()),
            }
        );
    }

    #[test]
    fn empty_folder_returns_error() {
        assert!(terminal_spawn_spec("", "   ", /*wt_available=*/ true).is_err());
    }
}
