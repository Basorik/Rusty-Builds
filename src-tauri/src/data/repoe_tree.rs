//! RePoE passive tree types.
//!
//! Loads from `data/repoe/passive_skill_trees/Default.json`.
//! This tree contains raw stat IDs and integer values — used by the
//! calculation engine via `modifier::parser::resolve()`. The GGG tree (`data/tree/`)
//! continues to serve frontend rendering and display text.

use crate::data::{DataError, DataLoader};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::path::Path;

/// Top-level structure of a RePoE passive skill tree file.
/// JSON keys are stringified integers; serde_json parses them directly as `u32`.
#[derive(Debug, Deserialize)]
pub struct RePoETree {
    pub passives: FxHashMap<u32, RePoEPassive>,
}

/// A single passive node from the RePoE tree.
/// Stats are raw internal IDs mapped to fixed integer values.
#[derive(Debug, Clone, Deserialize)]
pub struct RePoEPassive {
    pub hash: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub stats: FxHashMap<String, i64>,
    #[serde(default)]
    pub is_keystone: bool,
    #[serde(default)]
    pub is_notable: bool,
    #[serde(default)]
    pub is_jewel_socket: bool,
    #[serde(default)]
    pub is_ascendancy_starting_node: bool,
    pub ascendancy: Option<String>,
    #[serde(default)]
    pub skill_points: u32,
}

impl DataLoader for RePoETree {
    fn load_from_file(path: &Path) -> Result<Self, DataError> {
        let json = std::fs::read_to_string(path)?;
        Self::load_from_json(&json)
    }

    fn load_from_json(json: &str) -> Result<Self, DataError> {
        let tree: RePoETree = serde_json::from_str(json)?;
        Ok(tree)
    }
}

impl RePoETree {
    /// Look up a passive by its hash (node ID used by the frontend/GGG tree).
    pub fn get_passive(&self, hash: u32) -> Option<&RePoEPassive> {
        self.passives.get(&hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_default_tree() -> RePoETree {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/repoe/passive_skill_trees/Default.json");
        RePoETree::load_from_file(&path).expect("Failed to load RePoE Default tree")
    }

    #[test]
    fn test_load_and_count() {
        let tree = load_default_tree();
        assert!(
            tree.passives.len() > 2000,
            "Expected >2000 passives, got {}",
            tree.passives.len()
        );
    }

    #[test]
    fn test_node_127_stats() {
        let tree = load_default_tree();
        let node = tree.get_passive(127).expect("Node 127 should exist");
        assert_eq!(node.name, "Life on Kill and Recoup");
        assert_eq!(node.stats.get("base_life_gained_on_enemy_death"), Some(&15));
        assert_eq!(
            node.stats.get("damage_taken_goes_to_life_over_4_seconds_%"),
            Some(&4)
        );
    }

    #[test]
    fn test_ascendancy_node() {
        let tree = load_default_tree();
        // Find any ascendancy node
        let asc_node = tree.passives.values().find(|p| p.ascendancy.is_some());
        assert!(
            asc_node.is_some(),
            "Should have at least one ascendancy node"
        );
    }
}
