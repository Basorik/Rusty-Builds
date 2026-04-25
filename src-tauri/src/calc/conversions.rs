use crate::data::StatId;
use crate::modifier::{CalcContext, ModDB};

///Tracking for damage values per element through conversion
#[derive(Debug, Clone, Default)]
pub struct DamageSet {
    pub physical: f64,
    pub lightning: f64,
    pub cold: f64,
    pub fire: f64,
    pub chaos: f64,
}
impl DamageSet {
    /// Sum of all elements (for average hit DPS calculation)
    pub fn total(&self) -> f64 {
        self.physical + self.lightning + self.cold + self.fire + self.chaos
    }
}

///Per source table of conversions
pub struct SourceConv {
    pub conv_mults: [f64; 5],
    pub remaining_mult: f64,
    pub gains: [f64; 5],
}

/// Full conversion table for one skill cast.
pub struct ConversionTable {
    /// Indexed by source element: [Phys, Light, Cold, Fire, Chaos].
    pub src: [SourceConv; 5],
}

/// Builds `SourceConv` for one source element using PoB's two-layer conversion priority.
///
/// Priority (mirrors PoB `buildConversionTable`):
/// 1. Sum all skill-layer conversion % into `skill_pct[dst]` (gem-internal conversions).
///    If the total exceeds 100%, scale all entries down; global layer is discarded entirely.
/// 2. Sum all global-layer conversion % into `global_pct[dst]` (passives/gear/buffs).
///    If `skill_total + global_total > 100%`, scale global down to fill remaining space.
/// 3. `remaining_mult = 1.0 - clamped_total` — fraction that stays as its source type.
/// 4. Gain-as-extra is uncapped and does not reduce the source amount.
fn build_source_conv(
    db: &ModDB,
    ctx: &CalcContext,
    skill_stats: &[(usize, StatId)],
    global_stats: &[(usize, StatId)],
    gain_stats: &[(usize, StatId)],
) -> SourceConv {
    // Skill conversion layer
    let mut skill_pct = [0.0_f64; 5];
    for &(dst, stat) in skill_stats {
        skill_pct[dst] = db.sum_base(stat, ctx).max(0.0);
    }
    let skill_total: f64 = skill_pct.iter().sum();
    if skill_total > 100.0 {
        let scale = 100.0 / skill_total;
        for v in &mut skill_pct {
            *v *= scale;
        }
    }
    let skill_used: f64 = skill_pct.iter().sum::<f64>().min(100.0);

    // Global conversion layer — fills remaining space after skill conversion
    let mut global_pct = [0.0_f64; 5];
    for &(dst, stat) in global_stats {
        global_pct[dst] = db.sum_base(stat, ctx).max(0.0);
    }
    let global_total: f64 = global_pct.iter().sum();
    let remaining_space = (100.0 - skill_used).max(0.0);
    if global_total > remaining_space && global_total > 0.0 {
        let scale = remaining_space / global_total;
        for v in &mut global_pct {
            *v *= scale;
        }
    }

    // Combine into 0.0–1.0 fractions
    let mut conv_mults = [0.0_f64; 5];
    for i in 0..5 {
        conv_mults[i] = (skill_pct[i] + global_pct[i]) / 100.0;
    }
    let remaining_mult = (1.0 - conv_mults.iter().sum::<f64>()).max(0.0);

    // Gain-as-extra: uncapped, additive, does not reduce source
    let mut gains = [0.0_f64; 5];
    for &(dst, stat) in gain_stats {
        gains[dst] = db.sum_base(stat, ctx).max(0.0) / 100.0;
    }

    SourceConv {
        conv_mults,
        remaining_mult,
        gains,
    }
}

pub fn build_conversion_table(db: &ModDB, ctx: &CalcContext) -> ConversionTable {
    // Element indices: Physical=0, Lightning=1, Cold=2, Fire=3, Chaos=4
    // Conversion flows strictly left-to-right along this chain (PoE rule).
    // Chaos has no outbound conversion paths.

    // Physical (src=0) → Lightning(1), Cold(2), Fire(3), Chaos(4)
    let phys = build_source_conv(
        db,
        ctx,
        &[
            (1, StatId::SkillPhysicalDamageConvertToLightning),
            (2, StatId::SkillPhysicalDamageConvertToCold),
            (3, StatId::SkillPhysicalDamageConvertToFire),
            (4, StatId::SkillPhysicalDamageConvertToChaos),
        ],
        &[
            (1, StatId::PhysicalDamageConvertToLightning),
            (2, StatId::PhysicalDamageConvertToCold),
            (3, StatId::PhysicalDamageConvertToFire),
            (4, StatId::PhysicalDamageConvertToChaos),
        ],
        &[
            (1, StatId::PhysicalDamageGainAsLightning),
            (2, StatId::PhysicalDamageGainAsCold),
            (3, StatId::PhysicalDamageGainAsFire),
            (4, StatId::PhysicalDamageGainAsChaos),
        ],
    );

    // Lightning (src=1) → Cold(2), Fire(3), Chaos(4)
    let light = build_source_conv(
        db,
        ctx,
        &[
            (2, StatId::SkillLightningDamageConvertToCold),
            (3, StatId::SkillLightningDamageConvertToFire),
            (4, StatId::SkillLightningDamageConvertToChaos),
        ],
        &[
            (2, StatId::LightningDamageConvertToCold),
            (3, StatId::LightningDamageConvertToFire),
            (4, StatId::LightningDamageConvertToChaos),
        ],
        &[(4, StatId::LightningDamageGainAsChaos)],
    );

    // Cold (src=2) → Fire(3), Chaos(4)
    let cold = build_source_conv(
        db,
        ctx,
        &[
            (3, StatId::SkillColdDamageConvertToFire),
            (4, StatId::SkillColdDamageConvertToChaos),
        ],
        &[
            (3, StatId::ColdDamageConvertToFire),
            (4, StatId::ColdDamageConvertToChaos),
        ],
        &[(3, StatId::ColdDamageGainAsFire)],
    );

    // Fire (src=3) → Chaos(4)
    let fire = build_source_conv(
        db,
        ctx,
        &[(4, StatId::SkillFireDamageConvertToChaos)],
        &[(4, StatId::FireDamageConvertToChaos)],
        &[(4, StatId::FireDamageGainAsChaos)],
    );

    // Chaos (src=4) — no outbound conversion in PoE 1
    let chaos = SourceConv {
        conv_mults: [0.0; 5],
        remaining_mult: 1.0,
        gains: [0.0; 5],
    };

    ConversionTable {
        src: [phys, light, cold, fire, chaos],
    }
}

