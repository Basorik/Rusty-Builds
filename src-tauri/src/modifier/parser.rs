use crate::data::{ StatId, SourceId };
use super::types::*;
use smallvec::smallvec;
use rustc_hash::FxHashMap;
use regex::Regex;
use std::sync:: OnceLock;

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
        m.insert("+# to Strength",     |v, s| vec![simple_mod(StatId::Strength, ModType::Base, v[0], s)]);
        m.insert("+# to Dexterity",    |v, s| vec![simple_mod(StatId::Dexterity, ModType::Base, v[0], s)]);
        m.insert("+# to Intelligence", |v, s| vec![simple_mod(StatId::Intelligence, ModType::Base, v[0], s)]);
        m.insert("+# to maximum Life", |v, s| vec![simple_mod(StatId::Life, ModType::Base, v[0], s)]);
        m.insert("+# to maximum Mana", |v, s| vec![simple_mod(StatId::Mana, ModType::Base, v[0], s)]);
        m.insert("+# to maximum Energy Shield", |v, s| vec![simple_mod(StatId::EnergyShield, ModType::Base, v[0], s)]);
        m.insert("+# to Accuracy Rating", |v, s| vec![simple_mod(StatId::Accuracy, ModType::Base, v[0], s)]);

        // ── Simple Inc: "X% increased <Stat>" ──
        m.insert("#% increased maximum Life",          |v, s| vec![simple_mod(StatId::Life, ModType::Inc, v[0], s)]);
        m.insert("#% increased maximum Mana",          |v, s| vec![simple_mod(StatId::Mana, ModType::Inc, v[0], s)]);
        m.insert("#% increased maximum Energy Shield", |v, s| vec![simple_mod(StatId::EnergyShield, ModType::Inc, v[0], s)]);
        m.insert("#% increased Evasion Rating",        |v, s| vec![simple_mod(StatId::Evasion, ModType::Inc, v[0], s)]);
        m.insert("#% increased Armour",                |v, s| vec![simple_mod(StatId::Armour, ModType::Inc, v[0], s)]);
        m.insert("#% increased Attack Speed",          |v, s| vec![flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::ATTACK)]);
        m.insert("#% increased Cast Speed",            |v, s| vec![flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::CAST)]);
        m.insert("#% increased Movement Speed",        |v, s| vec![simple_mod(StatId::MovementSpeed, ModType::Inc, v[0], s)]);
        m.insert("#% increased Physical Damage",       |v, s| vec![simple_mod(StatId::PhysicalDamage, ModType::Inc, v[0], s)]);
        m.insert("#% increased Spell Damage",          |v, s| vec![flagged_mod(StatId::Damage, ModType::Inc, v[0], s, ModFlag::SPELL)]);
        m.insert("#% increased Elemental Damage",      |v, s| vec![simple_mod(StatId::ElementalDamage, ModType::Inc, v[0], s)]);
        m.insert("#% increased Critical Strike Chance", |v, s| vec![simple_mod(StatId::CritChance, ModType::Inc, v[0], s)]);
        m.insert("#% increased Mana Regeneration Rate", |v, s| vec![simple_mod(StatId::ManaRegeneration, ModType::Inc, v[0], s)]);
        m.insert("#% increased Projectile Damage",     |v, s| vec![flagged_mod(StatId::Damage, ModType::Inc, v[0], s, ModFlag::PROJECTILE)]);

        // ── Resistances: "+X% to <Element> Resistance" ──
        m.insert("+#% to Fire Resistance",      |v, s| vec![simple_mod(StatId::FireResist, ModType::Base, v[0], s)]);
        m.insert("+#% to Cold Resistance",      |v, s| vec![simple_mod(StatId::ColdResist, ModType::Base, v[0], s)]);
        m.insert("+#% to Lightning Resistance", |v, s| vec![simple_mod(StatId::LightningResist, ModType::Base, v[0], s)]);
        m.insert("+#% to Chaos Resistance",     |v, s| vec![simple_mod(StatId::ChaosResist, ModType::Base, v[0], s)]);
        // Expands to 3 modifiers
        m.insert("+#% to all Elemental Resistances", |v, s| vec![
            simple_mod(StatId::FireResist, ModType::Base, v[0], s),
            simple_mod(StatId::ColdResist, ModType::Base, v[0], s),
            simple_mod(StatId::LightningResist, ModType::Base, v[0], s),
        ]);

        // ── Multi-stat expansion: "X% increased A and B" ──
        m.insert("#% increased Evasion Rating and Armour", |v, s| vec![
            simple_mod(StatId::Evasion, ModType::Inc, v[0], s),
            simple_mod(StatId::Armour, ModType::Inc, v[0], s),
        ]);
        m.insert("#% increased Attack and Cast Speed", |v, s| vec![
            flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::ATTACK),
            flagged_mod(StatId::Speed, ModType::Inc, v[0], s, ModFlag::CAST),
        ]);

        // ── Dual-attribute: "+X to A and B" ──
        m.insert("+# to Strength and Intelligence", |v, s| vec![
            simple_mod(StatId::Strength, ModType::Base, v[0], s),
            simple_mod(StatId::Intelligence, ModType::Base, v[0], s),
        ]);
        m.insert("+# to Strength and Dexterity", |v, s| vec![
            simple_mod(StatId::Strength, ModType::Base, v[0], s),
            simple_mod(StatId::Dexterity, ModType::Base, v[0], s),
        ]);
        m.insert("+# to Dexterity and Intelligence", |v, s| vec![
            simple_mod(StatId::Dexterity, ModType::Base, v[0], s),
            simple_mod(StatId::Intelligence, ModType::Base, v[0], s),
        ]);

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
pub fn parse_display_text(text: &str, source: SourceId) -> Vec<Modifier> {
    let re = re_number();

    // Step 1: extract all numbers
    let values: Vec<f64> = re.find_iter(text)
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    // Step 2: replace numbers with # to form template key
    let template = re.replace_all(text, "#");

    // Step 3: O(1) lookup
    if let Some(handler) = template_map().get(template.as_ref()) {
        // Step 4: call handler
        handler(&values, source)
    } else {
        log::debug!("Unrecognized stat template: {:?} (from {:?})", template, text);
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
    stat: StatId, mod_type: ModType, value: f64, source: SourceId,
    tags: smallvec::SmallVec<[ModTag; 2]>,
) -> Modifier {
    Modifier { stat, mod_type, value, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), source, tags }
}

/// Helper: build a modifier scoped to specific ModFlag(s) (e.g. Attack/Cast/Spell).
fn flagged_mod(stat: StatId, mod_type: ModType, value: f64, source: SourceId, flags: ModFlag) -> Modifier {
    Modifier { stat, mod_type, value, flags, keywords: KeywordFlag::empty(), source, tags: smallvec![] }
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