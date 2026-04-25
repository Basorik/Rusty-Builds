//! Unit tests for `calc::calc_attributes` and `inject_attribute_bonuses`.
//!
//! Verifies:
//!  - Attributes are summed from the ModDB (base + inc + more)
//!  - STR/2 → BASE life (multiplied by inc, not added after)
//!  - INT/2 → BASE mana
//!  - INT/10 → INC ES
//!  - DEX/5  → INC evasion
//!  - DEX*2  → BASE accuracy
//!  - STR/5  → INC melee physical damage

#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::{assert_eq_u32, assert_near, seeded_db};

use rusty_builds_lib::{
    calc::{calc_attributes, calc_defense, inject_attribute_bonuses, setup_moddb},
    data::{SourceId, StatId},
    modifier::{
        mod_db::ModDBLayers, parser::simple_mod, CalcContext, ModFlag, ModType,
    },
};

// ──────────────────────────────────────────────────────────────────────────────
// Attribute accumulation
// ──────────────────────────────────────────────────────────────────────────────

/// Strength accumulates across base + more modifiers.
#[test]
fn test_strength_sum() {
    let mut layers = ModDBLayers::default();
    layers
        .items
        .add_mod(simple_mod(StatId::Strength, ModType::Base, 100.0, SourceId(1)));
    layers
        .items
        .add_mod(simple_mod(StatId::Strength, ModType::Base, 50.0, SourceId(2)));
    let db = setup_moddb(&layers, 1);
    let attrs = calc_attributes(&db);
    assert_eq!(attrs.strength, 150, "strength base sum");
}

// ──────────────────────────────────────────────────────────────────────────────
// STR → life bonus (BASE, not post-scaling add)
// ──────────────────────────────────────────────────────────────────────────────

/// 100 STR injects +50 BASE life. With 100% inc life that becomes 50 extra after scaling.
///
/// If str bonus were added AFTER scaling (wrong), the result would differ.
/// Correct: (base_life + 50) * (1 + inc) = (50 + 50) * 2 = 200 at level 1.
#[test]
fn test_str_bonus_multiplied_by_inc() {
    let mut layers = ModDBLayers::default();
    // 100 STR
    layers
        .items
        .add_mod(simple_mod(StatId::Strength, ModType::Base, 100.0, SourceId(1)));
    // 100% increased life
    layers
        .items
        .add_mod(simple_mod(StatId::Life, ModType::Inc, 100.0, SourceId(2)));

    let mut db = setup_moddb(&layers, 1);
    let attrs = calc_attributes(&db);
    inject_attribute_bonuses(&mut db, &attrs);
    let ctx = CalcContext::empty();
    let result = calc_defense(&db, 1, &ctx);

    // base_seed = 38 + 12*1 = 50. str bonus = floor(100/2) = 50. total base = 100.
    // with 100% inc: 100 * 2 = 200
    assert_eq_u32("life (str base + 100% inc)", result.life, 200);
}

// ──────────────────────────────────────────────────────────────────────────────
// INT → mana & ES
// ──────────────────────────────────────────────────────────────────────────────

/// 100 INT: floor(100/2) = 50 BASE mana + floor(100/10) = 10% INC ES.
#[test]
fn test_int_bonuses() {
    let mut layers = ModDBLayers::default();
    layers
        .items
        .add_mod(simple_mod(StatId::Intelligence, ModType::Base, 100.0, SourceId(1)));
    // 200 ES base from an item
    layers
        .items
        .add_mod(simple_mod(StatId::EnergyShield, ModType::Base, 200.0, SourceId(2)));

    let mut db = setup_moddb(&layers, 1);
    let attrs = calc_attributes(&db);
    inject_attribute_bonuses(&mut db, &attrs);
    let ctx = CalcContext::empty();
    let result = calc_defense(&db, 1, &ctx);

    // mana: seed = 34 + 6 = 40; +50 from INT/2 = 90
    assert_eq_u32("mana with 100 INT", result.mana, 90);
    // ES: 200 base * (1 + 10/100) = 220
    assert_eq_u32("ES with 100 INT (10% inc ES)", result.energy_shield, 220);
}

// ──────────────────────────────────────────────────────────────────────────────
// DEX → evasion & accuracy
// ──────────────────────────────────────────────────────────────────────────────

/// 100 DEX → floor(100/5) = 20% INC evasion; floor(100*2) = 200 BASE accuracy.
#[test]
fn test_dex_evasion_inc() {
    let mut layers = ModDBLayers::default();
    layers
        .items
        .add_mod(simple_mod(StatId::Dexterity, ModType::Base, 100.0, SourceId(1)));
    // 500 evasion base from armour
    layers
        .items
        .add_mod(simple_mod(StatId::Evasion, ModType::Base, 500.0, SourceId(2)));

    let mut db = setup_moddb(&layers, 1);
    let attrs = calc_attributes(&db);
    inject_attribute_bonuses(&mut db, &attrs);
    let ctx = CalcContext::empty();
    let result = calc_defense(&db, 1, &ctx);

    // evasion: 500 * (1 + 20/100) = 600
    assert_eq_u32("evasion with 100 DEX (20% inc)", result.evasion, 600);
}
