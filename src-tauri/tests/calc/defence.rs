//! Unit tests for `calc::calc_defense`.
//!
//! Each test builds a minimal `ModDB` (seeded with level-based base values via
//! `setup_moddb`) and verifies one specific formula against the expected value
//! documented in PoB's CalcDefence.lua.
//!
//! Tolerance: ±1 for integer stats, ±0.1 for f64 stats.

#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::{assert_eq_u32, assert_near, seeded_db};

use rusty_builds_lib::{
    calc::calc_defense,
    data::{SourceId, StatId},
    modifier::{parser::simple_mod, ModType},
};

// ──────────────────────────────────────────────────────────────────────────────
// Life
// ──────────────────────────────────────────────────────────────────────────────

/// PoB life formula: `(38 + 12*level + flat + floor(str/2)) * (1 + inc/100) * more`
///
/// Level 90 Marauder with 0 STR bonus, 200% increased life.
/// Expected: (38 + 90*12 + 0) * (1 + 200/100) = 1118 * 3 = 3354
#[test]
fn test_life_inc_only() {
    let (db, ctx) = seeded_db(90, |layers| {
        layers
            .items
            .add_mod(simple_mod(StatId::Life, ModType::Inc, 200.0, SourceId(1)));
    });
    let result = calc_defense(&db, 90, &ctx);
    // base = 38 + 90*12 = 1118; *3 for 200% inc → 3354
    assert_eq_u32("life with 200% inc", result.life, 3354);
}

/// STR/2 bonus is injected as a BASE mod BEFORE the inc/more multiplication.
/// Level 1, 100 STR, 0% inc: life = (38 + 12 + 50) * 1 = 100
#[test]
fn test_str_life_bonus_is_base() {
    let (db, ctx) = seeded_db(1, |layers| {
        // inject STR bonus as attribute::inject_attribute_bonuses would
        layers
            .items
            .add_mod(simple_mod(StatId::Life, ModType::Base, 50.0, SourceId(1)));
    });
    let result = calc_defense(&db, 1, &ctx);
    // base from seed = 38 + 12*1 = 50; +50 from str bonus = 100
    assert_eq_u32("life with 100 STR bonus (base)", result.life, 100);
}

/// 200% inc life + str bonus both scale together.
/// Level 1, +50 STR bonus, 200% inc → (50 + 50) * 3 = 300
#[test]
fn test_str_bonus_scales_with_inc() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers
            .items
            .add_mod(simple_mod(StatId::Life, ModType::Base, 50.0, SourceId(1)));
        layers
            .items
            .add_mod(simple_mod(StatId::Life, ModType::Inc, 200.0, SourceId(2)));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_u32("life: str+inc both scale", result.life, 300);
}

// ──────────────────────────────────────────────────────────────────────────────
// Mana
// ──────────────────────────────────────────────────────────────────────────────

/// PoB mana formula: `(34 + 6*level + floor(int/2)) * (1 + inc/100) * more`
/// Level 50, 0 INT, 100% inc: (34 + 300) * 2 = 668
#[test]
fn test_mana_inc() {
    let (db, ctx) = seeded_db(50, |layers| {
        layers
            .items
            .add_mod(simple_mod(StatId::Mana, ModType::Inc, 100.0, SourceId(1)));
    });
    let result = calc_defense(&db, 50, &ctx);
    // base = 34 + 50*6 = 334; *2 = 668
    assert_eq_u32("mana with 100% inc", result.mana, 668);
}

// ──────────────────────────────────────────────────────────────────────────────
// Resistance caps & overcap
// ──────────────────────────────────────────────────────────────────────────────

/// Default cap is 75. With +135% fire resist (from mods): net = -60 + 135 = 75 (capped).
/// Overcap = 135 - 60 - 75 = 0.  Wait — total = -60 + 135 = 75, cap = 75, overcap = 0.
#[test]
fn test_resist_at_cap() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::FireResist,
            ModType::Base,
            135.0,
            SourceId(1),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_stat("fire resist at cap", result.fire_resist, 75);
    assert_eq_stat("fire resist cap", result.fire_resist_cap, 75);
    assert_eq_stat("fire overcap at exactly cap", result.fire_resist_overcap, 0);
}

