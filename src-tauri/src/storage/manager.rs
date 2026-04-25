use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;
use uuid::Uuid;

use crate::data::Class;
use crate::storage::builds::SavedBuildData;

pub struct StorageManager {
    builds_dir: PathBuf,
}

impl StorageManager {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, StorageError> {
        let builds_dir = app
            .path()
            .app_data_dir()
            .map_err(|_| StorageError::PathResolution)?
            .join("builds");
        std::fs::create_dir_all(&builds_dir)?;
        Ok(Self { builds_dir })
    }
    pub fn list_builds(&self) -> Result<Vec<BuildSummary>, StorageError> {
        let mut summaries = Vec::new();

        for entry in std::fs::read_dir(&self.builds_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only process .json files
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            // Extract the UUID from the filename (strip .json)
            let id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(stem) => stem.to_string(),
                None => continue,
            };

            // Read and deserialize the build file
            let contents = std::fs::read_to_string(&path)?;
            let data: SavedBuildData = serde_json::from_str(&contents)?;

            // Get last-modified from file metadata as seconds since epoch
            let last_modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| {
                    let secs = t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
                    Some(secs.to_string())
                })
                .unwrap_or_default();

            // Format class display name (e.g. "Juggernaut" or "Marauder")
            let class = format_class_name(&data.class);

            let node_count = data.selected_nodes.selected_node_ids.len() as u32;

            summaries.push(BuildSummary {
                id,
                name: data.name,
                class,
                level: data.level,
                node_count,
                last_modified,
            });
        }

        // Most recently modified first
        summaries.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        Ok(summaries)
    }
    pub fn save_build(&self, data: &SavedBuildData) -> Result<String, StorageError> {
        let id = Uuid::new_v4().to_string();
        let path = self.builds_dir.join(format!("{}.json", id));
        let json = serde_json::to_string_pretty(data)?;
        let mut tmp = tempfile::NamedTempFile::new_in(&self.builds_dir)?;
        tmp.write_all(json.as_bytes())?;
        tmp.persist(&path)?;
        Ok(id)
    }
    pub fn load_build(&self, id: &str) -> Result<SavedBuildData, StorageError> {
        validate_build_id(id)?;
        let path = self.builds_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(StorageError::NotFound(id.to_string()));
        }
        let contents = std::fs::read_to_string(&path)?;
        let data: SavedBuildData = serde_json::from_str(&contents)?;
        Ok(data)
    }

    pub fn delete_build(&self, id: &str) -> Result<(), StorageError> {
        validate_build_id(id)?;
        let path = self.builds_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(StorageError::NotFound(id.to_string()));
        }
        std::fs::remove_file(&path)?;
        Ok(())
    }

    pub fn rename_build(&self, id: &str, new_name: &str) -> Result<(), StorageError> {
        validate_build_id(id)?;
        let path = self.builds_dir.join(format!("{}.json", id));
        if !path.exists() {
            return Err(StorageError::NotFound(id.to_string()));
        }
        let contents = std::fs::read_to_string(&path)?;
        let mut data: SavedBuildData = serde_json::from_str(&contents)?;
        data.name = new_name.to_string();
        let json = serde_json::to_string_pretty(&data)?;
        let mut tmp = tempfile::NamedTempFile::new_in(&self.builds_dir)?;
        tmp.write_all(json.as_bytes())?;
        tmp.persist(&path)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Type)]
pub struct BuildSummary {
    pub id: String, // UUID filename without .json
    pub name: String,
    pub class: String, // Display name: "Marauder", "Juggernaut", etc.
    pub level: u32,
    pub node_count: u32,
    pub last_modified: String, // ISO 8601 timestamp
}

/// Validates that a build ID looks like a UUID (hex + hyphens only).
/// Prevents path traversal attacks from user-supplied IDs.
fn validate_build_id(id: &str) -> Result<(), StorageError> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err(StorageError::InvalidId(id.to_string()));
    }
    Ok(())
}

/// Produces a human-readable class name. If an ascendancy is selected, returns
/// the ascendancy name (e.g. "Juggernaut"); otherwise returns the base class
/// (e.g. "Marauder").
fn format_class_name(class: &Class) -> String {
    match class {
        Class::Marauder(Some(a)) => format!("{a:?}"),
        Class::Marauder(None) => "Marauder".into(),
        Class::Ranger(Some(a)) => format!("{a:?}"),
        Class::Ranger(None) => "Ranger".into(),
        Class::Witch(Some(a)) => format!("{a:?}"),
        Class::Witch(None) => "Witch".into(),
        Class::Duelist(Some(a)) => format!("{a:?}"),
        Class::Duelist(None) => "Duelist".into(),
        Class::Templar(Some(a)) => format!("{a:?}"),
        Class::Templar(None) => "Templar".into(),
        Class::Shadow(Some(a)) => format!("{a:?}"),
        Class::Shadow(None) => "Shadow".into(),
        Class::Scion(Some(a)) => format!("{a:?}"),
        Class::Scion(None) => "Scion".into(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to resolve app data path")]
    PathResolution,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Build not found: {0}")]
    NotFound(String),
    #[error("Invalid build ID: {0}")]
    InvalidId(String),
    #[error("File Persist error: {0}")]
    Persist(#[from] tempfile::PersistError),
}
