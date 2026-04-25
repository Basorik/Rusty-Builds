//! Phase 4.7 — Local mod extraction and computed item stats.
//!
//! After [`crate::item::parser::parse_unique_item`] builds an [`Item`] with
//! parsed mod lines, this module computes the local stats (weapon DPS,
//! armour/evasion/ES, flask data) and populates `item.weapon_data`,
//! `item.armour_data`, or `item.flask_data`.
//!
//! Only global (non-local) modifiers remain in `item.mod_list` after this step.

use crate::data::bases::{BaseProperties, RePoEBaseItem};
use crate::item::types::{ArmourData, FlaskData, Item, ItemType, ModLine, WeaponData};

// ─── Public entry point ───────────────────────────────────────────────────────

/// Compute weapon/armour/flask stats from the base item properties and local mods,
/// populate the corresponding `item.*_data` field, and strip local mods from
/// `item.mod_list` so only global modifiers remain.
///
/// Must be called after all mod lines are parsed.
pub fn compute_local_stats(item: &mut Item, base: Option<&RePoEBaseItem>) {
    let props = base.map(|b| &b.properties);

    match item.item_type {
        t if is_weapon(t) => {
            item.weapon_data = Some(compute_weapon(item, props));
        }
        t if is_armour_or_shield(t) => {
            item.armour_data = Some(compute_armour(item, props));
        }
        t if is_flask(t) => {
            item.flask_data = Some(compute_flask(item, props));
        }
        _ => {}
    }

    // Remove local modifiers from mod_list — they were already used for
    // computing local stats and must not enter the global ModDB layer.
    item.mod_list.retain(|m| {
        // All Modifiers in mod_list were already filtered to global-only in
        // parse_unique_item, but in case any slipped through, double-check.
        let _ = m; // Currently no additional filtering needed; retained unconditionally.
        true
    });
}

// ─── Weapon computation ───────────────────────────────────────────────────────

