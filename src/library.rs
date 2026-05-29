use crate::domain::book::Book;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub book: Book,
    pub file_path: PathBuf,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Library {
    pub entries: HashMap<String, LibraryEntry>,
}

impl Library {
    pub fn get_app_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "FastSpeedReader") {
            let dir = proj_dirs.data_local_dir().to_path_buf();
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            dir
        } else {
            let dir = std::env::current_dir().unwrap_or_default().join(".fast-ereader");
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            dir
        }
    }

    pub fn get_library_file_path() -> PathBuf {
        Self::get_app_dir().join("library.json")
    }

    pub fn load() -> Self {
        let path = Self::get_library_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(lib) = serde_json::from_str(&content) {
                    return lib;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_library_file_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
