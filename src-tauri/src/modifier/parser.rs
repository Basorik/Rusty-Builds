use super::stat_table;
use super::types::*;
use crate::data::{SourceId, StatId};
use regex::Regex;
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use std::sync::OnceLock;

type ParseHandler = fn(values: &[f64], source: SourceId) -> Vec<Modifier>;

/// Single regex for extracting all numbers from a stat string.
/// Compiled once, used for every stat line.
static RE_NUMBER: OnceLock<Regex> = OnceLock::new();

fn re_number() -> &'static Regex {
    // Match only unsigned numbers — the sign (+ or -) stays in the template as a
    // literal character so keys like "+# to maximum Life" match correctly.
    RE_NUMBER.get_or_init(|| Regex::new(r"\d+\.?\d*").unwrap())
}

/// The template lookup table. Built once at first call, lives forever.
/// Key: stat text with all numbers replaced by `#` (e.g. "#% increased maximum Life")
/// Value: handler function that builds Modifier(s) from the extracted numbers
static TEMPLATE_MAP: OnceLock<FxHashMap<&'static str, ParseHandler>> = OnceLock::new();

fn template_map() -> &'static FxHashMap<&'static str, ParseHandler> {
    TEMPLATE_MAP.get_or_init(|| {
        let mut m: FxHashMap<&'static str, ParseHandler> = FxHashMap::default();

        // ── Simple Base: "+X to <Stat>" ──
        m.insert("+# to Strength", |v, s| {
            vec![simple_mod(StatId::Strength, ModType::Base, v[0], s)]
        });
        m.insert("+# to Dexterity", |v, s| {
            vec![simple_mod(StatId::Dexterity, ModType::Base, v[0], s)]
        });
        m.insert("+# to Intelligence", |v, s| {
            vec![simple_mod(StatId::Intelligence, ModType::Base, v[0], s)]
        });
        m.insert("+# to maximum Life", |v, s| {
            vec![simple_mod(StatId::Life, ModType::Base, v[0], s)]
        });
        m.insert("+# to maximum Mana", |v, s| {
            vec![simple_mod(StatId::Mana, ModType::Base, v[0], s)]
        });
        m.insert("+# to maximum Energy Shield", |v, s| {
            vec![simple_mod(StatId::EnergyShield, ModType::Base, v[0], s)]
        });
        m.insert("+# to Accuracy Rating", |v, s| {
            vec![simple_mod(StatId::Accuracy, ModType::Base, v[0], s)]
        });
        m.insert("+# to Armour", |v, s| {
            vec![simple_mod(StatId::Armour, ModType::Base, v[0], s)]
        });

        // ── Simple Inc: "X% increased <Stat>" ──
        m.insert("#% increased maximum Life", |v, s| {
            vec![simple_mod(StatId::Life, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased maximum Mana", |v, s| {
            vec![simple_mod(StatId::Mana, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased maximum Energy Shield", |v, s| {
            vec![simple_mod(StatId::EnergyShield, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Evasion Rating", |v, s| {
            vec![simple_mod(StatId::Evasion, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Armour", |v, s| {
            vec![simple_mod(StatId::Armour, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Attack Speed", |v, s| {
            vec![flagged_mod(
                StatId::Speed,
                ModType::Inc,
                v[0],
                s,
                ModFlag::ATTACK,
            )]
        });
        m.insert("#% increased Cast Speed", |v, s| {
            vec![flagged_mod(
                StatId::Speed,
                ModType::Inc,
                v[0],
                s,
                ModFlag::CAST,
            )]
        });
        m.insert("#% increased Movement Speed", |v, s| {
            vec![simple_mod(StatId::MovementSpeed, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Physical Damage", |v, s| {
            vec![simple_mod(StatId::PhysicalDamage, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Spell Damage", |v, s| {
            vec![flagged_mod(
                StatId::Damage,
                ModType::Inc,
                v[0],
                s,
                ModFlag::SPELL,
            )]
        });
        m.insert("#% increased Elemental Damage", |v, s| {
            vec![simple_mod(StatId::ElementalDamage, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Critical Strike Chance", |v, s| {
            vec![simple_mod(StatId::CritChance, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Mana Regeneration Rate", |v, s| {
            vec![simple_mod(StatId::ManaRegeneration, ModType::Inc, v[0], s)]
        });
        m.insert("#% increased Projectile Damage", |v, s| {
            vec![flagged_mod(
                StatId::Damage,
                ModType::Inc,
                v[0],
                s,
                ModFlag::PROJECTILE,
            )]
        });

        // ── Resistances: "+X% to <Element> Resistance" ──
        m.insert("+#% to Fire Resistance", |v, s| {
            vec![simple_mod(StatId::FireResist, ModType::Base, v[0], s)]
        });
        m.insert("+#% to Cold Resistance", |v, s| {
            vec![simple_mod(StatId::ColdResist, ModType::Base, v[0], s)]
        });
        m.insert("+#% to Lightning Resistance", |v, s| {
            vec![simple_mod(StatId::LightningResist, ModType::Base, v[0], s)]
        });
        m.insert("+#% to Chaos Resistance", |v, s| {
            vec![simple_mod(StatId::ChaosResist, ModType::Base, v[0], s)]
        });
        // Expands to 3 modifiers
        m.insert("+#% to all Elemental Resistances", |v, s| {
            vec![
                simple_mod(StatId::FireResist, ModType::Base, v[0], s),
                simple_mod(StatId::ColdResist, ModType::Base, v[0], s),
                simple_mod(StatId::LightningResist, ModType::Base, v[0], s),
            ]
        });

        // ── Multi-stat expansion: "X% increased A and B" ──
        m.insert("#% increased Evasion Rating and Armour", |v, s| {
            vec![
                simple_mod(StatId::Evasion, ModType::Inc, v[0], s),
                simple_mod(StatId::Armour, ModType::Inc, v[0], s),
            ]
        });
        m.insert("#% increased Attack and Cast Speed", |v, s| {
            vec![
                flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::ATTACK),
                flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::CAST),
            ]
        });

        // ── Dual-attribute: "+X to A and B" ──
        m.insert("+# to Strength and Intelligence", |v, s| {
            vec![
                simple_mod(StatId::Strength, ModType::Base, v[0], s),
                simple_mod(StatId::Intelligence, ModType::Base, v[0], s),
            ]
        });
        m.insert("+# to Strength and Dexterity", |v, s| {
            vec![
                simple_mod(StatId::Strength, ModType::Base, v[0], s),
                simple_mod(StatId::Dexterity, ModType::Base, v[0], s),
            ]
        });
        m.insert("+# to Dexterity and Intelligence", |v, s| {
            vec![
                simple_mod(StatId::Dexterity, ModType::Base, v[0], s),
                simple_mod(StatId::Intelligence, ModType::Base, v[0], s),
            ]
        });

        // ... expand table in later phases (see "Growing the table" below)
        m
    })
}

/// Parse a single display-text stat line into Modifier(s).
/// Returns empty Vec for unrecognized patterns (logged as warning).
///
/// Pipeline:
///   1. Extract all numbers from the text
///   2. Replace numbers with `#` to form a template key
///   3. Look up the template in the handler table (O(1) FxHashMap)
///   4. Call the handler with the extracted numbers + source
///
/// This handles human-readable stat text from passive nodes and items.
/// Gem/skill stats use the SkillStatMap direct mapping path instead (Phase 3).
/// Single entry point for all stat ID resolution.
///
/// Checks the static table first (covering all SkillStatMap entries with full
/// flag + condition tag data), then falls back to structural pattern matching
/// for the remaining patterned IDs not covered by SkillStatMap.
///
/// Returns an empty Vec when the stat ID is completely unknown — callers
/// log a debug message in that case.
pub fn resolve(stat_id: &str, value: f64, source: SourceId) -> Vec<Modifier> {
    if let Some(defs) = stat_table::stat_table().get(stat_id) {
        return defs
            .iter()
            .map(|d| stat_table::apply(d, value, source))
            .collect();
    }
    log::debug!("Unresolved stat: {}", stat_id);
    Vec::new()
}

pub fn parse_display_text(text: &str, source: SourceId) -> Vec<Modifier> {
    let re = re_number();

    // Step 1: extract all numbers
    let values: Vec<f64> = re
        .find_iter(text)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    // Step 2: replace numbers with # to form template key
    let template = re.replace_all(text, "#");

    // Step 3: O(1) lookup
    if let Some(handler) = template_map().get(template.as_ref()) {
        // Step 4: call handler
        handler(&values, source)
    } else {
        log::debug!(
            "Unrecognized stat template: {:?} (from {:?})",
            template,
            text
        );
        Vec::new()
    }
}

/// Helper: build a simple unconditional modifier (no flags, no tags).
pub fn simple_mod(stat: StatId, mod_type: ModType, value: f64, source: SourceId) -> Modifier {
    Modifier {
        stat,
        mod_type,
        value,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        source,
        tags: smallvec![],
    }
}

/// Helper: build a modifier with condition tags (for Phase 5+ conditional stats).
#[allow(dead_code)]
fn tagged_mod(
    stat: StatId,
    mod_type: ModType,
    value: f64,
    source: SourceId,
    tags: smallvec::SmallVec<[ModTag; 2]>,
) -> Modifier {
    Modifier {
        stat,
        mod_type,
        value,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        source,
        tags,
    }
}

/// Helper: build a modifier scoped to specific ModFlag(s) (e.g. Attack/Cast/Spell).
pub fn flagged_mod(
    stat: StatId,
    mod_type: ModType,
    value: f64,
    source: SourceId,
    flags: ModFlag,
) -> Modifier {
    Modifier {
        stat,
        mod_type,
        value,
        flags,
        keywords: KeywordFlag::empty(),
        source,
        tags: smallvec![],
    }
}

/// Helper: build a flag modifier (no numeric value).
#[allow(dead_code)]
fn flag_mod(stat: StatId, source: SourceId) -> Modifier {
    Modifier {
        stat,
        mod_type: ModType::Flag,
        value: 1.0,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        source,
        tags: smallvec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src() -> SourceId {
        SourceId(0)
    }

    // ── Helper: assert a single Modifier matches (stat, mod_type, value, flags) ──
    fn assert_mod(m: &Modifier, stat: StatId, mod_type: ModType, value: f64, flags: ModFlag) {
        assert_eq!(m.stat, stat, "stat mismatch for {:?}", stat);
        assert_eq!(m.mod_type, mod_type, "mod_type mismatch");
        assert!(
            (m.value - value).abs() < 1e-9,
            "value mismatch: {} != {}",
            m.value,
            value
        );
        assert_eq!(m.flags, flags, "flags mismatch");
        assert!(m.tags.is_empty(), "expected no tags");
    }

    // ── Suffix _+% → Inc ──

    #[test]
    fn inc_maximum_life() {
        let r = resolve("maximum_life_+%", 20.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Life, ModType::Inc, 20.0, ModFlag::empty());
    }

    #[test]
    fn inc_maximum_mana() {
        let r = resolve("maximum_mana_+%", 15.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Mana, ModType::Inc, 15.0, ModFlag::empty());
    }

    #[test]
    fn inc_maximum_energy_shield() {
        let r = resolve("maximum_energy_shield_+%", 12.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::EnergyShield,
            ModType::Inc,
            12.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn inc_evasion_rating() {
        let r = resolve("evasion_rating_+%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Evasion, ModType::Inc, 10.0, ModFlag::empty());
    }

    #[test]
    fn inc_physical_damage() {
        let r = resolve("physical_damage_+%", 8.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::PhysicalDamage,
            ModType::Inc,
            8.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn inc_spell_damage_has_cast_flag() {
        // SSM maps spell_damage_+% to ModFlag::CAST (PoB: ModFlag.Cast), not SPELL
        let r = resolve("spell_damage_+%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Damage, ModType::Inc, 10.0, ModFlag::CAST);
    }

    #[test]
    fn inc_attack_damage_has_attack_flag() {
        let r = resolve("attack_damage_+%", 5.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Damage, ModType::Inc, 5.0, ModFlag::ATTACK);
    }

    #[test]
    fn inc_critical_strike_chance() {
        let r = resolve("critical_strike_chance_+%", 25.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::CritChance,
            ModType::Inc,
            25.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn inc_global_critical_strike_chance() {
        let r = resolve("global_critical_strike_chance_+%", 25.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::CritChance,
            ModType::Inc,
            25.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn inc_mana_regeneration_rate() {
        let r = resolve("mana_regeneration_rate_+%", 30.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::ManaRegeneration,
            ModType::Inc,
            30.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn inc_life_regeneration_rate() {
        let r = resolve("life_regeneration_rate_+%", 5.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::LifeRegeneration,
            ModType::Inc,
            5.0,
            ModFlag::empty(),
        );
    }

    // ── Suffix _% → Base ──

    #[test]
    fn base_shield_block() {
        let r = resolve("shield_block_%", 3.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::BlockChance,
            ModType::Base,
            3.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn base_fire_resistance() {
        let r = resolve("fire_resistance_%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::FireResist,
            ModType::Base,
            10.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn base_cold_resistance() {
        let r = resolve("cold_resistance_%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::ColdResist,
            ModType::Base,
            10.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn base_lightning_resistance() {
        let r = resolve("lightning_resistance_%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::LightningResist,
            ModType::Base,
            10.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn base_chaos_resistance() {
        let r = resolve("chaos_resistance_%", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::ChaosResist,
            ModType::Base,
            10.0,
            ModFlag::empty(),
        );
    }

    // ── Suffix _+ → Base ──

    #[test]
    fn base_strength_plus() {
        let r = resolve("strength_+", 10.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::Strength,
            ModType::Base,
            10.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn base_crit_multiplier_plus() {
        let r = resolve("critical_strike_multiplier_+", 20.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::CritMultiplier,
            ModType::Base,
            20.0,
            ModFlag::empty(),
        );
    }

    // ── Prefix base_ → Base ──

    #[test]
    fn prefix_base_strength() {
        let r = resolve("base_strength", 30.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::Strength,
            ModType::Base,
            30.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn prefix_base_dexterity() {
        let r = resolve("base_dexterity", 14.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::Dexterity,
            ModType::Base,
            14.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn prefix_base_intelligence() {
        let r = resolve("base_intelligence", 16.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(
            &r[0],
            StatId::Intelligence,
            ModType::Base,
            16.0,
            ModFlag::empty(),
        );
    }

    #[test]
    fn prefix_base_maximum_life() {
        let r = resolve("base_maximum_life", 40.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Life, ModType::Base, 40.0, ModFlag::empty());
    }

    #[test]
    fn prefix_base_maximum_mana() {
        let r = resolve("base_maximum_mana", 20.0, src());
        assert_eq!(r.len(), 1);
        assert_mod(&r[0], StatId::Mana, ModType::Base, 20.0, ModFlag::empty());
    }

    // ── Multi-modifier expansion ──

    #[test]
    fn combined_evasion_and_armour() {
        let r = resolve(
            "evasion_and_physical_damage_reduction_rating_+%",
            8.0,
            src(),
        );
        assert_eq!(r.len(), 2);
        assert_mod(&r[0], StatId::Evasion, ModType::Inc, 8.0, ModFlag::empty());
        assert_mod(&r[1], StatId::Armour, ModType::Inc, 8.0, ModFlag::empty());
    }

    // ── Unknown IDs return empty ──

    #[test]
    fn unknown_stat_returns_empty() {
        assert!(resolve("gain_x_rage_on_melee_hit", 1.0, src()).is_empty());
        assert!(resolve("max_frenzy_charges", 1.0, src()).is_empty());
        assert!(resolve("display_can_take_character_start_point", 1.0, src()).is_empty());
    }
}
