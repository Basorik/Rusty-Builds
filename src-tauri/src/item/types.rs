use crate::modifier::Modifier;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    // Identity
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub item_type: ItemType,
    pub rarity: Rarity,

    // Flags
    pub corrupted: bool,
    pub mirrored: bool,
    pub fractured: bool,
    pub synthesised: bool,
    pub influences: InfluenceSet,

    // Requirements (from base + mods)
    pub requirements: ItemRequirements,
    pub item_level: u32,
    pub quality: u32,

    // Mod lines (parsed from text or resolved from mod IDs)
    pub implicit_lines: Vec<ModLine>,
    pub explicit_lines: Vec<ModLine>,
    pub crafted_lines: Vec<ModLine>,
    pub enchant_lines: Vec<ModLine>,

    // PoB variant support
    pub variant_list: Vec<String>,
    pub selected_variant: usize,

    // Computed local stats
    pub weapon_data: Option<WeaponData>,
    pub armour_data: Option<ArmourData>,
    pub flask_data: Option<FlaskData>,

    // Final output: global-only modifiers that enter ModDB
    #[serde(skip)]
    pub mod_list: Vec<Modifier>,

    // Inventory tracking — 0 means not assigned yet
    pub inventory_id: u32,
    /// Optional overrides for base item property values (e.g. exact phys damage roll).
    /// When set, these take precedence over the base item's property ranges in
    /// `compute_local_stats`.
    pub base_overrides: Option<BasePropertyOverride>,
}

