use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentRepository {
    pub path: String,
    pub name: String,
}

#[derive(Default, Serialize, Deserialize)]
struct RecentFile {
    paths: Vec<String>,
}

pub struct RecentStore {
    path: PathBuf,
}

impl RecentStore {
    pub fn from_app(app: &AppHandle) -> Result<Self, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        Ok(Self {
            path: dir.join("recent-repositories.json"),
        })
    }

    pub fn list(&self) -> Result<Vec<RecentRepository>, String> {
        Ok(self.load()?.paths.into_iter().map(entry_for).collect())
    }

    pub fn remember(&self, path: &str) -> Result<Vec<RecentRepository>, String> {
        let mut file = self.load()?;
        file.paths.retain(|existing| !same_path(existing, path));
        file.paths.insert(0, path.to_string());
        self.save(&file)?;
        Ok(file.paths.into_iter().map(entry_for).collect())
    }

    pub fn remove(&self, path: &str) -> Result<Vec<RecentRepository>, String> {
        let mut file = self.load()?;
        file.paths.retain(|existing| !same_path(existing, path));
        self.save(&file)?;
        Ok(file.paths.into_iter().map(entry_for).collect())
    }

    fn load(&self) -> Result<RecentFile, String> {
        if !self.path.exists() {
            return Ok(RecentFile::default());
        }
        let raw = fs::read_to_string(&self.path).map_err(|error| error.to_string())?;
        serde_json::from_str(&raw).or_else(|_| Ok(RecentFile::default()))
    }

    fn save(&self, file: &RecentFile) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(file).map_err(|error| error.to_string())?;
        fs::write(&self.path, raw).map_err(|error| error.to_string())
    }
}

fn entry_for(path: String) -> RecentRepository {
    let name = Path::new(&path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(&path)
        .to_string();
    RecentRepository { path, name }
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

    fn live_store() -> (tempfile::TempDir, RecentStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = RecentStore {
            path: dir.path().join("recent-repositories.json"),
        };
        (dir, store)
    }

    #[test]
    fn remember_promotes_and_dedupes() {
        let (_dir, store) = live_store();
        store.remember(r"C:\work\one").unwrap();
        store.remember(r"C:\work\two").unwrap();
        store.remember(r"c:/work/one").unwrap();

        let listed = store.list().unwrap();
        assert_eq!(
            listed,
            vec![
                entry_for(r"c:/work/one".into()),
                entry_for(r"C:\work\two".into()),
            ]
        );
    }

    #[test]
    fn remove_drops_a_path() {
        let (_dir, store) = live_store();
        store.remember(r"C:\work\one").unwrap();
        store.remember(r"C:\work\two").unwrap();
        let listed = store.remove(r"C:\work\one").unwrap();
        assert_eq!(listed, vec![entry_for(r"C:\work\two".into())]);
    }

    #[test]
    fn corrupt_file_starts_empty() {
        let (_dir, store) = live_store();
        fs::write(&store.path, "{not-json").unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