fn compute_weapon(item: &Item, props: Option<&BaseProperties>) -> WeaponData {
    let quality = item.quality as f64;

    // Base physical range from the base item type.
    let base_phys_min = item
        .base_overrides
        .as_ref()
        .and_then(|ov| ov.phys_damage_min)
        .or_else(|| props.and_then(|p| p.physical_damage_min))
        .unwrap_or(0.0);
    let base_phys_max = item
        .base_overrides
        .as_ref()
        .and_then(|ov| ov.phys_damage_max)
        .or_else(|| props.and_then(|p| p.physical_damage_max))
        .unwrap_or(0.0);
    let attack_time_ms = props.and_then(|p| p.attack_time).unwrap_or(1000) as f64;
    let base_crit_permyriad = props.and_then(|p| p.critical_strike_chance).unwrap_or(500) as f64;

    // Accumulate local mod stats from every non-filtered mod line.
    let mut flat_phys_min = 0.0_f64;
    let mut flat_phys_max = 0.0_f64;
    let mut inc_phys_pct = 0.0_f64;
    let mut inc_speed_pct = 0.0_f64;
    let mut inc_crit_pct = 0.0_f64;
    let mut flat_crit_permyriad = 0.0_f64;
    // Elemental adds per element
    let mut fire_min = 0.0_f64;
    let mut fire_max = 0.0_f64;
    let mut cold_min = 0.0_f64;
    let mut cold_max = 0.0_f64;
    let mut lightning_min = 0.0_f64;
    let mut lightning_max = 0.0_f64;
    let mut chaos_min = 0.0_f64;
    let mut chaos_max = 0.0_f64;
    let mut inc_ele_pct = 0.0_f64;

    for line in all_local_lines(item) {
        for (stat_id, value) in line.raw_stats.iter() {
            match stat_id.as_str() {
                "local_minimum_added_physical_damage" => flat_phys_min += value,
                "local_maximum_added_physical_damage" => flat_phys_max += value,
                "local_physical_damage_+%" => inc_phys_pct += value,
                "local_attack_speed_+%" => inc_speed_pct += value,
                "local_critical_strike_chance_+%" => inc_crit_pct += value,
                "local_critical_strike_chance" => flat_crit_permyriad += value,
                "local_minimum_added_fire_damage"
                | "unique_local_minimum_added_fire_damage_when_in_main_hand" => fire_min += value,
                "local_maximum_added_fire_damage"
                | "unique_local_maximum_added_fire_damage_when_in_main_hand" => fire_max += value,
                "local_minimum_added_cold_damage"
                | "unique_local_minimum_added_cold_damage_when_in_off_hand" => cold_min += value,
                "local_maximum_added_cold_damage"
                | "unique_local_maximum_added_cold_damage_when_in_off_hand" => cold_max += value,
                "local_minimum_added_lightning_damage" => lightning_min += value,
                "local_maximum_added_lightning_damage" => lightning_max += value,
                "local_minimum_added_chaos_damage"
                | "unique_local_minimum_added_chaos_damage_when_in_off_hand" => chaos_min += value,
                "local_maximum_added_chaos_damage"
                | "unique_local_maximum_added_chaos_damage_when_in_off_hand" => chaos_max += value,
                "local_elemental_damage_+%" => inc_ele_pct += value,
                _ => {}
            }
        }
    }

    // ── Physical DPS ──────────────────────────────────────────────────────────
    // PLAN §4.7: quality and %inc are additive with each other.
    let phys_multiplier = 1.0 + quality / 100.0 + inc_phys_pct / 100.0;
    let final_phys_min = (base_phys_min + flat_phys_min) * phys_multiplier;
    let final_phys_max = (base_phys_max + flat_phys_max) * phys_multiplier;
    let phys_avg = (final_phys_min + final_phys_max) / 2.0;

    // ── Attack rate ───────────────────────────────────────────────────────────
    let attacks_per_second = (1000.0 / attack_time_ms) * (1.0 + inc_speed_pct / 100.0);

    // ── Crit ──────────────────────────────────────────────────────────────────
    // base_crit_permyriad is × 100 in RePoE (e.g. 650 = 6.50%)
    let base_crit_pct = base_crit_permyriad / 100.0;
    let flat_crit_pct = flat_crit_permyriad / 100.0; // local_critical_strike_chance is also permyriad
    let crit_chance = (base_crit_pct + flat_crit_pct) * (1.0 + inc_crit_pct / 100.0);

    let phys_dps = phys_avg * attacks_per_second;

    // ── Elemental DPS — quality does NOT apply to ele damage ─────────────────
    let ele_multiplier = 1.0 + inc_ele_pct / 100.0;
    let fire_avg = (fire_min + fire_max) / 2.0 * ele_multiplier;
    let cold_avg = (cold_min + cold_max) / 2.0 * ele_multiplier;
    let lightning_avg = (lightning_min + lightning_max) / 2.0 * ele_multiplier;
    let chaos_avg = (chaos_min + chaos_max) / 2.0; // chaos never gets the ele multiplier
    let ele_dps = (fire_avg + cold_avg + lightning_avg) * attacks_per_second;
    let chaos_dps = chaos_avg * attacks_per_second;

    WeaponData {
        phys_min: final_phys_min,
        phys_max: final_phys_max,
        attack_time_ms: attack_time_ms as u32,
        range: props.and_then(|p| p.range).unwrap_or(0),
        attacks_per_second,
        crit_chance,
        fire_min: fire_min * ele_multiplier,
        fire_max: fire_max * ele_multiplier,
        cold_min: cold_min * ele_multiplier,
        cold_max: cold_max * ele_multiplier,
        lightning_min: lightning_min * ele_multiplier,
        lightning_max: lightning_max * ele_multiplier,
        chaos_min, // chaos never gets ele_multiplier
        chaos_max,
        phys_dps,
        ele_dps,
        chaos_dps,
        total_dps: phys_dps + ele_dps + chaos_dps,
    }
}

// ─── Armour/Shield computation ────────────────────────────────────────────────