/// Compute inc/more multiplier for one element type.
///
/// Mirrors PoB's unpack(modNames) pattern — each element queries its own stat
/// plus generic "Damage" (and Elemental for fire/cold/lightning).
/// The CalcContext must already have ATTACK/SPELL/HIT flags set by the caller.
fn element_scale(db: &ModDB, ctx: &CalcContext, elem: usize) -> f64 {
    // Element indices: Physical=0, Lightning=1, Cold=2, Fire=3, Chaos=4
    let (primary, elemental) = match elem {
        0 => (StatId::PhysicalDamage, false),
        1 => (StatId::LightningDamage, true),
        2 => (StatId::ColdDamage, true),
        3 => (StatId::FireDamage, true),
        4 => (StatId::ChaosDamage, false),
        _ => return 1.0,
    };

    let mut inc = db.sum_inc(primary, ctx) + db.sum_inc(StatId::Damage, ctx);
    let mut more = db.product_more(primary, ctx) * db.product_more(StatId::Damage, ctx);

    if elemental {
        inc += db.sum_inc(StatId::ElementalDamage, ctx);
        more *= db.product_more(StatId::ElementalDamage, ctx);
    }

    (1.0 + inc / 100.0) * more
}

/// Apply conversion and scale each element by its inc/more modifiers in one pass.
///
/// Processes elements in topological order (Physical → Lightning → Cold → Fire → Chaos)
/// so that converted damage goes through the SOURCE element's inc/more pipeline first,
/// then the DESTINATION element's. This matches PoB's recursive `calcDamage()` behaviour.
///
/// The returned `DamageSet` has inc/more already folded in. Do NOT apply inc/more
/// again in offense.rs after calling this function.
///
/// Note: DealNoX flags (Avatar of Fire) must be applied by the caller after this
/// returns, since they require ModDB + CalcContext access.
pub fn apply_conversion(
    base: &DamageSet,
    table: &ConversionTable,
    db: &ModDB,
    ctx: &CalcContext,
) -> DamageSet {
    let base_arr = [
        base.physical,
        base.lightning,
        base.cold,
        base.fire,
        base.chaos,
    ];

    // ── Phase 1: resolve chained conversions ─────────────────────────────
    // Elements can only convert to higher-indexed elements (phys<light<cold<fire<chaos).
    // A single forward pass resolves all chains (e.g. phys→light→fire).
    //
    // `flow[i]`    — amount of element i after conversions out (stays as element i).
    // `total_in[i]`— total flowing INTO element i (base + converted from all upstream).
    //                Captured before applying remaining_mult so gain-as-extra uses it.
    let mut flow = base_arr;
    let mut total_in = base_arr;
    for src in 0..5_usize {
        let total = flow[src];
        total_in[src] = total; // save BEFORE reducing by remaining
        for dst in (src + 1)..5 {
            flow[dst] += total * table.src[src].conv_mults[dst];
        }
        flow[src] = total * table.src[src].remaining_mult;
    }

    // ── Phase 2: add gain-as-extra contributions ──────────────────────────
    // Gains are a copy added to the destination; they do NOT reduce the source.
    // They are based on total_in[src] (all damage flowing through src, including
    // amounts that were converted INTO src from upstream elements).
    let mut gains = [0.0_f64; 5];
    for src in 0..5_usize {
        for dst in (src + 1)..5 {
            gains[dst] += total_in[src] * table.src[src].gains[dst];
        }
    }

    // ── Phase 3: apply per-element inc/more scaling ───────────────────────
    let result: [f64; 5] = std::array::from_fn(|dst| {
        (flow[dst] + gains[dst]) * element_scale(db, ctx, dst)
    });

    DamageSet {
        physical: result[0],
        lightning: result[1],
        cold: result[2],
        fire: result[3],
        chaos: result[4],
    }
}
