// AUTO-GENERATED from SkillStatMap.norm.json — manual additions go in the section below.
// Re-run `bun run tool:gen-ssm` to refresh (manual additions are preserved).
//
// Covers all 707 SkillStatMap entries: tree passives, gem stats, item mod IDs.
// For stat IDs not in SkillStatMap, add entries in the manual section at the bottom.

use rustc_hash::FxHashMap;
use smallvec::{smallvec, SmallVec};
use std::sync::OnceLock;

use super::types::{KeywordFlag, ModFlag, ModTag, ModType, Modifier};
use crate::data::{SourceId, StatId};

// ── StatDef ─────────────────────────────────────────────────────────────────
/// One resolved modifier definition from the stat table.
/// Combined with a runtime `value` and `source` to produce a `Modifier`.
///
/// `tags` hold condition/multiplier context for Phase 5 (stored now, evaluated later).
#[derive(Debug, Clone)]
pub struct StatDef {
    pub stat: StatId,
    pub mod_type: ModType,
    pub flags: ModFlag,
    pub keywords: KeywordFlag,
    /// Divide the raw stat value by this before creating the Modifier (usually 1.0).
    pub div: f64,
    /// Condition/multiplier tags — checked in Phase 5, stored unconditionally now.
    pub tags: SmallVec<[ModTag; 2]>,
}

// ── Static table ─────────────────────────────────────────────────────────────
static STAT_TABLE: OnceLock<FxHashMap<&'static str, SmallVec<[StatDef; 1]>>> = OnceLock::new();

/// Return the global stat ID → StatDef table, building it on first call.
pub fn stat_table() -> &'static FxHashMap<&'static str, SmallVec<[StatDef; 1]>> {
    STAT_TABLE.get_or_init(build)
}

// ── Helpers ──────────────────────────────────────────────────────────────────
fn def(stat: StatId, mod_type: ModType) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        div: 1.0,
        tags: smallvec![],
    }
}
fn flagged_def(stat: StatId, mod_type: ModType, flags: ModFlag) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags,
        keywords: KeywordFlag::empty(),
        div: 1.0,
        tags: smallvec![],
    }
}
fn kw_def(stat: StatId, mod_type: ModType, keywords: KeywordFlag) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords,
        div: 1.0,
        tags: smallvec![],
    }
}
fn flagged_kw_def(
    stat: StatId,
    mod_type: ModType,
    flags: ModFlag,
    keywords: KeywordFlag,
) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags,
        keywords,
        div: 1.0,
        tags: smallvec![],
    }
}
fn div_def(stat: StatId, mod_type: ModType, div: f64) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        div,
        tags: smallvec![],
    }
}
fn kw_div_def(stat: StatId, mod_type: ModType, keywords: KeywordFlag, div: f64) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords,
        div,
        tags: smallvec![],
    }
}
fn tagged_def(stat: StatId, mod_type: ModType, tags: SmallVec<[ModTag; 2]>) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords: KeywordFlag::empty(),
        div: 1.0,
        tags,
    }
}
fn flagged_tagged_def(
    stat: StatId,
    mod_type: ModType,
    flags: ModFlag,
    tags: SmallVec<[ModTag; 2]>,
) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags,
        keywords: KeywordFlag::empty(),
        div: 1.0,
        tags,
    }
}
fn kw_tagged_def(
    stat: StatId,
    mod_type: ModType,
    keywords: KeywordFlag,
    tags: SmallVec<[ModTag; 2]>,
) -> StatDef {
    StatDef {
        stat,
        mod_type,
        flags: ModFlag::empty(),
        keywords,
        div: 1.0,
        tags,
    }
}

fn build() -> FxHashMap<&'static str, SmallVec<[StatDef; 1]>> {
    let mut m: FxHashMap<&'static str, SmallVec<[StatDef; 1]>> =
        FxHashMap::with_capacity_and_hasher(1392, Default::default());
    build_ssm(&mut m);
    build_gem_stats(&mut m);
    build_manual(&mut m);
    m
}

