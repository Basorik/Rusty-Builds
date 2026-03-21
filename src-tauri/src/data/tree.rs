use crate::data::{DataError, DataLoader};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveTree {
    pub nodes: FxHashMap<String, PassiveNode>,
    pub groups: FxHashMap<String, PassiveGroup>,
    pub classes: Vec<ClassData>,
    #[serde(rename = "alternate_ascendancies")]
    pub bloodlines: Vec<BloodlineData>,
    pub constants: Option<TreeConstants>,
    pub points: Option<TreePoints>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeConstants {
    #[serde(rename = "skillsPerOrbit")]
    pub skills_per_orbit: Option<Vec<u32>>,
    #[serde(rename = "orbitRadii")]
    pub orbit_radii: Option<Vec<u32>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreePoints {
    #[serde(rename = "totalPoints")]
    pub total_points: u32,
    #[serde(rename = "ascendancyPoints")]
    pub ascendancy_points: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveNode {
    #[serde(rename = "skill", default)]
    pub id: Option<u32>,
    pub name: Option<String>,
    #[serde(rename = "ascendancyName")]
    pub ascendancy_name: Option<String>,
    #[serde(default)]
    pub stats: Vec<String>,
    #[serde(rename = "isKeystone", default)]
    pub is_keystone: bool,
    #[serde(rename = "isNotable", default)]
    pub is_notable: bool,
    #[serde(rename = "isMastery", default)]
    pub is_mastery: bool,
    #[serde(rename = "isJewelSocket", default)]
    pub is_jewel_socket: bool,
    #[serde(rename = "isAscendancyStart", default)]
    pub is_ascendancy_start: bool,
    #[serde(rename = "classStartIndex")]
    pub class_start_index: Option<u32>,
    #[serde(rename = "grantedStrength", default)]
    pub granted_strength: u32,
    #[serde(rename = "grantedDexterity", default)]
    pub granted_dexterity: u32,
    #[serde(rename = "grantedIntelligence", default)]
    pub granted_intelligence: u32,
    #[serde(rename = "grantedPassivePoints", default)]
    pub granted_passive_points: u32,
    #[serde(rename = "masteryEffects", default)]
    pub mastery_effects: Vec<MasteryEffect>,
    #[serde(rename = "out", default)]
    pub out_connections: Vec<String>,
    #[serde(rename = "in", default)]
    pub in_connections: Vec<String>,
    pub group: Option<u32>,
    pub orbit: Option<u32>,
    #[serde(rename = "orbitIndex")]
    pub orbit_index: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MasteryEffect {
    pub effect: u32,
    pub stats: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PassiveGroup {
    #[serde(rename = "x")]
    pub x_pos: f64,
    #[serde(rename = "y")]
    pub y_pos: f64,
    pub orbits: Vec<u32>,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassData {
    pub name: String,
    pub base_str: u32,
    pub base_dex: u32,
    pub base_int: u32,
    pub ascendancies: Vec<AscendancyData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AscendancyData {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BloodlineData {
    pub id: String,
    pub name: String,
}

impl DataLoader for PassiveTree {
    fn load_from_file(path: &Path) -> Result<Self, DataError> {
        let contents = std::fs::read_to_string(path)?;
        Self::load_from_json(&contents)
    }

    fn load_from_json(json: &str) -> Result<Self, DataError> {
        Ok(serde_json::from_str(json)?)
    }
}

impl PassiveTree {
    /// Look up a node by its numeric skill ID.
    pub fn get_node(&self, skill_id: u32) -> Option<&PassiveNode> {
        self.nodes.get(&skill_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TREE_JSON: &str = include_str!("../../data/tree/3.27.0g/data.json");

    #[test]
    fn test_load_tree() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        assert!(!tree.nodes.is_empty(), "Tree should have nodes");
        assert!(!tree.classes.is_empty(), "Tree should have classes");
        assert!(
            !tree.bloodlines.is_empty(),
            "Tree should have bloodline data"
        );
    }
}
