use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IdeChoice {
    #[default]
    Vscode,
    Cursor,
    #[serde(rename = "visual-studio")]
    VisualStudio,
    Rider,
    Custom { command: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProjectSettings {
    #[serde(default)]
    pub ide: IdeChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ProjectEntry {
    path: String,
    #[serde(flatten)]
    settings: ProjectSettings,
}

#[derive(Default, Serialize, Deserialize)]
struct ProjectSettingsFile {
    #[serde(default)]
    projects: Vec<ProjectEntry>,
}

pub struct ProjectSettingsStore {
    path: PathBuf,
}

impl ProjectSettingsStore {
    pub fn from_app(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        Ok(Self {
            path: dir.join("project-settings.json"),
        })
    }

    pub fn get(&self, path: &str) -> Result<ProjectSettings, String> {
        let file = self.load()?;
        Ok(file
            .projects
            .into_iter()
            .find(|entry| same_path(&entry.path, path))
            .map(|entry| entry.settings)
            .unwrap_or_default())
    }

    pub fn set_ide(&self, path: &str, ide: IdeChoice) -> Result<ProjectSettings, String> {
        let mut file = self.load()?;
        if let Some(entry) = file.projects.iter_mut().find(|entry| same_path(&entry.path, path)) {
            entry.settings.ide = ide;
        } else {
            file.projects.push(ProjectEntry {
                path: path.to_string(),
                settings: ProjectSettings { ide },
            });
        }
        self.save(&file)?;
        self.get(path)
    }

    fn load(&self) -> Result<ProjectSettingsFile, String> {
        if !self.path.exists() {
            return Ok(ProjectSettingsFile::default());
        }
        let raw = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        serde_json::from_str(&raw).or_else(|_| Ok(ProjectSettingsFile::default()))
    }

    fn save(&self, file: &ProjectSettingsFile) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(file).map_err(|error| error.to_string())?;
        fs::write(&self.path, raw).map_err(|error| error.to_string())
    }
}

fn same_path(left: &str, right: &str) -> bool {
    normalize(left) == normalize(right)
}

fn normalize(path: &str) -> String {
    let replaced = path.replace('/', "\\");
    if cfg!(windows) {
        return replaced.to_ascii_lowercase();
    }
    replaced
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live_store() -> (tempfile::TempDir, ProjectSettingsStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = ProjectSettingsStore {
            path: dir.path().join("project-settings.json"),
        };
        (dir, store)
    }

    #[test]
    fn missing_file_defaults_to_vscode() {
        let (_dir, store) = live_store();
        assert_eq!(store.get(r"C:\work\alpha").unwrap(), ProjectSettings::default());
    }

    #[test]
    fn set_ide_round_trips() {
        let (_dir, store) = live_store();
        let saved = store
            .set_ide(r"C:\work\alpha", IdeChoice::Cursor)
            .unwrap();
        assert_eq!(saved.ide, IdeChoice::Cursor);
        assert_eq!(store.get(r"C:\work\alpha").unwrap().ide, IdeChoice::Cursor);
    }

    #[test]
    fn set_ide_normalizes_paths() {
        let (_dir, store) = live_store();
        store.set_ide(r"C:\work\alpha", IdeChoice::Rider).unwrap();
        assert_eq!(store.get(r"c:/work/alpha").unwrap().ide, IdeChoice::Rider);
    }

    #[test]
    fn set_ide_merges_without_clobbering_other_projects() {
        let (_dir, store) = live_store();
        store.set_ide(r"C:\work\alpha", IdeChoice::Cursor).unwrap();
        store.set_ide(r"C:\work\beta", IdeChoice::Rider).unwrap();
        assert_eq!(store.get(r"C:\work\alpha").unwrap().ide, IdeChoice::Cursor);
        assert_eq!(store.get(r"C:\work\beta").unwrap().ide, IdeChoice::Rider);
    }

    #[test]
    fn corrupt_file_defaults_to_vscode() {
        let (_dir, store) = live_store();
        fs::write(&store.path, "{not-json").unwrap();
        assert_eq!(store.get(r"C:\work\alpha").unwrap(), ProjectSettings::default());
    }
}
