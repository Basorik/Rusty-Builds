//! Shared test utilities for calc integration tests.
//!
//! Two levels of helpers:
//!
//! * [`ModDbBuilder`] — builds a bare `ModDB` manually (no `GameData` needed).
//!   Use this for pure unit tests that verify a single formula in isolation.
//!
//! * [`SeededModDb`] — calls `setup_moddb` on top of `ModDbBuilder` so that
//!   the standard level-based seeds (life base, resistance penalty, etc.) are
//!   already present.  Lets tests focus on the one mod they're interested in.

#![allow(dead_code)]

use rusty_builds_lib::{
    calc::setup_moddb,
    data::{SourceId, StatId},
    modifier::{
        mod_db::{ModDB, ModDBLayers},
        parser::simple_mod,
        CalcContext, ModType,
    },
};

// ──────────────────────────────────────────────────────────────────────────────
// ModDbBuilder
// ──────────────────────────────────────────────────────────────────────────────

/// Fluent builder that assembles a `ModDB` for tests.
///
/// ```rust
/// let db = ModDbBuilder::new()
///     .add_base(StatId::Life, 500.0)
///     .add_inc(StatId::Life, 120.0)
///     .add_flag(StatId::ChaosInoculation)
///     .build();
/// ```
pub struct ModDbBuilder {
    db: ModDB,
    src: SourceId,
}

impl ModDbBuilder {
    pub fn new() -> Self {
        Self {
            db: ModDB::new(),
            src: SourceId(999),
        }
    }

    pub fn add_base(mut self, stat: StatId, value: f64) -> Self {
        self.db
            .add_mod(simple_mod(stat, ModType::Base, value, self.src));
        self
    }

    pub fn add_inc(mut self, stat: StatId, value: f64) -> Self {
        self.db
            .add_mod(simple_mod(stat, ModType::Inc, value, self.src));
        self
    }

    pub fn add_more(mut self, stat: StatId, value: f64) -> Self {
        self.db
            .add_mod(simple_mod(stat, ModType::More, value, self.src));
        self
    }

    pub fn add_flag(mut self, stat: StatId) -> Self {
        self.db
            .add_mod(simple_mod(stat, ModType::Flag, 1.0, self.src));
        self
    }

    /// Finalise and return the `ModDB`.
    pub fn build(self) -> ModDB {
        self.db
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SeededModDb — level-seeded DB ready for calc function tests
// ──────────────────────────────────────────────────────────────────────────────

/// Returns a `ModDB` that has already been seeded with level-based base values
/// (life/mana base, resistance penalties/caps, block caps, charge maxes, etc.)
/// from `setup_moddb`, plus any additional mods you supply via `extra`.
///
/// `level` — character level (controls seeded life/mana pools).
/// `extra`  — closure that receives a mutable `ModDBLayers` for the `items`
///            layer — add any flat/inc/more mods that the test needs.
///
/// # Example
///
/// ```rust
/// // Level 90 character with +200% increased life
/// let (db, ctx) = seeded_db(90, |layers| {
///     layers.items.add_mod(simple_mod(StatId::Life, ModType::Inc, 200.0, SourceId(1)));
/// });
/// let life = db.calculate(StatId::Life, &ctx);
/// ```
pub fn seeded_db(level: u32, extra: impl FnOnce(&mut ModDBLayers)) -> (ModDB, CalcContext) {
    let mut layers = ModDBLayers::default();
    extra(&mut layers);
    let db = setup_moddb(&layers, level);
    let ctx = CalcContext::empty();
    (db, ctx)
}

// ──────────────────────────────────────────────────────────────────────────────
// Tolerance helpers
// ──────────────────────────────────────────────────────────────────────────────

/// Assert two f64 values are within `±delta`.  Panics with a descriptive
/// message on failure so test output is immediately useful.
pub fn assert_near(label: &str, actual: f64, expected: f64, delta: f64) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= delta,
        "{label}: expected {expected} ±{delta}, got {actual} (diff={diff:.4})"
    );
}

/// Assert an integer stat matches exactly (±0).
pub fn assert_eq_stat(label: &str, actual: i32, expected: i32) {
    assert_eq!(
        actual, expected,
        "{label}: expected {expected}, got {actual}"
    );
}

/// Assert a u32 stat matches exactly.
pub fn assert_eq_u32(label: &str, actual: u32, expected: u32) {
    assert_eq!(
        actual, expected,
        "{label}: expected {expected}, got {actual}"
    );
}