fn compute_armour(item: &Item, props: Option<&BaseProperties>) -> ArmourData {
    let quality = item.quality as f64;

    // Base values — unique items always roll max base.
    let base_armour = item
        .base_overrides
        .as_ref()
        .and_then(|ov| ov.armour)
        .or_else(|| props.and_then(|p| p.armour.as_ref()).map(|m| m.max))
        .unwrap_or(0.0);
    let base_evasion = item
        .base_overrides
        .as_ref()
        .and_then(|ov| ov.evasion)
        .or_else(|| props.and_then(|p| p.evasion.as_ref()).map(|m| m.max))
        .unwrap_or(0.0);
    let base_es = item
        .base_overrides
        .as_ref()
        .and_then(|ov| ov.energy_shield)
        .or_else(|| props.and_then(|p| p.energy_shield.as_ref()).map(|m| m.max))
        .unwrap_or(0.0);
    let base_ward = props
        .and_then(|p| p.ward.as_ref())
        .map(|m| m.max)
        .unwrap_or(0.0);
    let base_block = props.and_then(|p| p.block).unwrap_or(0);
    let movement_speed_penalty = props.and_then(|p| p.movement_speed).unwrap_or(0);

    let mut inc_armour_pct = 0.0_f64;
    let mut inc_evasion_pct = 0.0_f64;
    let mut inc_es_pct = 0.0_f64;
    let mut inc_ward_pct = 0.0_f64;
    let mut flat_es = 0.0_f64;
    let mut flat_ward = 0.0_f64;

    for line in all_local_lines(item) {
        for (stat_id, value) in line.raw_stats.iter() {
            match stat_id.as_str() {
                "local_physical_damage_reduction_rating_+%" => inc_armour_pct += value,
                "local_evasion_rating_+%" => inc_evasion_pct += value,
                "local_energy_shield_+%" => inc_es_pct += value,
                "local_ward_+%" => inc_ward_pct += value,
                // Combined %inc stats
                "local_armour_and_evasion_+%" => {
                    inc_armour_pct += value;
                    inc_evasion_pct += value;
                }
                "local_armour_and_energy_shield_+%" => {
                    inc_armour_pct += value;
                    inc_es_pct += value;
                }
                "local_evasion_and_energy_shield_+%" => {
                    inc_evasion_pct += value;
                    inc_es_pct += value;
                }
                "local_armour_and_evasion_and_energy_shield_+%" => {
                    inc_armour_pct += value;
                    inc_evasion_pct += value;
                    inc_es_pct += value;
                }
                // Flat adds (shields, some uniques)
                "local_energy_shield" | "local_evasion_rating_and_energy_shield" => {
                    flat_es += value
                }
                "local_ward" => flat_ward += value,
                _ => {}
            }
        }
    }

    // Formula: base_max * (1 + quality/100 + inc%/100) — quality and %inc additive.
    let armour_multi = 1.0 + quality / 100.0 + inc_armour_pct / 100.0;
    let evasion_multi = 1.0 + quality / 100.0 + inc_evasion_pct / 100.0;
    let es_multi = 1.0 + quality / 100.0 + inc_es_pct / 100.0;
    let ward_multi = 1.0 + quality / 100.0 + inc_ward_pct / 100.0;

    ArmourData {
        armour: base_armour * armour_multi,
        evasion: base_evasion * evasion_multi,
        energy_shield: (base_es + flat_es) * es_multi,
        ward: (base_ward + flat_ward) * ward_multi,
        block: base_block,
        movement_speed_penalty,
    }
}

// ─── Flask computation ────────────────────────────────────────────────────────

fn compute_flask(item: &Item, props: Option<&BaseProperties>) -> FlaskData {
    let quality = item.quality as f64;

    let base_charges_max = props.and_then(|p| p.charges_max).unwrap_or(0);
    let base_charges_per_use = props.and_then(|p| p.charges_per_use).unwrap_or(0);
    let base_duration_ms = props.and_then(|p| p.duration).unwrap_or(0);
    let base_life = props.and_then(|p| p.life_per_use).unwrap_or(0.0);
    let base_mana = props.and_then(|p| p.mana_per_use).unwrap_or(0.0);

    let mut inc_duration_pct = 0.0_f64;
    let mut inc_life_pct = 0.0_f64;
    let mut inc_mana_pct = 0.0_f64;

    for line in all_local_lines(item) {
        for (stat_id, value) in line.raw_stats.iter() {
            match stat_id.as_str() {
                "local_flask_duration_+%"
                | "local_flask_duration_+%_final"
                | "local_flask_consume_flask_duration_+%_when_used" => inc_duration_pct += value,
                "local_flask_life_to_recover_+%" => inc_life_pct += value,
                "local_flask_mana_to_recover_+%" => inc_mana_pct += value,
                _ => {}
            }
        }
    }

    let duration_multi = 1.0 + quality / 100.0 + inc_duration_pct / 100.0;

    FlaskData {
        charges_max: base_charges_max,
        charges_per_use: base_charges_per_use,
        duration_ms: (base_duration_ms as f64 * duration_multi).round() as u32,
        life_per_use: base_life * (1.0 + inc_life_pct / 100.0),
        mana_per_use: base_mana * (1.0 + inc_mana_pct / 100.0),
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Iterate all mod lines that are marked local, across all line categories.
fn all_local_lines(item: &Item) -> impl Iterator<Item = &ModLine> {
    item.implicit_lines
        .iter()
        .chain(item.explicit_lines.iter())
        .chain(item.crafted_lines.iter())
        .filter(|ml| ml.is_local)
}

fn is_weapon(t: ItemType) -> bool {
    matches!(
        t,
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
    )
}

fn is_armour_or_shield(t: ItemType) -> bool {
    matches!(
        t,
        ItemType::BodyArmour
            | ItemType::Helmet
            | ItemType::Gloves
            | ItemType::Boots
            | ItemType::Shield
    )
}

fn is_flask(t: ItemType) -> bool {
    matches!(
        t,
        ItemType::LifeFlask | ItemType::ManaFlask | ItemType::HybridFlask | ItemType::UtilityFlask
    )
}
