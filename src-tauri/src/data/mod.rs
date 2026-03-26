pub mod bases;
pub mod gems;
pub mod mods;
pub mod skills;
pub mod stat_id;
pub mod tree;
pub mod uniques;

pub use stat_id::StatId;
pub use tree::{ PassiveTree, PassiveNode};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub u32);

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait DataLoader: Sized {
    fn load_from_file(path: &Path) -> Result<Self, DataError>;
    fn load_from_json(json: &str) -> Result<Self, DataError>;
}

pub struct GameData {
    pub tree: PassiveTree,
    pub source_names: Vec<String>,
}

impl GameData {
    pub fn load_from_json(tree_json: &str) -> Result<Self, DataError> {
        let tree = PassiveTree::load_from_json(tree_json)?;
        Ok(GameData {
            tree,
            source_names: Vec::new(),
        })
    }

    pub fn intern_source(&mut self, name: &str) -> SourceId {
        if let Some(pos) = self.source_names.iter().position(|s| s == name) {
            return SourceId(pos as u32);
        }
        let id = SourceId(self.source_names.len() as u32);
        self.source_names.push(name.to_owned());
        id
    }
}
