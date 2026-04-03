use rustc_hash::FxHashMap;
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::data::stat_id;
use specta::Type;

/// A gem placed in a socket group — tracks identity, level, quality, and computed stats.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct GemInstance {
    pub gem_id: String,
    pub name: String,
    pub is_support: bool,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
    /// Computed stats at the current level/quality (stat_id → value).
    pub stats: BTreeMap<String, f64>,
    pub mana_cost: Option<f64>,
    pub crit_chance: Option<f64>,
    pub damage_effectiveness: Option<f64>,
    pub mana_multiplier: Option<f64>,
    pub cooldown: Option<f64>,
    pub attack_speed_multiplier: Option<f64>,
}

/// A group of linked gems — one active skill plus its supports.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct SkillGroup {
    pub id: u32,
    pub label: String,
    pub gems: Vec<GemInstance>,
    pub enabled: bool,
    /// Support compatibility entries (only populated on analysis).
    pub compatibility: Vec<SupportCompatEntry>,
}

/// Whether a support gem is compatible with a specific active gem.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct SupportCompatEntry {
    pub support_gem_id: String,
    pub active_gem_id: String,
    pub compatible: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantedEffect {
    pub name: String,
    #[serde(default)]
    pub base_type_name: Option<String>,
    #[serde(default)]
    pub cast_time: f64,
    #[serde(default)]
    pub color: u32,
    #[serde(default)]
    pub description: Option<String>,

    // Support-only fields
    #[serde(default)]
    pub support: bool,
    #[serde(default)]
    pub require_skill_types: Vec<u32>, // postfix boolean expression
    #[serde(default)]
    pub exclude_skill_types: Vec<u32>, // postfix boolean expression
    #[serde(default)]
    pub add_skill_types: Vec<u32>, // types added to supported active
    #[serde(default)]
    pub add_flags: FxHashMap<String, bool>, // e.g. {"totem": true, "mine": true}
    #[serde(default)]
    pub support_gems_only: bool,
    #[serde(default)]
    pub is_trigger: bool,
    #[serde(default)]
    pub weapon_types: Option<FxHashMap<String, bool>>,
    #[serde(default)]
    pub has_global_effect: bool,

    // Skill type set — numeric SkillType IDs → true
    // JSON may be a map {"1": true} or a 1-indexed sparse array [null, true, null, ...]
    #[serde(default, deserialize_with = "deserialize_skill_types")]
    pub skill_types: FxHashMap<u32, bool>,

    // Stat system
    #[serde(default)]
    pub stats: Vec<String>, // ordered stat ID names (index corresponds to level values)
    #[serde(default)]
    pub constant_stats: Vec<(String, f64)>, // stats with fixed values: [["stat_id", value], ...]
    #[serde(default)]
    pub quality_stats: QualityStats, // keyed by quality type
    #[serde(default, deserialize_with = "map_or_empty_seq")]
    pub stat_map: FxHashMap<String, Value>, // stat_id → modifier template(s) (complex, keep as Value for now)
    #[serde(default)]
    pub base_mods: Vec<Value>, // pre-built modifier objects

    // Damage scaling (for effectiveness interpolation)
    #[serde(default)]
    pub base_effectiveness: Option<f64>,
    #[serde(default)]
    pub incremental_effectiveness: Option<f64>,

    // Flags
    #[serde(default, deserialize_with = "map_or_empty_seq")]
    pub base_flags: FxHashMap<String, bool>, // {"spell": true, "area": true, ...}

    // Levels — per-level data (see GrantedEffectLevel)
    // May be an array or a map keyed by level number depending on the source file.
    #[serde(default, deserialize_with = "deserialize_levels")]
    pub levels: Vec<GrantedEffectLevel>,

    // Other fields used by PoB
    #[serde(default)]
    pub stat_description_scope: Option<String>,
    #[serde(default)]
    pub minion_list: Option<Vec<String>>,
    #[serde(default)]
    pub plus_version_of: Option<String>, // exceptional (transfigured) gems
    #[serde(default)]
    pub legacy: bool,
}

