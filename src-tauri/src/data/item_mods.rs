use serde::{Deserialize, Deserializer};

pub const MOD_DOMAINS: &[&str] = &[
    "item",
    "crafted",
    "flask",
    "abyss_jewel",
    "unveiled",
    "tincture",
    "misc",
];

#[derive(Debug, Deserialize)]
pub struct RePoEMod {
    pub domain: String,
    pub generation_type: String, // "prefix", "suffix", "unique", "corrupted", etc.
    pub groups: Vec<String>,     // Mutual exclusion groups
    pub implicit_tags: Vec<String>,
    pub is_essence_only: bool,
    pub name: String,
    pub required_level: u32,
    pub spawn_weights: Vec<SpawnWeight>,
    pub stats: Vec<ModStat>, // Raw stat IDs + ranges
    #[serde(default)]
    pub adds_tags: Vec<String>,
    #[serde(default)]
    pub grants_effects: Vec<serde_json::Value>, // Complex; defer full parsing
}

#[derive(Debug, Deserialize)]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Debug, Deserialize)]
pub struct ModStat {
    pub id: String,
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize)]
pub struct StatMeta {
    pub is_local: bool,
}
