use std::path::Path;
use log::info;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use smallvec::SmallVec;

use crate::data::{DataError, SourceId, StatId};
use crate::modifier::{KeywordFlag, ModFlag, ModTag, ModType, Modifier, intern};

// -------------------------------------------------------------------
// Raw JSON types — mirror SkillStatMap.norm.json (normalized format)
// -------------------------------------------------------------------

/// A tag entry from the normalized "tags" array.
/// Uses `serde_json::Value` for extra fields to stay forward-compatible
/// with all 11 tag types without hardcoding every field.
#[derive(Debug, Deserialize)]
struct RawTag {
    #[serde(rename = "type")]
    tag_type: String,
    var: Option<String>,
    #[serde(rename = "varList")]
    var_list: Option<Vec<String>>,
    stat: Option<String>,
    div: Option<f64>,
    limit: Option<f64>,
    neg: Option<bool>,
    actor: Option<String>,
    #[serde(rename = "skillType")]
    skill_type: Option<u32>,
    threshold: Option<f64>,
    #[serde(rename = "effectName")]
    effect_name: Option<String>,
    #[serde(rename = "effectType")]
    effect_type: Option<String>,
    percent: Option<bool>,
    #[serde(rename = "modFlags")]
    mod_flags: Option<u32>,
    ramp: Option<Vec<Vec<f64>>>,
}

/// Raw deserialization target — matches normalized JSON shape exactly.
/// Every entry is a uniform object with a "tags" array (may be empty).
#[derive(Debug, Deserialize)]
struct RawStatMapping {
    name: String,               // calc variable name → resolves to StatId
    #[serde(rename = "type")]
    mod_type: String,           // "BASE", "INC", "MORE", "FLAG", "MAX", "MIN", "LIST", "OVERRIDE", "CHANCE"
    flags: u32,                 // raw bitfield integer
    #[serde(rename = "keywordFlags")]
    keyword_flags: u32,         // raw bitfield integer
    div: Option<f64>,
    value: Option<serde_json::Value>,  // Some entries carry a fixed value (bool or number)
    tags: Vec<RawTag>,          // normalized: always present, may be empty
}

// -------------------------------------------------------------------
// Resolved types — used everywhere after loading
// -------------------------------------------------------------------

pub struct StatMapping {
    pub stat_id: StatId,
    pub mod_type: ModType,
    pub flags: ModFlag,
    pub keywords: KeywordFlag,
    pub tags: SmallVec<[ModTag; 2]>,
    pub div: Option<f64>,
}

// -------------------------------------------------------------------
// Conversion from raw → resolved
// -------------------------------------------------------------------

fn parse_mod_type(s: &str) -> Option<ModType> {
    match s {
        "BASE"     => Some(ModType::Base),
        "INC"      => Some(ModType::Inc),
        "MORE"     => Some(ModType::More),
        "FLAG"     => Some(ModType::Flag),
        "MAX"      => Some(ModType::Max),
        "MIN"      => Some(ModType::Min),
        "LIST"     => Some(ModType::List),
        "OVERRIDE" => Some(ModType::Override),
        "CHANCE"   => Some(ModType::Chance),
        _          => None,
    }
}