/// Per-level data for a granted effect.
/// Stat values are stored positionally ("1", "2", etc.) matching the `stats` array.
/// Use `serde_json::Value` for flexible deserialization, then extract values by index.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantedEffectLevel {
    pub level_requirement: Option<u32>,
    #[serde(default, deserialize_with = "optional_map_or_seq")]
    pub cost: Option<FxHashMap<String, f64>>, // {"Mana": 9, "Life": 5, ...}
    #[serde(default)]
    pub crit_chance: Option<f64>,
    #[serde(default)]
    pub damage_effectiveness: Option<f64>,
    #[serde(default)]
    pub mana_multiplier: Option<f64>, // support gems: cost multiplier
    #[serde(default)]
    pub cooldown: Option<f64>,
    #[serde(default)]
    pub stored_uses: Option<u32>,
    #[serde(default)]
    pub attack_speed_multiplier: Option<f64>,
    #[serde(default)]
    pub attack_time: Option<f64>,
    #[serde(default)]
    pub base_multiplier: Option<f64>,
    #[serde(default)]
    pub stat_interpolation: Option<Vec<u32>>, // 1=static, 2=linear, 3=effectiveness
    /// Positional stat values. Key is the string index ("1", "2", etc.).
    /// Corresponds to the parent GrantedEffect's `stats` array.
    #[serde(flatten)]
    pub stat_values: FxHashMap<String, Value>,
}
/// Quality stats, keyed by quality type.
/// "Default" is standard quality. "Alternate1"/"Alternate2"/"Alternate3" are alt qualities.
/// Each value is a list of [stat_id, value_per_quality_point] tuples.
/// PoB emits `[]` for empty quality stats instead of `{}`, so we accept both.
#[derive(Debug, Clone, Default)]
pub struct QualityStats(pub FxHashMap<String, Vec<(String, f64)>>);

impl<'de> Deserialize<'de> for QualityStats {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct QsVisitor;
        impl<'de> Visitor<'de> for QsVisitor {
            type Value = QualityStats;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a map or empty array")
            }
            // [] → treat as empty map
            fn visit_seq<A: SeqAccess<'de>>(self, _: A) -> Result<QualityStats, A::Error> {
                Ok(QualityStats(FxHashMap::default()))
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<QualityStats, M::Error> {
                let mut result = FxHashMap::default();
                while let Some((k, v)) = map.next_entry::<String, Vec<(String, f64)>>()? {
                    result.insert(k, v);
                }
                Ok(QualityStats(result))
            }
        }
        deserializer.deserialize_any(QsVisitor)
    }
}

pub fn does_type_expression_match(
    check_types: &[u32],
    skill_types: &FxHashMap<u32, bool>,
    minion_types: Option<&FxHashMap<u32, bool>>,
) -> bool {
    let mut stack: Vec<bool> = Vec::with_capacity(check_types.len());
    for &st in check_types {
        match st {
            skill_type::OR => {
                let b = stack.pop().unwrap_or(false);
                if let Some(top) = stack.last_mut() {
                    *top = *top || b;
                }
            }
            skill_type::AND => {
                let b = stack.pop().unwrap_or(false);
                if let Some(top) = stack.last_mut() {
                    *top = *top && b;
                }
            }
            skill_type::NOT => {
                if let Some(top) = stack.last_mut() {
                    *top = !*top;
                }
            }
            _ => {
                let has = skill_types.get(&st).copied().unwrap_or(false)
                    || minion_types.map_or(false, |m| m.get(&st).copied().unwrap_or(false));
                stack.push(has);
            }
        }
    }
    stack.iter().any(|&v| v)
}
/// With a given support gem and the current active skill types return if the support gem will work
pub fn can_support(
    support: &GrantedEffect,
    active_skill_types: &FxHashMap<u32, bool>,
    active_is_gem: bool,
) -> bool {
    if support.support_gems_only && !active_is_gem {
        return false;
    }
    if !support.exclude_skill_types.is_empty()
        && does_type_expression_match(&support.exclude_skill_types, active_skill_types, None)
    {
        return false;
    }

    support.require_skill_types.is_empty()
        || does_type_expression_match(&support.require_skill_types, active_skill_types, None)
}

