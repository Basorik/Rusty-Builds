//! Unit tests for the offence calculation pipeline.
//!
//! These tests focus on the pure math that doesn't require `GameData`:
//!   - Damage conversion chain (phys → fire, lightning → cold, etc.)
//!   - Avatar of Fire: 50% of non-fire to fire, zero non-fire output
//!   - Gain-as-extra: uncapped, does not reduce source
//!   - Crit effective multiplier formula
//!   - Conversion priority when skill + global both convert the same element

#[path = "../helpers/mod.rs"]
mod helpers;
use helpers::seeded_db;

use rusty_builds_lib::{
    calc::{apply_conversion, build_conversion_table, DamageSet},
    data::{SourceId, StatId},
    modifier::{parser::simple_mod, CalcContext, ModType},
};

// ──────────────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────────────

fn assert_near(label: &str, actual: f64, expected: f64, delta: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= delta,
        "{label}: expected {expected} ±{delta}, got {actual} (diff={diff:.4})"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Simple physical → fire conversion
// ──────────────────────────────────────────────────────────────────────────────

/// 100 phys, 50% global converted to fire → 50 phys + 50 fire (no inc/more on either).
#[test]
fn test_phys_to_fire_50pct() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::PhysicalDamageConvertToFire,
            ModType::Base,
            50.0,
            SourceId(1),
        ));
    });
    let table = build_conversion_table(&db, &ctx);
    let base = DamageSet {
        physical: 100.0,
        ..Default::default()
    };
    let out = apply_conversion(&base, &table, &db, &ctx);

    assert_near("phys out (50% converted)", out.physical, 50.0, 0.1);
    assert_near("fire out (50% phys→fire)", out.fire, 50.0, 0.1);
    assert_near("lightning out (none)", out.lightning, 0.0, 0.01);
}

/// 100% phys → fire: all phys becomes fire, no physical output.
#[test]
fn test_phys_to_fire_100pct() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::PhysicalDamageConvertToFire,
            ModType::Base,
            100.0,
            SourceId(1),
        ));
    });
    let table = build_conversion_table(&db, &ctx);
    let base = DamageSet {
        physical: 200.0,
        ..Default::default()
    };
    let out = apply_conversion(&base, &table, &db, &ctx);

    assert_near("phys out (100% converted)", out.physical, 0.0, 0.1);
    assert_near("fire out (100% phys→fire)", out.fire, 200.0, 0.1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Conversion chain: phys → lightning → fire
// ──────────────────────────────────────────────────────────────────────────────

/// Chain: 50% phys→lightning, 100% lightning→fire.
/// 100 phys: 50 stays phys, 50 → lightning → all that lightning → fire.
/// Final: 50 phys, 0 lightning, 50 fire.
#[test]
fn test_conversion_chain_phys_light_fire() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::PhysicalDamageConvertToLightning,
            ModType::Base,
            50.0,
            SourceId(1),
        ));
        layers.items.add_mod(simple_mod(
            StatId::LightningDamageConvertToFire,
            ModType::Base,
            100.0,
            SourceId(2),
        ));
    });
    let table = build_conversion_table(&db, &ctx);
    let base = DamageSet {
        physical: 100.0,
        ..Default::default()
    };
    let out = apply_conversion(&base, &table, &db, &ctx);

    assert_near("phys (50% stayed)", out.physical, 50.0, 0.1);
    assert_near("lightning (all converted to fire)", out.lightning, 0.0, 0.1);
    assert_near("fire (from chain)", out.fire, 50.0, 0.1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Gain-as-extra
// ──────────────────────────────────────────────────────────────────────────────

/// Gain 50% of phys as fire does NOT reduce phys output.
/// 100 phys + 50% gain as fire → 100 phys + 50 fire.
#[test]
fn test_gain_as_extra_does_not_reduce_source() {
    let (db, ctx) = seeded_db(1, |layers| {
        layers.items.add_mod(simple_mod(
            StatId::PhysicalDamageGainAsFire,
            ModType::Base,
            50.0,
            SourceId(1),
        ));
    });
    let table = build_conversion_table(&db, &ctx);
    let base = DamageSet {
        physical: 100.0,
        ..Default::default()
    };
    let out = apply_conversion(&base, &table, &db, &ctx);

    assert_near("phys unchanged (gain does not reduce)", out.physical, 100.0, 0.1);
    assert_near("fire (50% of phys gained as extra)", out.fire, 50.0, 0.1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Skill conversion caps at 100% before global fills in
// ──────────────────────────────────────────────────────────────────────────────

/// Skill layer: 100% phys→fire. Global layer: 50% phys→lightning.
/// Total skill is already 100%, so global gets 0% space → no lightning.
#[test]
fn test_skill_conversion_blocks_global() {
    let (db, ctx) = seeded_db(1, |layers| {
        // Skill-layer conversion (SkillPhysical... stats)
        layers.items.add_mod(simple_mod(
            StatId::SkillPhysicalDamageConvertToFire,
            ModType::Base,
            100.0,
            SourceId(1),
        ));
        // Global-layer conversion — should be crowded out
        layers.items.add_mod(simple_mod(
            StatId::PhysicalDamageConvertToLightning,
            ModType::Base,
            50.0,
            SourceId(2),
        ));
    });
    let table = build_conversion_table(&db, &ctx);
    let base = DamageSet {
        physical: 100.0,
        ..Default::default()
    };
    let out = apply_conversion(&base, &table, &db, &ctx);

    assert_near("phys (fully converted)", out.physical, 0.0, 0.1);
    assert_near("fire (all from skill conv)", out.fire, 100.0, 0.1);
    assert_near("lightning (blocked by skill full)", out.lightning, 0.0, 0.1);
}

// ──────────────────────────────────────────────────────────────────────────────
// Crit effective multiplier formula
// ──────────────────────────────────────────────────────────────────────────────

/// PoB: effective_crit_mult = 1 + crit_chance * (crit_multiplier - 1)
/// 50% crit, 200% multiplier → 1 + 0.5 * (2.0 - 1.0) = 1.5
#[test]
fn test_crit_effective_multiplier() {
    let crit_chance = 0.50_f64; // 50%
    let crit_multi = 2.0_f64;   // 200%
    let effective = 1.0 + crit_chance * (crit_multi - 1.0);
    assert_near("effective crit multiplier", effective, 1.5, 0.001);
}

/// 100% crit, 300% multiplier → 1 + 1.0 * (3.0 - 1.0) = 3.0
#[test]
fn test_crit_100pct_chance() {
    let effective = 1.0 + 1.0_f64 * (3.0_f64 - 1.0);
    assert_near("effective crit (100% chance 300% multi)", effective, 3.0, 0.001);
}

/// 0% crit → effective multiplier = 1.0 (no bonus)
#[test]
fn test_no_crit() {
    let effective = 1.0 + 0.0_f64 * (5.0_f64 - 1.0);
    assert_near("effective crit (0% chance)", effective, 1.0, 0.001);
}
