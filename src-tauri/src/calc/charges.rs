use rustc_hash::FxHashMap;

use crate::{
    data::StatId,
    modifier::{intern, CalcContext, ModDB},
};

pub struct ChargeState {
    pub power: u32,
    pub frenzy: u32,
    pub endurance: u32,
    pub multipliers: FxHashMap<&'static str, f64>,
}

pub fn process_charges(db: &ModDB, ctx: &CalcContext) -> ChargeState {
    // Check if charge usage is enabled (Phase 7: from config tab)
    // For Phase 5: assume charges are active if configured

    let power_max = db.sum_base(StatId::PowerChargesMax, ctx) as u32;
    let frenzy_max = db.sum_base(StatId::FrenzyChargesMax, ctx) as u32;
    let endurance_max = db.sum_base(StatId::EnduranceChargesMax, ctx) as u32;

    // Set multipliers for Multiplier("PowerCharge"), etc.
    let mut multipliers = FxHashMap::default();
    multipliers.insert(intern("PowerCharge"), power_max as f64);
    multipliers.insert(intern("FrenzyCharge"), frenzy_max as f64);
    multipliers.insert(intern("EnduranceCharge"), endurance_max as f64);

    // PoB charge bonuses (from CalcSetup initModDB):
    // Power: +40% crit chance per charge → inject as INC with Multiplier tag
    // Frenzy: +4% attack speed, +4% cast speed, +4% more damage per charge
    // Endurance: +4% phys damage reduction per charge

    // These bonuses are injected into the ModDB as modifiers with Multiplier tags
    // The tags are already in stat_table.rs — they just need the multiplier values
    // in CalcContext to be evaluated by effective_value()

    ChargeState {
        power: power_max,
        frenzy: frenzy_max,
        endurance: endurance_max,
        multipliers,
    }
}