/// Build the stat table for a skill at a given level and quality.
/// Returns a map of stat_id → computed value.
///
/// Mirrors PoB's `calcLib.buildSkillInstanceStats()`.
pub fn build_skill_instance_stats(
    effect: &GrantedEffect,
    level: u32,
    quality: u32,
    quality_id: &str,
) -> FxHashMap<String, f64> {
    let mut stats: FxHashMap<String, f64> = FxHashMap::default();
    let level_idx = (level as usize).saturating_sub(1);
    let level_data = match effect.levels.get(level_idx) {
        Some(ld) => ld,
        None => return stats,
    };

    // Calculate quality stats
    if quality > 0 {
        if let Some(qs) = effect
            .quality_stats
            .0
            .get(quality_id)
            .or_else(|| effect.quality_stats.0.get("Default"))
        {
            for (stat_id, value_per_point) in qs {
                let val = (value_per_point * quality as f64).trunc();
                *stats.entry(stat_id.clone()).or_default() += val;
            }
        }
    }

    // Calculate per level stats
    let actor_level = level_data.level_requirement.unwrap_or(1) as f64;
    for (index, stat_id) in effect.stats.iter().enumerate() {
        let key = (index + 1).to_string();
        let raw_value = level_data
            .stat_values
            .get(&key)
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let interp_mode = level_data
            .stat_interpolation
            .as_ref()
            .and_then(|si| si.get(index))
            .copied()
            .unwrap_or(1);

        let stat_value = match interp_mode {
            3 => {
                // Effectiveness interpolation — PoB CalcTools.lua formula:
                // availableEffectiveness =
                //   (SkillDamageBaseEffectiveness + SkillDamageIncrementalEffectiveness * (actorLevel - 1))
                //   * grantedEffect.baseEffectiveness
                //   * (1 + grantedEffect.incrementalEffectiveness) ^ (actorLevel - 1)
                // statValue = round(availableEffectiveness * level[index])
                const SKILL_DMG_BASE_EFF: f64 = 3.885209;
                const SKILL_DMG_INC_EFF: f64 = 0.360246;
                let game_scaling = SKILL_DMG_BASE_EFF + SKILL_DMG_INC_EFF * (actor_level - 1.0);
                let base_eff = effect.base_effectiveness.unwrap_or(1.0);
                let inc_eff = effect.incremental_effectiveness.unwrap_or(0.0);
                let effectiveness =
                    game_scaling * base_eff * (1.0 + inc_eff).powf(actor_level - 1.0);
                (effectiveness * raw_value).round()
            }
            2 => {
                // Linear interpolation between level brackets
                // Simplified: use raw_value (full interpolation needs adjacent levels)
                raw_value.round()
            }
            _ => raw_value, // Static (mode 1): use as-is
        };

        *stats.entry(stat_id.clone()).or_default() += stat_value;
    }

    // Calculate constant stats
    for (stat_id, value) in &effect.constant_stats {
        *stats.entry(stat_id.clone()).or_default() += value;
    }

    return stats;
}
/// Resolve which supports apply to an active gem within a skill group.
/// Returns the list of applicable support GrantedEffect IDs.
///
/// Mirrors PoB's `calcs.createActiveSkill()` two-pass logic:
/// Pass 1: Check compatibility and accumulate addSkillTypes from compatible supports.
///         Repeat until no new supports are added (handles cross-dependencies).
/// Pass 2: Build final list of compatible supports.
pub fn resolve_supports(
    active_effect: &GrantedEffect,
    supports: &[(String, &GrantedEffect)],
    active_is_gem: bool,
) -> Vec<String> {
    // Start with active gems skill types
    let mut effective_types = active_effect.skill_types.clone();

    // Pass 1
    let mut added_new = true;
    let mut compatible: Vec<bool> = vec![false; supports.len()];
    while added_new {
        added_new = false;
        for (i, (_, support)) in supports.iter().enumerate() {
            if compatible[i] || !support.support {
                continue;
            }
            if can_support(support, &effective_types, active_is_gem) {
                compatible[i] = true;
                added_new = true;
                for &st in &support.add_skill_types {
                    effective_types.insert(st, true);
                }
            }
        }
    }

    // Pass 2 to recheck against final type set
    supports
        .iter()
        .enumerate()
        .filter(|(i, (_, support))| {
            support.support && can_support(support, &effective_types, active_is_gem)
        })
        .map(|(_, (id, _))| id.clone())
        .collect()
}

// Deserializers

/// Deserializes a `FxHashMap<String, V>` that may appear as `[]` (empty array) in the JSON.
fn map_or_empty_seq<'de, D, V>(d: D) -> Result<FxHashMap<String, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct MorS<V>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>> Visitor<'de> for MorS<V> {
        type Value = FxHashMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a map or empty array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
            Ok(FxHashMap::default())
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut result = FxHashMap::default();
            while let Some((k, v)) = map.next_entry::<String, V>()? {
                result.insert(k, v);
            }
            Ok(result)
        }
    }
    d.deserialize_any(MorS(PhantomData))
}

