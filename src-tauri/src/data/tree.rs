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
