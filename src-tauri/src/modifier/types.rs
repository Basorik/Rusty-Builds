use std::sync::RwLock;
use bitflags::bitflags;
use rustc_hash::FxHashSet;
use smallvec::SmallVec;
use crate::data::{StatId, SourceId};

// ── String interner ─────────────────────────────────────────────────
// ModTag strings (condition vars, multiplier vars, stat names, effect names)
// are a fixed set loaded once from game data. We intern them into a global
// set so every ModTag stores a `&'static str` — no per-tag heap allocation,
// cheap Clone (Copy), and O(1) equality via pointer comparison.

static INTERNED: RwLock<Option<FxHashSet<&'static str>>> = RwLock::new(None);

/// Intern a string, returning a `&'static str` that lives for the process lifetime.
/// Identical strings return the same pointer. Thread-safe.
pub fn intern(s: &str) -> &'static str {
    // Fast path: already interned
    {
        let guard = INTERNED.read().unwrap();
        if let Some(set) = guard.as_ref() {
            if let Some(&existing) = set.get(s) {
                return existing;
            }
        }
    }
    // Slow path: allocate and insert
    let mut guard = INTERNED.write().unwrap();
    let set = guard.get_or_insert_with(FxHashSet::default);
    // Double-check after acquiring write lock
    if let Some(&existing) = set.get(s) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    set.insert(leaked);
    leaked
}

// General type for mod - controls how the mod is accumulated
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModType {
    Base,      // +X flat (e.g., "+20 to Strength")
    Inc,       // X% increased (additive with other increases) — PoB calls this "INC"
    More,      // X% more (multiplicative, applied separately)
    Flag,      // Boolean flag (e.g., "Cannot be Stunned")
    Override,  // Overrides the stat entirely
    List,      // List-type mod (appended, not summed)
    Max,       // Only the highest value takes effect (e.g., "PoisonStackLimit")
    Min,       // Only the lowest value takes effect
    Chance,    // X% chance (e.g., "20% chance to Poison") — PoB calls this "CHANCE"
}

// Flags for actions that the mod applies to
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ModFlag: u32 {
        const ATTACK    = 1 << 0;
        const CAST      = 1 << 1;  // cast speed / cast-only mods (PoB: ModFlag.Cast)
        const SPELL     = 1 << 2;
        const HIT       = 1 << 3;
        const DOT       = 1 << 4;
        const MELEE     = 1 << 5;
        const RANGED    = 1 << 6;
        const AREA      = 1 << 7;
        const PROJECTILE = 1 << 8;
        const MINE      = 1 << 9;
        const TRAP      = 1 << 10;
        const TOTEM     = 1 << 11;
    }
}

bitflags! {
    /// Element/keyword flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeywordFlag: u32 {
        const PHYSICAL  = 1 << 0;
        const FIRE      = 1 << 1;
        const COLD      = 1 << 2;
        const LIGHTNING = 1 << 3;
        const CHAOS     = 1 << 4;
        // ... add more
    }
}

/// Tags that control when the mod is applied, on top of the ModFlags.
/// All string fields are interned via `intern()` — no per-tag heap allocation.
#[derive(Debug, Clone)]
pub enum ModTag {
    Condition(&'static str),            // condition var name, e.g. "LowLife", "Onslaught"
    ActorCondition {                    // condition on a specific actor (self, enemy, parent, etc.)
        var: &'static str,
        actor: Option<&'static str>,
    },
    Multiplier(&'static str),           // multiplier var name, e.g. "EnduranceCharge"
    MultiplierThreshold {               // multiplier that only kicks in above a threshold
        var: &'static str,
        threshold: f64,
    },
    PerStat {                           // per-X-of-stat scaling (e.g. "1% per 10 Dexterity")
        stat: &'static str,
        div: f64,
    },
    PercentStat {                       // stat percent contribution (e.g. "X% of Strength added as Y")
        stat: &'static str,
        percent: bool,
    },
    StatThreshold {                     // mod only active above a stat threshold
        stat: &'static str,
        threshold: f64,
    },
    SkillType(u32),                     // requires a specific active skill type flag
    GlobalEffect {                      // applies a named global effect (aura, curse, etc.)
        effect_name: &'static str,
        effect_type: Option<&'static str>,
    },
    ModFlagOr(u32),                     // alternative ModFlag check (OR instead of AND)
    DistanceRamp(Vec<[f64; 2]>),        // damage ramp based on projectile distance
    SlotName(u8),                       // (legacy) item slot identifier
}

// Main Modifier class
#[derive(Debug, Clone)]
pub struct Modifier {
    pub stat: StatId,
    pub mod_type: ModType,
    pub value: f64,
    pub flags: ModFlag,
    pub keywords: KeywordFlag,
    pub source: SourceId,
    pub tags: SmallVec<[ModTag; 2]>
}

/// Context passed to every ModDB query.
/// Contains the current character state needed to evaluate conditional mods.
/// Start with a minimal struct—expand as you add conditional support.
pub struct CalcContext {
    pub flags: ModFlag,       // what kind of action we're calculating
    pub key_flags: KeywordFlag,
    // Phase 5+: add fields for conditions, multipliers, etc.
    // pub power_charges: u32,
    // pub is_leeching: bool,
}

impl CalcContext {
    /// Default context with no flags—used in Phase 2 before conditions matter.
    pub fn empty() -> Self {
        Self {
            flags: ModFlag::empty(),
            key_flags: KeywordFlag::empty(),
        }
    }
}