fn build_ssm(m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {
    // ── Generated entries (SkillStatMap) ─────────────────────────────────────
    m.insert(
        "accuracy_rating",
        smallvec![def(StatId::Accuracy, ModType::Base)],
    );
    m.insert(
        "accuracy_rating_+%",
        smallvec![def(StatId::Accuracy, ModType::Inc)],
    );
    m.insert(
        "accuracy_rating_+%_when_on_low_life",
        smallvec![tagged_def(
            StatId::Accuracy,
            ModType::Inc,
            smallvec![ModTag::Condition("LowLife")]
        )],
    );
    m.insert(
        "active_skill_added_damage_+%_final",
        smallvec![def(StatId::AddedDamage, ModType::More)],
    );
    m.insert(
        "active_skill_additive_minion_damage_modifiers_apply_to_all_damage_at_%_value",
        smallvec![
            def(StatId::MinionDamageAppliesToPlayer, ModType::Flag),
            def(StatId::ImprovedMinionDamageAppliesToPlayer, ModType::Max),
        ],
    );
    m.insert(
        "active_skill_additive_spell_damage_modifiers_apply_to_attack_damage_at_%_value",
        smallvec![
            def(StatId::SpellDamageAppliesToAttacks, ModType::Flag),
            def(StatId::ImprovedSpellDamageAppliesToAttacks, ModType::Max),
        ],
    );
    m.insert(
        "active_skill_ailment_damage_+%_final",
        smallvec![kw_def(
            StatId::Damage,
            ModType::More,
            KeywordFlag::from_bits_truncate(524288)
        )],
    );
    m.insert(
        "active_skill_area_damage_+%_final",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(512)
        )],
    );
    m.insert(
        "active_skill_area_of_effect_+%_final",
        smallvec![def(StatId::AreaOfEffect, ModType::More)],
    );
    m.insert(
        "active_skill_area_of_effect_+%_final_per_endurance_charge",
        smallvec![tagged_def(
            StatId::AreaOfEffect,
            ModType::More,
            smallvec![ModTag::Multiplier("EnduranceCharge")]
        )],
    );
    m.insert(
        "active_skill_area_of_effect_+%_final_when_cast_on_frostbolt",
        smallvec![tagged_def(
            StatId::AreaOfEffect,
            ModType::More,
            smallvec![ModTag::Condition("CastOnFrostbolt")]
        )],
    );
    m.insert(
        "active_skill_area_of_effect_radius_+%_final",
        smallvec![def(StatId::AreaOfEffect, ModType::More)],
    );
    m.insert(
        "active_skill_attack_damage_+%_final_per_endurance_charge",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::Multiplier("EnduranceCharge")]
        )],
    );
    m.insert(
        "active_skill_attack_speed_+%_final",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::More,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "active_skill_base_area_of_effect_radius",
        smallvec![def(StatId::SdRadius, ModType::Base)],
    );
    m.insert(
        "active_skill_base_radius_+",
        smallvec![def(StatId::SdRadiusExtra, ModType::Base)],
    );
    m.insert(
        "active_skill_base_secondary_area_of_effect_radius",
        smallvec![def(StatId::SdRadiusSecondary, ModType::Base)],
    );
    m.insert(
        "active_skill_base_tertiary_area_of_effect_radius",
        smallvec![def(StatId::SdRadiusTertiary, ModType::Base)],
    );
    m.insert(
        "active_skill_beam_splits_instead_of_chaining",
        smallvec![
            def(StatId::NoAdditionalChains, ModType::Flag),
            def(StatId::AdditionalChainsAddSplitsInstead, ModType::Flag),
        ],
    );
    m.insert(
        "active_skill_bleeding_damage_+%_final",
        smallvec![kw_def(
            StatId::Damage,
            ModType::More,
            KeywordFlag::from_bits_truncate(4194304)
        )],
    );
    m.insert(
        "active_skill_bleeding_damage_+%_final_in_blood_stance",
        smallvec![kw_tagged_def(
            StatId::Damage,
            ModType::More,
            KeywordFlag::from_bits_truncate(4194304),
            smallvec![ModTag::Condition("BloodStance")]
        )],
    );
    m.insert(
        "active_skill_brands_allowed_on_enemy_+",
        smallvec![def(StatId::BrandsAttachedLimit, ModType::Base)],
    );
    m.insert(
        "active_skill_cast_speed_+%_final",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::More,
            ModFlag::from_bits_truncate(16)
        )],
    );
    m.insert(
        "active_skill_chill_as_though_damage_+%_final",
        smallvec![def(StatId::ChillAsThoughDealing, ModType::More)],
    );
    m.insert(
        "active_skill_chill_effect_+%_final",
        smallvec![def(StatId::EnemyChillEffect, ModType::More)],
    );
    m.insert(
        "active_skill_damage_+%_final",
        smallvec![def(StatId::Damage, ModType::More)],
    );
    m.insert(
        "active_skill_damage_+%_final_vs_stunned_enemies",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::More,
            smallvec![ModTag::ActorCondition {
                var: "Stunned",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "active_skill_damage_+%_final_when_cast_on_frostbolt",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("CastOnFrostbolt")]
        )],
    );
    m.insert(
        "active_skill_freeze_duration_+%_final",
        smallvec![def(StatId::EnemyFreezeDuration, ModType::More)],
    );
    m.insert(
        "active_skill_ignite_damage_+%_final",
        smallvec![kw_def(
            StatId::Damage,
            ModType::More,
            KeywordFlag::from_bits_truncate(8388608)
        )],
    );
    m.insert(
        "active_skill_main_hand_weapon_damage_+%_final",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::More,
            smallvec![ModTag::Condition("MainHandAttack")]
        )],
    );
    m.insert(
        "active_skill_merged_damage_+%_final_while_dual_wielding",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::More,
            smallvec![ModTag::Condition("DualWielding")]
        )],
    );
    m.insert(
        "active_skill_minion_added_damage_+%_final",
        smallvec![def(StatId::SdMinionDamageEffectiveness, ModType::Base)],
    );
    m.insert(
        "active_skill_minion_attack_speed_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_bleeding_damage_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_damage_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_energy_shield_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_life_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_movement_velocity_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_minion_physical_damage_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "active_skill_physical_damage_+%_final",
        smallvec![def(StatId::PhysicalDamage, ModType::More)],
    );
    m.insert(
        "active_skill_poison_duration_+%_final",
        smallvec![def(StatId::EnemyPoisonDuration, ModType::More)],
    );
    m.insert(
        "active_skill_projectile_damage_+%_final",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(1024)
        )],
    );
    m.insert(
        "active_skill_projectile_speed_+%_final",
        smallvec![def(StatId::ProjectileSpeed, ModType::More)],
    );
    m.insert(
        "active_skill_quality_duration_+%_final",
        smallvec![def(StatId::Duration, ModType::More)],
    );
    m.insert(
        "active_skill_returning_projectile_damage_+%_final",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::More,
            smallvec![ModTag::Condition("ReturningProjectile")]
        )],
    );
    m.insert(
        "active_skill_shock_as_though_damage_+%_final",
        smallvec![def(StatId::ShockAsThoughDealing, ModType::More)],
    );
    m.insert(
        "active_skill_shock_effect_+%_final",
        smallvec![def(StatId::EnemyShockEffect, ModType::More)],
    );
    m.insert(
        "active_skill_trap_throwing_speed_+%_final",
        smallvec![def(StatId::TrapThrowingSpeed, ModType::More)],
    );
    m.insert(
        "added_damage_+%_final",
        smallvec![def(StatId::AddedDamage, ModType::More)],
    );
    m.insert(
        "additional_base_critical_strike_chance",
        smallvec![div_def(StatId::CritChance, ModType::Base, 100_f64)],
    );
    m.insert(
        "additional_beam_only_chains",
        smallvec![def(StatId::BeamChainCountMax, ModType::Base)],
    );
    m.insert(
        "additional_chance_to_freeze_chilled_enemies_%",
        smallvec![flagged_tagged_def(
            StatId::EnemyFreezeChance,
            ModType::Base,
            ModFlag::from_bits_truncate(4),
            smallvec![ModTag::ActorCondition {
                var: "Chilled",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "additional_chance_to_take_critical_strike_%",
        smallvec![def(StatId::SelfExtraCritChance, ModType::Base)],
    );
    m.insert(
        "additional_critical_strike_chance_per_10_shield_maximum_energy_shield_permyriad",
        smallvec![StatDef {
            stat: StatId::CritChance,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::EnergyShieldOnWeapon2,
                div: 10_f64
            }]
        }],
    );
    m.insert(
        "additional_critical_strike_chance_permyriad_while_affected_by_elusive",
        smallvec![StatDef {
            stat: StatId::CritChance,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![ModTag::Condition("Elusive")]
        }],
    );
    m.insert(
        "additional_weapon_base_attack_time_ms",
        smallvec![StatDef {
            stat: StatId::Speed,
            mod_type: ModType::Base,
            flags: ModFlag::from_bits_truncate(1),
            keywords: KeywordFlag::empty(),
            div: 1000_f64,
            tags: smallvec![]
        }],
    );
    m.insert(
        "additive_arrow_speed_modifiers_apply_to_area_of_effect",
        smallvec![def(
            StatId::SdArrowSpeedAppliesToAreaOfEffect,
            ModType::Base
        )],
    );
    m.insert(
        "additive_mine_duration_modifiers_apply_to_buff_effect_duration",
        smallvec![def(StatId::SdMineDurationAppliesToSkill, ModType::Base)],
    );
    m.insert(
        "ailment_damage_+%_per_frenzy_charge",
        smallvec![kw_tagged_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(524288),
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "all_damage_can_freeze",
        smallvec![
            def(StatId::PhysicalCanFreeze, ModType::Flag),
            def(StatId::LightningCanFreeze, ModType::Flag),
            def(StatId::FireCanFreeze, ModType::Flag),
            def(StatId::ChaosCanFreeze, ModType::Flag),
        ],
    );
    m.insert(
        "all_damage_can_ignite",
        smallvec![
            def(StatId::PhysicalCanIgnite, ModType::Flag),
            def(StatId::LightningCanIgnite, ModType::Flag),
            def(StatId::ColdCanIgnite, ModType::Flag),
            def(StatId::ChaosCanIgnite, ModType::Flag),
        ],
    );
    m.insert(
        "all_damage_can_ignite_freeze_shock",
        smallvec![
            def(StatId::PhysicalCanIgnite, ModType::Flag),
            def(StatId::LightningCanIgnite, ModType::Flag),
            def(StatId::ColdCanIgnite, ModType::Flag),
            def(StatId::ChaosCanIgnite, ModType::Flag),
            def(StatId::PhysicalCanFreeze, ModType::Flag),
            def(StatId::LightningCanFreeze, ModType::Flag),
            def(StatId::FireCanFreeze, ModType::Flag),
            def(StatId::ChaosCanFreeze, ModType::Flag),
            def(StatId::PhysicalCanShock, ModType::Flag),
            def(StatId::ColdCanShock, ModType::Flag),
            def(StatId::FireCanShock, ModType::Flag),
            def(StatId::ChaosCanShock, ModType::Flag),
        ],
    );
    m.insert(
        "all_damage_can_shock",
        smallvec![
            def(StatId::PhysicalCanShock, ModType::Flag),
            def(StatId::ColdCanShock, ModType::Flag),
            def(StatId::FireCanShock, ModType::Flag),
            def(StatId::ChaosCanShock, ModType::Flag),
        ],
    );
    m.insert(
        "always_freeze",
        smallvec![def(StatId::EnemyFreezeChance, ModType::Base)],
    );
    m.insert(
        "always_ignite",
        smallvec![def(StatId::EnemyIgniteChance, ModType::Base)],
    );
    m.insert(
        "always_pierce",
        smallvec![def(StatId::PierceAllTargets, ModType::Flag)],
    );
    m.insert(
        "always_shock",
        smallvec![def(StatId::EnemyShockChance, ModType::Base)],
    );
    m.insert(
        "area_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(512)
        )],
    );
    m.insert(
        "area_of_effect_+%_final_per_removable_power_frenzy_or_endurance_charge",
        smallvec![flagged_tagged_def(
            StatId::AreaOfEffect,
            ModType::More,
            ModFlag::from_bits_truncate(2),
            smallvec![ModTag::Multiplier("RemovableTotalCharges")]
        )],
    );
    m.insert(
        "area_of_effect_+%_per_50_strength",
        smallvec![def(StatId::SdAreaOfEffect, ModType::Base)],
    );
    m.insert(
        "area_of_effect_+%_while_not_dual_wielding",
        smallvec![tagged_def(
            StatId::AreaOfEffect,
            ModType::Inc,
            smallvec![ModTag::Condition("DualWielding")]
        )],
    );
    m.insert(
        "arrow_base_number_of_targets_to_pierce",
        smallvec![kw_def(
            StatId::PierceCount,
            ModType::Base,
            KeywordFlag::from_bits_truncate(2048)
        )],
    );
    m.insert(
        "attack_and_cast_speed_+%",
        smallvec![def(StatId::Speed, ModType::Inc)],
    );
    m.insert(
        "attack_critical_strike_chance_+%",
        smallvec![flagged_def(
            StatId::CritChance,
            ModType::Inc,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "attack_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "attack_damage_+%_per_450_evasion",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::PerStat {
                stat: StatId::Evasion,
                div: 450_f64
            }]
        )],
    );
    m.insert(
        "attack_damage_+%_per_450_physical_damage_reduction_rating",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::PerStat {
                stat: StatId::Armour,
                div: 450_f64
            }]
        )],
    );
    m.insert(
        "attack_maximum_added_chaos_damage",
        smallvec![kw_def(
            StatId::ChaosMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_maximum_added_cold_damage",
        smallvec![kw_def(
            StatId::ColdMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_maximum_added_fire_damage",
        smallvec![kw_def(
            StatId::FireMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_maximum_added_lightning_damage",
        smallvec![kw_def(
            StatId::LightningMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_maximum_added_physical_damage",
        smallvec![kw_def(
            StatId::PhysicalMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_maximum_added_physical_damage_with_weapons",
        smallvec![flagged_kw_def(
            StatId::PhysicalMax,
            ModType::Base,
            ModFlag::from_bits_truncate(8192),
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_chaos_damage",
        smallvec![kw_def(
            StatId::ChaosMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_cold_damage",
        smallvec![kw_def(
            StatId::ColdMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_fire_damage",
        smallvec![kw_def(
            StatId::FireMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_lightning_damage",
        smallvec![kw_def(
            StatId::LightningMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_physical_damage",
        smallvec![kw_def(
            StatId::PhysicalMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_minimum_added_physical_damage_with_weapons",
        smallvec![flagged_kw_def(
            StatId::PhysicalMin,
            ModType::Base,
            ModFlag::from_bits_truncate(8192),
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "attack_skill_mana_leech_from_any_damage_permyriad",
        smallvec![StatDef {
            stat: StatId::DamageManaLeech,
            mod_type: ModType::Base,
            flags: ModFlag::from_bits_truncate(1),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![]
        }],
    );
    m.insert(
        "attack_skills_additional_ballista_totems_allowed",
        smallvec![tagged_def(
            StatId::ActiveTotemLimit,
            ModType::Base,
            smallvec![ModTag::SkillType(125)]
        )],
    );
    m.insert(
        "attack_skills_have_added_lightning_damage_equal_to_%_of_maximum_mana",
        smallvec![
            flagged_tagged_def(
                StatId::LightningMin,
                ModType::Base,
                ModFlag::from_bits_truncate(1),
                smallvec![ModTag::PercentStat { stat: StatId::Mana }]
            ),
            flagged_tagged_def(
                StatId::LightningMax,
                ModType::Base,
                ModFlag::from_bits_truncate(1),
                smallvec![ModTag::PercentStat { stat: StatId::Mana }]
            ),
        ],
    );
    m.insert(
        "attack_speed_+%",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "attack_speed_+%_when_on_low_life",
        smallvec![flagged_tagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::Condition("LowLife")]
        )],
    );
    m.insert(
        "attack_speed_+%_with_atleast_20_rage",
        smallvec![flagged_tagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::MultiplierThreshold {
                var: "Rage",
                threshold: 20_f64
            }]
        )],
    );
    m.insert(
        "attacks_impale_on_hit_%_chance",
        smallvec![kw_def(
            StatId::ImpaleChance,
            ModType::Base,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "aura_effect_+%",
        smallvec![def(StatId::AuraEffect, ModType::Inc)],
    );
    m.insert(
        "avoid_interruption_while_using_this_skill_%",
        smallvec![def(StatId::AvoidInterruptStun, ModType::Base)],
    );
    m.insert(
        "banner_area_of_effect_+%_final_per_resource",
        smallvec![tagged_def(
            StatId::AreaOfEffect,
            ModType::More,
            smallvec![
                ModTag::Multiplier("BannerValour"),
                ModTag::Condition("BannerPlanted")
            ]
        )],
    );
    m.insert(
        "banner_buff_effect_+%_final_per_resource",
        smallvec![tagged_def(
            StatId::AuraEffect,
            ModType::More,
            smallvec![
                ModTag::Multiplier("BannerValour"),
                ModTag::Condition("BannerPlanted")
            ]
        )],
    );
    m.insert(
        "base_active_skill_totem_level",
        smallvec![def(StatId::SdTotemLevel, ModType::Base)],
    );
    m.insert(
        "base_added_cooldown_count",
        smallvec![def(StatId::AdditionalCooldownUses, ModType::Base)],
    );
    m.insert(
        "base_ailment_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(524288)
        )],
    );
    m.insert(
        "base_all_ailment_duration_+%",
        smallvec![def(StatId::EnemyAilmentDuration, ModType::Inc)],
    );
    m.insert(
        "base_arrow_speed_+%",
        smallvec![kw_def(
            StatId::ProjectileSpeed,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(2048)
        )],
    );
    m.insert(
        "base_attack_speed_+%_per_frenzy_charge",
        smallvec![flagged_tagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "base_aura_area_of_effect_+%",
        smallvec![kw_def(
            StatId::AreaOfEffect,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "base_avoid_stun_%",
        smallvec![def(StatId::AvoidStun, ModType::Base)],
    );
    m.insert(
        "base_bleed_duration_+%",
        smallvec![def(StatId::EnemyBleedDuration, ModType::Inc)],
    );
    m.insert(
        "base_bleed_on_hit_still_%_of_physical_damage_to_deal_per_minute",
        smallvec![div_def(StatId::SkillData, ModType::List, 60_f64)],
    );
    m.insert(
        "base_block_%_damage_taken",
        smallvec![def(StatId::BlockEffect, ModType::Base)],
    );
    m.insert(
        "base_cannot_be_damaged",
        smallvec![def(StatId::ConditionCannotBeDamaged, ModType::Flag)],
    );
    m.insert(
        "base_cannot_be_stunned",
        smallvec![def(StatId::StunImmune, ModType::Flag)],
    );
    m.insert(
        "base_cast_speed_+%",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(16)
        )],
    );
    m.insert(
        "base_chance_to_deal_triple_damage_%",
        smallvec![def(StatId::TripleDamageChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_dodge_%",
        smallvec![def(StatId::AttackDodgeChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_dodge_spells_%",
        smallvec![def(StatId::SpellDodgeChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_freeze_%",
        smallvec![def(StatId::EnemyFreezeChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_ignite_%",
        smallvec![def(StatId::EnemyIgniteChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_poison_on_hit_%",
        smallvec![def(StatId::PoisonChance, ModType::Base)],
    );
    m.insert(
        "base_chance_to_shock_%",
        smallvec![def(StatId::EnemyShockChance, ModType::Base)],
    );
    m.insert(
        "base_chaos_damage_resistance_%",
        smallvec![def(StatId::ChaosResist, ModType::Base)],
    );
    m.insert(
        "base_chaos_damage_to_deal_per_minute",
        smallvec![div_def(StatId::SkillData, ModType::List, 60_f64)],
    );
    m.insert(
        "base_cold_damage_resistance_%",
        smallvec![def(StatId::ColdResist, ModType::Base)],
    );
    m.insert(
        "base_cold_damage_to_deal_per_minute",
        smallvec![div_def(StatId::SkillData, ModType::List, 60_f64)],
    );
    m.insert(
        "base_cooldown_modifier_ms",
        smallvec![div_def(StatId::CooldownRecovery, ModType::Base, 1000_f64)],
    );
    m.insert(
        "base_cooldown_speed_+%",
        smallvec![def(StatId::CooldownRecovery, ModType::Inc)],
    );
    m.insert(
        "base_critical_strike_multiplier_+",
        smallvec![def(StatId::CritMultiplier, ModType::Base)],
    );
    m.insert(
        "base_curse_duration_+%",
        smallvec![kw_def(
            StatId::Duration,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(2)
        )],
    );
    m.insert(
        "base_deal_no_chaos_damage",
        smallvec![def(StatId::DealNoChaos, ModType::Flag)],
    );
    m.insert(
        "base_elemental_status_ailment_duration_+%",
        smallvec![def(StatId::EnemyElementalAilmentDuration, ModType::Inc)],
    );
    m.insert(
        "base_energy_shield_leech_from_spell_damage_permyriad",
        smallvec![StatDef {
            stat: StatId::DamageEnergyShieldLeech,
            mod_type: ModType::Base,
            flags: ModFlag::from_bits_truncate(2),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![]
        }],
    );
    m.insert(
        "base_evasion_rating",
        smallvec![def(StatId::Evasion, ModType::Base)],
    );
    m.insert(
        "base_fire_damage_resistance_%",
        smallvec![def(StatId::FireResist, ModType::Base)],
    );
    m.insert(
        "base_fire_damage_to_deal_per_minute",
        smallvec![div_def(StatId::SkillData, ModType::List, 60_f64)],
    );
    m.insert(
        "base_global_chance_to_knockback_%",
        smallvec![def(StatId::EnemyKnockbackChance, ModType::Base)],
    );
    m.insert(
        "base_graft_skill_tul_summon_maximum_allowed_demons",
        smallvec![def(StatId::ActiveHivebornLimit, ModType::Base)],
    );
    m.insert(
        "base_holy_strike_maximum_number_of_animated_weapons",
        smallvec![def(StatId::ActiveHolyStrikeMinionLimit, ModType::Base)],
    );
    m.insert(
        "base_inflict_cold_exposure_on_hit_%_chance",
        smallvec![def(StatId::ColdExposureChance, ModType::Base)],
    );
    m.insert(
        "base_inflict_fire_exposure_on_hit_%_chance",
        smallvec![def(StatId::FireExposureChance, ModType::Base)],
    );
    m.insert(
        "base_inflict_lightning_exposure_on_hit_%_chance",
        smallvec![def(StatId::LightningExposureChance, ModType::Base)],
    );
    m.insert(
        "base_killed_monster_dropped_item_quantity_+%",
        smallvec![def(StatId::LootQuantity, ModType::Inc)],
    );
    m.insert(
        "base_killed_monster_dropped_item_rarity_+%",
        smallvec![def(StatId::LootRarity, ModType::Inc)],
    );
    m.insert(
        "base_life_cost_+%",
        smallvec![def(StatId::LifeCost, ModType::Inc)],
    );
    m.insert(
        "base_life_gain_per_target",
        smallvec![flagged_def(
            StatId::LifeOnHit,
            ModType::Base,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "base_life_leech_from_attack_damage_permyriad",
        smallvec![StatDef {
            stat: StatId::DamageLifeLeech,
            mod_type: ModType::Base,
            flags: ModFlag::from_bits_truncate(1),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![]
        }],
    );
    m.insert(
        "base_life_leech_from_chaos_damage_permyriad",
        smallvec![div_def(
            StatId::ChaosDamageLifeLeech,
            ModType::Base,
            100_f64
        )],
    );
    m.insert(
        "base_life_regeneration_rate_per_minute",
        smallvec![div_def(StatId::LifeRegen, ModType::Base, 60_f64)],
    );
    m.insert(
        "base_life_reservation_+%",
        smallvec![def(StatId::LifeReserved, ModType::Inc)],
    );
    m.insert(
        "base_life_reservation_efficiency_+%",
        smallvec![def(StatId::LifeReservationEfficiency, ModType::Inc)],
    );
    m.insert(
        "base_lightning_damage_resistance_%",
        smallvec![def(StatId::LightningResist, ModType::Base)],
    );
    m.insert(
        "base_mana_cost_+",
        smallvec![def(StatId::ManaCostNoMult, ModType::Base)],
    );
    m.insert(
        "base_mana_cost_-%",
        smallvec![def(StatId::ManaCost, ModType::Inc)],
    );
    m.insert(
        "base_mana_leech_from_elemental_damage_permyriad",
        smallvec![div_def(
            StatId::ElementalDamageManaLeech,
            ModType::Base,
            100_f64
        )],
    );
    m.insert(
        "base_mana_regeneration_rate_per_minute",
        smallvec![div_def(StatId::ManaRegen, ModType::Base, 60_f64)],
    );
    m.insert(
        "base_mana_reservation_+%",
        smallvec![def(StatId::ManaReserved, ModType::Inc)],
    );
    m.insert(
        "base_mana_reservation_efficiency_+%",
        smallvec![def(StatId::ManaReservationEfficiency, ModType::Inc)],
    );
    m.insert(
        "base_max_number_of_absolution_sentinels",
        smallvec![def(StatId::ActiveSentinelOfAbsolutionLimit, ModType::Base)],
    );
    m.insert(
        "base_maximum_cold_damage_resistance_%",
        smallvec![def(StatId::ColdResistMax, ModType::Base)],
    );
    m.insert(
        "base_maximum_energy_shield",
        smallvec![def(StatId::EnergyShield, ModType::Base)],
    );
    m.insert(
        "base_maximum_fire_damage_resistance_%",
        smallvec![def(StatId::FireResistMax, ModType::Base)],
    );
    m.insert(
        "base_maximum_lightning_damage_resistance_%",
        smallvec![def(StatId::LightningResistMax, ModType::Base)],
    );
    m.insert(
        "base_melee_attack_repeat_count",
        smallvec![
            tagged_def(
                StatId::RepeatCount,
                ModType::Base,
                smallvec![ModTag::ModFlagOr(83886080)]
            ),
            tagged_def(
                StatId::RepeatCount,
                ModType::Base,
                smallvec![ModTag::SkillType(13)]
            ),
        ],
    );
    m.insert(
        "base_minion_duration_+%",
        smallvec![tagged_def(
            StatId::Duration,
            ModType::Inc,
            smallvec![ModTag::SkillType(88)]
        )],
    );
    m.insert(
        "base_movement_velocity_+%",
        smallvec![def(StatId::MovementSpeed, ModType::Inc)],
    );
    m.insert(
        "base_nonlethal_fire_damage_%_of_maximum_energy_shield_taken_per_minute",
        smallvec![StatDef {
            stat: StatId::FireDegen,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 6000_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::EnergyShield,
                div: 1_f64
            }]
        }],
    );
    m.insert(
        "base_nonlethal_fire_damage_%_of_maximum_life_taken_per_minute",
        smallvec![StatDef {
            stat: StatId::FireDegen,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 6000_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::Life,
                div: 1_f64
            }]
        }],
    );
    m.insert(
        "base_number_of_arbalists",
        smallvec![def(StatId::ActiveArbalistLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_arrows",
        smallvec![kw_def(
            StatId::ProjectileCount,
            ModType::Base,
            KeywordFlag::from_bits_truncate(2048)
        )],
    );
    m.insert(
        "base_number_of_champions_of_light_allowed",
        smallvec![def(StatId::ActiveSentinelOfPurityLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_golems_allowed",
        smallvec![def(StatId::ActiveGolemLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_living_lightning_allowed",
        smallvec![def(StatId::ActiveLivingLightningLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_projectiles",
        smallvec![def(StatId::ProjectileCount, ModType::Base)],
    );
    m.insert(
        "base_number_of_projectiles_in_spiral_nova",
        smallvec![def(StatId::ProjectileCount, ModType::Base)],
    );
    m.insert(
        "base_number_of_raging_spirits_allowed",
        smallvec![def(StatId::ActiveRagingSpiritLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_reapers_allowed",
        smallvec![def(StatId::ActiveReaperLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_relics_allowed",
        smallvec![def(StatId::ActiveHolyRelicLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_restless_dead_allowed",
        smallvec![def(StatId::ShamblingUndeadLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_sigils_allowed_per_target",
        smallvec![def(StatId::BrandsAttachedLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_skeletons_allowed",
        smallvec![def(StatId::ActiveSkeletonLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_spectres_allowed",
        smallvec![def(StatId::ActiveSpectreLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_totems_allowed",
        smallvec![def(StatId::ActiveTotemLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_void_spawns_allowed",
        smallvec![def(StatId::ActiveVoidSpawnLimit, ModType::Base)],
    );
    m.insert(
        "base_number_of_zombies_allowed",
        smallvec![def(StatId::ActiveZombieLimit, ModType::Base)],
    );
    m.insert(
        "base_penetrate_elemental_resistances_%",
        smallvec![def(StatId::ElementalPenetration, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_%_of_maximum_energy_shield_to_deal_per_minute",
        smallvec![StatDef {
            stat: StatId::PhysicalDegen,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 6000_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::EnergyShield,
                div: 1_f64
            }]
        }],
    );
    m.insert(
        "base_physical_damage_%_of_maximum_life_to_deal_per_minute",
        smallvec![StatDef {
            stat: StatId::PhysicalDegen,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 6000_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::Life,
                div: 1_f64
            }]
        }],
    );
    m.insert(
        "base_physical_damage_%_to_convert_to_chaos",
        smallvec![def(StatId::PhysicalDamageConvertToChaos, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_%_to_convert_to_cold",
        smallvec![def(StatId::PhysicalDamageConvertToCold, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_%_to_convert_to_fire",
        smallvec![def(StatId::PhysicalDamageConvertToFire, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_%_to_convert_to_lightning",
        smallvec![def(StatId::PhysicalDamageConvertToLightning, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_over_time_taken_+%",
        smallvec![def(StatId::PhysicalDamageTakenOverTime, ModType::Inc)],
    );
    m.insert(
        "base_physical_damage_reduction_rating",
        smallvec![def(StatId::Armour, ModType::Base)],
    );
    m.insert(
        "base_physical_damage_to_deal_per_minute",
        smallvec![div_def(StatId::SkillData, ModType::List, 60_f64)],
    );
    m.insert(
        "base_poison_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(2097152)
        )],
    );
    m.insert(
        "base_poison_duration_+%",
        smallvec![def(StatId::EnemyPoisonDuration, ModType::Inc)],
    );
    m.insert(
        "base_projectile_speed_+%",
        smallvec![def(StatId::ProjectileSpeed, ModType::Inc)],
    );
    m.insert(
        "base_reduce_enemy_cold_resistance_%",
        smallvec![def(StatId::ColdPenetration, ModType::Base)],
    );
    m.insert(
        "base_reduce_enemy_fire_resistance_%",
        smallvec![def(StatId::FirePenetration, ModType::Base)],
    );
    m.insert(
        "base_reduce_enemy_lightning_resistance_%",
        smallvec![def(StatId::LightningPenetration, ModType::Base)],
    );
    m.insert(
        "base_reservation_+%",
        smallvec![def(StatId::Reserved, ModType::Inc)],
    );
    m.insert(
        "base_reservation_efficiency_+%",
        smallvec![def(StatId::ReservationEfficiency, ModType::Inc)],
    );
    m.insert(
        "base_resist_all_elements_%",
        smallvec![def(StatId::ElementalResist, ModType::Base)],
    );
    m.insert(
        "base_secondary_skill_effect_duration",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "base_self_critical_strike_multiplier_-%",
        smallvec![def(StatId::SelfCritMultiplier, ModType::Inc)],
    );
    m.insert(
        "base_self_freeze_duration_-%",
        smallvec![def(StatId::SelfFreezeDuration, ModType::Inc)],
    );
    m.insert(
        "base_self_ignite_duration_-%",
        smallvec![def(StatId::SelfIgniteDuration, ModType::Inc)],
    );
    m.insert(
        "base_self_shock_duration_-%",
        smallvec![def(StatId::SelfShockDuration, ModType::Inc)],
    );
    m.insert(
        "base_sigil_repeat_frequency_ms",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "base_skill_area_of_effect_+%",
        smallvec![def(StatId::AreaOfEffect, ModType::Inc)],
    );
    m.insert(
        "base_skill_cost_life_instead_of_mana",
        smallvec![def(StatId::CostLifeInsteadOfMana, ModType::Flag)],
    );
    m.insert(
        "base_skill_cost_life_instead_of_mana_%",
        smallvec![def(StatId::HybridManaAndLifeCost_Life, ModType::Base)],
    );
    m.insert(
        "base_skill_effect_duration",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "base_skill_reserve_life_instead_of_mana",
        smallvec![def(StatId::BloodMagicReserved, ModType::Flag)],
    );
    m.insert(
        "base_skill_show_average_damage_instead_of_dps",
        smallvec![def(StatId::SdShowAverage, ModType::Base)],
    );
    m.insert(
        "base_spell_block_%",
        smallvec![def(StatId::SpellBlockChance, ModType::Base)],
    );
    m.insert(
        "base_spell_cast_time_ms_override",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "base_spell_cooldown_speed_+%",
        smallvec![def(StatId::CooldownRecovery, ModType::Inc)],
    );
    m.insert(
        "base_spell_repeat_count",
        smallvec![flagged_def(
            StatId::RepeatCount,
            ModType::Base,
            ModFlag::from_bits_truncate(16)
        )],
    );
    m.insert(
        "base_stun_duration_+%",
        smallvec![def(StatId::EnemyStunDuration, ModType::Inc)],
    );
    m.insert(
        "base_stun_recovery_+%",
        smallvec![def(StatId::StunRecovery, ModType::Inc)],
    );
    m.insert(
        "base_stun_threshold_reduction_+%",
        smallvec![def(StatId::EnemyStunThreshold, ModType::Inc)],
    );
    m.insert(
        "base_tertiary_skill_effect_duration",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "base_totem_duration",
        smallvec![div_def(StatId::TotemDuration, ModType::Base, 1000_f64)],
    );
    m.insert(
        "bleed_duration_is_skill_duration",
        smallvec![def(StatId::SdBleedDurationIsSkillDuration, ModType::Base)],
    );
    m.insert(
        "bleed_on_hit_with_attacks_%",
        smallvec![flagged_def(
            StatId::BleedChance,
            ModType::Base,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "bleed_on_melee_attack_chance_%",
        smallvec![flagged_def(
            StatId::BleedChance,
            ModType::Base,
            ModFlag::from_bits_truncate(256)
        )],
    );
    m.insert(
        "bleeding_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(4194304)
        )],
    );
    m.insert(
        "bleeding_stacks_up_to_x_times",
        smallvec![def(StatId::BleedStacksMax, ModType::Override)],
    );
    m.insert(
        "blind_effect_+%",
        smallvec![tagged_def(
            StatId::BlindEffect,
            ModType::Inc,
            smallvec![ModTag::GlobalEffect {
                effect_name: "Vaal Blade Flurry",
                effect_type: Some("Debuff")
            }]
        )],
    );
    m.insert(
        "block_while_dual_wielding_%",
        smallvec![tagged_def(
            StatId::BlockChance,
            ModType::Base,
            smallvec![ModTag::Condition("DualWielding")]
        )],
    );
    m.insert(
        "brand_atttached_duration_is_infinite",
        smallvec![def(StatId::UnlimitedBrandDuration, ModType::Flag)],
    );
    m.insert(
        "brand_cannot_be_recalled",
        smallvec![def(StatId::ConditionCannotRecallBrand, ModType::Flag)],
    );
    m.insert(
        "buff_effect_duration_+%_per_removable_endurance_charge",
        smallvec![tagged_def(
            StatId::Duration,
            ModType::Inc,
            smallvec![ModTag::Multiplier("RemovableEnduranceCharge")]
        )],
    );
    m.insert(
        "buff_effect_duration_+%_per_removable_endurance_charge_limited_to_5",
        smallvec![tagged_def(
            StatId::Duration,
            ModType::Inc,
            smallvec![ModTag::Multiplier("RemovableEnduranceCharge")]
        )],
    );
    m.insert(
        "buff_time_passed_-%",
        smallvec![def(StatId::BuffExpireFaster, ModType::More)],
    );
    m.insert(
        "burn_damage_+%",
        smallvec![kw_def(
            StatId::FireDamage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(134217728)
        )],
    );
    m.insert(
        "cannot_be_knocked_back",
        smallvec![def(StatId::KnockbackImmune, ModType::Flag)],
    );
    m.insert(
        "cannot_be_stunned_while_leeching",
        smallvec![def(StatId::AvoidStun, ModType::Base)],
    );
    m.insert(
        "cannot_cause_bleeding",
        smallvec![def(StatId::CannotBleed, ModType::Flag)],
    );
    m.insert(
        "cannot_inflict_additional_poisons",
        smallvec![def(StatId::CannotMultiplePoison, ModType::Flag)],
    );
    m.insert(
        "cannot_inflict_status_ailments",
        smallvec![
            def(StatId::CannotShock, ModType::Flag),
            def(StatId::CannotChill, ModType::Flag),
            def(StatId::CannotFreeze, ModType::Flag),
            def(StatId::CannotIgnite, ModType::Flag),
            def(StatId::CannotScorch, ModType::Flag),
            def(StatId::CannotBrittle, ModType::Flag),
            def(StatId::CannotSap, ModType::Flag),
        ],
    );
    m.insert(
        "cannot_pierce",
        smallvec![def(StatId::CannotPierce, ModType::Flag)],
    );
    m.insert(
        "cannot_poison_poisoned_enemies",
        smallvec![
            def(StatId::ConditionNonPoisonedOnly, ModType::Flag),
            def(StatId::PoisonStackLimit, ModType::Min),
        ],
    );
    m.insert(
        "cast_linked_spells_on_attack_crit_%",
        smallvec![def(StatId::SdChanceToTriggerOnCrit, ModType::Base)],
    );
    m.insert(
        "cast_linked_spells_on_melee_kill_%",
        smallvec![def(StatId::SdChanceToTriggerOnMeleeKill, ModType::Base)],
    );
    m.insert(
        "cast_on_damage_taken_threshold",
        smallvec![def(StatId::SdTriggeredByDamageTaken, ModType::Base)],
    );
    m.insert(
        "cast_on_stunned_%",
        smallvec![def(StatId::SdChanceToTriggerOnStun, ModType::Base)],
    );
    m.insert(
        "cast_speed_+%_granted_from_skill",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(16)
        )],
    );
    m.insert(
        "cast_spell_on_linked_attack_crit",
        smallvec![
            def(StatId::SdTriggeredByCoc, ModType::Base),
            def(StatId::SdTriggerOnCrit, ModType::Base),
        ],
    );
    m.insert(
        "cast_spell_on_linked_melee_kill",
        smallvec![def(StatId::SdTriggeredByMeleeKill, ModType::Base)],
    );
    m.insert(
        "cast_spell_while_linked_skill_channelling",
        smallvec![def(StatId::SdTriggeredWhileChannelling, ModType::Base)],
    );
    m.insert(
        "cast_time_overrides_attack_duration",
        smallvec![def(StatId::SdCastTimeOverridesAttackTime, ModType::Base)],
    );
    m.insert(
        "cast_when_cast_curse_%",
        smallvec![def(StatId::SdChanceToTriggerCurseOnCurse, ModType::Base)],
    );
    m.insert(
        "chance_for_extra_damage_roll_%",
        smallvec![def(StatId::LuckyHitsChance, ModType::Base)],
    );
    m.insert(
        "chance_to_be_frozen_%",
        smallvec![def(StatId::SelfFreezeChance, ModType::Base)],
    );
    m.insert(
        "chance_to_be_ignited_%",
        smallvec![def(StatId::SelfIgniteChance, ModType::Base)],
    );
    m.insert(
        "chance_to_be_knocked_back_%",
        smallvec![def(StatId::SelfKnockbackChance, ModType::Base)],
    );
    m.insert(
        "chance_to_be_pierced_%",
        smallvec![def(StatId::SelfPierceChance, ModType::Base)],
    );
    m.insert(
        "chance_to_be_shocked_%",
        smallvec![def(StatId::SelfShockChance, ModType::Base)],
    );
    m.insert(
        "chance_to_bleed_on_hit_%_chance_in_blood_stance",
        smallvec![flagged_tagged_def(
            StatId::BleedChance,
            ModType::Base,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::Condition("BloodStance")]
        )],
    );
    m.insert(
        "chance_to_bleed_on_hit_%_vs_maimed",
        smallvec![tagged_def(
            StatId::BleedChance,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Maimed",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "chance_to_cast_on_kill_%",
        smallvec![def(StatId::SdTriggeredBySquirmingTerror, ModType::Base)],
    );
    m.insert(
        "chance_to_deal_double_damage_%",
        smallvec![def(StatId::DoubleDamageChance, ModType::Base)],
    );
    m.insert(
        "chance_to_deal_double_damage_%_per_10_intelligence",
        smallvec![tagged_def(
            StatId::DoubleDamageChance,
            ModType::Base,
            smallvec![ModTag::PerStat {
                stat: StatId::Intelligence,
                div: 10_f64
            }]
        )],
    );
    m.insert(
        "chance_to_deal_double_damage_%_vs_bleeding_enemies",
        smallvec![tagged_def(
            StatId::DoubleDamageChance,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Bleeding",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "chance_to_double_stun_duration_%",
        smallvec![def(StatId::DoubleEnemyStunDurationChance, ModType::Base)],
    );
    m.insert(
        "chance_to_freeze_shock_ignite_%",
        smallvec![
            def(StatId::EnemyFreezeChance, ModType::Base),
            def(StatId::EnemyShockChance, ModType::Base),
            def(StatId::EnemyIgniteChance, ModType::Base),
        ],
    );
    m.insert(
        "chance_to_scorch_%",
        smallvec![def(StatId::EnemyScorchChance, ModType::Base)],
    );
    m.insert(
        "channelled_skill_damage_+%",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::SkillType(57)]
        )],
    );
    m.insert(
        "chaos_damage_+%",
        smallvec![def(StatId::ChaosDamage, ModType::Inc)],
    );
    m.insert(
        "chaos_damage_taken_+%",
        smallvec![def(StatId::ChaosDamageTaken, ModType::Inc)],
    );
    m.insert(
        "chaos_dot_multiplier_+",
        smallvec![def(StatId::ChaosDotMultiplier, ModType::Base)],
    );
    m.insert(
        "chill_and_freeze_duration_+%",
        smallvec![
            def(StatId::EnemyChillDuration, ModType::Inc),
            def(StatId::EnemyFreezeDuration, ModType::Inc),
        ],
    );
    m.insert(
        "chill_duration_+%",
        smallvec![def(StatId::EnemyChillDuration, ModType::Inc)],
    );
    m.insert(
        "chill_effect_+%",
        smallvec![def(StatId::EnemyChillEffect, ModType::Inc)],
    );
    m.insert(
        "cold_ailment_duration_+%",
        smallvec![
            def(StatId::EnemyChillDuration, ModType::Inc),
            def(StatId::EnemyFreezeDuration, ModType::Inc),
            def(StatId::EnemyBrittleDuration, ModType::Inc),
        ],
    );
    m.insert(
        "cold_ailment_effect_+%",
        smallvec![
            def(StatId::EnemyChillEffect, ModType::Inc),
            def(StatId::EnemyFreezeEffect, ModType::Inc),
            def(StatId::EnemyBrittleEffect, ModType::Inc),
        ],
    );
    m.insert(
        "cold_damage_%_to_add_as_fire",
        smallvec![def(StatId::ColdDamageGainAsFire, ModType::Base)],
    );
    m.insert(
        "cold_damage_+%",
        smallvec![def(StatId::ColdDamage, ModType::Inc)],
    );
    m.insert(
        "cold_damage_taken_+%",
        smallvec![def(StatId::ColdDamageTaken, ModType::Inc)],
    );
    m.insert(
        "cold_dot_multiplier_+",
        smallvec![def(StatId::ColdDotMultiplier, ModType::Base)],
    );
    m.insert(
        "consecrated_ground_effect_+%",
        smallvec![def(StatId::ConsecratedGroundEffect, ModType::Inc)],
    );
    m.insert(
        "consecrated_ground_enemy_damage_taken_+%",
        smallvec![tagged_def(
            StatId::DamageTakenConsecratedGround,
            ModType::Inc,
            smallvec![ModTag::Condition("OnConsecratedGround")]
        )],
    );
    m.insert(
        "cooldown_recovery_rate_+%_per_100_ward",
        smallvec![tagged_def(
            StatId::CooldownRecovery,
            ModType::Inc,
            smallvec![ModTag::PerStat {
                stat: StatId::Ward,
                div: 100_f64
            }]
        )],
    );
    m.insert(
        "corpse_explosion_monster_life_%",
        smallvec![div_def(StatId::SkillData, ModType::List, 100_f64)],
    );
    m.insert(
        "corpse_explosion_monster_life_permillage_fire",
        smallvec![div_def(StatId::SkillData, ModType::List, 1000_f64)],
    );
    m.insert(
        "critical_ailment_dot_multiplier_+",
        smallvec![tagged_def(
            StatId::DotMultiplier,
            ModType::Base,
            smallvec![ModTag::Condition("CriticalStrike")]
        )],
    );
    m.insert(
        "critical_multiplier_+%_per_100_max_es_on_shield",
        smallvec![tagged_def(
            StatId::CritMultiplier,
            ModType::Base,
            smallvec![ModTag::PerStat {
                stat: StatId::EnergyShieldOnWeapon2,
                div: 100_f64
            }]
        )],
    );
    m.insert(
        "critical_poison_dot_multiplier_+",
        smallvec![kw_tagged_def(
            StatId::DotMultiplier,
            ModType::Base,
            KeywordFlag::from_bits_truncate(2097152),
            smallvec![ModTag::Condition("CriticalStrike")]
        )],
    );
    m.insert(
        "critical_strike_chance_+%",
        smallvec![def(StatId::CritChance, ModType::Inc)],
    );
    m.insert(
        "critical_strike_chance_+%_per_power_charge",
        smallvec![tagged_def(
            StatId::CritChance,
            ModType::Inc,
            smallvec![ModTag::Multiplier("PowerCharge")]
        )],
    );
    m.insert(
        "critical_strike_chance_+%_vs_bleeding_enemies",
        smallvec![tagged_def(
            StatId::CritChance,
            ModType::Inc,
            smallvec![ModTag::ActorCondition {
                var: "Bleeding",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "critical_strike_chance_+%_vs_blinded_enemies",
        smallvec![tagged_def(
            StatId::CritChance,
            ModType::Inc,
            smallvec![ModTag::ActorCondition {
                var: "Blinded",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "critical_strike_chance_+%_vs_shocked_enemies",
        smallvec![tagged_def(
            StatId::CritChance,
            ModType::Inc,
            smallvec![ModTag::ActorCondition {
                var: "Shocked",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "critical_strike_chance_against_enemies_on_full_life_+%",
        smallvec![tagged_def(
            StatId::CritChance,
            ModType::Inc,
            smallvec![ModTag::ActorCondition {
                var: "FullLife",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "critical_strike_multiplier_+_if_dexterity_higher_than_intelligence",
        smallvec![def(StatId::SdCritMultiplier, ModType::Base)],
    );
    m.insert(
        "critical_strike_multiplier_+_per_power_charge",
        smallvec![tagged_def(
            StatId::CritMultiplier,
            ModType::Base,
            smallvec![ModTag::Multiplier("PowerCharge")]
        )],
    );
    m.insert(
        "curse_area_of_effect_+%",
        smallvec![kw_def(
            StatId::AreaOfEffect,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(2)
        )],
    );
    m.insert(
        "curse_cast_speed_+%",
        smallvec![flagged_def(
            StatId::Speed,
            ModType::Inc,
            ModFlag::from_bits_truncate(16)
        )],
    );
    m.insert(
        "curse_effect_+%",
        smallvec![def(StatId::CurseEffect, ModType::Inc)],
    );
    m.insert(
        "curse_effect_+%_vs_players",
        smallvec![def(StatId::CurseEffectAgainstPlayer, ModType::Inc)],
    );
    m.insert(
        "curse_maximum_doom",
        smallvec![def(StatId::MaxDoom, ModType::Base)],
    );
    m.insert(
        "curse_skill_effect_duration_+%",
        smallvec![kw_def(
            StatId::Duration,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(2)
        )],
    );
    m.insert("damage_+%", smallvec![def(StatId::Damage, ModType::Inc)]);
    m.insert(
        "damage_+%_if_you_have_consumed_a_corpse_recently",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("ConsumedCorpseRecently")]
        )],
    );
    m.insert(
        "damage_+%_on_full_energy_shield",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("FullEnergyShield")]
        )],
    );
    m.insert(
        "damage_+%_per_chain",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::PerStat {
                stat: StatId::Chain,
                div: 1_f64
            }]
        )],
    );
    m.insert(
        "damage_+%_per_endurance_charge",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Multiplier("EnduranceCharge")]
        )],
    );
    m.insert(
        "damage_+%_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "damage_+%_per_power_charge",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Multiplier("PowerCharge")]
        )],
    );
    m.insert(
        "damage_+%_vs_enemies_on_full_life",
        smallvec![kw_tagged_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(786432),
            smallvec![ModTag::ActorCondition {
                var: "FullLife",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "damage_+%_vs_frozen_enemies",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(4),
            smallvec![ModTag::ActorCondition {
                var: "Frozen",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "damage_+%_when_on_full_life",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("FullLife")]
        )],
    );
    m.insert(
        "damage_+%_when_on_low_life",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("LowLife")]
        )],
    );
    m.insert(
        "damage_+%_while_es_leeching",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("LeechingEnergyShield")]
        )],
    );
    m.insert(
        "damage_+%_while_life_leeching",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("LeechingLife")]
        )],
    );
    m.insert(
        "damage_+%_while_mana_leeching",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![ModTag::Condition("LeechingMana")]
        )],
    );
    m.insert(
        "damage_+%_with_hits_and_ailments",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(786432)
        )],
    );
    m.insert(
        "damage_over_time_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(8)
        )],
    );
    m.insert(
        "damage_vs_enemies_on_low_life_+%",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(4),
            smallvec![ModTag::ActorCondition {
                var: "LowLife",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "damaging_ailments_deal_damage_+%_faster",
        smallvec![
            def(StatId::BleedFaster, ModType::Inc),
            def(StatId::PoisonFaster, ModType::Inc),
            def(StatId::IgniteBurnFaster, ModType::Inc),
        ],
    );
    m.insert(
        "deal_chaos_damage_per_second_for_10_seconds_on_hit",
        smallvec![def(StatId::SdDecay, ModType::Base)],
    );
    m.insert(
        "deal_no_elemental_damage",
        smallvec![
            def(StatId::DealNoFire, ModType::Flag),
            def(StatId::DealNoCold, ModType::Flag),
            def(StatId::DealNoLightning, ModType::Flag),
        ],
    );
    m.insert(
        "degen_effect_+%",
        smallvec![def(StatId::DamageTakenOverTime, ModType::Inc)],
    );
    m.insert(
        "discharge_damage_+%_if_3_charge_types_removed",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::Inc,
            smallvec![
                ModTag::Multiplier("RemovableEnduranceCharge"),
                ModTag::Multiplier("RemovableFrenzyCharge"),
                ModTag::Multiplier("RemovablePowerCharge")
            ]
        )],
    );
    m.insert(
        "display_mirage_warriors_no_spirit_strikes",
        smallvec![def(StatId::SdTriggeredBySaviour, ModType::Base)],
    );
    m.insert(
        "display_skill_minions_level_is_corpse_level",
        smallvec![def(StatId::SdMinionLevelIsEnemyLevel, ModType::Base)],
    );
    m.insert(
        "display_this_skill_cooldown_does_not_recover_during_buff",
        smallvec![def(StatId::NoCooldownRecoveryInDuration, ModType::Flag)],
    );
    m.insert(
        "dot_multiplier_+",
        smallvec![def(StatId::DotMultiplier, ModType::Base)],
    );
    m.insert(
        "dual_wield_inherent_attack_speed_+%_final",
        smallvec![flagged_tagged_def(
            StatId::Speed,
            ModType::More,
            ModFlag::from_bits_truncate(1),
            smallvec![ModTag::Condition("DualWielding")]
        )],
    );
    m.insert(
        "elemental_damage_+%",
        smallvec![def(StatId::ElementalDamage, ModType::Inc)],
    );
    m.insert(
        "elemental_damage_with_attack_skills_+%",
        smallvec![kw_def(
            StatId::ElementalDamage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "elusive_effect_+%",
        smallvec![def(StatId::ElusiveEffect, ModType::Max)],
    );
    m.insert(
        "enemies_you_shock_take_%_increased_physical_damage",
        smallvec![tagged_def(
            StatId::PhysicalDamageTaken,
            ModType::Inc,
            smallvec![ModTag::Condition("Shocked")]
        )],
    );
    m.insert(
        "enemy_phys_reduction_%_penalty_vs_hit",
        smallvec![def(StatId::EnemyPhysicalDamageReduction, ModType::Base)],
    );
    m.insert(
        "energy_shield_leech_from_any_damage_permyriad",
        smallvec![div_def(
            StatId::DamageEnergyShieldLeech,
            ModType::Base,
            100_f64
        )],
    );
    m.insert(
        "energy_shield_recharge_rate_+%",
        smallvec![def(StatId::EnergyShieldRecharge, ModType::Inc)],
    );
    m.insert(
        "faster_bleed_%",
        smallvec![def(StatId::BleedFaster, ModType::Inc)],
    );
    m.insert(
        "faster_burn_%",
        smallvec![def(StatId::IgniteBurnFaster, ModType::Inc)],
    );
    m.insert(
        "faster_poison_%",
        smallvec![def(StatId::PoisonFaster, ModType::Inc)],
    );
    m.insert(
        "fire_damage_%_to_add_as_chaos",
        smallvec![def(StatId::FireDamageGainAsChaos, ModType::Base)],
    );
    m.insert(
        "fire_damage_+%",
        smallvec![def(StatId::FireDamage, ModType::Inc)],
    );
    m.insert(
        "fire_damage_taken_+%",
        smallvec![def(StatId::FireDamageTaken, ModType::Inc)],
    );
    m.insert(
        "fire_dot_multiplier_+",
        smallvec![def(StatId::FireDotMultiplier, ModType::Base)],
    );
    m.insert(
        "flask_effect_+%",
        smallvec![def(StatId::FlaskEffect, ModType::Inc)],
    );
    m.insert(
        "flask_mana_to_recover_+%",
        smallvec![def(StatId::FlaskManaRecovery, ModType::Inc)],
    );
    m.insert(
        "fortify_duration_+%",
        smallvec![def(StatId::FortifyDuration, ModType::Inc)],
    );
    m.insert(
        "freeze_as_though_dealt_damage_+%",
        smallvec![def(StatId::FreezeAsThoughDealing, ModType::More)],
    );
    m.insert(
        "freeze_duration_+%",
        smallvec![def(StatId::EnemyFreezeDuration, ModType::Inc)],
    );
    m.insert(
        "gain_fortify_on_melee_hit_ms",
        smallvec![div_def(
            StatId::FortifyDuration,
            ModType::Override,
            1000_f64
        )],
    );
    m.insert(
        "gain_x_rage_on_attack_hit",
        smallvec![tagged_def(
            StatId::ConditionCanGainRage,
            ModType::Flag,
            smallvec![ModTag::GlobalEffect {
                effect_name: "Rage",
                effect_type: Some("Buff")
            }]
        )],
    );
    m.insert(
        "global_always_hit",
        smallvec![def(StatId::SdCannotBeEvaded, ModType::Base)],
    );
    m.insert(
        "global_bleed_on_hit",
        smallvec![def(StatId::BleedChance, ModType::Base)],
    );
    m.insert(
        "global_knockback",
        smallvec![def(StatId::EnemyKnockbackChance, ModType::Base)],
    );
    m.insert(
        "global_maximum_added_chaos_damage",
        smallvec![def(StatId::ChaosMax, ModType::Base)],
    );
    m.insert(
        "global_maximum_added_cold_damage",
        smallvec![def(StatId::ColdMax, ModType::Base)],
    );
    m.insert(
        "global_maximum_added_fire_damage_vs_burning_enemies",
        smallvec![tagged_def(
            StatId::FireMax,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Burning",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "global_maximum_added_lightning_damage",
        smallvec![def(StatId::LightningMax, ModType::Base)],
    );
    m.insert(
        "global_maximum_added_physical_damage_vs_bleeding_enemies",
        smallvec![tagged_def(
            StatId::PhysicalMax,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Bleeding",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "global_minimum_added_chaos_damage",
        smallvec![def(StatId::ChaosMin, ModType::Base)],
    );
    m.insert(
        "global_minimum_added_cold_damage",
        smallvec![def(StatId::ColdMin, ModType::Base)],
    );
    m.insert(
        "global_minimum_added_fire_damage_vs_burning_enemies",
        smallvec![tagged_def(
            StatId::FireMin,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Burning",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "global_minimum_added_lightning_damage",
        smallvec![def(StatId::LightningMin, ModType::Base)],
    );
    m.insert(
        "global_minimum_added_physical_damage_vs_bleeding_enemies",
        smallvec![tagged_def(
            StatId::PhysicalMin,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Bleeding",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "global_poison_on_hit",
        smallvec![def(StatId::PoisonChance, ModType::Base)],
    );
    m.insert(
        "golem_buff_effect_+%",
        smallvec![def(StatId::BuffEffect, ModType::Inc)],
    );
    m.insert(
        "golem_cooldown_recovery_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "hex_applied_when_trap_triggers",
        smallvec![def(StatId::SdTriggeredByTrapTrigger, ModType::Base)],
    );
    m.insert(
        "hit_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(4)
        )],
    );
    m.insert(
        "holy_relic_trigger_on_parent_attack_%",
        smallvec![def(StatId::SdChanceToTriggerOnParentAttack, ModType::Base)],
    );
    m.insert(
        "ignite_duration_+%",
        smallvec![def(StatId::EnemyIgniteDuration, ModType::Inc)],
    );
    m.insert(
        "impale_debuff_effect_+%",
        smallvec![def(StatId::ImpaleEffect, ModType::Inc)],
    );
    m.insert(
        "impale_on_hit_%_chance",
        smallvec![def(StatId::ImpaleChance, ModType::Base)],
    );
    m.insert(
        "impale_phys_reduction_%_penalty",
        smallvec![def(
            StatId::EnemyImpalePhysicalDamageReduction,
            ModType::Base
        )],
    );
    m.insert(
        "keystone_strong_bowman",
        smallvec![def(StatId::IronGrip, ModType::Flag)],
    );
    m.insert(
        "kill_enemy_on_hit_if_under_10%_life",
        smallvec![def(StatId::CullPercent, ModType::Max)],
    );
    m.insert(
        "kill_normal_or_magic_enemy_on_hit_if_under_x%_life",
        smallvec![tagged_def(
            StatId::CullPercent,
            ModType::Max,
            smallvec![ModTag::ActorCondition {
                var: "RareOrUnique",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "knockback_distance_+%",
        smallvec![def(StatId::EnemyKnockbackDistance, ModType::Inc)],
    );
    m.insert(
        "life_granted_when_hit_by_attacks",
        smallvec![flagged_def(
            StatId::SelfLifeOnHit,
            ModType::Base,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "life_granted_when_killed",
        smallvec![def(StatId::SelfLifeOnKill, ModType::Base)],
    );
    m.insert(
        "life_leech_does_not_stop_at_full_life",
        smallvec![def(StatId::CanLeechLifeOnFullLife, ModType::Flag)],
    );
    m.insert(
        "life_leech_from_any_damage_permyriad",
        smallvec![div_def(StatId::DamageLifeLeech, ModType::Base, 100_f64)],
    );
    m.insert(
        "life_leech_from_physical_attack_damage_permyriad",
        smallvec![StatDef {
            stat: StatId::PhysicalDamageLifeLeech,
            mod_type: ModType::Base,
            flags: ModFlag::from_bits_truncate(1),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![]
        }],
    );
    m.insert(
        "life_leech_on_any_damage_when_hit_permyriad",
        smallvec![def(StatId::SelfDamageLifeLeech, ModType::Base)],
    );
    m.insert(
        "life_regeneration_rate_per_minute_%",
        smallvec![div_def(StatId::LifeRegenPercent, ModType::Base, 60_f64)],
    );
    m.insert(
        "lightning_ailment_duration_+%",
        smallvec![
            def(StatId::EnemyShockDuration, ModType::Inc),
            def(StatId::EnemySapDuration, ModType::Inc),
        ],
    );
    m.insert(
        "lightning_ailment_effect_+%",
        smallvec![
            def(StatId::EnemyShockEffect, ModType::Inc),
            def(StatId::EnemySapEffect, ModType::Inc),
        ],
    );
    m.insert(
        "lightning_damage_%_to_add_as_chaos",
        smallvec![def(StatId::LightningDamageGainAsChaos, ModType::Base)],
    );
    m.insert(
        "lightning_damage_+%",
        smallvec![def(StatId::LightningDamage, ModType::Inc)],
    );
    m.insert(
        "lightning_damage_cannot_shock",
        smallvec![def(StatId::LightningCannotShock, ModType::Flag)],
    );
    m.insert(
        "lightning_damage_taken_+%",
        smallvec![def(StatId::LightningDamageTaken, ModType::Inc)],
    );
    m.insert(
        "link_buff_effect_on_self_+%",
        smallvec![def(StatId::LinkEffectOnSelf, ModType::Inc)],
    );
    m.insert(
        "main_hand_local_maximum_added_physical_damage",
        smallvec![flagged_kw_def(
            StatId::PhysicalMax,
            ModType::Base,
            ModFlag::from_bits_truncate(8192),
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "main_hand_local_minimum_added_physical_damage",
        smallvec![flagged_kw_def(
            StatId::PhysicalMin,
            ModType::Base,
            ModFlag::from_bits_truncate(8192),
            KeywordFlag::from_bits_truncate(65536)
        )],
    );
    m.insert(
        "mana_gain_per_target",
        smallvec![def(StatId::ManaOnHit, ModType::Base)],
    );
    m.insert(
        "mana_granted_when_hit_by_attacks",
        smallvec![flagged_def(
            StatId::SelfManaOnHit,
            ModType::Base,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "mana_granted_when_killed",
        smallvec![def(StatId::SelfManaOnKill, ModType::Base)],
    );
    m.insert(
        "mana_leech_from_any_damage_permyriad",
        smallvec![div_def(StatId::DamageManaLeech, ModType::Base, 100_f64)],
    );
    m.insert(
        "mana_leech_on_any_damage_when_hit_permyriad",
        smallvec![def(StatId::SelfDamageManaLeech, ModType::Base)],
    );
    m.insert(
        "mark_skills_curse_effect_+%",
        smallvec![tagged_def(
            StatId::CurseEffect,
            ModType::Inc,
            smallvec![ModTag::SkillType(109)]
        )],
    );
    m.insert(
        "maximum_added_cold_damage_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::ColdMax,
            ModType::Base,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "maximum_added_cold_damage_vs_chilled_enemies",
        smallvec![tagged_def(
            StatId::ColdMax,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Chilled",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "maximum_added_fire_damage_vs_ignited_enemies",
        smallvec![tagged_def(
            StatId::FireMax,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Ignited",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "maximum_energy_shield_leech_amount_per_leech_+%",
        smallvec![def(StatId::MaxEnergyShieldLeechRate, ModType::Inc)],
    );
    m.insert(
        "maximum_life_+%_for_corpses_you_create",
        smallvec![def(StatId::CorpseLife, ModType::Inc)],
    );
    m.insert(
        "maximum_life_leech_amount_per_leech_+%",
        smallvec![def(StatId::MaxLifeLeechRate, ModType::Inc)],
    );
    m.insert(
        "melee_attack_number_of_spirit_strikes",
        smallvec![def(StatId::AdditionalStrikeTarget, ModType::Base)],
    );
    m.insert(
        "melee_counterattack_trigger_on_block_%",
        smallvec![def(
            StatId::SdChanceToTriggerCounterattackOnBlock,
            ModType::Base
        )],
    );
    m.insert(
        "melee_counterattack_trigger_on_hit_%",
        smallvec![def(
            StatId::SdChanceToTriggerCounterAttackOnHit,
            ModType::Base
        )],
    );
    m.insert(
        "melee_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(256)
        )],
    );
    m.insert(
        "melee_damage_vs_bleeding_enemies_+%",
        smallvec![flagged_tagged_def(
            StatId::PhysicalDamage,
            ModType::Inc,
            ModFlag::from_bits_truncate(256),
            smallvec![ModTag::ActorCondition {
                var: "Bleeding",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "melee_physical_damage_+%",
        smallvec![flagged_def(
            StatId::PhysicalDamage,
            ModType::Inc,
            ModFlag::from_bits_truncate(256)
        )],
    );
    m.insert(
        "melee_range_+",
        smallvec![
            def(StatId::MeleeWeaponRange, ModType::Base),
            def(StatId::UnarmedRange, ModType::Base),
        ],
    );
    m.insert(
        "melee_weapon_range_+",
        smallvec![def(StatId::MeleeWeaponRange, ModType::Base)],
    );
    m.insert(
        "mine_critical_strike_chance_+%_per_power_charge",
        smallvec![kw_tagged_def(
            StatId::CritChance,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(8192),
            smallvec![ModTag::Multiplier("PowerCharge")]
        )],
    );
    m.insert(
        "mine_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(8192)
        )],
    );
    m.insert(
        "mine_detonation_radius_+%",
        smallvec![def(StatId::MineDetonationAreaOfEffect, ModType::Inc)],
    );
    m.insert(
        "mine_duration_+%",
        smallvec![def(StatId::MineDuration, ModType::Base)],
    );
    m.insert(
        "mine_laying_speed_+%",
        smallvec![def(StatId::MineLayingSpeed, ModType::Inc)],
    );
    m.insert(
        "mine_projectile_speed_+%_per_frenzy_charge",
        smallvec![kw_tagged_def(
            StatId::ProjectileSpeed,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(8192),
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "mine_throwing_speed_+%_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::MineLayingSpeed,
            ModType::Inc,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "minimum_added_cold_damage_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::ColdMin,
            ModType::Base,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "minimum_added_cold_damage_vs_chilled_enemies",
        smallvec![tagged_def(
            StatId::ColdMin,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Chilled",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "minimum_added_fire_damage_vs_ignited_enemies",
        smallvec![tagged_def(
            StatId::FireMin,
            ModType::Base,
            smallvec![ModTag::ActorCondition {
                var: "Ignited",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "minion_actor_level_is_user_level_up_to_maximum",
        smallvec![def(StatId::SdMinionLevelIsPlayerLevel, ModType::Base)],
    );
    m.insert(
        "minion_additional_physical_damage_reduction_%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_ailment_damage_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_always_crit",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_area_of_effect_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_attack_speed_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_attack_speed_+%_when_on_low_life",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_base_physical_damage_%_to_convert_to_lightning",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_cast_speed_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_chance_to_deal_double_damage_%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_cooldown_recovery_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_critical_strike_chance_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_critical_strike_multiplier_+",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_damage_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_damage_+%_on_full_life",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_elemental_resistance_%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_life_leech_from_elemental_damage_permyriad",
        smallvec![
            div_def(StatId::MinionModifier, ModType::List, 100_f64),
            div_def(StatId::MinionModifier, ModType::List, 100_f64),
            div_def(StatId::MinionModifier, ModType::List, 100_f64),
        ],
    );
    m.insert(
        "minion_life_regeneration_rate_per_minute_%",
        smallvec![div_def(StatId::MinionModifier, ModType::List, 60_f64)],
    );
    m.insert(
        "minion_maximum_all_elemental_resistances_%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_maximum_life_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_melee_damage_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_melee_range_+",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_movement_speed_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minion_skill_area_of_effect_+%",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minions_deal_%_of_physical_damage_as_additional_chaos_damage",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "minions_have_%_chance_to_inflict_wither_on_hit",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "modifiers_to_buff_effect_duration_also_affect_soul_prevention_duration",
        smallvec![def(
            StatId::SdSkillEffectAppliesToSoulGainPrevention,
            ModType::Base
        )],
    );
    m.insert(
        "modifiers_to_number_of_projectiles_instead_apply_to_chaining",
        smallvec![
            def(StatId::NoAdditionalProjectiles, ModType::Flag),
            def(StatId::AdditionalProjectilesAddChainsInstead, ModType::Flag),
        ],
    );
    m.insert(
        "modifiers_to_number_of_projectiles_instead_apply_to_splitting",
        smallvec![
            def(StatId::NoAdditionalProjectiles, ModType::Flag),
            def(StatId::AdditionalProjectilesAddSplitsInstead, ModType::Flag),
        ],
    );
    m.insert(
        "modifiers_to_projectile_count_do_not_apply",
        smallvec![def(StatId::NoAdditionalProjectiles, ModType::Flag)],
    );
    m.insert(
        "modifiers_to_skill_effect_duration_also_affect_soul_prevention_duration",
        smallvec![def(
            StatId::SdSkillEffectAppliesToSoulGainPrevention,
            ModType::Base
        )],
    );
    m.insert(
        "monster_base_block_%",
        smallvec![def(StatId::BlockChance, ModType::Base)],
    );
    m.insert(
        "monster_inherent_damage_taken_+%_final",
        smallvec![def(StatId::DamageTaken, ModType::More)],
    );
    m.insert(
        "never_any_ailment",
        smallvec![
            def(StatId::CannotShock, ModType::Flag),
            def(StatId::CannotChill, ModType::Flag),
            def(StatId::CannotFreeze, ModType::Flag),
            def(StatId::CannotIgnite, ModType::Flag),
            def(StatId::CannotScorch, ModType::Flag),
            def(StatId::CannotBrittle, ModType::Flag),
            def(StatId::CannotSap, ModType::Flag),
            def(StatId::CannotBleed, ModType::Flag),
            def(StatId::CannotPoison, ModType::Flag),
        ],
    );
    m.insert(
        "never_chill",
        smallvec![def(StatId::CannotChill, ModType::Flag)],
    );
    m.insert(
        "never_freeze",
        smallvec![def(StatId::CannotFreeze, ModType::Flag)],
    );
    m.insert(
        "never_ignite",
        smallvec![def(StatId::CannotIgnite, ModType::Flag)],
    );
    m.insert(
        "never_shock",
        smallvec![def(StatId::CannotShock, ModType::Flag)],
    );
    m.insert(
        "nightblade_elusive_grants_critical_strike_multiplier_+_to_supported_skills",
        smallvec![def(StatId::NightbladeElusiveCritMultiplier, ModType::Base)],
    );
    m.insert(
        "no_critical_strike_multiplier",
        smallvec![def(StatId::NoCritMultiplier, ModType::Flag)],
    );
    m.insert(
        "no_mana_cost",
        smallvec![def(StatId::ManaCost, ModType::More)],
    );
    m.insert(
        "non_curse_aura_effect_+%",
        smallvec![tagged_def(
            StatId::AuraEffect,
            ModType::Inc,
            smallvec![ModTag::SkillType(79)]
        )],
    );
    m.insert(
        "non_damaging_ailment_effect_+%",
        smallvec![
            def(StatId::EnemyChillEffect, ModType::Inc),
            def(StatId::EnemyShockEffect, ModType::Inc),
            def(StatId::EnemyFreezeEffect, ModType::Inc),
            def(StatId::EnemyScorchEffect, ModType::Inc),
            def(StatId::EnemyBrittleEffect, ModType::Inc),
            def(StatId::EnemySapEffect, ModType::Inc),
        ],
    );
    m.insert(
        "number_of_additional_arrows",
        smallvec![kw_def(
            StatId::ProjectileCount,
            ModType::Base,
            KeywordFlag::from_bits_truncate(2048)
        )],
    );
    m.insert(
        "number_of_additional_curses_allowed",
        smallvec![def(StatId::EnemyCurseLimit, ModType::Base)],
    );
    m.insert(
        "number_of_additional_forks_base",
        smallvec![
            def(StatId::ForkTwice, ModType::Flag),
            def(StatId::ForkCountMax, ModType::Base),
        ],
    );
    m.insert(
        "number_of_additional_mines_to_place",
        smallvec![def(StatId::MineThrowCount, ModType::Base)],
    );
    m.insert(
        "number_of_additional_projectiles",
        smallvec![def(StatId::ProjectileCount, ModType::Base)],
    );
    m.insert(
        "number_of_additional_remote_mines_allowed",
        smallvec![def(StatId::ActiveMineLimit, ModType::Base)],
    );
    m.insert(
        "number_of_additional_totems_allowed",
        smallvec![def(StatId::ActiveTotemLimit, ModType::Base)],
    );
    m.insert(
        "number_of_additional_traps_allowed",
        smallvec![def(StatId::ActiveTrapLimit, ModType::Base)],
    );
    m.insert(
        "number_of_additional_traps_to_throw",
        smallvec![def(StatId::TrapThrowCount, ModType::Base)],
    );
    m.insert(
        "number_of_archer_skeletons_to_summon",
        smallvec![def(StatId::MinionPerCastCount, ModType::Base)],
    );
    m.insert(
        "number_of_chains",
        smallvec![def(StatId::ChainCountMax, ModType::Base)],
    );
    m.insert(
        "number_of_mage_skeletons_to_summon",
        smallvec![def(StatId::MinionPerCastCount, ModType::Base)],
    );
    m.insert(
        "number_of_melee_skeletons_to_summon",
        smallvec![def(StatId::MinionPerCastCount, ModType::Base)],
    );
    m.insert(
        "number_of_spider_minions_allowed",
        smallvec![def(StatId::ActiveSpiderLimit, ModType::Base)],
    );
    m.insert(
        "number_of_tigers_allowed",
        smallvec![def(StatId::ActiveTigerLimit, ModType::Base)],
    );
    m.insert(
        "number_of_wolves_allowed",
        smallvec![def(StatId::ActiveWolfLimit, ModType::Base)],
    );
    m.insert(
        "off_hand_base_weapon_attack_duration_ms",
        smallvec![def(StatId::SdSetOffHandAttackTime, ModType::Base)],
    );
    m.insert(
        "off_hand_critical_strike_chance_+_per_10_es_on_shield",
        smallvec![StatDef {
            stat: StatId::CritChance,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![ModTag::PerStat {
                stat: StatId::EnergyShieldOnWeapon2,
                div: 10_f64
            }]
        }],
    );
    m.insert(
        "off_hand_local_maximum_added_cold_damage",
        smallvec![def(StatId::SdSetOffHandColdMax, ModType::Base)],
    );
    m.insert(
        "off_hand_local_maximum_added_fire_damage",
        smallvec![def(StatId::SdSetOffHandFireMax, ModType::Base)],
    );
    m.insert(
        "off_hand_local_maximum_added_physical_damage",
        smallvec![def(StatId::SdSetOffHandPhysicalMax, ModType::Base)],
    );
    m.insert(
        "off_hand_local_minimum_added_cold_damage",
        smallvec![def(StatId::SdSetOffHandColdMin, ModType::Base)],
    );
    m.insert(
        "off_hand_local_minimum_added_fire_damage",
        smallvec![def(StatId::SdSetOffHandFireMin, ModType::Base)],
    );
    m.insert(
        "off_hand_local_minimum_added_physical_damage",
        smallvec![def(StatId::SdSetOffHandPhysicalMin, ModType::Base)],
    );
    m.insert(
        "off_hand_maximum_added_cold_damage_per_15_shield_evasion",
        smallvec![tagged_def(
            StatId::ColdMax,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::EvasionOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_maximum_added_fire_damage_per_15_shield_armour",
        smallvec![tagged_def(
            StatId::FireMax,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::ArmourOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_maximum_added_physical_damage_per_15_shield_armour",
        smallvec![tagged_def(
            StatId::PhysicalMax,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::ArmourOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_maximum_added_physical_damage_per_15_shield_armour_and_evasion_rating",
        smallvec![tagged_def(
            StatId::PhysicalMax,
            ModType::Base,
            smallvec![ModTag::Condition("OffHandAttack")]
        )],
    );
    m.insert(
        "off_hand_minimum_added_cold_damage_per_15_shield_evasion",
        smallvec![tagged_def(
            StatId::ColdMin,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::EvasionOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_minimum_added_fire_damage_per_15_shield_armour",
        smallvec![tagged_def(
            StatId::FireMin,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::ArmourOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_minimum_added_physical_damage_per_15_shield_armour",
        smallvec![tagged_def(
            StatId::PhysicalMin,
            ModType::Base,
            smallvec![
                ModTag::Condition("OffHandAttack"),
                ModTag::PerStat {
                    stat: StatId::ArmourOnWeapon2,
                    div: 15_f64
                }
            ]
        )],
    );
    m.insert(
        "off_hand_minimum_added_physical_damage_per_15_shield_armour_and_evasion_rating",
        smallvec![tagged_def(
            StatId::PhysicalMin,
            ModType::Base,
            smallvec![ModTag::Condition("OffHandAttack")]
        )],
    );
    m.insert(
        "offering_skill_effect_duration_per_corpse",
        smallvec![StatDef {
            stat: StatId::PrimaryDuration,
            mod_type: ModType::Base,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 1000_f64,
            tags: smallvec![ModTag::Multiplier("CorpseConsumedRecently")]
        }],
    );
    m.insert(
        "offering_spells_effect_+%",
        smallvec![def(StatId::BuffEffect, ModType::Inc)],
    );
    m.insert(
        "override_off_hand_base_critical_strike_chance_to_5%",
        smallvec![def(StatId::SkillData, ModType::List)],
    );
    m.insert(
        "physical_damage_%_to_add_as_chaos",
        smallvec![def(StatId::PhysicalDamageGainAsChaos, ModType::Base)],
    );
    m.insert(
        "physical_damage_%_to_add_as_cold",
        smallvec![def(StatId::PhysicalDamageGainAsCold, ModType::Base)],
    );
    m.insert(
        "physical_damage_%_to_add_as_fire",
        smallvec![def(StatId::PhysicalDamageGainAsFire, ModType::Base)],
    );
    m.insert(
        "physical_damage_%_to_add_as_lightning",
        smallvec![def(StatId::PhysicalDamageGainAsLightning, ModType::Base)],
    );
    m.insert(
        "physical_damage_+%",
        smallvec![def(StatId::PhysicalDamage, ModType::Inc)],
    );
    m.insert(
        "physical_damage_+%_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::PhysicalDamage,
            ModType::Inc,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "physical_damage_reduction_rating_+%",
        smallvec![def(StatId::Armour, ModType::Inc)],
    );
    m.insert(
        "physical_damage_taken_+%",
        smallvec![def(StatId::PhysicalDamageTaken, ModType::Inc)],
    );
    m.insert(
        "physical_weapon_damage_+%_per_10_str",
        smallvec![flagged_tagged_def(
            StatId::PhysicalDamage,
            ModType::Inc,
            ModFlag::from_bits_truncate(8192),
            smallvec![ModTag::PerStat {
                stat: StatId::Strength,
                div: 10_f64
            }]
        )],
    );
    m.insert(
        "pierce_%",
        smallvec![def(StatId::PierceChance, ModType::Base)],
    );
    m.insert(
        "placing_traps_cooldown_recovery_+%",
        smallvec![kw_def(
            StatId::CooldownRecovery,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(4096)
        )],
    );
    m.insert(
        "poison_dot_multiplier_+",
        smallvec![kw_def(
            StatId::DotMultiplier,
            ModType::Base,
            KeywordFlag::from_bits_truncate(2097152)
        )],
    );
    m.insert(
        "poison_duration_is_skill_duration",
        smallvec![def(StatId::SdPoisonDurationIsSkillDuration, ModType::Base)],
    );
    m.insert(
        "projectile_base_number_of_targets_to_pierce",
        smallvec![def(StatId::PierceCount, ModType::Base)],
    );
    m.insert(
        "projectile_behaviour_only_explode",
        smallvec![def(StatId::CannotSplit, ModType::Flag)],
    );
    m.insert(
        "projectile_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(1024)
        )],
    );
    m.insert(
        "projectile_damage_+%_final_if_pierced_enemy",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(1024),
            smallvec![ModTag::StatThreshold {
                stat: StatId::PiercedCount,
                threshold: 1_f64,
                upper: false,
                threshold_stat: None
            }]
        )],
    );
    m.insert(
        "projectile_damage_+%_if_pierced_enemy",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(1024),
            smallvec![ModTag::StatThreshold {
                stat: StatId::PiercedCount,
                threshold: 1_f64,
                upper: false,
                threshold_stat: None
            }]
        )],
    );
    m.insert(
        "projectile_damage_+%_per_remaining_chain",
        smallvec![
            flagged_tagged_def(
                StatId::Damage,
                ModType::Inc,
                ModFlag::from_bits_truncate(1024),
                smallvec![ModTag::PerStat {
                    stat: StatId::ChainRemaining,
                    div: 1_f64
                }]
            ),
            flagged_tagged_def(
                StatId::Damage,
                ModType::Inc,
                ModFlag::from_bits_truncate(2048),
                smallvec![ModTag::PerStat {
                    stat: StatId::ChainRemaining,
                    div: 1_f64
                }]
            ),
        ],
    );
    m.insert(
        "projectile_damage_modifiers_apply_to_skill_dot",
        smallvec![def(StatId::SdDotIsProjectile, ModType::Base)],
    );
    m.insert(
        "projectile_damage_taken_+%",
        smallvec![def(StatId::ProjectileDamageTaken, ModType::Inc)],
    );
    m.insert(
        "projectile_number_to_split",
        smallvec![def(StatId::SplitCount, ModType::Base)],
    );
    m.insert(
        "projectiles_always_pierce_you",
        smallvec![def(StatId::AlwaysPierceSelf, ModType::Flag)],
    );
    m.insert(
        "projectiles_cannot_split",
        smallvec![def(StatId::CannotSplit, ModType::Flag)],
    );
    m.insert(
        "projectiles_fork",
        smallvec![
            def(StatId::ForkOnce, ModType::Flag),
            def(StatId::ForkCountMax, ModType::Base),
        ],
    );
    m.insert(
        "receive_bleeding_chance_%_when_hit_by_attack",
        smallvec![def(StatId::SelfBleedChance, ModType::Base)],
    );
    m.insert(
        "reduce_enemy_chaos_resistance_%",
        smallvec![def(StatId::ChaosPenetration, ModType::Base)],
    );
    m.insert(
        "reduce_enemy_elemental_resistance_%",
        smallvec![def(StatId::ElementalPenetration, ModType::Base)],
    );
    m.insert(
        "remote_mined_by_support",
        smallvec![
            def(StatId::ManaCostGainAsReservation, ModType::Flag),
            def(StatId::LifeCostGainAsReservation, ModType::Flag),
        ],
    );
    m.insert(
        "returning_projectiles_always_pierce",
        smallvec![tagged_def(
            StatId::PierceAllTargets,
            ModType::Flag,
            smallvec![ModTag::Condition("ReturningProjectile")]
        )],
    );
    m.insert(
        "secondary_maximum_base_chaos_damage",
        smallvec![def(StatId::ChaosMax, ModType::Base)],
    );
    m.insert(
        "secondary_maximum_base_cold_damage",
        smallvec![def(StatId::ColdMax, ModType::Base)],
    );
    m.insert(
        "secondary_maximum_base_fire_damage",
        smallvec![def(StatId::FireMax, ModType::Base)],
    );
    m.insert(
        "secondary_maximum_base_lightning_damage",
        smallvec![def(StatId::LightningMax, ModType::Base)],
    );
    m.insert(
        "secondary_maximum_base_physical_damage",
        smallvec![def(StatId::PhysicalMax, ModType::Base)],
    );
    m.insert(
        "secondary_minimum_base_chaos_damage",
        smallvec![def(StatId::ChaosMin, ModType::Base)],
    );
    m.insert(
        "secondary_minimum_base_cold_damage",
        smallvec![def(StatId::ColdMin, ModType::Base)],
    );
    m.insert(
        "secondary_minimum_base_fire_damage",
        smallvec![def(StatId::FireMin, ModType::Base)],
    );
    m.insert(
        "secondary_minimum_base_lightning_damage",
        smallvec![def(StatId::LightningMin, ModType::Base)],
    );
    m.insert(
        "secondary_minimum_base_physical_damage",
        smallvec![def(StatId::PhysicalMin, ModType::Base)],
    );
    m.insert(
        "secondary_skill_effect_duration_+%",
        smallvec![def(StatId::SecondaryDuration, ModType::Inc)],
    );
    m.insert(
        "set_base_avoid_projectiles_%_chance",
        smallvec![def(StatId::AvoidProjectilesChance, ModType::Base)],
    );
    m.insert(
        "set_base_cannot_be_damaged",
        smallvec![def(StatId::DamageTaken, ModType::More)],
    );
    m.insert(
        "set_energy_shield_recharge_rate_per_minute_%",
        smallvec![div_def(
            StatId::EnergyShieldRecharge,
            ModType::Override,
            6000_f64
        )],
    );
    m.insert(
        "set_immune_to_curses",
        smallvec![def(StatId::CurseImmune, ModType::Flag)],
    );
    m.insert(
        "set_max_endurance_charges",
        smallvec![def(StatId::EnduranceChargesMax, ModType::Override)],
    );
    m.insert(
        "set_max_frenzy_charges",
        smallvec![def(StatId::FrenzyChargesMax, ModType::Override)],
    );
    m.insert(
        "set_max_power_charges",
        smallvec![def(StatId::PowerChargesMax, ModType::Override)],
    );
    m.insert(
        "set_maximum_life_is_one",
        smallvec![def(StatId::Life, ModType::Override)],
    );
    m.insert(
        "shield_charge_damage_+%_maximum",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(4)
        )],
    );
    m.insert(
        "shock_duration_+%",
        smallvec![def(StatId::EnemyShockDuration, ModType::Inc)],
    );
    m.insert(
        "shock_effect_+%",
        smallvec![def(StatId::EnemyShockEffect, ModType::Inc)],
    );
    m.insert(
        "shock_maximum_magnitude_+",
        smallvec![def(StatId::ShockMax, ModType::Base)],
    );
    m.insert(
        "shock_minimum_damage_taken_increase_%+",
        smallvec![def(StatId::ShockMinimum, ModType::Base)],
    );
    m.insert(
        "sigil_attached_target_damage_+%_final",
        smallvec![tagged_def(
            StatId::Damage,
            ModType::More,
            smallvec![ModTag::MultiplierThreshold {
                var: "BrandsAttachedToEnemy",
                threshold: 1_f64
            }]
        )],
    );
    m.insert(
        "sigil_attached_target_hit_damage_+%_final",
        smallvec![flagged_tagged_def(
            StatId::Damage,
            ModType::More,
            ModFlag::from_bits_truncate(4),
            smallvec![ModTag::Condition("TargetingBrandedEnemy")]
        )],
    );
    m.insert(
        "sigil_repeat_frequency_+%",
        smallvec![def(StatId::BrandActivationFrequency, ModType::Inc)],
    );
    m.insert(
        "skill_area_of_effect_+%_final_in_sand_stance",
        smallvec![tagged_def(
            StatId::AreaOfEffect,
            ModType::More,
            smallvec![ModTag::Condition("SandStance")]
        )],
    );
    m.insert(
        "skill_buff_effect_+%",
        smallvec![def(StatId::BuffEffect, ModType::Inc)],
    );
    m.insert(
        "skill_can_own_mirage_archers",
        smallvec![def(StatId::SdTriggeredByMirageArcher, ModType::Base)],
    );
    m.insert(
        "skill_cannot_gain_repeat_bonuses",
        smallvec![def(StatId::NoRepeatBonuses, ModType::Flag)],
    );
    m.insert(
        "skill_cold_damage_%_to_convert_to_chaos",
        smallvec![def(StatId::SkillColdDamageConvertToChaos, ModType::Base)],
    );
    m.insert(
        "skill_cold_damage_%_to_convert_to_fire",
        smallvec![def(StatId::SkillColdDamageConvertToFire, ModType::Base)],
    );
    m.insert(
        "skill_convert_%_physical_damage_to_random_element",
        smallvec![def(StatId::PhysicalDamageConvertToRandom, ModType::Base)],
    );
    m.insert(
        "skill_double_hits_when_dual_wielding",
        smallvec![def(StatId::SdDoubleHitsWhenDualWielding, ModType::Base)],
    );
    m.insert(
        "skill_effect_duration_+%",
        smallvec![def(StatId::Duration, ModType::Inc)],
    );
    m.insert(
        "skill_effect_duration_+%_per_removable_frenzy_charge",
        smallvec![tagged_def(
            StatId::Duration,
            ModType::Inc,
            smallvec![ModTag::Multiplier("RemovableFrenzyCharge")]
        )],
    );
    m.insert(
        "skill_fire_damage_%_to_convert_to_chaos",
        smallvec![def(StatId::SkillFireDamageConvertToChaos, ModType::Base)],
    );
    m.insert(
        "skill_has_trigger_from_unique_item",
        smallvec![def(StatId::SdTriggeredByUnique, ModType::Base)],
    );
    m.insert(
        "skill_lightning_damage_%_to_convert_to_chaos",
        smallvec![def(
            StatId::SkillLightningDamageConvertToChaos,
            ModType::Base
        )],
    );
    m.insert(
        "skill_lightning_damage_%_to_convert_to_cold",
        smallvec![def(
            StatId::SkillLightningDamageConvertToCold,
            ModType::Base
        )],
    );
    m.insert(
        "skill_lightning_damage_%_to_convert_to_fire",
        smallvec![def(
            StatId::SkillLightningDamageConvertToFire,
            ModType::Base
        )],
    );
    m.insert(
        "skill_minion_explosion_life_%",
        smallvec![div_def(StatId::SkillData, ModType::List, 100_f64)],
    );
    m.insert(
        "skill_physical_damage_%_to_convert_to_chaos",
        smallvec![def(
            StatId::SkillPhysicalDamageConvertToChaos,
            ModType::Base
        )],
    );
    m.insert(
        "skill_physical_damage_%_to_convert_to_cold",
        smallvec![def(StatId::SkillPhysicalDamageConvertToCold, ModType::Base)],
    );
    m.insert(
        "skill_physical_damage_%_to_convert_to_fire",
        smallvec![def(StatId::SkillPhysicalDamageConvertToFire, ModType::Base)],
    );
    m.insert(
        "skill_physical_damage_%_to_convert_to_lightning",
        smallvec![def(
            StatId::SkillPhysicalDamageConvertToLightning,
            ModType::Base
        )],
    );
    m.insert(
        "skill_repeat_count",
        smallvec![tagged_def(
            StatId::RepeatCount,
            ModType::Base,
            smallvec![ModTag::SkillType(26)]
        )],
    );
    m.insert(
        "skill_triggered_by_snipe",
        smallvec![def(StatId::SdTriggeredBySnipe, ModType::Base)],
    );
    m.insert(
        "skill_triggered_when_you_focus_chance_%",
        smallvec![StatDef {
            stat: StatId::SkillData,
            mod_type: ModType::List,
            flags: ModFlag::empty(),
            keywords: KeywordFlag::empty(),
            div: 100_f64,
            tags: smallvec![ModTag::SkillType(35), ModTag::SkillType(2)]
        }],
    );
    m.insert(
        "snipe_triggered_skill_ailment_damage_+%_final_per_stage",
        smallvec![def(StatId::snipeAilmentMulti, ModType::Base)],
    );
    m.insert(
        "snipe_triggered_skill_damage_+%_final",
        smallvec![def(StatId::Damage, ModType::More)],
    );
    m.insert(
        "snipe_triggered_skill_hit_damage_+%_final_per_stage",
        smallvec![def(StatId::snipeHitMulti, ModType::Base)],
    );
    m.insert(
        "spell_base_fire_damage_%_maximum_life",
        smallvec![div_def(StatId::SkillData, ModType::List, 100_f64)],
    );
    m.insert(
        "spell_cast_time_added_to_cooldown_if_triggered",
        smallvec![def(
            StatId::SpellCastTimeAddedToCooldownIfTriggered,
            ModType::Flag
        )],
    );
    m.insert(
        "spell_cast_time_cannot_be_modified",
        smallvec![def(StatId::SdFixedCastTime, ModType::Base)],
    );
    m.insert(
        "spell_critical_strike_chance_+%",
        smallvec![flagged_def(
            StatId::CritChance,
            ModType::Inc,
            ModFlag::from_bits_truncate(2)
        )],
    );
    m.insert(
        "spell_damage_+%",
        smallvec![flagged_def(
            StatId::Damage,
            ModType::Inc,
            ModFlag::from_bits_truncate(2)
        )],
    );
    m.insert(
        "spell_damage_+%_per_10_int",
        smallvec![def(StatId::SdDamage, ModType::Base)],
    );
    m.insert(
        "spell_damage_modifiers_apply_to_skill_dot",
        smallvec![def(StatId::SdDotIsSpell, ModType::Base)],
    );
    m.insert(
        "spell_has_trigger_from_crafted_item_mod",
        smallvec![def(StatId::SdTriggeredByCraft, ModType::Base)],
    );
    m.insert(
        "spell_impale_on_crit_%_chance",
        smallvec![flagged_tagged_def(
            StatId::ImpaleChance,
            ModType::Base,
            ModFlag::from_bits_truncate(2),
            smallvec![ModTag::Condition("CriticalStrike")]
        )],
    );
    m.insert(
        "spell_maximum_added_chaos_damage",
        smallvec![kw_def(
            StatId::ChaosMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_maximum_added_cold_damage",
        smallvec![kw_def(
            StatId::ColdMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_maximum_added_fire_damage",
        smallvec![kw_def(
            StatId::FireMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_maximum_added_lightning_damage",
        smallvec![kw_def(
            StatId::LightningMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_maximum_added_physical_damage",
        smallvec![kw_def(
            StatId::PhysicalMax,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_maximum_base_chaos_damage",
        smallvec![def(StatId::ChaosMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_cold_damage",
        smallvec![def(StatId::ColdMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_cold_damage_+_per_10_intelligence",
        smallvec![def(StatId::ColdMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_cold_damage_per_removable_frenzy_charge",
        smallvec![def(StatId::ColdMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_fire_damage",
        smallvec![def(StatId::FireMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_fire_damage_per_removable_endurance_charge",
        smallvec![def(StatId::FireMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_lightning_damage",
        smallvec![def(StatId::LightningMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_lightning_damage_per_removable_power_charge",
        smallvec![def(StatId::LightningMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_physical_damage",
        smallvec![def(StatId::PhysicalMax, ModType::Base)],
    );
    m.insert(
        "spell_maximum_base_physical_damage_%_of_ward",
        smallvec![def(StatId::PhysicalMax, ModType::Base)],
    );
    m.insert(
        "spell_minimum_added_chaos_damage",
        smallvec![kw_def(
            StatId::ChaosMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_minimum_added_cold_damage",
        smallvec![kw_def(
            StatId::ColdMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_minimum_added_fire_damage",
        smallvec![kw_def(
            StatId::FireMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_minimum_added_lightning_damage",
        smallvec![kw_def(
            StatId::LightningMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_minimum_added_physical_damage",
        smallvec![kw_def(
            StatId::PhysicalMin,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "spell_minimum_base_chaos_damage",
        smallvec![def(StatId::ChaosMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_cold_damage",
        smallvec![def(StatId::ColdMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_cold_damage_+_per_10_intelligence",
        smallvec![def(StatId::ColdMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_cold_damage_per_removable_frenzy_charge",
        smallvec![def(StatId::ColdMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_fire_damage",
        smallvec![def(StatId::FireMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_fire_damage_per_removable_endurance_charge",
        smallvec![def(StatId::FireMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_lightning_damage",
        smallvec![def(StatId::LightningMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_lightning_damage_per_removable_power_charge",
        smallvec![def(StatId::LightningMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_physical_damage",
        smallvec![def(StatId::PhysicalMin, ModType::Base)],
    );
    m.insert(
        "spell_minimum_base_physical_damage_%_of_ward",
        smallvec![def(StatId::PhysicalMin, ModType::Base)],
    );
    m.insert(
        "spell_uncastable_if_triggerable",
        smallvec![def(StatId::SdTriggered, ModType::Base)],
    );
    m.insert(
        "spells_impale_on_hit_%_chance",
        smallvec![kw_def(
            StatId::ImpaleChance,
            ModType::Base,
            KeywordFlag::from_bits_truncate(131072)
        )],
    );
    m.insert(
        "strong_casting",
        smallvec![def(StatId::IronWill, ModType::Flag)],
    );
    m.insert(
        "stun_duration_+%_vs_enemies_that_are_on_full_life",
        smallvec![tagged_def(
            StatId::EnemyStunDuration,
            ModType::Inc,
            smallvec![ModTag::ActorCondition {
                var: "FullLife",
                actor: Some("enemy")
            }]
        )],
    );
    m.insert(
        "stun_threshold_+%",
        smallvec![def(StatId::StunThreshold, ModType::Inc)],
    );
    m.insert(
        "summon_cold_resistance_+",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "summon_fire_resistance_+",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "summon_lightning_resistance_+",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "summon_sacred_wisps_on_hit",
        smallvec![def(StatId::SdTriggeredBySacredWisps, ModType::Base)],
    );
    m.insert(
        "summon_totem_cast_speed_+%",
        smallvec![def(StatId::TotemPlacementSpeed, ModType::Inc)],
    );
    m.insert(
        "support_added_cooldown_count_if_not_instant",
        smallvec![tagged_def(
            StatId::AdditionalCooldownUses,
            ModType::Base,
            smallvec![ModTag::SkillType(74)]
        )],
    );
    m.insert(
        "support_additional_trap_mine_%_chance_for_1_additional_trap_mine",
        smallvec![
            div_def(StatId::MineThrowCount, ModType::Base, 100_f64),
            div_def(StatId::TrapThrowCount, ModType::Base, 100_f64),
        ],
    );
    m.insert(
        "support_additional_trap_mine_%_chance_for_2_additional_trap_mine",
        smallvec![
            div_def(StatId::MineThrowCount, ModType::Base, 50_f64),
            div_def(StatId::TrapThrowCount, ModType::Base, 50_f64),
        ],
    );
    m.insert(
        "support_additional_trap_mine_%_chance_for_3_additional_trap_mine",
        smallvec![
            div_def(StatId::MineThrowCount, ModType::Base, 33.333333333333_f64),
            div_def(StatId::TrapThrowCount, ModType::Base, 33.333333333333_f64),
        ],
    );
    m.insert(
        "support_autocast_instant_spells",
        smallvec![def(StatId::SdTriggeredByAutomation, ModType::Base)],
    );
    m.insert(
        "support_autocast_warcries",
        smallvec![def(StatId::SdTriggeredByAutoexertion, ModType::Base)],
    );
    m.insert(
        "support_barrage_attack_time_+%_per_projectile_fired",
        smallvec![tagged_def(
            StatId::SkillAttackTime,
            ModType::More,
            smallvec![ModTag::PerStat {
                stat: StatId::ProjectileCount,
                div: 1_f64
            }]
        )],
    );
    m.insert(
        "support_barrage_trap_and_mine_throwing_time_+%_final_per_projectile_fired",
        smallvec![
            tagged_def(
                StatId::SkillMineThrowingTime,
                ModType::More,
                smallvec![ModTag::PerStat {
                    stat: StatId::ProjectileCount,
                    div: 1_f64
                }]
            ),
            tagged_def(
                StatId::SkillTrapThrowingTime,
                ModType::More,
                smallvec![ModTag::PerStat {
                    stat: StatId::ProjectileCount,
                    div: 1_f64
                }]
            ),
        ],
    );
    m.insert(
        "support_cast_on_life_spent",
        smallvec![def(
            StatId::SdTriggeredByFoulbornKitavaThirst,
            ModType::Base
        )],
    );
    m.insert(
        "support_cast_on_mana_spent",
        smallvec![def(StatId::SdTriggeredByKitavaThirst, ModType::Base)],
    );
    m.insert(
        "support_chain_count_+%_final",
        smallvec![def(StatId::ChainCountMax, ModType::More)],
    );
    m.insert(
        "support_makes_skill_mine_pvp_damage_+%_final",
        smallvec![def(StatId::PvpDamageMultiplier, ModType::More)],
    );
    m.insert(
        "support_minion_damage_minion_life_+%_final",
        smallvec![def(StatId::MinionModifier, ModType::List)],
    );
    m.insert(
        "support_swift_affliction_skill_effect_and_damaging_ailment_duration_+%_final",
        smallvec![
            def(StatId::Duration, ModType::More),
            def(StatId::DamagingAilmentDuration, ModType::More),
        ],
    );
    m.insert(
        "support_trap_damage_+%_final",
        smallvec![kw_def(
            StatId::Damage,
            ModType::More,
            KeywordFlag::from_bits_truncate(4096)
        )],
    );
    m.insert(
        "supported_active_skill_gem_level_+",
        smallvec![def(StatId::SupportedGemProperty, ModType::List)],
    );
    m.insert(
        "supported_active_skill_gem_quality_%",
        smallvec![def(StatId::SupportedGemProperty, ModType::List)],
    );
    m.insert(
        "supported_aura_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(43)]
        )],
    );
    m.insert(
        "supported_chaos_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(49)]
        )],
    );
    m.insert(
        "supported_cold_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(33)]
        )],
    );
    m.insert(
        "supported_curse_skill_gem_level_+",
        smallvec![kw_def(
            StatId::SupportedGemProperty,
            ModType::List,
            KeywordFlag::from_bits_truncate(2)
        )],
    );
    m.insert(
        "supported_elemental_skill_gem_level_+",
        smallvec![kw_def(
            StatId::SupportedGemProperty,
            ModType::List,
            KeywordFlag::from_bits_truncate(224)
        )],
    );
    m.insert(
        "supported_fire_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(32)]
        )],
    );
    m.insert(
        "supported_lightning_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(34)]
        )],
    );
    m.insert(
        "supported_minion_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(9)]
        )],
    );
    m.insert(
        "supported_physical_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(86)]
        )],
    );
    m.insert(
        "supported_strike_skill_gem_level_+",
        smallvec![tagged_def(
            StatId::SupportedGemProperty,
            ModType::List,
            smallvec![ModTag::SkillType(25)]
        )],
    );
    m.insert(
        "throw_X_additional_traps_if_dual_wielding",
        smallvec![tagged_def(
            StatId::TrapThrowCount,
            ModType::Base,
            smallvec![ModTag::Condition("DualWielding")]
        )],
    );
    m.insert(
        "totem_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(16384)
        )],
    );
    m.insert(
        "totem_duration_+%",
        smallvec![def(StatId::TotemDuration, ModType::Inc)],
    );
    m.insert(
        "totem_life_+%",
        smallvec![def(StatId::TotemLife, ModType::Inc)],
    );
    m.insert(
        "totem_life_+%_final",
        smallvec![def(StatId::TotemLife, ModType::More)],
    );
    m.insert(
        "totem_support_gem_level",
        smallvec![def(StatId::SdTotemLevel, ModType::Base)],
    );
    m.insert(
        "totems_regenerate_%_life_per_minute",
        smallvec![kw_div_def(
            StatId::LifeRegenPercent,
            ModType::Base,
            KeywordFlag::from_bits_truncate(16384),
            60_f64
        )],
    );
    m.insert(
        "trap_critical_strike_multiplier_+_per_power_charge",
        smallvec![kw_tagged_def(
            StatId::CritMultiplier,
            ModType::Base,
            KeywordFlag::from_bits_truncate(4096),
            smallvec![ModTag::Multiplier("PowerCharge")]
        )],
    );
    m.insert(
        "trap_damage_+%",
        smallvec![kw_def(
            StatId::Damage,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(4096)
        )],
    );
    m.insert(
        "trap_duration_+%",
        smallvec![def(StatId::TrapDuration, ModType::Base)],
    );
    m.insert(
        "trap_throwing_speed_+%",
        smallvec![def(StatId::TrapThrowingSpeed, ModType::Inc)],
    );
    m.insert(
        "trap_throwing_speed_+%_per_frenzy_charge",
        smallvec![tagged_def(
            StatId::TrapThrowingSpeed,
            ModType::Inc,
            smallvec![ModTag::Multiplier("FrenzyCharge")]
        )],
    );
    m.insert(
        "trap_throwing_speed_+%_while_wielding_2hand",
        smallvec![tagged_def(
            StatId::TrapThrowingSpeed,
            ModType::Inc,
            smallvec![ModTag::Condition("UsingTwoHandedWeapon")]
        )],
    );
    m.insert(
        "trap_trigger_radius_+%",
        smallvec![def(StatId::TrapTriggerAreaOfEffect, ModType::Inc)],
    );
    m.insert(
        "trauma_strike_self_damage_per_trauma",
        smallvec![def(StatId::TraumaSelfDamageTakenLife, ModType::Base)],
    );
    m.insert(
        "treat_enemy_resistances_as_negated_on_elemental_damage_hit_%_chance",
        smallvec![div_def(
            StatId::HitsInvertEleResChance,
            ModType::Chance,
            100_f64
        )],
    );
    m.insert(
        "trigger_on_attack_hit_against_rare_or_unique",
        smallvec![def(StatId::SdTriggerMarkOnRareOrUnique, ModType::Base)],
    );
    m.insert(
        "triggered_by_kinetic_instability_support",
        smallvec![
            def(StatId::SdTriggeredByKineticFlux, ModType::Base),
            def(StatId::SdTriggerOnCrit, ModType::Base),
        ],
    );
    m.insert(
        "triggered_by_spiritual_cry",
        smallvec![def(StatId::SdTriggeredByGeneralsCry, ModType::Base)],
    );
    m.insert(
        "triggered_skill_damage_+%",
        smallvec![tagged_def(
            StatId::TriggeredDamage,
            ModType::Inc,
            smallvec![ModTag::SkillType(41)]
        )],
    );
    m.insert(
        "triggered_vicious_hex_explosion",
        smallvec![def(StatId::SdTriggeredWhenHexEnds, ModType::Base)],
    );
    m.insert(
        "unique_cospris_malice_cold_spells_triggered",
        smallvec![
            def(StatId::SdTriggeredByCospris, ModType::Base),
            def(StatId::SdTriggerOnCrit, ModType::Base),
        ],
    );
    m.insert(
        "unique_mjolner_lightning_spells_triggered",
        smallvec![def(StatId::SdTriggeredByMjolner, ModType::Base)],
    );
    m.insert(
        "warcry_cooldown_speed_+%",
        smallvec![kw_def(
            StatId::CooldownRecovery,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(4)
        )],
    );
    m.insert(
        "warcry_count_power_from_enemies",
        smallvec![def(StatId::UsesWarcryPower, ModType::Flag)],
    );
    m.insert(
        "warcry_grant_damage_+%_to_exerted_attacks",
        smallvec![flagged_def(
            StatId::ExertIncrease,
            ModType::Inc,
            ModFlag::from_bits_truncate(1)
        )],
    );
    m.insert(
        "warcry_speed_+%",
        smallvec![kw_def(
            StatId::WarcrySpeed,
            ModType::Inc,
            KeywordFlag::from_bits_truncate(4)
        )],
    );
    m.insert(
        "withered_on_hit_chance_%",
        smallvec![def(StatId::ConditionCanWither, ModType::Flag)],
    );
    m.insert(
        "withered_on_hit_for_2_seconds_%_chance",
        smallvec![def(StatId::ConditionCanWither, ModType::Flag)],
    );
}

fn build_gem_stats(_m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {
    // Auto-generated by gen_gem_stat_table.ts — do not edit here.
    // Re-run `bun run tool:gen-gem-stats` to refresh.
}

fn build_manual(m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {
    // ── Manual additions ────────────────────────────────────────────────
    // Stat IDs not in SkillStatMap.json — primarily tree passives and item mods
    // that use PoE's raw stat ID naming conventions.
    //
    // ── Defensive pool stats ──
    m.insert(
        "maximum_life_+%",
        smallvec![def(StatId::Life, ModType::Inc)],
    );
    m.insert(
        "maximum_life_+",
        smallvec![def(StatId::Life, ModType::Base)],
    );
    m.insert(
        "maximum_life_%",
        smallvec![def(StatId::Life, ModType::Base)],
    );
    m.insert(
        "base_maximum_life",
        smallvec![def(StatId::Life, ModType::Base)],
    );
    m.insert(
        "additional_maximum_life",
        smallvec![def(StatId::Life, ModType::Base)],
    );

    m.insert(
        "maximum_mana_+%",
        smallvec![def(StatId::Mana, ModType::Inc)],
    );
    m.insert(
        "maximum_mana_+",
        smallvec![def(StatId::Mana, ModType::Base)],
    );
    m.insert(
        "base_maximum_mana",
        smallvec![def(StatId::Mana, ModType::Base)],
    );
    m.insert(
        "additional_maximum_mana",
        smallvec![def(StatId::Mana, ModType::Base)],
    );

    m.insert(
        "maximum_energy_shield_+%",
        smallvec![def(StatId::EnergyShield, ModType::Inc)],
    );
    m.insert(
        "maximum_energy_shield_+",
        smallvec![def(StatId::EnergyShield, ModType::Base)],
    );
    m.insert(
        "base_maximum_energy_shield",
        smallvec![def(StatId::EnergyShield, ModType::Base)],
    );
    m.insert(
        "additional_maximum_energy_shield",
        smallvec![def(StatId::EnergyShield, ModType::Base)],
    );

    m.insert(
        "evasion_rating_+%",
        smallvec![def(StatId::Evasion, ModType::Inc)],
    );
    m.insert(
        "evasion_rating_+",
        smallvec![def(StatId::Evasion, ModType::Base)],
    );
    m.insert(
        "base_evasion_rating",
        smallvec![def(StatId::Evasion, ModType::Base)],
    );
    m.insert(
        "additional_evasion_rating",
        smallvec![def(StatId::Evasion, ModType::Base)],
    );

    // Evasion + Armour combined node (expands to two modifiers)
    m.insert(
        "evasion_and_physical_damage_reduction_rating_+%",
        smallvec![
            def(StatId::Evasion, ModType::Inc),
            def(StatId::Armour, ModType::Inc),
        ],
    );

    // ── Attributes ──
    m.insert(
        "strength_+%",
        smallvec![def(StatId::Strength, ModType::Inc)],
    );
    m.insert(
        "strength_+",
        smallvec![def(StatId::Strength, ModType::Base)],
    );
    m.insert(
        "base_strength",
        smallvec![def(StatId::Strength, ModType::Base)],
    );
    m.insert(
        "additional_strength",
        smallvec![def(StatId::Strength, ModType::Base)],
    );

    m.insert(
        "dexterity_+%",
        smallvec![def(StatId::Dexterity, ModType::Inc)],
    );
    m.insert(
        "dexterity_+",
        smallvec![def(StatId::Dexterity, ModType::Base)],
    );
    m.insert(
        "base_dexterity",
        smallvec![def(StatId::Dexterity, ModType::Base)],
    );
    m.insert(
        "additional_dexterity",
        smallvec![def(StatId::Dexterity, ModType::Base)],
    );

    m.insert(
        "intelligence_+%",
        smallvec![def(StatId::Intelligence, ModType::Inc)],
    );
    m.insert(
        "intelligence_+",
        smallvec![def(StatId::Intelligence, ModType::Base)],
    );
    m.insert(
        "base_intelligence",
        smallvec![def(StatId::Intelligence, ModType::Base)],
    );
    m.insert(
        "additional_intelligence",
        smallvec![def(StatId::Intelligence, ModType::Base)],
    );

    // ── Regeneration ──
    m.insert(
        "life_regeneration_rate_+%",
        smallvec![def(StatId::LifeRegeneration, ModType::Inc)],
    );
    m.insert(
        "life_regeneration_rate_%",
        smallvec![def(StatId::LifeRegeneration, ModType::Base)],
    );
    m.insert(
        "base_life_regeneration_rate",
        smallvec![def(StatId::LifeRegeneration, ModType::Base)],
    );

    m.insert(
        "mana_regeneration_rate_+%",
        smallvec![def(StatId::ManaRegeneration, ModType::Inc)],
    );
    m.insert(
        "mana_regeneration_rate_%",
        smallvec![def(StatId::ManaRegeneration, ModType::Base)],
    );
    m.insert(
        "base_mana_regeneration_rate",
        smallvec![def(StatId::ManaRegeneration, ModType::Base)],
    );

    // ── Block ──
    m.insert(
        "shield_block_%",
        smallvec![def(StatId::BlockChance, ModType::Base)],
    );
    m.insert(
        "block_%",
        smallvec![def(StatId::BlockChance, ModType::Base)],
    );
    m.insert(
        "spell_block_%",
        smallvec![def(StatId::SpellBlockChance, ModType::Base)],
    );
    m.insert(
        "spell_block_chance_%",
        smallvec![def(StatId::SpellBlockChance, ModType::Base)],
    );

    // ── Resistances ──
    m.insert(
        "fire_resistance_%",
        smallvec![def(StatId::FireResist, ModType::Base)],
    );
    m.insert(
        "fire_resistance_+%",
        smallvec![def(StatId::FireResist, ModType::Inc)],
    );
    m.insert(
        "cold_resistance_%",
        smallvec![def(StatId::ColdResist, ModType::Base)],
    );
    m.insert(
        "cold_resistance_+%",
        smallvec![def(StatId::ColdResist, ModType::Inc)],
    );
    m.insert(
        "lightning_resistance_%",
        smallvec![def(StatId::LightningResist, ModType::Base)],
    );
    m.insert(
        "lightning_resistance_+%",
        smallvec![def(StatId::LightningResist, ModType::Inc)],
    );
    m.insert(
        "chaos_resistance_%",
        smallvec![def(StatId::ChaosResist, ModType::Base)],
    );
    m.insert(
        "chaos_resistance_+%",
        smallvec![def(StatId::ChaosResist, ModType::Inc)],
    );

    // ── Damage ──
    m.insert(
        "base_physical_damage",
        smallvec![def(StatId::PhysicalDamage, ModType::Base)],
    );
    m.insert(
        "physical_attack_damage_+%",
        smallvec![flagged_def(
            StatId::PhysicalDamage,
            ModType::Inc,
            ModFlag::ATTACK
        )],
    );
    m.insert(
        "base_elemental_damage",
        smallvec![def(StatId::ElementalDamage, ModType::Base)],
    );
    m.insert(
        "attack_damage_+",
        smallvec![flagged_def(StatId::Damage, ModType::Base, ModFlag::ATTACK)],
    );
    m.insert(
        "base_attack_damage",
        smallvec![flagged_def(StatId::Damage, ModType::Base, ModFlag::ATTACK)],
    );
    m.insert(
        "spell_damage_+",
        smallvec![flagged_def(StatId::Damage, ModType::Base, ModFlag::SPELL)],
    );
    m.insert(
        "base_spell_damage",
        smallvec![flagged_def(StatId::Damage, ModType::Base, ModFlag::SPELL)],
    );

    // ── Critical strike ──
    m.insert(
        "global_critical_strike_chance_+%",
        smallvec![def(StatId::CritChance, ModType::Inc)],
    );
    m.insert(
        "critical_strike_multiplier_+",
        smallvec![def(StatId::CritMultiplier, ModType::Base)],
    );
    m.insert(
        "global_critical_strike_multiplier_+",
        smallvec![def(StatId::CritMultiplier, ModType::Base)],
    );
}

// ── Utility ──────────────────────────────────────────────────────────────────

/// Build a `Modifier` from a `StatDef` + runtime value + source.
pub fn apply(def: &StatDef, value: f64, source: SourceId) -> Modifier {
    Modifier {
        stat: def.stat,
        mod_type: def.mod_type,
        value: value / def.div,
        flags: def.flags,
        keywords: def.keywords,
        source,
        tags: def.tags.clone(),
    }
}