/// Deserializes `Option<FxHashMap<String, f64>>` that may appear as `[]` or `null`.
fn optional_map_or_seq<'de, D>(d: D) -> Result<Option<FxHashMap<String, f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Vis;
    impl<'de> Visitor<'de> for Vis {
        type Value = Option<FxHashMap<String, f64>>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a map, null, or empty array")
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, _: A) -> Result<Self::Value, A::Error> {
            Ok(Some(FxHashMap::default()))
        }
        fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
            let mut result = FxHashMap::default();
            while let Some((k, v)) = m.next_entry::<String, f64>()? {
                result.insert(k, v);
            }
            Ok(Some(result))
        }
    }
    d.deserialize_any(Vis)
}

/// Deserializes `skillTypes` which is either a `{"id": true}` map or a 1-indexed sparse
/// array like `[null, true, null, true]` where the array index is the skill type ID.
fn deserialize_skill_types<'de, D>(d: D) -> Result<FxHashMap<u32, bool>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Vis;
    impl<'de> Visitor<'de> for Vis {
        type Value = FxHashMap<u32, bool>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a map or 1-indexed sparse boolean array")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut map = FxHashMap::default();
            let mut idx: u32 = 1;
            while let Some(v) = seq.next_element::<Option<bool>>()? {
                if v == Some(true) {
                    map.insert(idx, true);
                }
                idx += 1;
            }
            Ok(map)
        }
        fn visit_map<M: MapAccess<'de>>(self, mut m: M) -> Result<Self::Value, M::Error> {
            let mut map = FxHashMap::default();
            while let Some((k, v)) = m.next_entry::<u32, bool>()? {
                map.insert(k, v);
            }
            Ok(map)
        }
    }
    d.deserialize_any(Vis)
}

/// Deserializes `levels` which may be an array `[{...}]` or a map `{"1": {...}, "20": {...}}`.
/// When it's a map the keys (level numbers) are discarded — only the values are kept.
fn deserialize_levels<'de, D>(d: D) -> Result<Vec<GrantedEffectLevel>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Vis;
    impl<'de> Visitor<'de> for Vis {
        type Value = Vec<GrantedEffectLevel>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "an array or map of GrantedEffectLevel")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while let Some(entry) = seq.next_element::<Option<GrantedEffectLevel>>()? {
                if let Some(level) = entry {
                    v.push(level);
                }
            }
            Ok(v)
        }
        fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
            let mut v = Vec::new();
            while let Some((_key, entry)) =
                map.next_entry::<serde::de::IgnoredAny, Option<GrantedEffectLevel>>()?
            {
                if let Some(level) = entry {
                    v.push(level);
                }
            }
            Ok(v)
        }
    }
    d.deserialize_any(Vis)
}
/// SkillType IDs from PoB's Global.lua / ActiveSkillType.dat (as of PoE 3.17+).
/// Used in requireSkillTypes / excludeSkillTypes postfix boolean expressions.
/// Stored as raw u32 values (not an enum) because they appear in runtime-evaluated expressions.
/// Source: PathOfBuildingCommunity/PathOfBuilding – src/Data/Global.lua  SkillType table.
#[allow(dead_code)]
pub mod skill_type {
    // ── Core damage/cast modes ─────────────────────────────────────────────
    pub const ATTACK: u32 = 1;
    pub const SPELL: u32 = 2;
    /// Skills that fire projectiles.
    pub const PROJECTILE: u32 = 3;
    /// Attack requires dual wielding (only Dual Strike).
    pub const DUAL_WIELD_ONLY: u32 = 4;
    pub const BUFF: u32 = 5;
    // 6 removed (was CanDualWield)
    /// Attack only uses the main hand (removed in 3.5, kept for 2.6 compat).
    pub const MAIN_HAND_ONLY: u32 = 7;
    // 8 removed (was only used on Cleave)
    pub const MINION: u32 = 9;
    /// Skill hits (not set on attacks since all attacks hit by default).
    pub const DAMAGE: u32 = 10;
    pub const AREA: u32 = 11;
    pub const DURATION: u32 = 12;
    pub const REQUIRES_SHIELD: u32 = 13;
    pub const PROJECTILE_SPEED: u32 = 14;
    pub const HAS_RESERVATION: u32 = 15;
    pub const RESERVATION_BECOMES_COST: u32 = 16;
    /// Skill can be turned into a trap.
    pub const TRAPPABLE: u32 = 17;
    /// Skill can be turned into a totem.
    pub const TOTEMABLE: u32 = 18;
    /// Skill can be turned into a mine.
    pub const MINEABLE: u32 = 19;
    /// Causes elemental status effects without hitting (e.g. Herald of Ash).
    pub const ELEMENTAL_STATUS: u32 = 20;
    pub const MINIONS_CAN_EXPLODE: u32 = 21;
    // 22 removed (was AttackCanTotem)
    pub const CHAINS: u32 = 23;
    pub const MELEE: u32 = 24;
    pub const MELEE_SINGLE_TARGET: u32 = 25;
    /// Spell can repeat via Spell Echo.
    pub const MULTICASTABLE: u32 = 26;
    pub const TOTEM_CASTS_ALONE: u32 = 27;
    /// Attack can repeat via Multistrike.
    pub const MULTISTRIKEABLE: u32 = 28;
    /// Deals burning damage.
    pub const CAUSES_BURNING: u32 = 29;
    pub const SUMMONS_TOTEM: u32 = 30;
    pub const TOTEM_CASTS_WHEN_NOT_DETACHED: u32 = 31;

