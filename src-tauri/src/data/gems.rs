use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer, Serialize};
use smallvec::SmallVec;
use specta::Type;

use crate::modifier::stat_table::StatDef;

/// A gem stat slot with its stat_table definitions pre-resolved at load time.
/// Eliminates runtime `stat_table` lookups during calculation.
#[derive(Debug, Clone)]
pub struct ResolvedGemStat {
    /// Pre-resolved StatDef entries from stat_table (after stat_conversions).
    pub defs: SmallVec<[StatDef; 1]>,
}

/// Deserializes a Vec that may be either absent or explicitly `null` in JSON.
fn null_as_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Clone, Deserialize)]
pub struct RePoEGem {
    pub active_skill: Option<ActiveSkill>,
    pub base_item: Option<GemItem>,
    pub cast_time: Option<u32>,
    pub color: GemColor,
    pub display_name: Option<String>,
    pub is_support: bool,
    pub per_level: FxHashMap<u32, GemLevel>,
    pub stat_translation_file: Option<String>,
    #[serde(rename = "static")]
    pub static_data: GemStaticProps,
    #[serde(deserialize_with = "null_as_empty_vec", default)]
    pub tags: Vec<String>,
    pub support_gem: Option<SupportSkill>,
    pub secondary_granted_effect: Option<serde_json::Value>,
    /// Pre-resolved stat definitions, positionally aligned with `static_data.stats`.
    /// Populated by `GameData::pre_resolve_gems()` after loading.
    #[serde(skip)]
    pub resolved_stats: Vec<Option<ResolvedGemStat>>,
    /// Pre-resolved quality stat definitions.
    /// Each entry is (stat_id_string, ResolvedGemStat) for quality_stats.
    #[serde(skip)]
    pub resolved_quality_stats: Vec<(String, ResolvedGemStat)>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct ActiveSkill {
    pub display_name: String,
    pub id: String,
    pub is_manually_casted: bool,
    pub is_skill_totem: bool,
    #[serde(default)]
    pub stat_conversions: FxHashMap<String, String>,
    #[serde(deserialize_with = "null_as_empty_vec", default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub weapon_restrictions: Vec<String>,
    pub skill_totem_life_multiplier: Option<f64>,
    pub minion_types: Option<Vec<String>>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GemItem {
    pub display_name: String,
    pub id: String,
    pub max_level: u32,
    pub release_state: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GemLevel {
    pub costs: Option<FxHashMap<String, Option<f64>>>,
    pub required_level: Option<u32>,
    #[serde(deserialize_with = "null_as_empty_vec", default)]
    pub stats: Vec<Option<GemStat>>,
    pub damage_effectiveness: Option<i32>,
    pub cooldown: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub stored_uses: Option<u32>,
    pub attack_speed_multiplier: Option<f64>,
    pub damage_multiplier: Option<f64>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GemStat {
    pub value: Option<f64>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub stat_type: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GemStaticProps {
    pub crit_chance: Option<u32>,
    pub damage_effectiveness: Option<i32>,
    #[serde(default)]
    pub quality_stats: Vec<GemQualityStat>,
    #[serde(deserialize_with = "null_as_empty_vec", default)]
    pub stats: Vec<Option<GemStat>>,
    pub cooldown: Option<u32>,
    pub stored_uses: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub attack_speed_multiplier: Option<f64>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct GemQualityStat {
    pub stats: FxHashMap<String, f64>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SupportSkill {
    pub allowed_types: Option<Vec<String>>,
    pub excluded_types: Option<Vec<String>>,
    pub letter: String,
    pub supports_gems_only: bool,
    pub added_types: Option<Vec<String>>,
    pub added_minion_types: Option<Vec<String>>,
}

/// Gem color. RePoE gems.json uses single-letter codes ("r", "g", "b").
/// GemSummary IPC serializes as full lowercase words ("red", "green", "blue", "white").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum GemColor {
    #[serde(alias = "r")]
    Red,
    #[serde(alias = "g")]
    Green,
    #[serde(alias = "b")]
    Blue,
    #[serde(alias = "w")]
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

pub fn compute_gem_stats(gem: &RePoEGem, level: u32, quality: u32) -> Vec<(String, f64)> {
    let mut result = Vec::new();

    let level_data = match gem.per_level.get(&level) {
        Some(ld) => ld,
        None => return result,
    };

    // Per-level stats — positionally aligned with static.stats
    for (i, static_slot) in gem.static_data.stats.iter().enumerate() {
        let Some(stat_def) = static_slot.as_ref() else {
            continue;
        };
        let Some(stat_id) = stat_def.id.as_ref() else {
            continue;
        };

        if stat_def.stat_type.as_deref() == Some("implicit") {
            continue;
        }

        let value = level_data
            .stats
            .get(i)
            .and_then(|s| s.as_ref())
            .and_then(|s| s.value)
            .or(stat_def.value)
            .unwrap_or(0.0);

        result.push((stat_id.clone(), value));
    }

    // Quality stats — divide by 1000, multiply by quality percentage
    if quality > 0 {
        for qs in &gem.static_data.quality_stats {
            for (stat_id, raw_value) in &qs.stats {
                let value = (raw_value / 1000.0) * quality as f64;
                result.push((stat_id.clone(), value));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modifier::stat_table::stat_table;

    fn load_test_gems() -> FxHashMap<String, RePoEGem> {
        let gems_json = std::fs::read_to_string("data/repoe/gems.json")
            .expect("gems.json must exist for tests — run `bun run tool:fetch-data`");
        serde_json::from_str(&gems_json).expect("Failed to parse gems.json")
    }

    #[test]
    fn test_gem_stat_coverage_above_threshold() {
        let gems = load_test_gems();
        let table = stat_table();

        let mut total = 0u32;
        let mut resolved = 0u32;

        for gem in gems.values() {
            // Apply stat_conversions for this gem
            let conversions = gem.active_skill.as_ref().map(|a| &a.stat_conversions);

            for stat_slot in &gem.static_data.stats {
                let Some(stat) = stat_slot.as_ref() else {
                    continue;
                };
                let Some(id) = stat.id.as_ref() else { continue };
                if stat.stat_type.as_deref() == Some("implicit") {
                    continue;
                }

                total += 1;

                let resolved_id = conversions
                    .and_then(|c| c.get(id.as_str()))
                    .map(|s| s.as_str())
                    .unwrap_or(id.as_str());

                if table.contains_key(resolved_id) {
                    resolved += 1;
                }
            }
        }

        let pct = (resolved as f64 / total as f64) * 100.0;
        eprintln!("Gem stat coverage: {resolved}/{total} ({pct:.1}%)");
        // Minimum threshold: we resolve at least 30% of gem stat slots.
        // Current: ~38% (583+SSM entries). Will increase as overrides are added.
        assert!(
            pct >= 30.0,
            "Gem stat coverage dropped below 30%: {pct:.1}%"
        );
    }
}
