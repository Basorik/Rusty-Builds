use crate::calc::conversions;
use crate::calc::conversions::DamageSet;
use crate::calc::perform::AttributeResult;
use crate::calc::perform::OffenceResult;
use crate::data::StatId;
use crate::data::{skills::GemRef, skills::SkillGroup, GameData};
use crate::item::types::ItemType;
use crate::modifier::{CalcContext, KeywordFlag, ModDB, ModFlag};
use crate::Item;
use crate::ItemSlot;
use rustc_hash::FxHashMap;

/// Monster evasion by level (1–100). Source: data/pob/Misc.json `monsterEvasionTable`.
/// Index 0 = level 1, index 99 = level 100.
const MONSTER_EVASION: [u32; 100] = [
    67, 86, 104, 124, 144, 166, 188, 211, 234, 259, 285, 311, 339, 368, 397, 428, 460, 493, 527,
    563, 600, 638, 677, 718, 760, 804, 849, 896, 944, 994, 1046, 1100, 1155, 1212, 1271, 1332,
    1395, 1460, 1528, 1597, 1669, 1743, 1819, 1898, 1979, 2063, 2150, 2239, 2331, 2426, 2524, 2626,
    2730, 2837, 2948, 3063, 3180, 3302, 3427, 3556, 3689, 3826, 3967, 4112, 4262, 4416, 4575, 4739,
    4907, 5081, 5260, 5444, 5633, 5828, 6029, 6235, 6448, 6667, 6892, 7124, 7362, 7608, 7860, 8120,
    8388, 8663, 8946, 9237, 9536, 9844, 10160, 10486, 10821, 11165, 11519, 11883, 12258, 12643,
    13038, 13445,
];

