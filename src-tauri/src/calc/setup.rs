use crate::{
    data::{SourceId, StatId},
    modifier::{parser, ModDB, ModDBLayers, ModType, Modifier},
};

pub fn setup_moddb(layers: &ModDBLayers, level: u32) -> ModDB {
    let mut db = layers.merged();
    seed_base_values(&mut db, level);
    db
}

fn seed_base_values(db: &mut ModDB, level: u32) {
    let src = SourceId(0);
    // --- Resource pool bases ---
    // Life: 38 base + 12 per level (PoB: initModDB)
    db.add_mod(parser::simple_mod(
        StatId::Life,
        ModType::Base,
        38.0 + 12.0 * level as f64,
        src,
    ));
    // Mana: 34 base + 6 per level (PoB: initModDB, base=34)
    db.add_mod(parser::simple_mod(
        StatId::Mana,
        ModType::Base,
        34.0 + 6.0 * level as f64,
        src,
    ));
    // Mana regen: 1.75% of max mana per second (PoB: initModDB "ManaRegenerationPercent")
    db.add_mod(parser::simple_mod(
        StatId::ManaRegenPercent,
        ModType::Base,
        1.75,
        src,
    ));

    // --- Resistance caps (default 75%) ---
    db.add_mod(parser::simple_mod(
        StatId::FireResistMax,
        ModType::Base,
        75.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::ColdResistMax,
        ModType::Base,
        75.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::LightningResistMax,
        ModType::Base,
        75.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::ChaosResistMax,
        ModType::Base,
        75.0,
        src,
    ));

    // --- Resistance penalties
    db.add_mod(parser::simple_mod(
        StatId::FireResist,
        ModType::Base,
        -60.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::ColdResist,
        ModType::Base,
        -60.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::LightningResist,
        ModType::Base,
        -60.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::ChaosResist,
        ModType::Base,
        -60.0,
        src,
    ));

    // --- Block caps ---
    db.add_mod(parser::simple_mod(
        StatId::BlockChanceMax,
        ModType::Base,
        75.0,
        src,
    ));

    // --- Base accuracy (2 per level, before DEX injection) ---
    db.add_mod(parser::simple_mod(
        StatId::Accuracy,
        ModType::Base,
        2.0 * level as f64,
        src,
    ));

    db.add_mod(parser::simple_mod(
        StatId::SpellBlockChanceMax,
        ModType::Base,
        75.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::SpellDodgeChanceMax,
        ModType::Base,
        75.0,
        src,
    ));

    // --- Charge maximums ---
    db.add_mod(parser::simple_mod(
        StatId::PowerChargesMax,
        ModType::Base,
        3.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::FrenzyChargesMax,
        ModType::Base,
        3.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::EnduranceChargesMax,
        ModType::Base,
        3.0,
        src,
    ));

    // --- Leech caps ---
    db.add_mod(parser::simple_mod(
        StatId::MaxLifeLeechRate,
        ModType::Base,
        20.0,
        src,
    )); // % of max life/sec
    db.add_mod(parser::simple_mod(
        StatId::MaxManaLeechRate,
        ModType::Base,
        20.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::MaxLifeLeechInstance,
        ModType::Base,
        10.0,
        src,
    )); // % per instance
    db.add_mod(parser::simple_mod(
        StatId::MaxManaLeechInstance,
        ModType::Base,
        10.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::MaxEnergyShieldLeechRate,
        ModType::Base,
        10.0,
        src,
    ));
    db.add_mod(parser::simple_mod(
        StatId::MaxEnergyShieldLeechInstance,
        ModType::Base,
        10.0,
        src,
    ));

    // --- Damage reduction cap ---
    db.add_mod(parser::simple_mod(
        StatId::DamageReductionMax,
        ModType::Base,
        90.0,
        src,
    ));

    // --- Base crit multiplier ---
    // +50% extra damage on crit = 150% total (PoB: initModDB seeds CritMultiplier BASE 50).
    // CalcOffence computes: crit_multiplier = 1.0 + max(0, sum("BASE","CritMultiplier") / 100).
    // Without this seed the base would be 1.0 (100% crit mult = no bonus), undercounting by 50%.
    db.add_mod(parser::simple_mod(
        StatId::CritMultiplier,
        ModType::Base,
        50.0,
        src,
    ));

    // --- Dual-wield inherent bonuses ---
    // +10% more attack speed, +15% block chance (PoB: initModDB)
    // These are conditional on DualWielding — add with Condition tag
    // (Will be handled via Condition("DualWielding") tag when implemented)

    // --- Charge stat bonuses (per charge) ---
    // Power charge: +40% crit chance per charge (Multiplier tag)
    // Frenzy charge: +4% attack speed, +4% cast speed, +4% more damage per charge
    // Endurance charge: +4% phys damage reduction, +4% all max elemental resist per charge
    // (These are injected as modifiers with Multiplier("PowerCharge") etc. tags)
}
