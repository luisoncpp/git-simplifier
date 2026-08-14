use std::path::{Path, PathBuf};
use tauri::AppHandle;

use super::prefs::PrefsStore;
use super::terminal::{
    is_windows_terminal_available, spawn_spec, TerminalSpawnSpec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConfig<'a> {
    pub custom_terminal: &'a str,
    pub wt_available: bool,
}

pub fn resolve_bash_program(saved: &str, guessed: &str) -> String {
    let trimmed = saved.trim();
    if !trimmed.is_empty() {
        trimmed.to_string()
    } else {
        guessed.trim().to_string()
    }
}

pub fn bash_spawn_spec(
    config: TerminalConfig,
    bash_program: &str,
    folder: &str,
) -> Result<TerminalSpawnSpec, String> {
    let trimmed_folder = folder.trim();
    if trimmed_folder.is_empty() {
        return Err("Repository path is empty".into());
    }
    let trimmed_bash = bash_program.trim();
    if trimmed_bash.is_empty() {
        return Err("Bash executable could not be resolved".into());
    }
    let custom = config.custom_terminal.trim();
    if custom.is_empty() {
        return Ok(default_bash_spec(trimmed_bash, trimmed_folder, config.wt_available));
    }
    Ok(custom_terminal_bash_spec(custom, trimmed_bash, trimmed_folder))
}

fn default_bash_spec(bash: &str, folder: &str, wt_available: bool) -> TerminalSpawnSpec {
    if wt_available {
        return TerminalSpawnSpec {
            program: "wt".into(),
            args: vec!["-d".into(), folder.into(), bash.into()],
            cwd: None,
        };
    }
    TerminalSpawnSpec {
        program: bash.into(),
        args: Vec::new(),
        cwd: Some(folder.into()),
    }
}

fn custom_terminal_bash_spec(custom: &str, bash: &str, folder: &str) -> TerminalSpawnSpec {
    let path = Path::new(custom);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.eq_ignore_ascii_case("wt") {
        return TerminalSpawnSpec {
            program: custom.into(),
            args: vec!["-d".into(), folder.into(), bash.into()],
            cwd: None,
        };
    }
    if stem.eq_ignore_ascii_case("powershell") || stem.eq_ignore_ascii_case("pwsh") {
        return TerminalSpawnSpec {
            program: custom.into(),
            args: vec!["-NoExit".into(), "-Command".into(), format!("& '{}'", bash)],
            cwd: Some(folder.into()),
        };
    }
    if stem.eq_ignore_ascii_case("bash") || stem.eq_ignore_ascii_case("git-bash") {
        return TerminalSpawnSpec {
            program: custom.into(),
            args: Vec::new(),
            cwd: Some(folder.into()),
        };
    }
    TerminalSpawnSpec {
        program: custom.into(),
        args: vec!["-e".into(), bash.into()],
        cwd: Some(folder.into()),
    }
}

pub fn spawn_bash(custom_terminal: &str, bash_program: &str, folder: &str) -> Result<(), String> {
    let config = TerminalConfig {
        custom_terminal,
        wt_available: is_windows_terminal_available(),
    };
    let spec = bash_spawn_spec(config, bash_program, folder)?;
    spawn_spec(&spec)
}

pub fn env_guessed_bash_path() -> String {
    #[cfg(windows)]
    {
        for candidate in windows_bash_candidates() {
            if candidate.exists() {
                return candidate.to_string_lossy().into_owned();
            }
        }
        String::new()
    }
    #[cfg(not(windows))]
    {
        for candidate in ["/bin/bash", "/usr/bin/bash", "/usr/local/bin/bash"] {
            if Path::new(candidate).exists() {
                return candidate.to_string();
            }
        }
        "bash".to_string()
    }
}