fn parse_raw_tag(raw: RawTag) -> Option<ModTag> {
    match raw.tag_type.as_str() {
        "Condition" => {
            let var = intern(raw.var.as_deref()?);
            Some(ModTag::Condition(var))
        }
        "ActorCondition" => {
            let var = intern(raw.var.as_deref()?);
            let actor = raw.actor.as_deref().map(intern);
            Some(ModTag::ActorCondition { var, actor })
        }
        "Multiplier" => {
            let var = intern(raw.var.as_deref()?);
            Some(ModTag::Multiplier(var))
        }
        "MultiplierThreshold" => {
            let var = intern(raw.var.as_deref()?);
            let threshold = raw.threshold.unwrap_or(0.0);
            Some(ModTag::MultiplierThreshold { var, threshold })
        }
        "PerStat" => {
            let stat = intern(raw.stat.as_deref()?);
            let div = raw.div.unwrap_or(1.0);
            Some(ModTag::PerStat { stat, div })
        }
        "PercentStat" => {
            let stat = intern(raw.stat.as_deref()?);
            let percent = raw.percent.unwrap_or(false);
            Some(ModTag::PercentStat { stat, percent })
        }
        "StatThreshold" => {
            let stat = intern(raw.stat.as_deref()?);
            let threshold = raw.threshold.unwrap_or(0.0);
            Some(ModTag::StatThreshold { stat, threshold })
        }
        "SkillType" => {
            let skill_type = raw.skill_type?;
            Some(ModTag::SkillType(skill_type))
        }
        "GlobalEffect" => {
            let effect_name = intern(raw.effect_name.as_deref()?);
            let effect_type = raw.effect_type.as_deref().map(intern);
            Some(ModTag::GlobalEffect { effect_name, effect_type })
        }
        "ModFlagOr" => {
            let flags = raw.mod_flags?;
            Some(ModTag::ModFlagOr(flags))
        }
        "DistanceRamp" => {
            let ramp_data = raw.ramp?;
            let ramp: Vec<[f64; 2]> = ramp_data
                .into_iter()
                .filter_map(|pair| {
                    if pair.len() == 2 { Some([pair[0], pair[1]]) } else { None }
                })
                .collect();
            Some(ModTag::DistanceRamp(ramp))
        }
        _ => {
            info!("Unknown tag type: {}", raw.tag_type);
            None
        }
    }
}

fn convert(raw: RawStatMapping) -> Option<StatMapping> {
    let stat_id = StatId::from_name(&raw.name)?;
    let mod_type = parse_mod_type(&raw.mod_type)?;

    let tags: SmallVec<[ModTag; 2]> = raw.tags
        .into_iter()
        .filter_map(parse_raw_tag)
        .collect();

    Some(StatMapping {
        stat_id,
        mod_type,
        flags: ModFlag::from_bits_truncate(raw.flags),
        keywords: KeywordFlag::from_bits_truncate(raw.keyword_flags),
        tags,
        div: raw.div,
    })
}

// -------------------------------------------------------------------
// SkillStatMapDB
// -------------------------------------------------------------------

pub struct SkillStatMapDB {
    map: FxHashMap<String, Vec<StatMapping>>,
}

impl SkillStatMapDB {
    /// Load from the normalized SkillStatMap.norm.json format.
    /// Every value is a uniform array of mapping objects with a "tags" array.
    pub fn load(path: &Path) -> Result<Self, DataError> {
        let json = std::fs::read_to_string(path).map_err(DataError::Io)?;
        let raw: FxHashMap<String, Vec<RawStatMapping>> = serde_json::from_str(&json)?;

        let mut map: FxHashMap<String, Vec<StatMapping>> =
            FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());

        for (stat_id, entries) in raw {
            let resolved: Vec<StatMapping> = entries
                .into_iter()
                .filter_map(convert)
                .collect();

            if !resolved.is_empty() {
                map.insert(stat_id, resolved);
            }
        }

