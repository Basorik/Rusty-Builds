pub mod bases;
pub mod gems;
pub mod mods;
pub mod skills;
pub mod stat_id;
pub mod tree;
pub mod uniques;

use rustc_hash::FxHashMap;
pub use stat_id::StatId;
use std::path::{Path, PathBuf};
use thiserror::Error;
pub use tree::{PassiveNode, PassiveTree};

use crate::data::gems::GemItem;
use crate::data::skills::GrantedEffect;
use crate::DEFAULT_TREE_VERSION;
use log::info;

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
    pub gems: FxHashMap<String, GemItem>,
    pub skills: FxHashMap<String, GrantedEffect>,

    pub source_names: Vec<String>,
}

impl GameData {
    pub fn load_from_dir(resource_dir: PathBuf) -> Result<Self, DataError> {
        let tree_path = resource_dir
            .join("data/tree")
            .join(DEFAULT_TREE_VERSION)
            .join("data.json");
        let tree_json =
            std::fs::read_to_string(&tree_path).expect("Failed to read default tree data");

        let gems_path = resource_dir.join("data/pob/Gems.json");
        let gems_json = std::fs::read_to_string(&gems_path).expect("Failed to read Gems.json");

        let skill_paths = resource_dir.join("data/pob/Skills");
        let mut skills_json = Vec::<String>::new();
        for entry in std::fs::read_dir(skill_paths)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                skills_json.push(std::fs::read_to_string(path)?)
            }
        }

        let game_data = GameData::load_all_json(&tree_json, &gems_json, &skills_json)
            .expect("Failed to load game data");
        return Ok(game_data);
    }
    pub fn load_all_json(
        tree_json: &str,
        gems_json: &str,
        skills_json: &Vec<String>,
    ) -> Result<Self, DataError> {
        let tree = PassiveTree::load_from_json(tree_json)?;
        let gems = serde_json::from_str::<FxHashMap<String, GemItem>>(gems_json)?;
        let mut skills = FxHashMap::<String, GrantedEffect>::default();
        for json in skills_json {
            let chunk: FxHashMap<String, GrantedEffect> = serde_json::from_str(json)?;
            skills.extend(chunk);
        }
        Ok(GameData {
            tree,
            gems,
            skills,
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