#[cfg(windows)]
fn windows_bash_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(git) = find_program_in_path("git.exe").or_else(|| find_program_in_path("git")) {
        if let Some(parent) = git.parent() {
            if let Some(root) = parent.parent() {
                candidates.push(root.join("bin").join("bash.exe"));
                candidates.push(root.join("usr").join("bin").join("bash.exe"));
            }
            candidates.push(parent.join("bin").join("bash.exe"));
            candidates.push(parent.join("usr").join("bin").join("bash.exe"));
        }
    }
    let prog_files = std::env::var("ProgramFiles").unwrap_or_default();
    let prog_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    for base in [&prog_files, &local, &prog_x86] {
        if !base.trim().is_empty() {
            candidates.push(PathBuf::from(base).join("Git").join("bin").join("bash.exe"));
            candidates.push(PathBuf::from(base).join("Git").join("usr").join("bin").join("bash.exe"));
            candidates.push(PathBuf::from(base).join("Programs").join("Git").join("bin").join("bash.exe"));
        }
    }
    if let Some(bash) = find_program_in_path("bash.exe") {
        candidates.push(bash);
    }
    candidates
}

fn find_program_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[tauri::command(async)]
pub fn set_bash_path(
    app: AppHandle,
    bash_path: String,
) -> Result<super::prefs::UiPreferences, String> {
    PrefsStore::from_app(&app)?.set_bash_path(bash_path)
}

#[tauri::command(async)]
pub fn open_in_bash(app: AppHandle, path: String) -> Result<(), String> {
    let prefs = PrefsStore::from_app(&app)?.load()?;
    let guessed = env_guessed_bash_path();
    let program = resolve_bash_program(&prefs.bash_path, &guessed);
    if program.trim().is_empty() {
        return Err("Bash executable could not be resolved".into());
    }
    spawn_bash(&prefs.terminal_path, &program, &path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_saved_or_guessed_bash() {
        assert_eq!(
            resolve_bash_program("C:\\tools\\bash.exe", "C:\\Git\\bin\\bash.exe"),
            "C:\\tools\\bash.exe"
        );
        assert_eq!(
            resolve_bash_program("  ", "C:\\Git\\bin\\bash.exe"),
            "C:\\Git\\bin\\bash.exe"
        );
    }

    #[test]
    fn default_terminal_with_wt_uses_wt_and_bash() {
        let config = TerminalConfig {
            custom_terminal: "",
            wt_available: true,
        };
        let spec = bash_spawn_spec(config, "C:\\Git\\bin\\bash.exe", "C:\\repo").unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "wt".into(),
                args: vec!["-d".into(), "C:\\repo".into(), "C:\\Git\\bin\\bash.exe".into()],
                cwd: None,
            }
        );
    }

    #[test]
    fn default_terminal_without_wt_spawns_bash_directly() {
        let config = TerminalConfig {
            custom_terminal: "",
            wt_available: false,
        };
        let spec = bash_spawn_spec(config, "C:\\Git\\bin\\bash.exe", "C:\\repo").unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "C:\\Git\\bin\\bash.exe".into(),
                args: vec![],
                cwd: Some("C:\\repo".into()),
            }
        );
    }

    #[test]
    fn custom_wt_uses_wt_spec() {
        let config = TerminalConfig {
            custom_terminal: "C:\\Windows\\wt.exe",
            wt_available: false,
        };
        let spec = bash_spawn_spec(config, "C:\\Git\\bin\\bash.exe", "C:\\repo").unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "C:\\Windows\\wt.exe".into(),
                args: vec!["-d".into(), "C:\\repo".into(), "C:\\Git\\bin\\bash.exe".into()],
                cwd: None,
            }
        );
    }

    #[test]
    fn custom_other_terminal_uses_dash_e() {
        let config = TerminalConfig {
            custom_terminal: "alacritty",
            wt_available: true,
        };
        let spec = bash_spawn_spec(config, "C:\\Git\\bin\\bash.exe", "C:\\repo").unwrap();
        assert_eq!(
            spec,
            TerminalSpawnSpec {
                program: "alacritty".into(),
                args: vec!["-e".into(), "C:\\Git\\bin\\bash.exe".into()],
                cwd: Some("C:\\repo".into()),
            }
        );
    }

    #[test]
    fn empty_folder_or_bash_returns_error() {
        let config = TerminalConfig {
            custom_terminal: "",
            wt_available: true,
        };
        assert!(bash_spawn_spec(config, "C:\\Git\\bin\\bash.exe", "  ").is_err());
        assert!(bash_spawn_spec(config, "  ", "C:\\repo").is_err());
    }
}
