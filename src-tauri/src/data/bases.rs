use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer};

fn deserialize_null_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(de)?.unwrap_or_default())
}

pub const EQUIPPABLE_CLASSES: &[&str] = &[
    "Body Armour",
    "Helmet",
    "Gloves",
    "Boots",
    "Shield",
    "Bow",
    "Claw",
    "Dagger",
    "Rune Dagger",
    "One Hand Sword",
    "Thrusting One Hand Sword",
    "One Hand Axe",
    "One Hand Mace",
    "Sceptre",
    "Two Hand Sword",
    "Two Hand Axe",
    "Two Hand Mace",
    "Staff",
    "Warstaff",
    "Wand",
    "Amulet",
    "Ring",
    "Belt",
    "Quiver",
    "LifeFlask",
    "ManaFlask",
    "HybridFlask",
    "UtilityFlask",
    "Jewel",
    "AbyssJewel",
    "Tincture",
    "FishingRod",
];

#[derive(Debug, Deserialize)]
pub struct RePoEBaseItem {
    pub name: String,
    pub item_class: String,
    pub drop_level: u32,
    pub tags: Vec<String>,
    pub implicits: Vec<String>,
    pub release_state: String,
    pub domain: String,
    pub inventory_width: Option<u32>,
    pub inventory_height: Option<u32>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub requirements: BaseRequirements,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub properties: BaseProperties,

    pub grants_buff: Option<GrantsBuff>,
}

/// Flat property bag — every item has this object; irrelevant fields are null.
#[derive(Debug, Deserialize, Default)]
pub struct BaseProperties {
    // Armour
    pub armour: Option<MinMax>,
    pub evasion: Option<MinMax>,
    pub energy_shield: Option<MinMax>,
    pub ward: Option<MinMax>,
    pub block: Option<u32>,          // Shields only
    pub movement_speed: Option<i32>, // Negative = penalty

    // Weapons
    pub physical_damage_min: Option<f64>,
    pub physical_damage_max: Option<f64>,
    pub attack_time: Option<u32>,            // Milliseconds
    pub critical_strike_chance: Option<u32>, // ×100 (e.g. 650 = 6.5%)
    pub range: Option<u32>,

    // Flasks
    pub charges_max: Option<u32>,
    pub charges_per_use: Option<u32>,
    pub duration: Option<u32>, // Milliseconds
    pub life_per_use: Option<f64>,
    pub mana_per_use: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BaseRequirements {
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub strength: u32,
    #[serde(default)]
    pub dexterity: u32,
    #[serde(default)]
    pub intelligence: u32,
}

#[derive(Debug, Deserialize)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize)]
pub struct GrantsBuff {
    pub id: String,
    pub stats: FxHashMap<String, f64>,
}
