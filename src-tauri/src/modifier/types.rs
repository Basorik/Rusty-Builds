use crate::data::{SourceId, StatId};
use bitflags::bitflags;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use std::sync::RwLock;

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
    Base,     // +X flat (e.g., "+20 to Strength")
    Inc,      // X% increased (additive with other increases) — PoB calls this "INC"
    More,     // X% more (multiplicative, applied separately)
    Flag,     // Boolean flag (e.g., "Cannot be Stunned")
    Override, // Overrides the stat entirely
    List,     // List-type mod (appended, not summed)
    Max,      // Only the highest value takes effect (e.g., "PoisonStackLimit")
    Min,      // Only the lowest value takes effect
    Chance,   // X% chance (e.g., "20% chance to Poison") — PoB calls this "CHANCE"
}

// ── ModFlag ─────────────────────────────────────────────────────────────────
// Bit layout mirrors PoB's ModFlag exactly (src/Data/Global.lua).
// The generated stat_table.rs embeds raw PoB numeric values via
// `ModFlag::from_bits_truncate(n)`, so both tables are always in agreement.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ModFlag: u32 {
        // ── Damage modes (PoB 0x01–0x10) ───────────────────────────────
        const ATTACK     = 0x00000001;  // ModFlag.Attack
        const SPELL      = 0x00000002;  // ModFlag.Spell
        const HIT        = 0x00000004;  // ModFlag.Hit
        const DOT        = 0x00000008;  // ModFlag.Dot
        const CAST       = 0x00000010;  // ModFlag.Cast  (cast speed / cast-only)

        // ── Damage sources (PoB 0x100–0x2000) ──────────────────────────
        const MELEE      = 0x00000100;  // ModFlag.Melee
        const AREA       = 0x00000200;  // ModFlag.Area
        const PROJECTILE = 0x00000400;  // ModFlag.Projectile
        const AILMENT    = 0x00000800;  // ModFlag.Ailment
        const MELEE_HIT  = 0x00001000;  // ModFlag.MeleeHit
        const WEAPON     = 0x00002000;  // ModFlag.Weapon

        // ── Weapon types (PoB 0x10000–0x02000000) ──────────────────────
        const AXE        = 0x00010000;  // ModFlag.Axe
        const BOW        = 0x00020000;  // ModFlag.Bow
        const CLAW       = 0x00040000;  // ModFlag.Claw
        const DAGGER     = 0x00080000;  // ModFlag.Dagger
        const MACE       = 0x00100000;  // ModFlag.Mace
        const STAFF      = 0x00200000;  // ModFlag.Staff
        const SWORD      = 0x00400000;  // ModFlag.Sword
        const WAND       = 0x00800000;  // ModFlag.Wand
        const UNARMED    = 0x01000000;  // ModFlag.Unarmed

        // ── Weapon classes (PoB 0x04000000–0x20000000) ─────────────────
        const WEAPON_MELEE  = 0x04000000;  // ModFlag.WeaponMelee
        const WEAPON_RANGED = 0x08000000;  // ModFlag.WeaponRanged
        const WEAPON_1H     = 0x10000000;  // ModFlag.Weapon1H
        const WEAPON_2H     = 0x20000000;  // ModFlag.Weapon2H

        // ── Composite helper masks ──────────────────────────────────────
        const WEAPON_MASK = 0x2FFF0000;  // ModFlag.WeaponMask
        const SOURCE_MASK = 0x00000600;  // ModFlag.SourceMask (Area | Projectile)
    }
}

// ── KeywordFlag ──────────────────────────────────────────────────────────────
// Bit layout mirrors PoB's KeywordFlag exactly (src/Data/Global.lua).
// Default matching: ANY of the specified flags must be present in the context.
// When MATCH_ALL is set on a modifier, ALL specified flags must be present.
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeywordFlag: u32 {
        // ── Skill categories ──────────────────────────────────────────────
        const AURA       = 0x00000001;  // KeywordFlag.Aura
        const CURSE      = 0x00000002;  // KeywordFlag.Curse
        const WARCRY     = 0x00000004;  // KeywordFlag.Warcry
        const MOVEMENT   = 0x00000008;  // KeywordFlag.Movement

        // ── Damage element keywords ───────────────────────────────────────
        const PHYSICAL   = 0x00000010;  // KeywordFlag.Physical
        const FIRE       = 0x00000020;  // KeywordFlag.Fire
        const COLD       = 0x00000040;  // KeywordFlag.Cold
        const LIGHTNING  = 0x00000080;  // KeywordFlag.Lightning
        const CHAOS      = 0x00000100;  // KeywordFlag.Chaos
        const VAAL       = 0x00000200;  // KeywordFlag.Vaal

        // ── Projectile / bow ──────────────────────────────────────────────
        const BOW        = 0x00000400;  // KeywordFlag.Bow
        const ARROW      = 0x00000800;  // KeywordFlag.Arrow

        // ── Deployment keywords ───────────────────────────────────────────
        const TRAP       = 0x00001000;  // KeywordFlag.Trap
        const MINE       = 0x00002000;  // KeywordFlag.Mine
        const TOTEM      = 0x00004000;  // KeywordFlag.Totem
        const MINION     = 0x00008000;  // KeywordFlag.Minion

        // ── Skill type keywords ───────────────────────────────────────────
        const ATTACK     = 0x00010000;  // KeywordFlag.Attack
        const SPELL      = 0x00020000;  // KeywordFlag.Spell
        const HIT        = 0x00040000;  // KeywordFlag.Hit
        const AILMENT    = 0x00080000;  // KeywordFlag.Ailment
        const BRAND      = 0x00100000;  // KeywordFlag.Brand

        // ── Ailment / DoT type keywords ───────────────────────────────────
        const POISON         = 0x00200000;  // KeywordFlag.Poison
        const BLEED          = 0x00400000;  // KeywordFlag.Bleed
        const IGNITE         = 0x00800000;  // KeywordFlag.Ignite
        const PHYSICAL_DOT   = 0x01000000;  // KeywordFlag.PhysicalDot
        const LIGHTNING_DOT  = 0x02000000;  // KeywordFlag.LightningDot
        const COLD_DOT       = 0x04000000;  // KeywordFlag.ColdDot
        const FIRE_DOT       = 0x08000000;  // KeywordFlag.FireDot
        const CHAOS_DOT      = 0x10000000;  // KeywordFlag.ChaosDot

        // ── Matching control ─────────────────────────────────────────────
        /// When set on a mod's keyword flags, ALL specified keyword bits must
        /// match (instead of the default ANY-match).  Strip this bit before
        /// doing the actual intersection test.
        const MATCH_ALL  = 0x40000000;  // KeywordFlag.MatchAll
    }
}