/// Allows overriding specific base property values when crafting items,
/// so the user can pick exact rolls within the base item's min–max range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BasePropertyOverride {
    pub phys_damage_min: Option<f64>,
    pub phys_damage_max: Option<f64>,
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WeaponData {
    // From base (properties.physical_damage_min/max, attack_time, critical_strike_chance, range)
    pub phys_min: f64,
    pub phys_max: f64,
    pub attack_time_ms: u32,
    pub range: u32,

    // Computed (after local mods + quality)
    pub attacks_per_second: f64, // 1000.0 / attack_time_ms * (1 + local_speed_inc/100)
    pub crit_chance: f64,        // properties.critical_strike_chance/100 * (1 + local_crit_inc/100)
    // Per-element min/max damage (after local adds + local elemental % applied).
    // Used by calc/offense.rs as base damage for each element source.
    // Chaos does NOT receive the local elemental damage multiplier.
    pub fire_min: f64,
    pub fire_max: f64,
    pub cold_min: f64,
    pub cold_max: f64,
    pub lightning_min: f64,
    pub lightning_max: f64,
    pub chaos_min: f64,
    pub chaos_max: f64,
    pub phys_dps: f64,
    pub ele_dps: f64, // fire + cold + lightning combined
    pub chaos_dps: f64,
    pub total_dps: f64,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArmourData {
    // Computed (after local mods + quality); null fields stay 0.0
    pub armour: f64,
    pub evasion: f64,
    pub energy_shield: f64,
    pub ward: f64,
    pub block: u32,
    pub movement_speed_penalty: i32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FlaskData {
    pub charges_max: u32,
    pub charges_per_use: u32,
    pub duration_ms: u32,
    pub life_per_use: f64,
    pub mana_per_use: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ItemRequirements {
    pub level: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLine {
    pub text: String,
    #[serde(skip)]
    pub modifiers: Vec<Modifier>,
    /// Raw `(stat_id, value)` pairs as returned by `InvertedTranslations::resolve_line`.
    /// Populated for all lines resolved through the stat-translation path.
    /// Used by `compute_local_stats` to read local stat values that are not
    /// represented in the global stat table (weapon/armour/flask local mods).
    pub raw_stats: Vec<(String, f64)>,
    pub is_local: bool,
    pub source: ModLineSource, // Implicit, Explicit, Crafted, Enchant, Fractured
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModLineSource {
    Implicit,
    Explicit,
    Crafted,
    Enchant,
    Fractured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    // Armour
    BodyArmour,
    Helmet,
    Gloves,
    Boots,
    Shield,
    // Weapons (1H)
    Claw,
    Dagger,
    RuneDagger,
    OneHandSword,
    ThrustingOneHandSword,
    OneHandAxe,
    OneHandMace,
    Sceptre,
    Wand,
    // Weapons (2H)
    Bow,
    TwoHandSword,
    TwoHandAxe,
    TwoHandMace,
    Staff,
    Warstaff,
    // Accessories
    Amulet,
    Ring,
    Belt,
    Quiver,
    // Flasks
    LifeFlask,
    ManaFlask,
    HybridFlask,
    UtilityFlask,
    // Jewels
    Jewel,
    AbyssJewel,
    // Other
    Tincture,
    FishingRod,
}

// impl ItemType {
//     pub fn from_class(class: &str) -> Option<Self> { /* match class → variant */
//     }
//     pub fn is_weapon(self) -> bool { /* 1H and 2H */
//     }
//     pub fn is_one_handed(self) -> bool { /* Claw, Dagger, RuneDagger, 1HSword, etc. */
//     }
//     pub fn is_two_handed(self) -> bool { /* Bow, 2HSword, etc. */
//     }
//     pub fn is_armour(self) -> bool { /* BodyArmour, Helmet, Gloves, Boots, Shield */
//     }
//     pub fn is_flask(self) -> bool { /* Life/Mana/Hybrid/Utility */
//     }
//     pub fn is_jewel(self) -> bool { /* Jewel, AbyssJewel */
//     }
//     pub fn is_accessory(self) -> bool { /* Amulet, Ring, Belt, Quiver */
//     }
// }

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct InfluenceSet: u8 {
        const SHAPER   = 1 << 0;
        const ELDER    = 1 << 1;
        const CRUSADER = 1 << 2;
        const HUNTER   = 1 << 3;
        const REDEEMER = 1 << 4;
        const WARLORD  = 1 << 5;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ItemSummary {
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub rarity: Rarity,
    pub corrupted: bool,
    pub item_level: u32,
    pub quality: u32,
    pub influences: u8,         // InfluenceSet bits
    pub total_dps: Option<f64>, // Some for weapons
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
}

/// Each equipment slot the player can fill.
/// Variants are stable integers — stored as u8 in IPC and used as HashMap keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[repr(u8)]
pub enum ItemSlot {
    Weapon1 = 0,
    Weapon2 = 1,
    Helmet = 2,
    BodyArmour = 3,
    Gloves = 4,
    Boots = 5,
    Amulet = 6,
    Ring1 = 7,
    Ring2 = 8,
    Belt = 9,
    Flask1 = 10,
    Flask2 = 11,
    Flask3 = 12,
    Flask4 = 13,
    Flask5 = 14,
}

impl ItemSlot {
    /// Returns true if `item_type` can legally go into this slot.
    ///
    /// `weapon1_type` — the `ItemType` already in Weapon1 (if any), used to
    /// enforce the 2H lock-out rule for Weapon2.
    pub fn is_compatible(
        slot: ItemSlot,
        item_type: ItemType,
        weapon1_type: Option<ItemType>,
    ) -> bool {
        match slot {
            ItemSlot::Weapon1 => matches!(
                item_type,
                ItemType::Claw
                    | ItemType::Dagger
                    | ItemType::RuneDagger
                    | ItemType::OneHandSword
                    | ItemType::ThrustingOneHandSword
                    | ItemType::OneHandAxe
                    | ItemType::OneHandMace
                    | ItemType::Sceptre
                    | ItemType::Wand
                    | ItemType::Bow
                    | ItemType::TwoHandSword
                    | ItemType::TwoHandAxe
                    | ItemType::TwoHandMace
                    | ItemType::Staff
                    | ItemType::Warstaff
                    | ItemType::FishingRod
            ),
            ItemSlot::Weapon2 => {
                let w1_is_2h = weapon1_type.map_or(false, |t| {
                    matches!(
                        t,
                        ItemType::Bow
                            | ItemType::TwoHandSword
                            | ItemType::TwoHandAxe
                            | ItemType::TwoHandMace
                            | ItemType::Staff
                            | ItemType::Warstaff
                    )
                });
                if w1_is_2h {
                    return false;
                }
                matches!(
                    item_type,
                    ItemType::Claw
                        | ItemType::Dagger
                        | ItemType::RuneDagger
                        | ItemType::OneHandSword
                        | ItemType::ThrustingOneHandSword
                        | ItemType::OneHandAxe
                        | ItemType::OneHandMace
                        | ItemType::Sceptre
                        | ItemType::Wand
                        | ItemType::Shield
                        | ItemType::Quiver
                )
            }
            ItemSlot::Helmet => matches!(item_type, ItemType::Helmet),
            ItemSlot::BodyArmour => matches!(item_type, ItemType::BodyArmour),
            ItemSlot::Gloves => matches!(item_type, ItemType::Gloves),
            ItemSlot::Boots => matches!(item_type, ItemType::Boots),
            ItemSlot::Amulet => matches!(item_type, ItemType::Amulet),
            ItemSlot::Ring1 | ItemSlot::Ring2 => matches!(item_type, ItemType::Ring),
            ItemSlot::Belt => matches!(item_type, ItemType::Belt),
            ItemSlot::Flask1
            | ItemSlot::Flask2
            | ItemSlot::Flask3
            | ItemSlot::Flask4
            | ItemSlot::Flask5 => {
                matches!(
                    item_type,
                    ItemType::LifeFlask
                        | ItemType::ManaFlask
                        | ItemType::HybridFlask
                        | ItemType::UtilityFlask
                )
            }
        }
    }
}