/// 135% fire resist gives overcap of 60 when the base penalty (-60) already fills the gap.
/// total = -60 + 135 = 75 (hits cap exactly). But if +195 fire: total = -60+195 = 135 → overcap = 60.
#[test]
fn test_resist_overcap() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::FireResist,
            ModType::Base,
            195.0,
            SourceId(1),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_stat(
        "fire resist (overcapped, effective = cap)",
        result.fire_resist,
        75,
    );
    assert_eq_stat("fire overcap = 60", result.fire_resist_overcap, 60);
}

/// Negative resist: with only the -60 penalty and no compensating mods, fire = -60.
#[test]
fn test_resist_negative() {
    let (db, ctx) = seeded_db(1, |_| {});
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_stat("fire resist (bare, -60 penalty)", result.fire_resist, -60);
    assert_eq_stat("cold resist (bare, -60 penalty)", result.cold_resist, -60);
    assert_eq_stat(
        "lightning resist (bare, -60 penalty)",
        result.lightning_resist,
        -60,
    );
}

/// Raised cap: "+8% to maximum Fire Resistance" → cap = 83; total of 78% goes through uncapped.
#[test]
fn test_raised_resist_cap() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::FireResist,
            ModType::Base,
            138.0,
            SourceId(1),
        ));
        layers.items.add_mod(simple_mod(
            StatId::FireResistMax,
            ModType::Base,
            8.0,
            SourceId(2),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    // total = -60 + 138 = 78; cap = 75 + 8 = 83; effective = 78, overcap = 0
    assert_eq_stat("fire resist with raised cap", result.fire_resist, 78);
    assert_eq_stat("fire resist cap raised to 83", result.fire_resist_cap, 83);
    assert_eq_stat(
        "fire overcap (under new cap)",
        result.fire_resist_overcap,
        0,
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Chaos Inoculation
// ──────────────────────────────────────────────────────────────────────────────

/// CI → life = 1 regardless of other mods.
#[test]
fn test_chaos_inoculation_life() {
    let (db, ctx) = seeded_db(90, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::ChaosInoculation,
            ModType::Flag,
            1.0,
            SourceId(1),
        ));
        layers
            .items
            .add_mod(simple_mod(StatId::Life, ModType::Inc, 300.0, SourceId(2)));
    });
    let result = calc_defense(&db, 90, &ctx);
    assert_eq_u32("CI forces life = 1", result.life, 1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Iron Reflexes
// ──────────────────────────────────────────────────────────────────────────────

/// Iron Reflexes: all evasion converts to armour, evasion → 0.
#[test]
fn test_iron_reflexes_converts_evasion() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::IronReflexes,
            ModType::Flag,
            1.0,
            SourceId(1),
        ));
        layers.items.add_mod(simple_mod(
            StatId::Evasion,
            ModType::Base,
            1000.0,
            SourceId(2),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_u32("evasion = 0 with IR", result.evasion, 0);
    assert_eq_u32("armour = evasion with IR", result.armour, 1000);
}

// ──────────────────────────────────────────────────────────────────────────────
// Block chance cap
// ──────────────────────────────────────────────────────────────────────────────

/// Block chance is capped at 75% by default.
#[test]
fn test_block_capped_at_75() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::BlockChance,
            ModType::Base,
            90.0,
            SourceId(1),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_near("block capped at 75", result.block_chance, 75.0, 0.1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Energy Shield
// ──────────────────────────────────────────────────────────────────────────────

/// 100 ES base + 100% inc = 200 ES.
#[test]
fn test_es_inc() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::EnergyShield,
            ModType::Base,
            100.0,
            SourceId(1),
        ));
        layers.items.add_mod(simple_mod(
            StatId::EnergyShield,
            ModType::Inc,
            100.0,
            SourceId(2),
        ));
    });
    let result = calc_defense(&db, 1, &ctx);
    assert_eq_u32("ES with 100% inc", result.energy_shield, 200);
}

// ──────────────────────────────────────────────────────────────────────────────
// helpers re-export (each integration test file must declare the module itself)
// ──────────────────────────────────────────────────────────────────────────────
fn assert_eq_stat(label: &str, actual: i32, expected: i32) {
    assert_eq!(
        actual, expected,
        "{label}: expected {expected}, got {actual}"
    );
}
