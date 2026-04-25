use crate::calc::perform::DefenceResult;
use crate::data::StatId;
use crate::modifier::{CalcContext, ModDB};

/// Calculate all defensive stats from the merged ModDB.
/// Matches PoB CalcDefence.lua logic (Phase 5.5).
pub fn calc_defense(db: &ModDB, _level: u32, ctx: &CalcContext) -> DefenceResult {
    // --- Keystones ---
    let chaos_inoculation = db.has_flag(StatId::ChaosInoculation, ctx);
    let iron_reflexes = db.has_flag(StatId::IronReflexes, ctx);
    let zealots_oath = db.has_flag(StatId::ZealotsOath, ctx);

    // --- Life ---
    // PoB: base*(1+inc/100)*more, min 1; with CI → 1
    let life = if chaos_inoculation {
        1u32
    } else {
        db.calculate(StatId::Life, ctx).round().max(1.0) as u32
    };

    // --- Mana ---
    // PoB: base*(1+inc/100)*more (34+6*lvl seeded in setup.rs)
    let mana = db.calculate(StatId::Mana, ctx).round().max(0.0) as u32;

    // --- Energy Shield ---
    // base from items/mods; INT/10 INC injected in attributes.rs
    let energy_shield = db.calculate(StatId::EnergyShield, ctx).round().max(0.0) as u32;

    // --- Evasion (pre-Iron Reflexes) ---
    // PoB: Evasion = base*(1+inc/100)*more. DEX/5 INC injected in attributes.rs.
    let evasion_raw = db.calculate(StatId::Evasion, ctx);

    // --- Armour ---
    // Iron Reflexes: all evasion converts to armour (added as base before armour calc)
    // PoB: if IronReflexes → armourBase += evasion, evasion = 0
    let armour = {
        let base = db.sum_base(StatId::Armour, ctx) + if iron_reflexes { evasion_raw } else { 0.0 };
        let inc = db.sum_inc(StatId::Armour, ctx);
        let more = db.product_more(StatId::Armour, ctx);
        (base * (1.0 + inc / 100.0) * more).round().max(0.0) as u32
    };

    let evasion = if iron_reflexes {
        0u32
    } else {
        evasion_raw.round().max(0.0) as u32
    };

    // --- Ward ---
    let ward = db.calculate(StatId::Ward, ctx).round().max(0.0) as u32;

    // --- Elemental Resistances ---
    // PoB: total = base*(1+inc/100)*more, where inc includes ElementalResist INC.
    // Cap = min(90, sum_base(ResistMax) + sum_base(ElementalResistMax)). Truncate (floor).
    // Clamp effective to -200..cap. Overcap = total - cap (clamped ≥ 0).
    let calc_ele_resist = |base_stat: StatId, max_stat: StatId| -> (i32, i32, i32) {
        let base = db.sum_base(base_stat, ctx);
        let inc = db.sum_inc(base_stat, ctx) + db.sum_inc(StatId::ElementalResist, ctx);
        let more = db.product_more(base_stat, ctx) * db.product_more(StatId::ElementalResist, ctx);
        let total = (base * (1.0 + inc / 100.0) * more).floor() as i32;
        let total = total.max(-200);
        // ElementalResistMax can also raise caps
        let cap = (db.sum_base(max_stat, ctx) + db.sum_base(StatId::ElementalResistMax, ctx))
            .min(90.0) as i32;
        let effective = total.min(cap);
        let overcap = (total - cap).max(0);
        (effective, cap, overcap)
    };

    // Chaos resist: no ElementalResist shared bonus
    let calc_chaos_resist = || -> (i32, i32, i32) {
        let total = db.calculate(StatId::ChaosResist, ctx).floor() as i32;
        let total = total.max(-200);
        let cap = db.sum_base(StatId::ChaosResistMax, ctx).min(90.0) as i32;
        let effective = total.min(cap);
        let overcap = (total - cap).max(0);
        (effective, cap, overcap)
    };

    let (fire_resist, fire_resist_cap, fire_resist_overcap) =
        calc_ele_resist(StatId::FireResist, StatId::FireResistMax);
    let (cold_resist, cold_resist_cap, cold_resist_overcap) =
        calc_ele_resist(StatId::ColdResist, StatId::ColdResistMax);
    let (lightning_resist, lightning_resist_cap, lightning_resist_overcap) =
        calc_ele_resist(StatId::LightningResist, StatId::LightningResistMax);
    let (chaos_resist, chaos_resist_cap, chaos_resist_overcap) = calc_chaos_resist();

    // --- Block Chance ---
    // PoB: base*(1+inc/100)*more, floor, capped at BlockChanceMax (75 from seed)
    let block_chance = {
        let cap = db.sum_base(StatId::BlockChanceMax, ctx);
        db.calculate(StatId::BlockChance, ctx)
            .floor()
            .min(cap)
            .max(0.0)
    };

    let spell_block_chance = {
        let cap = db.sum_base(StatId::SpellBlockChanceMax, ctx);
        db.calculate(StatId::SpellBlockChance, ctx)
            .floor()
            .min(cap)
            .max(0.0)
    };

    // --- Spell Suppression ---
    // PoB: chance capped at 100, effect = 50 + bonus (data.misc.SuppressionEffect = 50).
    // Store effective suppression chance (0-100).
    let suppression_chance = db
        .sum_base(StatId::SpellSuppressionChance, ctx)
        .min(100.0)
        .max(0.0);
    let spell_suppression = suppression_chance;

    // --- Dodge ---
    // Attack dodge: BASE-only (from evasion mechanics), capped at 75
    let attack_dodge = db
        .sum_base(StatId::AttackDodgeChance, ctx)
        .floor()
        .min(75.0)
        .max(0.0);

    // Spell dodge: BASE, capped at SpellDodgeChanceMax (75 from seed)
    let spell_dodge_max = db.sum_base(StatId::SpellDodgeChanceMax, ctx);
    let spell_dodge = db
        .sum_base(StatId::SpellDodgeChance, ctx)
        .floor()
        .min(spell_dodge_max)
        .max(0.0);

    // --- Regen Recovery Rate Multipliers ---
    // PoB: recoveryRateMod = (1+inc/100)*more for each resource
    let life_recovery_rate = (1.0 + db.sum_inc(StatId::LifeRecoveryRate, ctx) / 100.0)
        * db.product_more(StatId::LifeRecoveryRate, ctx);
    let mana_recovery_rate = (1.0 + db.sum_inc(StatId::ManaRecoveryRate, ctx) / 100.0)
        * db.product_more(StatId::ManaRecoveryRate, ctx);
    let es_recovery_rate = (1.0 + db.sum_inc(StatId::EnergyShieldRecoveryRate, ctx) / 100.0)
        * db.product_more(StatId::EnergyShieldRecoveryRate, ctx);

    // --- Life Regen ---
    // PoB: lifeRegenBase = flat_LifeRegen + Life * LifeRegenPercent/100
    //      * (1+ManaRegeneration_INC/100)*more * lifeRecoveryRateMod
    // With ZealotsOath: life regen → ES instead (so life_regen = 0)
    let life_regen_flat = db.sum_base(StatId::LifeRegen, ctx);
    let life_regen_pct = db.sum_base(StatId::LifeRegenPercent, ctx);
    let life_regen_inc = db.sum_inc(StatId::LifeRegen, ctx);
    let life_regen_more = db.product_more(StatId::LifeRegen, ctx);
    let life_regen_base = (life_regen_flat + life as f64 * life_regen_pct / 100.0)
        * (1.0 + life_regen_inc / 100.0)
        * life_regen_more;
    let life_regen = if zealots_oath {
        0.0
    } else {
        life_regen_base * life_recovery_rate
    };

    // --- Mana Regen ---
    // PoB: manaRegenBase = flat_ManaRegeneration + Mana * ManaRegenerationPercent/100
    //      * (1+ManaRegeneration_INC/100)*more * manaRecoveryRateMod
    // Default 1.75%/sec is seeded as ManaRegenPercent BASE in setup.rs
    let mana_regen_flat = db.sum_base(StatId::ManaRegeneration, ctx);
    let mana_regen_pct = db.sum_base(StatId::ManaRegenPercent, ctx);
    let mana_regen_inc = db.sum_inc(StatId::ManaRegeneration, ctx);
    let mana_regen_more = db.product_more(StatId::ManaRegeneration, ctx);
    let mana_regen = (mana_regen_flat + mana as f64 * mana_regen_pct / 100.0)
        * (1.0 + mana_regen_inc / 100.0)
        * mana_regen_more
        * mana_recovery_rate;

    // --- ES Regen ---
    // Zealot's Oath: life regen base → ES (multiplied by ES recovery rate, not life)
    // Normal: EnergyShieldRegenPercent BASE % of ES per second (e.g. from unique items)
    let es_regen = if zealots_oath {
        life_regen_base * es_recovery_rate
    } else {
        let es_regen_pct = db.sum_base(StatId::EnergyShieldRegenPercent, ctx);
        energy_shield as f64 * es_regen_pct / 100.0 * es_recovery_rate
    };

    // --- ES Recharge ---
    // PoB: Rate = ES * 0.20 * (1+EnergyShieldRecharge_INC/100)*more * esRecoveryRate
    //      Delay = 2.0 / (1 + EnergyShieldRechargeFaster_INC/100)
    let es_recharge_mod = (1.0 + db.sum_inc(StatId::EnergyShieldRecharge, ctx) / 100.0)
        * db.product_more(StatId::EnergyShieldRecharge, ctx);
    let es_recharge = energy_shield as f64 * 0.20 * es_recharge_mod * es_recovery_rate;
    let es_recharge_delay =
        2.0 / (1.0 + db.sum_inc(StatId::EnergyShieldRechargeFaster, ctx) / 100.0);

    // --- Leech Caps ---
    // Seeded in setup.rs: MaxLifeLeechRate = 20, MaxManaLeechRate = 20 (% of pool/sec)
    let life_leech_rate_max = db.sum_base(StatId::MaxLifeLeechRate, ctx) / 100.0 * life as f64;
    let mana_leech_rate_max = db.sum_base(StatId::MaxManaLeechRate, ctx) / 100.0 * mana as f64;

    // --- Movement Speed ---
    // PoB: Override check first; else (1+inc/100)*more
    let movement_speed_mod = if let Some(ov) = db.get_override(StatId::MovementSpeed, ctx) {
        ov / 100.0
    } else {
        let inc = db.sum_inc(StatId::MovementSpeed, ctx);
        let more = db.product_more(StatId::MovementSpeed, ctx);
        (1.0 + inc / 100.0) * more
    };

    // --- Reserved Resources ---
    let mana_reserved = db.sum_base(StatId::ManaReserved, ctx).max(0.0) as u32;
    let mana_unreserved = mana.saturating_sub(mana_reserved);
    let life_reserved = db.sum_base(StatId::LifeReserved, ctx).max(0.0) as u32;
    let life_unreserved = life.saturating_sub(life_reserved);

    DefenceResult {
        life,
        mana,
        energy_shield,
        armour,
        evasion,
        ward,
        fire_resist,
        fire_resist_cap,
        fire_resist_overcap,
        cold_resist,
        cold_resist_cap,
        cold_resist_overcap,
        lightning_resist,
        lightning_resist_cap,
        lightning_resist_overcap,
        chaos_resist,
        chaos_resist_cap,
        chaos_resist_overcap,
        block_chance,
        spell_block_chance,
        spell_suppression,
        attack_dodge,
        spell_dodge,
        life_regen,
        mana_regen,
        es_regen,
        es_recharge,
        es_recharge_delay,
        life_leech_rate_max,
        mana_leech_rate_max,
        movement_speed_mod,
        mana_unreserved,
        life_unreserved,
    }
}

/// Returns hit chance percentage (clamped 5-95%).
/// PoB formula: floor(accuracy / (accuracy + evasion^0.8) * 100 + 0.5)
fn hit_chance(evasion: f64, accuracy: f64) -> f64 {
    if accuracy <= 0.0 {
        return 5.0;
    }
    (accuracy / (accuracy + evasion.powf(0.8)) * 100.0 + 0.5)
        .floor()
        .clamp(5.0, 95.0)
}

/// Returns damage reduction percentage from armour against raw_damage
fn armour_reduction(armour: f64, raw_damage: f64) -> f64 {
    if armour == 0.0 && raw_damage == 0.0 {
        return 0.0;
    }
    armour / (armour + raw_damage * 5.0) * 100.0
}
