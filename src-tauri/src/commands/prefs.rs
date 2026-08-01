use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiPreferences {
    pub skip_review: bool,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self { skip_review: false }
    }
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
        let prefs = UiPreferences { skip_review };
        self.save(&prefs)?;
        Ok(prefs)
    }

    fn save(&self, prefs: &UiPreferences) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(prefs).map_err(|error| error.to_string())?;
        fs::write(&self.path, raw).map_err(|error| error.to_string())
    }
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
    fn corrupt_file_defaults_to_review() {
        let (_dir, store) = live_store();
        fs::write(&store.path, "{not-json").unwrap();
        assert_eq!(store.load().unwrap(), UiPreferences::default());
    }
}