    // ── Damage elements ────────────────────────────────────────────────────
    pub const FIRE: u32 = 32;
    pub const COLD: u32 = 33;
    pub const LIGHTNING: u32 = 34;

    // ── Trigger / trap / mine flags ────────────────────────────────────────
    pub const TRIGGERABLE: u32 = 35;
    pub const TRAPPED: u32 = 36;
    pub const MOVEMENT: u32 = 37;
    // 38 removed (was Cast)
    pub const DAMAGE_OVER_TIME: u32 = 39;
    pub const REMOTE_MINED: u32 = 40;
    pub const TRIGGERED: u32 = 41;

    // ── Skill categories ───────────────────────────────────────────────────
    pub const VAAL: u32 = 42;
    pub const AURA: u32 = 43;
    // 44 removed (was LightningSpell)
    pub const CAN_TARGET_UNUSABLE_CORPSE: u32 = 45;
    // 46 removed (was TriggeredAttack)
    pub const RANGED_ATTACK: u32 = 47;
    // 48 removed (was MinionSpell)
    pub const CHAOS: u32 = 49;
    pub const FIXED_SPEED_PROJECTILE: u32 = 50; // unused in live data
                                                // 51 removed
    /// Allows Burning Arrow / Vigilant Strike to be supported by AoE gems.
    pub const THRESHOLD_JEWEL_AREA: u32 = 52;
    pub const THRESHOLD_JEWEL_PROJECTILE: u32 = 53;
    pub const THRESHOLD_JEWEL_DURATION: u32 = 54;
    pub const THRESHOLD_JEWEL_RANGED_ATTACK: u32 = 55;
    // 56 removed
    pub const CHANNEL: u32 = 57;
    /// Allows Contagion/Blight/Scorching Ray to be supported by Controlled Destruction.
    pub const DEGEN_ONLY_SPELL_DAMAGE: u32 = 58;
    // 59 removed (was ColdSpell)
    /// Skill granted by item that auto-triggers; blocks trap/mine/totem supports.
    pub const INBUILT_TRIGGER: u32 = 60;
    pub const GOLEM: u32 = 61;
    pub const HERALD: u32 = 62;
    pub const AURA_AFFECTS_ENEMIES: u32 = 63;
    pub const NO_RUTHLESS: u32 = 64;
    pub const THRESHOLD_JEWEL_SPELL_DAMAGE: u32 = 65;
    /// Spell can cascade via Spell Cascade.
    pub const CASCADABLE: u32 = 66;
    /// Skill can be supported by Volley.
    pub const PROJECTILES_FROM_USER: u32 = 67;
    pub const MIRAGE_ARCHER_CAN_USE: u32 = 68;
    /// Excludes Volley from Vaal Fireball / Vaal Spark.
    pub const PROJECTILE_SPIRAL: u32 = 69;
    /// Excludes Volley from Spectral Shield Throw.
    pub const SINGLE_MAIN_PROJECTILE: u32 = 70;
    /// Prevents Summon Phantasm on Kill from applying to Dancing Dervish.
    pub const MINIONS_PERSIST_WHEN_SKILL_REMOVED: u32 = 71;
    /// Allows LMP/GMP on Rain of Arrows and Toxic Rain.
    pub const PROJECTILE_NUMBER: u32 = 72;
    pub const WARCRY: u32 = 73;
    pub const INSTANT: u32 = 74;
    pub const BRAND: u32 = 75;
    pub const DESTROYS_CORPSE: u32 = 76;
    pub const NON_HIT_CHILL: u32 = 77;
    pub const CHILLING_AREA: u32 = 78;
    pub const APPLIES_CURSE: u32 = 79;
    pub const CAN_RAPID_FIRE: u32 = 80;
    pub const AURA_DURATION: u32 = 81;
    pub const AREA_SPELL: u32 = 82;

