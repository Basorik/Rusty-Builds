use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use specta::Type;

/// A gem item from Gems.json — metadata about the gem, not the skill mechanics.
/// Skill mechanics (levels, stats, support matching) live on GrantedEffect in data/skills.rs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GemItem {
    pub name: String,
    #[serde(default)]
    pub base_type_name: Option<String>,
    pub game_id: String,
    pub granted_effect_id: String,
    #[serde(default)]
    pub secondary_granted_effect_id: Option<String>,
    pub natural_max_level: u32,
    pub req_str: u32,
    pub req_dex: u32,
    pub req_int: u32,
    pub tag_string: String,
    pub tags: FxHashMap<String, bool>,
    #[serde(default)]
    pub variant_id: Option<String>,
    #[serde(default, rename = "VaalGem")]
    pub vaal_gem: bool,
    #[serde(default)]
    pub secondary_effect_name: Option<String>,
}

/// Gem color based on attribute tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum GemColor {
    Red,
    Green,
    Blue,
    White,
}

/// Lightweight summary for the frontend gem selector dropdown.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GemSummary {
    pub id: String,
    pub name: String,
    pub tag_string: String,
    pub is_support: bool,
    pub color: GemColor,
    pub description: Option<String>,
}

impl GemItem {
    /// Returns the gem color based on attribute tags.
    /// Red = strength, Green = dexterity, Blue = intelligence, White = none/multi.
    pub fn gem_color(&self) -> GemColor {
        let s = self.tags.get("strength").copied().unwrap_or(false);
        let d = self.tags.get("dexterity").copied().unwrap_or(false);
        let i = self.tags.get("intelligence").copied().unwrap_or(false);
        if s && !d && !i {
            GemColor::Red
        } else if d && !s && !i {
            GemColor::Green
        } else if i && !s && !d {
            GemColor::Blue
        } else {
            GemColor::White
        }
    }
}