/// Tags that control when the mod is applied, on top of the ModFlags.
/// All string fields are interned via `intern()` — no per-tag heap allocation.
#[derive(Debug, Clone)]
pub enum ModTag {
    Condition(&'static str), // condition var name, e.g. "LowLife", "Onslaught"
    ActorCondition {
        // condition on a specific actor (self, enemy, parent, etc.)
        var: &'static str,
        actor: Option<&'static str>,
    },
    Multiplier(&'static str), // multiplier var name, e.g. "EnduranceCharge"
    MultiplierThreshold {
        // multiplier that only kicks in above a threshold
        var: &'static str,
        threshold: f64,
    },
    PerStat {
        // per-X-of-stat scaling (e.g. "1% per 10 Dexterity")
        stat: StatId,
        div: f64,
    },
    PercentStat {
        // stat percent contribution: value *= stat_val / 100
        // (e.g. "X% of Mana added as Lightning Damage")
        stat: StatId,
    },
    StatThreshold {
        // mod only active when stat meets a threshold.
        // upper = false (default): active when stat >= threshold ("while you have at least X")
        // upper = true: active when stat <= threshold ("while you have no X", "single projectile")
        stat: StatId,
        threshold: f64,
        upper: bool,                    // true = invert gate (≤ instead of ≥)
        threshold_stat: Option<StatId>, // compare stat vs another stat ("while at max charges")
    },
    SkillType(u32), // requires a specific active skill type flag
    GlobalEffect {
        // applies a named global effect (aura, curse, etc.)
        effect_name: &'static str,
        effect_type: Option<&'static str>,
    },
    ModFlagOr(u32), // alternative ModFlag check (OR instead of AND)
    SlotName(u8), // (legacy) item slot identifier                       // (legacy) item slot identifier
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
    pub tags: SmallVec<[ModTag; 2]>,
}

/// Context passed to every ModDB query.
/// Contains the current character state needed to evaluate conditional mods.
/// Start with a minimal struct—expand as you add conditional support.
#[derive(Clone)]
pub struct CalcContext {
    pub flags: ModFlag, // what kind of action we're calculating
    pub key_flags: KeywordFlag,
    /// Boolean conditions: "LowLife", "FullLife", "Leeching", "UsingShield", etc.
    pub conditions: FxHashMap<&'static str, bool>,
    /// Numeric multipliers: "PowerCharge" → count, "EnduranceCharge" → count, etc.
    pub multipliers: FxHashMap<&'static str, f64>,
    /// Pre-computed stat values for PerStat/StatThreshold/PercentStat tags
    pub stat_values: FxHashMap<StatId, f64>,
    /// Conditions on the enemy actor: "Frozen", "Cursed", "RareOrUnique", etc.
    pub enemy_conditions: FxHashMap<&'static str, bool>,
}

impl CalcContext {
    /// Default context with no flags—used in Phase 2 before conditions matter.
    pub fn empty() -> Self {
        Self {
            flags: ModFlag::empty(),
            key_flags: KeywordFlag::empty(),
            conditions: FxHashMap::default(),
            multipliers: FxHashMap::default(),
            stat_values: FxHashMap::default(),
            enemy_conditions: FxHashMap::default(),
        }
    }

    pub fn defense() -> Self {
        Self::empty()
    }
    pub fn attack(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::ATTACK | ModFlag::HIT,
            key_flags: keywords,
            ..Self::empty()
        }
    }
    /// For spell offence calculations
    pub fn spell(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::SPELL | ModFlag::HIT,
            key_flags: keywords,
            ..Self::empty()
        }
    }

    /// For DoT calculations
    pub fn dot(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::DOT,
            key_flags: keywords,
            ..Self::empty()
        }
    }
}
