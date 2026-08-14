use std::path::PathBuf;
use std::process::Command;

use tauri::AppHandle;

use super::prefs::{PrefsStore, UiPreferences};

pub fn guessed_codechart_path(local_app_data: &str, user_profile: &str) -> String {
    let base = if !local_app_data.trim().is_empty() {
        PathBuf::from(local_app_data)
    } else if !user_profile.trim().is_empty() {
        PathBuf::from(user_profile)
            .join("AppData")
            .join("Local")
    } else {
        return String::new();
    };
    base.join("codechart")
        .join("codechart.exe")
        .to_string_lossy()
        .into_owned()
}

pub fn resolve_codechart_program(saved: &str, guessed: &str) -> String {
    let trimmed = saved.trim();
    if trimmed.is_empty() {
        guessed.to_string()
    } else {
        trimmed.to_string()
    }
}

#[tauri::command(async)]
pub fn set_codechart_path(app: AppHandle, codechart_path: String) -> Result<UiPreferences, String> {
    PrefsStore::from_app(&app)?.set_codechart_path(codechart_path)
}

#[tauri::command(async)]
pub fn open_in_codechart(app: AppHandle, path: String) -> Result<(), String> {
    let store = PrefsStore::from_app(&app)?;
    let prefs = store.load()?;
    let guessed = env_guessed_codechart_path();
    let program = resolve_codechart_program(&prefs.codechart_path, &guessed);
    if program.trim().is_empty() {
        return Err("Codechart executable could not be resolved".into());
    }
    spawn_codechart(&program, &path)
}

pub fn env_guessed_codechart_path() -> String {
    let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
    let profile = std::env::var("USERPROFILE").unwrap_or_default();
    guessed_codechart_path(&local, &profile)
}

fn spawn_codechart(program: &str, folder: &str) -> Result<(), String> {
    let mut process = Command::new(program);
    process.arg(folder);
    hide_console(&mut process);
    process
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open Codechart: {error}"))
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
    fn guessed_path_uses_local_app_data() {
        assert_eq!(
            guessed_codechart_path(r"C:\Users\me\AppData\Local", ""),
            r"C:\Users\me\AppData\Local\codechart\codechart.exe"
        );
    }

    #[test]
    fn guessed_path_falls_back_to_user_profile() {
        assert_eq!(
            guessed_codechart_path("", r"C:\Users\me"),
            r"C:\Users\me\AppData\Local\codechart\codechart.exe"
        );
    }

    #[test]
    fn guessed_path_is_empty_when_env_is_missing() {
        assert_eq!(guessed_codechart_path("", ""), "");
        assert_eq!(guessed_codechart_path("   ", "  "), "");
    }

    #[test]
    fn saved_path_wins_when_non_empty() {
        assert_eq!(
            resolve_codechart_program(
                r"C:\tools\codechart.exe",
                r"C:\Users\me\AppData\Local\codechart\codechart.exe",
            ),
            r"C:\tools\codechart.exe"
        );
    }

    #[test]
    fn empty_saved_uses_guess() {
        assert_eq!(
            resolve_codechart_program(
                "   ",
                r"C:\Users\me\AppData\Local\codechart\codechart.exe",
            ),
            r"C:\Users\me\AppData\Local\codechart\codechart.exe"
        );
    }
}