/// Phase 5.7 — offence calculation (skill context setup; DPS pipeline in progress).
pub fn calc_offence(
    db: &ModDB,
    gem_ref: &GemRef,
    skill_groups: &[SkillGroup],
    game_data: &GameData,
    ctx: &CalcContext,
    attrs: &AttributeResult,
    equipped: &FxHashMap<ItemSlot, Item>,
    level: u32,
) -> OffenceResult {
    // 1. Resolve the gem instance.
    let group = match skill_groups.iter().find(|g| g.id == gem_ref.group_id) {
        Some(g) => g,
        None => return OffenceResult::default(),
    };
    let gem_instance = match group.gems.get(gem_ref.gem_index as usize) {
        Some(g) => g,
        None => return OffenceResult::default(),
    };

    // 2. Look up the RePoEGem definition (has cast_time and active_skill.types).
    let gem_def = match game_data.gems.get(&gem_instance.gem_id) {
        Some(g) => g,
        None => return OffenceResult::default(),
    };

    // 3. Determine skill type from active_skill.types.
    let active_types = gem_def
        .active_skill
        .as_ref()
        .map(|a| a.types.as_slice())
        .unwrap_or(&[]);
    let is_attack = active_types.contains(&"Attack".to_string());
    let is_spell = active_types.contains(&"Spell".to_string());
    let is_melee = active_types.contains(&"Melee".to_string());
    let is_projectile = active_types.contains(&"Projectile".to_string());

    // 4. Build offence-specific CalcContext.
    let mut off_ctx = ctx.clone();
    off_ctx.flags |= ModFlag::HIT;
    if is_attack {
        off_ctx.flags |= ModFlag::ATTACK;
    }
    if is_spell {
        off_ctx.flags |= ModFlag::SPELL;
    }
    if is_melee {
        off_ctx.flags |= ModFlag::MELEE;
    }
    if is_projectile {
        off_ctx.flags |= ModFlag::PROJECTILE;
    }

    // Weapon-type flags — derived from the equipped main-hand item type.
    // These allow mods like "increased Claw Damage" or "while wielding a Bow" to resolve correctly.
    if is_attack {
        let weapon_type = equipped.get(&ItemSlot::Weapon1).map(|w| w.item_type);
        off_ctx.flags |= weapon_flags_for(weapon_type);
    }

    // Calculate base damage, first add from weapon
    let mut base_dmg_min = DamageSet::default();
    let mut base_dmg_max = DamageSet::default();
    if is_attack {
        if let Some(wd) = equipped
            .get(&ItemSlot::Weapon1)
            .and_then(|w| w.weapon_data.as_ref())
        {
            base_dmg_min.physical = wd.phys_min;
            base_dmg_max.physical = wd.phys_max;
            base_dmg_min.fire = wd.fire_min;
            base_dmg_max.fire = wd.fire_max;
            base_dmg_min.cold = wd.cold_min;
            base_dmg_max.cold = wd.cold_max;
            base_dmg_min.lightning = wd.lightning_min;
            base_dmg_max.lightning = wd.lightning_max;
            base_dmg_min.chaos = wd.chaos_min;
            base_dmg_max.chaos = wd.chaos_max;
        }
    }
    // Step 2 add flat
    base_dmg_min.physical += db.sum_base(StatId::PhysicalMin, &off_ctx);
    base_dmg_max.physical += db.sum_base(StatId::PhysicalMax, &off_ctx);
    base_dmg_min.fire += db.sum_base(StatId::FireMin, &off_ctx);
    base_dmg_max.fire += db.sum_base(StatId::FireMax, &off_ctx);
    base_dmg_min.cold += db.sum_base(StatId::ColdMin, &off_ctx);
    base_dmg_max.cold += db.sum_base(StatId::ColdMax, &off_ctx);
    base_dmg_min.lightning += db.sum_base(StatId::LightningMin, &off_ctx);
    base_dmg_max.lightning += db.sum_base(StatId::LightningMax, &off_ctx);
    base_dmg_min.chaos += db.sum_base(StatId::ChaosMin, &off_ctx);
    base_dmg_max.chaos += db.sum_base(StatId::ChaosMax, &off_ctx);

    let base_damage = DamageSet {
        physical: (base_dmg_min.physical + base_dmg_max.physical) / 2.0,
        fire: (base_dmg_min.fire + base_dmg_max.fire) / 2.0,
        cold: (base_dmg_min.cold + base_dmg_max.cold) / 2.0,
        lightning: (base_dmg_min.lightning + base_dmg_max.lightning) / 2.0,
        chaos: (base_dmg_min.chaos + base_dmg_max.chaos) / 2.0,
    };

    //Handle conversions
    let conv_table = conversions::build_conversion_table(db, &off_ctx);
    let final_damage = conversions::apply_conversion(&base_damage, &conv_table, db, &off_ctx);

    // 5.7.5 — Critical Strikes
    // PoB: baseCrit = source.CritChance (weapon for attacks, gem static_data for spells)
    let crit_base_pct = if is_attack {
        // Weapon crit chance is post-local-mod, stored as a percentage (e.g. 6.5 = 6.5%)
        equipped
            .get(&ItemSlot::Weapon1)
            .and_then(|w| w.weapon_data.as_ref())
            .map(|wd| wd.crit_chance)
            .unwrap_or(0.0)
    } else {
        // Spell base crit from gem static_data.crit_chance — hundredths-of-percent (600 = 6.00%)
        gem_def
            .static_data
            .crit_chance
            .map(|c| c as f64 / 100.0)
            .unwrap_or(0.0)
    };
    let crit_flat = db.sum_base(StatId::CritChance, &off_ctx);
    let crit_inc = db.sum_inc(StatId::CritChance, &off_ctx);
    let crit_more = db.product_more(StatId::CritChance, &off_ctx);
    let crit_pct =
        ((crit_base_pct + crit_flat) * (1.0 + crit_inc / 100.0) * crit_more).clamp(0.0, 100.0);
    let crit_chance = crit_pct / 100.0; // 0.0–1.0

    // Crit multiplier — setup.rs seeds CritMultiplier BASE 50 (= 150% total minimum)
    let crit_multi_pct = db.sum_base(StatId::CritMultiplier, &off_ctx);
    let crit_multiplier = 1.0 + (crit_multi_pct / 100.0).max(0.0);

    // Effective damage multiplier from crits (weighted average: normal + crit)
    let effective_crit = 1.0 + crit_chance * (crit_multiplier - 1.0);

    // 5.7.6 — Speed
    // PoB: speed = base_rate * round((1 + inc/100) * more, 2dp)
    let speed = {
        let inc = db.sum_inc(StatId::Speed, &off_ctx);
        let more = db.product_more(StatId::Speed, &off_ctx);
        // Round combined speed modifier to 2 decimal places (matches PoB rounding)
        let speed_mod = ((1.0 + inc / 100.0) * more * 100.0).round() / 100.0;
        if is_attack {
            let base_rate = equipped
                .get(&ItemSlot::Weapon1)
                .and_then(|w| w.weapon_data.as_ref())
                .map(|wd| wd.attacks_per_second)
                .unwrap_or(1.2);
            (base_rate * speed_mod).max(0.0)
        } else {
            // cast_time is Option<u32> in milliseconds
            let cast_time_s = gem_def
                .cast_time
                .map(|ms| ms as f64 / 1000.0)
                .unwrap_or(1.0);
            let base_speed = 1.0 / cast_time_s;
            (base_speed * speed_mod).max(0.0)
        }
    };

    // 5.7.7 — Accuracy / Hit Chance (attacks only; spells always hit)
    let hit_chance_pct = if is_attack {
        let acc_base = db.sum_base(StatId::Accuracy, &off_ctx);
        let acc_inc = db.sum_inc(StatId::Accuracy, &off_ctx);
        let acc_more = db.product_more(StatId::Accuracy, &off_ctx);
        let accuracy = (acc_base * (1.0 + acc_inc / 100.0) * acc_more)
            .max(0.0)
            .floor();
        // Enemy evasion scaled to player level from PoB monsterEvasionTable.
        let idx = (level.clamp(1, 100) - 1) as usize;
        let enemy_evasion = MONSTER_EVASION[idx] as f64;
        // PoB hit-chance formula: floor(acc / (acc + evasion^0.8) * 100 + 0.5), clamp 5–95%
        (accuracy / (accuracy + enemy_evasion.powf(0.8)) * 100.0 + 0.5)
            .floor()
            .clamp(5.0, 95.0)
    } else {
        100.0
    };

    // 5.7.8 — Final hit DPS
    let avg_hit = final_damage.total();
    let hit_chance_f = hit_chance_pct / 100.0;
    let elem_mult = speed * hit_chance_f * effective_crit;
    let phys_dps = final_damage.physical * elem_mult;
    let fire_dps = final_damage.fire * elem_mult;
    let cold_dps = final_damage.cold * elem_mult;
    let light_dps = final_damage.lightning * elem_mult;
    let chaos_dps = final_damage.chaos * elem_mult;
    let hit_dps = avg_hit * elem_mult;

    // 5.7.9 — Ailment DoT (simplified Phase 5)
    // DoT context: preserve weapon/melee/projectile flags, remove HIT, add DOT.
    let mut dot_ctx = off_ctx.clone();
    dot_ctx.flags &= !ModFlag::HIT;
    dot_ctx.flags |= ModFlag::DOT;

    // Bleed — 70% of physical per second (attacks only, scaled by bleed application chance)
    let bleed_dps = if is_attack && final_damage.physical > 0.0 {
        let bleed_chance = db
            .calculate(StatId::BleedChance, &off_ctx)
            .clamp(0.0, 100.0)
            / 100.0;
        if bleed_chance > 0.0 {
            let mut bleed_ctx = dot_ctx.clone();
            bleed_ctx.flags |= ModFlag::AILMENT;
            bleed_ctx.key_flags |=
                KeywordFlag::BLEED | KeywordFlag::AILMENT | KeywordFlag::PHYSICAL_DOT;
            let dot_multi = 1.0 + db.sum_base(StatId::DotMultiplier, &bleed_ctx) / 100.0;
            final_damage.physical * 0.70 * dot_multi * bleed_chance
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Poison — 30% of (physical + chaos) per second, scaled by poison chance
    let poison_dps = {
        let poison_source = final_damage.physical + final_damage.chaos;
        if poison_source > 0.0 {
            let poison_chance = db
                .calculate(StatId::PoisonChance, &off_ctx)
                .clamp(0.0, 100.0)
                / 100.0;
            if poison_chance > 0.0 {
                let mut poison_ctx = dot_ctx.clone();
                poison_ctx.flags |= ModFlag::AILMENT;
                poison_ctx.key_flags |=
                    KeywordFlag::POISON | KeywordFlag::AILMENT | KeywordFlag::CHAOS_DOT;
                let dot_multi = 1.0
                    + db.sum_base(StatId::DotMultiplier, &poison_ctx) / 100.0
                    + db.sum_base(StatId::ChaosDotMultiplier, &poison_ctx) / 100.0;
                poison_source * 0.30 * dot_multi * poison_chance
            } else {
                0.0
            }
        } else {
            0.0
        }
    };

    // Ignite — 90% of fire per second, scaled by ignite chance
    let ignite_dps = if final_damage.fire > 0.0 {
        let ignite_chance = db
            .calculate(StatId::IgniteChance, &off_ctx)
            .clamp(0.0, 100.0)
            / 100.0;
        if ignite_chance > 0.0 {
            let mut ignite_ctx = dot_ctx.clone();
            ignite_ctx.flags |= ModFlag::AILMENT;
            ignite_ctx.key_flags |=
                KeywordFlag::IGNITE | KeywordFlag::AILMENT | KeywordFlag::FIRE_DOT;
            let dot_multi = 1.0
                + db.sum_base(StatId::DotMultiplier, &ignite_ctx) / 100.0
                + db.sum_base(StatId::FireDotMultiplier, &ignite_ctx) / 100.0;
            final_damage.fire * 0.90 * dot_multi * ignite_chance
        } else {
            0.0
        }
    } else {
        0.0
    };

    let dot_dps = bleed_dps + poison_dps + ignite_dps;

    OffenceResult {
        total_dps: hit_dps + dot_dps,
        hit_dps,
        average_hit: avg_hit * effective_crit,
        crit_chance: crit_pct,
        crit_multiplier,
        hit_chance: hit_chance_pct,
        attack_speed: if is_attack { speed } else { 0.0 },
        cast_speed: if !is_attack { speed } else { 0.0 },
        speed,
        is_attack,
        phys_dps,
        fire_dps,
        cold_dps,
        lightning_dps: light_dps,
        chaos_dps,
        dot_dps,
        bleed_dps,
        poison_dps,
        ignite_dps,
    }
}

/// Maps the equipped main-hand `ItemType` to its `ModFlag` combination.
///
/// Mirrors PoB's `weaponTypeInfo[type].flag` plus the class flags
/// (`WeaponMelee`/`WeaponRanged`, `Weapon1H`/`Weapon2H`).
/// Called with `None` when the slot is empty, which resolves to `UNARMED`.
fn weapon_flags_for(item_type: Option<ItemType>) -> ModFlag {
    match item_type {
        // ── 1-Handed Melee ────────────────────────────────────────────────────
        Some(ItemType::Claw) => {
            ModFlag::CLAW | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_1H
        }
        Some(ItemType::Dagger | ItemType::RuneDagger) => {
            ModFlag::DAGGER | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_1H
        }
        Some(ItemType::OneHandSword | ItemType::ThrustingOneHandSword) => {
            ModFlag::SWORD | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_1H
        }
        Some(ItemType::OneHandAxe) => {
            ModFlag::AXE | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_1H
        }
        Some(ItemType::OneHandMace | ItemType::Sceptre) => {
            ModFlag::MACE | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_1H
        }
        // ── 1-Handed Ranged ───────────────────────────────────────────────────
        Some(ItemType::Wand) => {
            ModFlag::WAND | ModFlag::WEAPON | ModFlag::WEAPON_RANGED | ModFlag::WEAPON_1H
        }
        // ── 2-Handed Melee ────────────────────────────────────────────────────
        Some(ItemType::TwoHandSword) => {
            ModFlag::SWORD | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_2H
        }
        Some(ItemType::TwoHandAxe) => {
            ModFlag::AXE | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_2H
        }
        Some(ItemType::TwoHandMace) => {
            ModFlag::MACE | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_2H
        }
        Some(ItemType::Staff | ItemType::Warstaff) => {
            ModFlag::STAFF | ModFlag::WEAPON | ModFlag::WEAPON_MELEE | ModFlag::WEAPON_2H
        }
        // ── 2-Handed Ranged ───────────────────────────────────────────────────
        Some(ItemType::Bow) => {
            ModFlag::BOW | ModFlag::WEAPON | ModFlag::WEAPON_RANGED | ModFlag::WEAPON_2H
        }
        // ── Unarmed / non-weapon slot ─────────────────────────────────────────
        _ => ModFlag::UNARMED,
    }
}
