//! PoB parity integration tests.
//!
//! Each test loads a JSON fixture from `tests/fixtures/`, calls `calculate()`
//! with the described build configuration, and asserts that key stats match
//! the known-good values recorded from Path of Building.
//!
//! Tolerance:
//!   - Integer stats (life, mana, ES, attributes): ±1
//!   - Resistance values: ±0 (integer, exact)
//!   - DPS / percentages: ±0.1%
//!
//! # Adding a new reference build
//!
//! 1. Set up the build in PoB 3.27.0g, note all stats from the Character panel.
//! 2. Create `tests/fixtures/<descriptive_name>.json` (copy an existing fixture).
//! 3. Fill in `class`, `level`, `node_ids`, and `expected`.
//! 4. Add a `#[test]` fn that calls `run_fixture("descriptive_name")`.

use rusty_builds_lib::{
    calc::calculate,
    data::{Class, GameData},
    item::types::{Item, ItemSlot},
    modifier::ModDBLayers,
};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::OnceLock;

// ──────────────────────────────────────────────────────────────────────────────
// One-time GameData load (shared across all tests in this binary)
// ──────────────────────────────────────────────────────────────────────────────

fn game_data() -> &'static GameData {
    static GD: OnceLock<GameData> = OnceLock::new();
    GD.get_or_init(|| {
        let resource_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        GameData::load_from_dir(resource_dir).expect("pob_parity: failed to load GameData")
    })
}

// ──────────────────────────────────────────────────────────────────────────────
// Fixture schema
// ──────────────────────────────────────────────────────────────────────────────

/// JSON schema for one reference build fixture.
/// Keys prefixed with `_` are ignored (treated as documentation comments).
#[derive(Debug, Deserialize)]
struct Fixture {
    class: String,
    level: u32,
    node_ids: Vec<u32>,
    expected: ExpectedStats,
}

#[derive(Debug, Deserialize)]
struct ExpectedStats {
    life: Option<u32>,
    mana: Option<u32>,
    energy_shield: Option<u32>,
    fire_resist: Option<i32>,
    cold_resist: Option<i32>,
    lightning_resist: Option<i32>,
    chaos_resist: Option<i32>,
    strength: Option<i32>,
    dexterity: Option<i32>,
    intelligence: Option<i32>,
    // Optional offence fields (added when fixtures include skill/gem data)
    total_dps: Option<f64>,
    crit_chance: Option<f64>,
    hit_chance: Option<f64>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Fixture runner
// ──────────────────────────────────────────────────────────────────────────────

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(format!("{name}.json"))
}

fn parse_class(s: &str) -> Class {
    match s {
        "Marauder" => Class::Marauder(None),
        "Ranger" => Class::Ranger(None),
        "Witch" => Class::Witch(None),
        "Duelist" => Class::Duelist(None),
        "Templar" => Class::Templar(None),
        "Shadow" => Class::Shadow(None),
        "Scion" => Class::Scion(None),
        other => panic!("Unknown class in fixture: {other}"),
    }
}

/// Load `tests/fixtures/<name>.json`, run `calculate()`, assert expected stats.
fn run_fixture(name: &str) {
    let path = fixture_path(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read fixture {name}: {e}"));
    let fixture: Fixture =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("Bad JSON in {name}: {e}"));

    let gd = game_data();
    let class = parse_class(&fixture.class);
    let equipped: FxHashMap<ItemSlot, Item> = FxHashMap::default();

    let mut layers = ModDBLayers::default();
    layers.rebuild_class(&class, &gd.tree);
    layers.rebuild_tree(&fixture.node_ids, gd);

    let result = calculate(&layers, fixture.level, &class, None, &[], &equipped, gd);
    let def = &result.defense;
    let attrs = &result.attributes;
    let exp = &fixture.expected;

    // ── integer stats (±1 tolerance) ───────────────────────────────────
    if let Some(expected_life) = exp.life {
        let diff = (def.life as i64 - expected_life as i64).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] life: expected {expected_life} ±1, got {}",
            def.life
        );
    }
    if let Some(expected_mana) = exp.mana {
        let diff = (def.mana as i64 - expected_mana as i64).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] mana: expected {expected_mana} ±1, got {}",
            def.mana
        );
    }
    if let Some(expected_es) = exp.energy_shield {
        let diff = (def.energy_shield as i64 - expected_es as i64).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] energy_shield: expected {expected_es} ±1, got {}",
            def.energy_shield
        );
    }

    // ── resistance values (exact) ──────────────────────────────────────
    if let Some(e) = exp.fire_resist {
        assert_eq!(def.fire_resist, e, "[{name}] fire_resist");
    }
    if let Some(e) = exp.cold_resist {
        assert_eq!(def.cold_resist, e, "[{name}] cold_resist");
    }
    if let Some(e) = exp.lightning_resist {
        assert_eq!(def.lightning_resist, e, "[{name}] lightning_resist");
    }
    if let Some(e) = exp.chaos_resist {
        assert_eq!(def.chaos_resist, e, "[{name}] chaos_resist");
    }

    // ── attributes (±1 tolerance) ──────────────────────────────────────
    if let Some(e) = exp.strength {
        let diff = (attrs.strength - e).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] strength: expected {e} ±1, got {}",
            attrs.strength
        );
    }
    if let Some(e) = exp.dexterity {
        let diff = (attrs.dexterity - e).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] dexterity: expected {e} ±1, got {}",
            attrs.dexterity
        );
    }
    if let Some(e) = exp.intelligence {
        let diff = (attrs.intelligence - e).unsigned_abs();
        assert!(
            diff <= 1,
            "[{name}] intelligence: expected {e} ±1, got {}",
            attrs.intelligence
        );
    }

    // ── offence (±0.1% relative tolerance) ────────────────────────────
    if let Some(expected_dps) = exp.total_dps {
        let off = &result.offence;
        let rel_diff = ((off.total_dps - expected_dps) / expected_dps.max(1.0)).abs() * 100.0;
        assert!(
            rel_diff <= 0.1,
            "[{name}] total_dps: expected {expected_dps} ±0.1%, got {} (diff={rel_diff:.4}%)",
            off.total_dps
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Reference builds
// ──────────────────────────────────────────────────────────────────────────────

/// Level 90 Marauder with zero tree nodes selected.
/// Verifies: life formula with STR class base, mana, attributes, bare resists.
/// To confirm expected values: load PoB 3.27.0g → new Marauder build → level 90 → 0 nodes.
#[test]
fn pob_parity_marauder_base_level90() {
    run_fixture("marauder_base_level90");
}
