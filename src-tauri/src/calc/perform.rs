use crate::calc::setup::setup_moddb;
use crate::calc::{attributes, charges, defense, offense};
use crate::data::GameData;
use crate::data::StatId;
use crate::modifier::CalcContext;
use crate::modifier::ModDBLayers;
use crate::Class;
use crate::GemRef;
use crate::Item;
use crate::ItemSlot;
use crate::SkillGroup;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub fn calculate(
    layers: &ModDBLayers,
    level: u32,
    class: &Class,
    active_gem: Option<&GemRef>,
    skill_groups: &[SkillGroup],
    equipped: &FxHashMap<ItemSlot, Item>,
    game_data: &GameData,
) -> CalcResult {
    //Setup and seed mod db
    let mut db = setup_moddb(layers, level);

    let attrs = attributes::calc_attributes(&db);
    attributes::inject_attribute_bonuses(&mut db, &attrs);

    //Set up calccontext
    let conditions = attributes::determine_conditions(&db, &attrs, equipped);
    let mut ctx = CalcContext::empty();
    ctx.conditions = conditions;

    let charges = charges::process_charges(&db, &ctx);
    ctx.multipliers = charges.multipliers;

    let defense = defense::calc_defense(&db, level, &ctx);

    // Step 8: Update context stat_values with computed pool sizes
    ctx.stat_values.insert(StatId::Life, defense.life as f64);
    ctx.stat_values.insert(StatId::Mana, defense.mana as f64);
    ctx.stat_values
        .insert(StatId::EnergyShield, defense.energy_shield as f64);
    ctx.stat_values
        .insert(StatId::Strength, attrs.strength as f64);
    ctx.stat_values
        .insert(StatId::Dexterity, attrs.dexterity as f64);
    ctx.stat_values
        .insert(StatId::Intelligence, attrs.intelligence as f64);

    // Step 9: Calculate offence (DPS, crit, speed, etc.)
    let offence = if let Some(gem_ref) = active_gem {
        offense::calc_offence(
            &db,
            gem_ref,
            skill_groups,
            game_data,
            &ctx,
            &attrs,
            equipped,
            level,
        )
    } else {
        OffenceResult::default()
    };

    CalcResult {
        defense,
        offence,
        attributes: attrs,
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CalcResult {
    pub defense: DefenceResult,
    pub offence: OffenceResult,
    pub attributes: AttributeResult,
}
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AttributeResult {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct DefenceResult {
    pub life: u32,
    pub mana: u32,
    pub energy_shield: u32,
    pub armour: u32,
    pub evasion: u32,
    pub ward: u32,
    pub fire_resist: i32, // Can be negative
    pub fire_resist_cap: i32,
    pub fire_resist_overcap: i32,
    pub cold_resist: i32,
    pub cold_resist_cap: i32,
    pub cold_resist_overcap: i32,
    pub lightning_resist: i32,
    pub lightning_resist_cap: i32,
    pub lightning_resist_overcap: i32,
    pub chaos_resist: i32,
    pub chaos_resist_cap: i32,
    pub chaos_resist_overcap: i32,
    pub block_chance: f64, // Attack block %
    pub spell_block_chance: f64,
    pub spell_suppression: f64,
    pub attack_dodge: f64,
    pub spell_dodge: f64,
    pub life_regen: f64, // Per second
    pub mana_regen: f64,
    pub es_regen: f64,
    pub es_recharge: f64,       // Per second (when recharging)
    pub es_recharge_delay: f64, // Seconds before recharge starts
    pub life_leech_rate_max: f64,
    pub mana_leech_rate_max: f64,
    pub movement_speed_mod: f64, // 1.0 = 100% base
    pub mana_unreserved: u32,
    pub life_unreserved: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct OffenceResult {
    pub total_dps: f64,
    pub hit_dps: f64,
    pub average_hit: f64,
    pub crit_chance: f64,     // Effective crit chance %
    pub crit_multiplier: f64, // e.g. 1.5 = 150%
    pub hit_chance: f64,      // Chance to hit % (attacks only)
    pub attack_speed: f64,    // Attacks/casts per second
    pub cast_speed: f64,
    /// Damage breakdown per element (after all modifiers)
    pub phys_dps: f64,
    pub fire_dps: f64,
    pub cold_dps: f64,
    pub lightning_dps: f64,
    pub chaos_dps: f64,
    /// DoT
    pub dot_dps: f64,
    pub bleed_dps: f64,
    pub poison_dps: f64,
    pub ignite_dps: f64,
    /// Speed info
    pub speed: f64, // Active attacks/casts per second
    pub is_attack: bool,
}
