use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::bash::env_guessed_bash_path;
use super::codechart::env_guessed_codechart_path;
use super::terminal::default_terminal_name;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiPreferences {
    pub skip_review: bool,
    #[serde(default)]
    pub codechart_path: String,
    #[serde(default)]
    pub terminal_path: String,
    #[serde(default)]
    pub bash_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiPreferencesResponse {
    pub skip_review: bool,
    pub codechart_path: String,
    pub guessed_codechart_path: String,
    pub terminal_path: String,
    pub default_terminal_name: String,
    pub bash_path: String,
    pub guessed_bash_path: String,
}

pub struct PrefsStore {
    path: PathBuf,
}

impl PrefsStore {
    pub fn from_app(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        Ok(Self {
            path: dir.join("ui-preferences.json"),
        })
    }

    pub fn load(&self) -> Result<UiPreferences, String> {
        if !self.path.exists() {
            return Ok(UiPreferences::default());
        }
        let raw = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        serde_json::from_str(&raw).or_else(|_| Ok(UiPreferences::default()))
    }

    pub fn set_skip_review(&self, skip_review: bool) -> Result<UiPreferences, String> {
        let mut prefs = self.load()?;
        prefs.skip_review = skip_review;
        self.save(&prefs)?;
        Ok(prefs)
    }

    pub fn set_codechart_path(&self, codechart_path: String) -> Result<UiPreferences, String> {
        let mut prefs = self.load()?;
        prefs.codechart_path = codechart_path;
        self.save(&prefs)?;
        Ok(prefs)
    }

    pub fn set_terminal_path(&self, terminal_path: String) -> Result<UiPreferences, String> {
        let mut prefs = self.load()?;
        prefs.terminal_path = terminal_path;
        self.save(&prefs)?;
        Ok(prefs)
    }

    pub fn set_bash_path(&self, bash_path: String) -> Result<UiPreferences, String> {
        let mut prefs = self.load()?;
        prefs.bash_path = bash_path;
        self.save(&prefs)?;
        Ok(prefs)
    }

    fn save(&self, prefs: &UiPreferences) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(prefs).map_err(|error| error.to_string())?;
        fs::write(&self.path, raw).map_err(|error| error.to_string())
    }
}

#[tauri::command(async)]
pub fn get_ui_preferences(app: AppHandle) -> Result<UiPreferencesResponse, String> {
    let store = PrefsStore::from_app(&app)?;
    let prefs = store.load()?;
    let guessed = env_guessed_codechart_path();
    let default_terminal = default_terminal_name();
    let guessed_bash = env_guessed_bash_path();
    Ok(UiPreferencesResponse {
        skip_review: prefs.skip_review,
        codechart_path: prefs.codechart_path,
        guessed_codechart_path: guessed,
        terminal_path: prefs.terminal_path,
        default_terminal_name: default_terminal,
        bash_path: prefs.bash_path,
        guessed_bash_path: guessed_bash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_store() -> (tempfile::TempDir, PrefsStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = PrefsStore {
            path: dir.path().join("ui-preferences.json"),
        };
        (dir, store)
    }

    #[test]
    fn missing_file_defaults_to_review() {
        let (_dir, store) = live_store();
        assert_eq!(store.load().unwrap(), UiPreferences::default());
    }

    #[test]
    fn set_skip_review_round_trips() {
        let (_dir, store) = live_store();
        let saved = store.set_skip_review(true).unwrap();
        assert_eq!(saved.skip_review, true);
        assert_eq!(store.load().unwrap().skip_review, true);
        store.set_skip_review(false).unwrap();
        assert_eq!(store.load().unwrap().skip_review, false);
    }

    #[test]
    fn set_codechart_path_round_trips() {
        let (_dir, store) = live_store();
        let saved = store
            .set_codechart_path(r"C:\tools\codechart.exe".into())
            .unwrap();
        assert_eq!(saved.codechart_path, r"C:\tools\codechart.exe");
        assert_eq!(
            store.load().unwrap().codechart_path,
            r"C:\tools\codechart.exe"
        );
    }

    #[test]
    fn set_terminal_path_round_trips() {
        let (_dir, store) = live_store();
        let saved = store
            .set_terminal_path(r"C:\tools\alacritty.exe".into())
            .unwrap();
        assert_eq!(saved.terminal_path, r"C:\tools\alacritty.exe");
        assert_eq!(
            store.load().unwrap().terminal_path,
            r"C:\tools\alacritty.exe"
        );
    }

    #[test]
    fn set_bash_path_round_trips() {
        let (_dir, store) = live_store();
        let saved = store
            .set_bash_path(r"C:\tools\bash.exe".into())
            .unwrap();
        assert_eq!(saved.bash_path, r"C:\tools\bash.exe");
        assert_eq!(store.load().unwrap().bash_path, r"C:\tools\bash.exe");
    }

    #[test]
    fn set_skip_review_preserves_other_prefs() {
        let (_dir, store) = live_store();
        store
            .set_codechart_path(r"C:\tools\codechart.exe".into())
            .unwrap();
        store
            .set_terminal_path(r"C:\tools\alacritty.exe".into())
            .unwrap();
        store
            .set_bash_path(r"C:\tools\bash.exe".into())
            .unwrap();
        store.set_skip_review(true).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.codechart_path, r"C:\tools\codechart.exe");
        assert_eq!(loaded.terminal_path, r"C:\tools\alacritty.exe");
        assert_eq!(loaded.bash_path, r"C:\tools\bash.exe");
    }

    #[test]
    fn corrupt_file_defaults_to_review() {
        let (_dir, store) = live_store();
        fs::write(&store.path, "{not-json").unwrap();
        assert_eq!(store.load().unwrap(), UiPreferences::default());
    }
}