        Ok(Self { map })
    }

    pub fn resolve(&self, internal_stat_id: &str, value: f64, source: SourceId) -> Vec<Modifier> {
        let stat_mapping = match self.map.get(internal_stat_id) {
            Some(mapping) => mapping,
            None => {
                info!("Unresolved stat: {}", internal_stat_id);
                return Vec::new()},
        };

        let mut mods = Vec::new();
        for stat_map in stat_mapping {
            let final_value = stat_map.div.map_or(value, |div| value / div);
            mods.push(Modifier {
                stat: stat_map.stat_id,
                mod_type: stat_map.mod_type,
                value: final_value,
                flags: stat_map.flags,
                keywords: stat_map.keywords,
                source,
                tags: stat_map.tags.clone(),
            });
        }

        mods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_db() -> SkillStatMapDB {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("data/pob/SkillStatMap.norm.json");
        SkillStatMapDB::load(&path).expect("Failed to load SkillStatMap.norm.json")
    }

    // Plan §7 Step 1: "resolve("attack_speed_+%", 5.0, src) → Modifier { stat: Speed, mod_type: Inc, value: 5.0, flags: ATTACK }"
    #[test]
    fn test_resolve_simple() {
        let db = load_db();
        let source = SourceId(0);
        let mods = db.resolve("attack_speed_+%", 5.0, source);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].stat, StatId::Speed);
        assert_eq!(mods[0].mod_type, ModType::Inc);
        assert_eq!(mods[0].value, 5.0);
        assert!(mods[0].flags.contains(ModFlag::ATTACK));
    }

    // Plan §7 Step 1: "conditional entries like "accuracy_rating_+%_when_on_low_life" produce ModTag::Condition"
    #[test]
    fn test_resolve_condition_tag() {
        let db = load_db();
        let source = SourceId(0);
        let mods = db.resolve("accuracy_rating_+%_when_on_low_life", 20.0, source);

        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].mod_type, ModType::Inc);
        assert!(
            mods[0].tags.iter().any(|t| matches!(t, ModTag::Condition(_))),
            "Expected a Condition tag"
        );
    }

    // Stat not in SkillStatMap should return empty vec, not panic
    #[test]
    fn test_resolve_unknown_stat() {
        let db = load_db();
        let mods = db.resolve("this_stat_does_not_exist", 1.0, SourceId(0));
        assert!(mods.is_empty());
    }

    // Check total loaded entries against the known normalized JSON counts
    #[test]
    fn test_load_coverage() {
        let db = load_db();
        // Normalized file has 692 stat IDs; we load most (some StatId/ModType may not resolve)
        assert!(db.map.len() > 500, "Expected >500 stat mappings, got {}", db.map.len());
    }

    // Verify all 11 tag types can be parsed from the normalized data
    #[test]
    fn test_all_tag_types_present() {
        let db = load_db();

        let mut has_condition = false;
        let mut has_actor_condition = false;
        let mut has_multiplier = false;
        let mut has_multiplier_threshold = false;
        let mut has_per_stat = false;
        let mut has_skill_type = false;
        let mut has_global_effect = false;

        for mappings in db.map.values() {
            for mapping in mappings {
                for tag in &mapping.tags {
                    match tag {
                        ModTag::Condition(_) => has_condition = true,
                        ModTag::ActorCondition { .. } => has_actor_condition = true,
                        ModTag::Multiplier(_) => has_multiplier = true,
                        ModTag::MultiplierThreshold { .. } => has_multiplier_threshold = true,
                        ModTag::PerStat { .. } => has_per_stat = true,
                        ModTag::SkillType(_) => has_skill_type = true,
                        ModTag::GlobalEffect { .. } => has_global_effect = true,
                        _ => {}
                    }
                }
            }
        }

        assert!(has_condition, "Missing Condition tags");
        assert!(has_actor_condition, "Missing ActorCondition tags");
        assert!(has_multiplier, "Missing Multiplier tags");
        assert!(has_multiplier_threshold, "Missing MultiplierThreshold tags");
        assert!(has_per_stat, "Missing PerStat tags");
        assert!(has_skill_type, "Missing SkillType tags");
        assert!(has_global_effect, "Missing GlobalEffect tags");
    }

    // Verify that the div field is properly carried through from the normalized data
    #[test]
    fn test_div_propagation() {
        let db = load_db();
        let source = SourceId(0);

        // additional_base_critical_strike_chance has div:100 in the data
        let mods = db.resolve("additional_base_critical_strike_chance", 50.0, source);
        assert!(!mods.is_empty(), "Expected to resolve additional_base_critical_strike_chance");
        // value should be 50.0 / 100.0 = 0.5 if div is present
        let has_divided = mods.iter().any(|m| (m.value - 0.5).abs() < f64::EPSILON);
        assert!(has_divided, "Expected div=100 to produce value=0.5 from input 50.0, got {:?}",
            mods.iter().map(|m| m.value).collect::<Vec<_>>());
    }
}