    // ── Postfix expression operators ───────────────────────────────────────
    pub const OR: u32 = 83;
    pub const AND: u32 = 84;
    pub const NOT: u32 = 85;

    // ── More skill categories ───────────────────────────────────────────────
    pub const PHYSICAL: u32 = 86;
    pub const APPLIES_MAIM: u32 = 87;
    pub const CREATES_MINION: u32 = 88;
    pub const GUARD: u32 = 89;
    pub const TRAVEL: u32 = 90;
    pub const BLINK: u32 = 91;
    pub const CAN_HAVE_BLESSING: u32 = 92;
    pub const PROJECTILES_NOT_FROM_USER: u32 = 93;
    pub const ATTACK_IN_PLACE_IS_DEFAULT: u32 = 94;
    pub const NOVA: u32 = 95;
    pub const INSTANT_NO_REPEAT_WHEN_HELD: u32 = 96;
    pub const INSTANT_SHIFT_ATTACK_FOR_LEFT_MOUSE: u32 = 97;
    pub const AURA_NOT_ON_CASTER: u32 = 98;
    pub const BANNER: u32 = 99;
    pub const RAIN: u32 = 100;
    pub const COOLDOWN: u32 = 101;
    pub const THRESHOLD_JEWEL_CHAINING: u32 = 102;
    pub const SLAM: u32 = 103;
    pub const STANCE: u32 = 104;
    /// Blood and Sand / Flesh and Stone.
    pub const NON_REPEATABLE: u32 = 105;
    pub const OTHER_THING_USES_SKILL: u32 = 106;
    pub const STEEL: u32 = 107;
    pub const HEX: u32 = 108;
    pub const MARK: u32 = 109;
    pub const AEGIS: u32 = 110;
    pub const ORB: u32 = 111;
    pub const KILL_NO_DAMAGE_MODIFIERS: u32 = 112;
    /// Elements cannot repeat.
    pub const RANDOM_ELEMENT: u32 = 113;
    pub const LATE_CONSUME_COOLDOWN: u32 = 114;
    /// Reliant on amount of mana spent.
    pub const ARCANE: u32 = 115;
    pub const FIXED_CAST_TIME: u32 = 116;
    pub const REQUIRES_OFF_HAND_NOT_WEAPON: u32 = 117;
    pub const LINK: u32 = 118;
    pub const BLESSING: u32 = 119;
    pub const ZERO_RESERVATION: u32 = 120;
    pub const DYNAMIC_COOLDOWN: u32 = 121;
    pub const MICROTRANSACTION: u32 = 122;
    pub const OWNER_CANNOT_USE: u32 = 123;
    pub const PROJECTILES_NUMBER_MODIFIERS_NOT_APPLIED: u32 = 124;
    pub const TOTEMS_ARE_BALLISTAE: u32 = 125;
    pub const SKILL_GRANTED_BY_SUPPORT: u32 = 126;
    pub const PREVENT_HEX_TRANSFER: u32 = 127;
    pub const MINIONS_ARE_UNDAMAGEABLE: u32 = 128;
    pub const INNATE_TRAUMA: u32 = 129;
    pub const DUAL_WIELD_REQUIRES_DIFFERENT_TYPES: u32 = 130;
    pub const NO_VOLLEY: u32 = 131;
    pub const RETALIATION: u32 = 132;
    pub const NEVER_EXERTABLE: u32 = 133;
    pub const DISALLOW_TRIGGER_SUPPORTS: u32 = 134;
    pub const PROJECTILE_CANNOT_RETURN: u32 = 135;
    pub const OFFERING: u32 = 136;
    pub const SUPPORTED_BY_BANE: u32 = 137;
    pub const WAND_ATTACK: u32 = 138;
    pub const GAINS_INTENSITY: u32 = 139;
    pub const CREATES_SENTINEL_MINION: u32 = 140;
    pub const SUPPORTED_BY_AUTO_EXERTION: u32 = 141;
}
