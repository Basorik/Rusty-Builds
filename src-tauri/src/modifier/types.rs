use bitflags::bitflags;
use smallvec::SmallVec;
use crate::data::{StatId, SourceId};

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

//Tags that control when the mod is applied on top of the modflags
#[derive(Debug, Clone)]
pub enum ModTag {
    Condition(StatId),
    Multiplier(StatId),
    SkillType(u32),
    SlotName(u8)

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