// Auto-generated from src-tauri/data/pob/SkillStatMap.json
// Calc variable names used by ModDB. Add new variants as the calc engine grows.

use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::sync::OnceLock;

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[repr(u16)]
pub enum StatId {
    Accuracy = 0,
    ActiveArbalistLimit = 1,
    ActiveGolemLimit = 2,
    ActiveHivebornLimit = 3,
    ActiveHolyRelicLimit = 4,
    ActiveHolyStrikeMinionLimit = 5,
    ActiveLivingLightningLimit = 6,
    ActiveMineLimit = 7,
    ActiveRagingSpiritLimit = 8,
    ActiveReaperLimit = 9,
    ActiveSentinelOfAbsolutionLimit = 10,
    ActiveSentinelOfPurityLimit = 11,
    ActiveSkeletonLimit = 12,
    ActiveSpectreLimit = 13,
    ActiveSpiderLimit = 14,
    ActiveTigerLimit = 15,
    ActiveTotemLimit = 16,
    ActiveTrapLimit = 17,
    ActiveVoidSpawnLimit = 18,
    ActiveWolfLimit = 19,
    ActiveZombieLimit = 20,
    AddedDamage = 21,
    AdditionalChainsAddSplitsInstead = 22,
    AdditionalCooldownUses = 23,
    AdditionalProjectilesAddChainsInstead = 24,
    AdditionalProjectilesAddSplitsInstead = 25,
    AdditionalStrikeTarget = 26,
    AlwaysPierceSelf = 27,
    AreaOfEffect = 28,
    Armour = 29,
    AttackDodgeChance = 30,
    AuraEffect = 31,
    AvoidInterruptStun = 32,
    AvoidProjectilesChance = 33,
    AvoidStun = 34,
    BeamChainCountMax = 35,
    BleedChance = 36,
    BleedFaster = 37,
    BleedStacksMax = 38,
    BlindEffect = 39,
    BlockChance = 40,
    BlockEffect = 41,
    BloodMagicReserved = 42,
    BrandActivationFrequency = 43,
    BrandsAttachedLimit = 44,
    BuffEffect = 45,
    CanLeechLifeOnFullLife = 46,
    CannotBleed = 47,
    CannotBrittle = 48,
    CannotChill = 49,
    CannotFreeze = 50,
    CannotIgnite = 51,
    CannotMultiplePoison = 52,
    CannotPierce = 53,
    CannotPoison = 54,
    CannotSap = 55,
    CannotScorch = 56,
    CannotShock = 57,
    CannotSplit = 58,
    ChainCountMax = 59,
    ChaosCanFreeze = 60,
    ChaosCanIgnite = 61,
    ChaosCanShock = 62,
    ChaosDamage = 63,
    ChaosDamageTaken = 64,
    ChaosDotMultiplier = 65,
    ChaosMax = 66,
    ChaosMin = 67,
    ChaosPenetration = 68,
    ChaosResist = 69,
    ChillAsThoughDealing = 70,
    ColdCanIgnite = 71,
    ColdCanShock = 72,
    ColdDamage = 73,
    ColdDamageGainAsFire = 74,
    ColdDamageTaken = 75,
    ColdDotMultiplier = 76,
    ColdExposureChance = 77,
    ColdMax = 78,
    ColdMin = 79,
    ColdPenetration = 80,
    ColdResist = 81,
    ColdResistMax = 82,
    ConditionCanGainRage = 83,
    ConditionCanWither = 84,
    ConditionCannotBeDamaged = 85,
    ConditionCannotRecallBrand = 86,
    ConditionNonPoisonedOnly = 87,
    ConsecratedGroundEffect = 88,
    CooldownRecovery = 89,
    CorpseLife = 90,
    CostLifeInsteadOfMana = 91,
    CritChance = 92,
    CritMultiplier = 93,
    CullPercent = 94,
    CurseEffect = 95,
    CurseEffectAgainstPlayer = 96,
    CurseImmune = 97,
    Damage = 98,
    DamageTaken = 99,
    DamageTakenConsecratedGround = 100,
    DamageTakenOverTime = 101,
    DamagingAilmentDuration = 102,
    DealNoChaos = 103,
    DealNoCold = 104,
    DealNoFire = 105,
    DealNoLightning = 106,
    Dexterity = 107,
    DotMultiplier = 108,
    DoubleDamageChance = 109,
    DoubleEnemyStunDurationChance = 110,
    Duration = 111,
    ElementalDamage = 112,
    ElementalPenetration = 113,
    ElementalResist = 114,
    ElusiveEffect = 115,
    EnduranceChargeCount = 116,
    EnduranceChargesMax = 117,
    EnemyAilmentDuration = 118,
    EnemyBleedDuration = 119,
    EnemyBrittleDuration = 120,
    EnemyBrittleEffect = 121,
    EnemyChillDuration = 122,
    EnemyChillEffect = 123,
    EnemyCurseLimit = 124,
    EnemyElementalAilmentDuration = 125,
    EnemyFreezeChance = 126,
    EnemyFreezeDuration = 127,
    EnemyFreezeEffect = 128,
    EnemyIgniteChance = 129,
    EnemyIgniteDuration = 130,
    EnemyKnockbackChance = 131,
    EnemyKnockbackDistance = 132,
    EnemyPoisonDuration = 133,
    EnemySapDuration = 134,
    EnemySapEffect = 135,
    EnemyScorchChance = 136,
    EnemyScorchEffect = 137,
    EnemyShockChance = 138,
    EnemyShockDuration = 139,
    EnemyShockEffect = 140,
    EnemyStunDuration = 141,
    EnergyShield = 142,
    EnergyShieldRecharge = 143,
    Evasion = 144,
    ExertIncrease = 145,
    FireCanFreeze = 146,
    FireCanShock = 147,
    FireDamage = 148,
    FireDamageGainAsChaos = 149,
    FireDamageTaken = 150,
    FireDotMultiplier = 151,
    FireExposureChance = 152,
    FireMax = 153,
    FireMin = 154,
    FirePenetration = 155,
    FireResist = 156,
    FireResistMax = 157,
    FlaskEffect = 158,
    FlaskManaRecovery = 159,
    ForkCountMax = 160,
    ForkOnce = 161,
    ForkTwice = 162,
    FortifyDuration = 163,
    FreezeAsThoughDealing = 164,
    FrenzyChargeCount = 165,
    FrenzyChargesMax = 166,
    HybridManaAndLifeCost_Life = 167,
    IgniteBurnFaster = 168,
    ImpaleChance = 169,
    ImpaleEffect = 170,
    ImprovedMinionDamageAppliesToPlayer = 171,
    ImprovedSpellDamageAppliesToAttacks = 172,
    Intelligence = 173,
    IronGrip = 174,
    IronWill = 175,
    IsLeeching = 176,
    KnockbackImmune = 177,
    Life = 178,
    LifeCost = 179,
    LifeCostGainAsReservation = 180,
    LifeGainOnHit = 181,
    LifeOnHit = 182,
    LifeRegeneration = 183,
    LifeReservationEfficiency = 184,
    LifeReserved = 185,
    LightningCanFreeze = 186,
    LightningCanIgnite = 187,
    LightningCannotShock = 188,
    LightningDamage = 189,
    LightningDamageGainAsChaos = 190,
    LightningDamageTaken = 191,
    LightningExposureChance = 192,
    LightningMax = 193,
    LightningMin = 194,
    LightningPenetration = 195,
    LightningResist = 196,
    LightningResistMax = 197,
    LinkEffectOnSelf = 198,
    LootQuantity = 199,
    LootRarity = 200,
    LuckyHitsChance = 201,
    Mana = 202,
    ManaCostGainAsReservation = 203,
    ManaCostNoMult = 204,
    ManaGainOnHit = 205,
    ManaOnHit = 206,
    ManaRegeneration = 207,
    ManaReservationEfficiency = 208,
    ManaReserved = 209,
    MaxDoom = 210,
    MaxEnergyShieldLeechRate = 211,
    MaxLifeLeechRate = 212,
    MeleeWeaponRange = 213,
    MineDetonationAreaOfEffect = 214,
    MineDuration = 215,
    MineLayingSpeed = 216,
    MineThrowCount = 217,
    MinionDamageAppliesToPlayer = 218,
    MinionModifier = 219,
    MinionPerCastCount = 220,
    MovementSpeed = 221,
    NightbladeElusiveCritMultiplier = 222,
    NoAdditionalChains = 223,
    NoAdditionalProjectiles = 224,
    NoCooldownRecoveryInDuration = 225,
    NoCritMultiplier = 226,
    NoRepeatBonuses = 227,
    PhysicalCanFreeze = 228,
    PhysicalCanIgnite = 229,
    PhysicalCanShock = 230,
    PhysicalDamage = 231,
    PhysicalDamageConvertToChaos = 232,
    PhysicalDamageConvertToCold = 233,
    PhysicalDamageConvertToFire = 234,
    PhysicalDamageConvertToLightning = 235,
    PhysicalDamageConvertToRandom = 236,
    PhysicalDamageGainAsChaos = 237,
    PhysicalDamageGainAsCold = 238,
    PhysicalDamageGainAsFire = 239,
    PhysicalDamageGainAsLightning = 240,
    PhysicalDamageTaken = 241,
    PhysicalDamageTakenOverTime = 242,
    PhysicalMax = 243,
    PhysicalMin = 244,
    PierceAllTargets = 245,
    PierceChance = 246,
    PierceCount = 247,
    PoisonChance = 248,
    PoisonFaster = 249,
    PoisonStackLimit = 250,
    PowerChargeCount = 251,
    PowerChargesMax = 252,
    ProjectileCount = 253,
    ProjectileDamageTaken = 254,
    ProjectileSpeed = 255,
    PvpDamageMultiplier = 256,
    RepeatCount = 257,
    ReservationEfficiency = 258,
    Reserved = 259,
    SecondaryDuration = 260,
    SelfBleedChance = 261,
    SelfDamageLifeLeech = 262,
    SelfDamageManaLeech = 263,
    SelfExtraCritChance = 264,
    SelfFreezeChance = 265,
    SelfIgniteChance = 266,
    SelfKnockbackChance = 267,
    SelfLifeOnHit = 268,
    SelfLifeOnKill = 269,
    SelfManaOnHit = 270,
    SelfManaOnKill = 271,
    SelfPierceChance = 272,
    SelfShockChance = 273,
    ShamblingUndeadLimit = 274,
    ShockAsThoughDealing = 275,
    ShockMax = 276,
    ShockMinimum = 277,
    SkillAttackTime = 278,
    SkillColdDamageConvertToChaos = 279,
    SkillColdDamageConvertToFire = 280,
    SkillData = 281,
    SkillFireDamageConvertToChaos = 282,
    SkillLightningDamageConvertToChaos = 283,
    SkillLightningDamageConvertToCold = 284,
    SkillLightningDamageConvertToFire = 285,
    SkillMineThrowingTime = 286,
    SkillPhysicalDamageConvertToChaos = 287,
    SkillPhysicalDamageConvertToCold = 288,
    SkillPhysicalDamageConvertToFire = 289,
    SkillPhysicalDamageConvertToLightning = 290,
    SkillTrapThrowingTime = 291,
    Speed = 292,
    SpellBlockChance = 293,
    SpellCastTimeAddedToCooldownIfTriggered = 294,
    SpellDamageAppliesToAttacks = 295,
    SpellDodgeChance = 296,
    SplitCount = 297,
    Strength = 298,
    StunImmune = 299,
    StunRecovery = 300,
    StunThreshold = 301,
    SupportedGemProperty = 302,
    TotalDamageTaken = 303,
    TotemDuration = 304,
    TotemLife = 305,
    TotemPlacementSpeed = 306,
    TrapDuration = 307,
    TrapThrowCount = 308,
    TrapThrowingSpeed = 309,
    TrapTriggerAreaOfEffect = 310,
    TraumaSelfDamageTakenLife = 311,
    TriggeredDamage = 312,
    TripleDamageChance = 313,
    UnarmedRange = 314,
    UnlimitedBrandDuration = 315,
    UsesWarcryPower = 316,
    WarcrySpeed = 317,
    snipeAilmentMulti = 318,
    snipeHitMulti = 319,
    // Added: missing from original generation
    BuffExpireFaster = 320,
    ChaosDamageLifeLeech = 321,
    ColdDamageLeech = 322,
    DamageEnergyShieldLeech = 323,
    DamageLifeLeech = 324,
    DamageManaLeech = 325,
    ElementalDamageManaLeech = 326,
    ElementalResistMax = 327,
    EnemyImpalePhysicalDamageReduction = 328,
    EnemyPhysicalDamageReduction = 329,
    EnemyStunThreshold = 330,
    FireDamageLeech = 331,
    FireDegen = 332,
    HitsInvertEleResChance = 333,
    LifeRegen = 334,
    LifeRegenPercent = 335,
    LightningDamageLeech = 336,
    ManaCost = 337,
    ManaRegen = 338,
    PhysicalDamageLifeLeech = 339,
    PhysicalDamageReduction = 340,
    PhysicalDegen = 341,
    PrimaryDuration = 342,
    SelfCritMultiplier = 343,
    SelfFreezeDuration = 344,
    SelfIgniteDuration = 345,
    SelfShockDuration = 346,
    // Phase 3.14: gem stat coverage expansion
    ActionSpeed = 347,
    BlindChance = 348,
    CharacterSize = 349,
    ChillChance = 350,
    FreezeChance = 351,
    IgniteChance = 352,
    KnockbackDistance = 353,
    ShockChance = 354,
    Stealth = 355,
    // SkillData keyed variants — one per SkillStatMap LIST key (Phase 3.14)
    SdAreaOfEffect = 356,
    SdChaosMax = 357,
    SdChaosMin = 358,
    SdColdMax = 359,
    SdColdMin = 360,
    SdCritMultiplier = 361,
    SdDamage = 362,
    SdFireMax = 363,
    SdFireMin = 364,
    SdLightningMax = 365,
    SdLightningMin = 366,
    SdPhysicalMax = 367,
    SdPhysicalMin = 368,
    SdArrowSpeedAppliesToAreaOfEffect = 369,
    SdBleedDurationIsSkillDuration = 370,
    SdCannotBeEvaded = 371,
    SdCastTimeOverridesAttackTime = 372,
    SdChanceToTriggerCounterAttackOnHit = 373,
    SdChanceToTriggerCounterattackOnBlock = 374,
    SdChanceToTriggerCurseOnCurse = 375,
    SdChanceToTriggerOnCrit = 376,
    SdChanceToTriggerOnMeleeKill = 377,
    SdChanceToTriggerOnParentAttack = 378,
    SdChanceToTriggerOnStun = 379,
    SdDecay = 380,
    SdDotIsProjectile = 381,
    SdDotIsSpell = 382,
    SdDoubleHitsWhenDualWielding = 383,
    SdFixedCastTime = 384,
    SdMineDurationAppliesToSkill = 385,
    SdMinionDamageEffectiveness = 386,
    SdMinionLevelIsEnemyLevel = 387,
    SdMinionLevelIsPlayerLevel = 388,
    SdPoisonDurationIsSkillDuration = 389,
    SdRadius = 390,
    SdRadiusExtra = 391,
    SdRadiusSecondary = 392,
    SdRadiusTertiary = 393,
    SdSetOffHandAttackTime = 394,
    SdSetOffHandColdMax = 395,
    SdSetOffHandColdMin = 396,
    SdSetOffHandFireMax = 397,
    SdSetOffHandFireMin = 398,
    SdSetOffHandPhysicalMax = 399,
    SdSetOffHandPhysicalMin = 400,
    SdShowAverage = 401,
    SdSkillEffectAppliesToSoulGainPrevention = 402,
    SdTotemLevel = 403,
    SdTriggerMarkOnRareOrUnique = 404,
    SdTriggerOnCrit = 405,
    SdTriggered = 406,
    SdTriggeredByAutoexertion = 407,
    SdTriggeredByAutomation = 408,
    SdTriggeredByCoc = 409,
    SdTriggeredByCospris = 410,
    SdTriggeredByCraft = 411,
    SdTriggeredByDamageTaken = 412,
    SdTriggeredByFoulbornKitavaThirst = 413,
    SdTriggeredByGeneralsCry = 414,
    SdTriggeredByKineticFlux = 415,
    SdTriggeredByKitavaThirst = 416,
    SdTriggeredByMeleeKill = 417,
    SdTriggeredByMirageArcher = 418,
    SdTriggeredByMjolner = 419,
    SdTriggeredBySacredWisps = 420,
    SdTriggeredBySaviour = 421,
    SdTriggeredBySnipe = 422,
    SdTriggeredBySquirmingTerror = 423,
    SdTriggeredByTrapTrigger = 424,
    SdTriggeredByUnique = 425,
    SdTriggeredWhenHexEnds = 426,
    SdTriggeredWhileChannelling = 427,
    // Per-stat tag runtime values (calc phase)
    Ward = 428,
    EnergyShieldOnWeapon2 = 429,
    EvasionOnWeapon2 = 430,
    ArmourOnWeapon2 = 431,
    Chain = 432,
    ChainRemaining = 433,
    PiercedCount = 434,
    // Phase 5: CalcSetup base values
    ChaosResistMax = 435,
    BlockChanceMax = 436,
    SpellBlockChanceMax = 437,
    MaxManaLeechRate = 438,
    MaxLifeLeechInstance = 439,
    MaxManaLeechInstance = 440,
    MaxEnergyShieldLeechInstance = 441,
    DamageReductionMax = 442,
    // Phase 5.5: Defence calc stats
    ChaosInoculation = 443,
    IronReflexes = 444,
    ZealotsOath = 445,
    SpellSuppressionChance = 446,
    SpellSuppressionEffect = 447,
    LifeRecoveryRate = 448,
    ManaRecoveryRate = 449,
    EnergyShieldRecoveryRate = 450,
    EnergyShieldRechargeFaster = 451,
    SpellDodgeChanceMax = 452,
    ManaRegenPercent = 453,
    EnergyShieldRegenPercent = 454,
    LightningDamageConvertToCold = 455,
    LightningDamageConvertToFire = 456,
    LightningDamageConvertToChaos = 457,
    ColdDamageConvertToFire = 458,
    ColdDamageConvertToChaos = 459,
    FireDamageConvertToChaos = 460,
}

impl StatId {
    /// Look up a StatId from its calc-variable name.
    /// Returns `None` for stats not yet in the enum.
    pub fn from_name(name: &str) -> Option<Self> {
        static MAP: OnceLock<FxHashMap<&'static str, StatId>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut m = FxHashMap::default();
            m.insert("Accuracy", Self::Accuracy);
            m.insert("ActiveArbalistLimit", Self::ActiveArbalistLimit);
            m.insert("ActiveGolemLimit", Self::ActiveGolemLimit);
            m.insert("ActiveHivebornLimit", Self::ActiveHivebornLimit);
            m.insert("ActiveHolyRelicLimit", Self::ActiveHolyRelicLimit);
            m.insert(
                "ActiveHolyStrikeMinionLimit",
                Self::ActiveHolyStrikeMinionLimit,
            );
            m.insert(
                "ActiveLivingLightningLimit",
                Self::ActiveLivingLightningLimit,
            );
            m.insert("ActiveMineLimit", Self::ActiveMineLimit);
            m.insert("ActiveRagingSpiritLimit", Self::ActiveRagingSpiritLimit);
            m.insert("ActiveReaperLimit", Self::ActiveReaperLimit);
            m.insert(
                "ActiveSentinelOfAbsolutionLimit",
                Self::ActiveSentinelOfAbsolutionLimit,
            );
            m.insert(
                "ActiveSentinelOfPurityLimit",
                Self::ActiveSentinelOfPurityLimit,
            );
            m.insert("ActiveSkeletonLimit", Self::ActiveSkeletonLimit);
            m.insert("ActiveSpectreLimit", Self::ActiveSpectreLimit);
            m.insert("ActiveSpiderLimit", Self::ActiveSpiderLimit);
            m.insert("ActiveTigerLimit", Self::ActiveTigerLimit);
            m.insert("ActiveTotemLimit", Self::ActiveTotemLimit);
            m.insert("ActiveTrapLimit", Self::ActiveTrapLimit);
            m.insert("ActiveVoidSpawnLimit", Self::ActiveVoidSpawnLimit);
            m.insert("ActiveWolfLimit", Self::ActiveWolfLimit);
            m.insert("ActiveZombieLimit", Self::ActiveZombieLimit);
            m.insert("AddedDamage", Self::AddedDamage);
            m.insert(
                "AdditionalChainsAddSplitsInstead",
                Self::AdditionalChainsAddSplitsInstead,
            );
            m.insert("AdditionalCooldownUses", Self::AdditionalCooldownUses);
            m.insert(
                "AdditionalProjectilesAddChainsInstead",
                Self::AdditionalProjectilesAddChainsInstead,
            );
            m.insert(
                "AdditionalProjectilesAddSplitsInstead",
                Self::AdditionalProjectilesAddSplitsInstead,
            );
            m.insert("AdditionalStrikeTarget", Self::AdditionalStrikeTarget);
            m.insert("AlwaysPierceSelf", Self::AlwaysPierceSelf);
            m.insert("AreaOfEffect", Self::AreaOfEffect);
            m.insert("Armour", Self::Armour);
            m.insert("AttackDodgeChance", Self::AttackDodgeChance);
            m.insert("AuraEffect", Self::AuraEffect);
            m.insert("AvoidInterruptStun", Self::AvoidInterruptStun);
            m.insert("AvoidProjectilesChance", Self::AvoidProjectilesChance);
            m.insert("AvoidStun", Self::AvoidStun);
            m.insert("BeamChainCountMax", Self::BeamChainCountMax);
            m.insert("BleedChance", Self::BleedChance);
            m.insert("BleedFaster", Self::BleedFaster);
            m.insert("BleedStacksMax", Self::BleedStacksMax);
            m.insert("BlindEffect", Self::BlindEffect);
            m.insert("BlockChance", Self::BlockChance);
            m.insert("BlockEffect", Self::BlockEffect);
            m.insert("BloodMagicReserved", Self::BloodMagicReserved);
            m.insert("BrandActivationFrequency", Self::BrandActivationFrequency);
            m.insert("BrandsAttachedLimit", Self::BrandsAttachedLimit);
            m.insert("BuffEffect", Self::BuffEffect);
            m.insert("CanLeechLifeOnFullLife", Self::CanLeechLifeOnFullLife);
            m.insert("CannotBleed", Self::CannotBleed);
            m.insert("CannotBrittle", Self::CannotBrittle);
            m.insert("CannotChill", Self::CannotChill);
            m.insert("CannotFreeze", Self::CannotFreeze);
            m.insert("CannotIgnite", Self::CannotIgnite);
            m.insert("CannotMultiplePoison", Self::CannotMultiplePoison);
            m.insert("CannotPierce", Self::CannotPierce);
            m.insert("CannotPoison", Self::CannotPoison);
            m.insert("CannotSap", Self::CannotSap);
            m.insert("CannotScorch", Self::CannotScorch);
            m.insert("CannotShock", Self::CannotShock);
            m.insert("CannotSplit", Self::CannotSplit);
            m.insert("ChainCountMax", Self::ChainCountMax);
            m.insert("ChaosCanFreeze", Self::ChaosCanFreeze);
            m.insert("ChaosCanIgnite", Self::ChaosCanIgnite);
            m.insert("ChaosCanShock", Self::ChaosCanShock);
            m.insert("ChaosDamage", Self::ChaosDamage);
            m.insert("ChaosDamageTaken", Self::ChaosDamageTaken);
            m.insert("ChaosDotMultiplier", Self::ChaosDotMultiplier);
            m.insert("ChaosMax", Self::ChaosMax);
            m.insert("ChaosMin", Self::ChaosMin);
            m.insert("ChaosPenetration", Self::ChaosPenetration);
            m.insert("ChaosResist", Self::ChaosResist);
            m.insert("ChillAsThoughDealing", Self::ChillAsThoughDealing);
            m.insert("ColdCanIgnite", Self::ColdCanIgnite);
            m.insert("ColdCanShock", Self::ColdCanShock);
            m.insert("ColdDamage", Self::ColdDamage);
            m.insert("ColdDamageGainAsFire", Self::ColdDamageGainAsFire);
            m.insert("ColdDamageTaken", Self::ColdDamageTaken);
            m.insert("ColdDotMultiplier", Self::ColdDotMultiplier);
            m.insert("ColdExposureChance", Self::ColdExposureChance);
            m.insert("ColdMax", Self::ColdMax);
            m.insert("ColdMin", Self::ColdMin);
            m.insert("ColdPenetration", Self::ColdPenetration);
            m.insert("ColdResist", Self::ColdResist);
            m.insert("ColdResistMax", Self::ColdResistMax);
            m.insert("Condition:CanGainRage", Self::ConditionCanGainRage);
            m.insert("Condition:CanWither", Self::ConditionCanWither);
            m.insert("Condition:CannotBeDamaged", Self::ConditionCannotBeDamaged);
            m.insert(
                "Condition:CannotRecallBrand",
                Self::ConditionCannotRecallBrand,
            );
            m.insert("Condition:NonPoisonedOnly", Self::ConditionNonPoisonedOnly);
            m.insert("ConsecratedGroundEffect", Self::ConsecratedGroundEffect);
            m.insert("CooldownRecovery", Self::CooldownRecovery);
            m.insert("CorpseLife", Self::CorpseLife);
            m.insert("CostLifeInsteadOfMana", Self::CostLifeInsteadOfMana);
            m.insert("CritChance", Self::CritChance);
            m.insert("CritMultiplier", Self::CritMultiplier);
            m.insert("CullPercent", Self::CullPercent);
            m.insert("CurseEffect", Self::CurseEffect);
            m.insert("CurseEffectAgainstPlayer", Self::CurseEffectAgainstPlayer);
            m.insert("CurseImmune", Self::CurseImmune);
            m.insert("Damage", Self::Damage);
            m.insert("DamageTaken", Self::DamageTaken);
            m.insert(
                "DamageTakenConsecratedGround",
                Self::DamageTakenConsecratedGround,
            );
            m.insert("DamageTakenOverTime", Self::DamageTakenOverTime);
            m.insert("DamagingAilmentDuration", Self::DamagingAilmentDuration);
            m.insert("DealNoChaos", Self::DealNoChaos);
            m.insert("DealNoCold", Self::DealNoCold);
            m.insert("DealNoFire", Self::DealNoFire);
            m.insert("DealNoLightning", Self::DealNoLightning);
            m.insert("Dexterity", Self::Dexterity);
            m.insert("DotMultiplier", Self::DotMultiplier);
            m.insert("DoubleDamageChance", Self::DoubleDamageChance);
            m.insert(
                "DoubleEnemyStunDurationChance",
                Self::DoubleEnemyStunDurationChance,
            );
            m.insert("Duration", Self::Duration);
            m.insert("ElementalDamage", Self::ElementalDamage);
            m.insert("ElementalPenetration", Self::ElementalPenetration);
            m.insert("ElementalResist", Self::ElementalResist);
            m.insert("ElusiveEffect", Self::ElusiveEffect);
            m.insert("EnduranceChargeCount", Self::EnduranceChargeCount);
            m.insert("EnduranceChargesMax", Self::EnduranceChargesMax);
            m.insert("EnemyAilmentDuration", Self::EnemyAilmentDuration);
            m.insert("EnemyBleedDuration", Self::EnemyBleedDuration);
            m.insert("EnemyBrittleDuration", Self::EnemyBrittleDuration);
            m.insert("EnemyBrittleEffect", Self::EnemyBrittleEffect);
            m.insert("EnemyChillDuration", Self::EnemyChillDuration);
            m.insert("EnemyChillEffect", Self::EnemyChillEffect);
            m.insert("EnemyCurseLimit", Self::EnemyCurseLimit);
            m.insert(
                "EnemyElementalAilmentDuration",
                Self::EnemyElementalAilmentDuration,
            );
            m.insert("EnemyFreezeChance", Self::EnemyFreezeChance);
            m.insert("EnemyFreezeDuration", Self::EnemyFreezeDuration);
            m.insert("EnemyFreezeEffect", Self::EnemyFreezeEffect);
            m.insert("EnemyIgniteChance", Self::EnemyIgniteChance);
            m.insert("EnemyIgniteDuration", Self::EnemyIgniteDuration);
            m.insert("EnemyKnockbackChance", Self::EnemyKnockbackChance);
            m.insert("EnemyKnockbackDistance", Self::EnemyKnockbackDistance);
            m.insert("EnemyPoisonDuration", Self::EnemyPoisonDuration);
            m.insert("EnemySapDuration", Self::EnemySapDuration);
            m.insert("EnemySapEffect", Self::EnemySapEffect);
            m.insert("EnemyScorchChance", Self::EnemyScorchChance);
            m.insert("EnemyScorchEffect", Self::EnemyScorchEffect);
            m.insert("EnemyShockChance", Self::EnemyShockChance);
            m.insert("EnemyShockDuration", Self::EnemyShockDuration);
            m.insert("EnemyShockEffect", Self::EnemyShockEffect);
            m.insert("EnemyStunDuration", Self::EnemyStunDuration);
            m.insert("EnergyShield", Self::EnergyShield);
            m.insert("EnergyShieldRecharge", Self::EnergyShieldRecharge);
            m.insert("Evasion", Self::Evasion);
            m.insert("ExertIncrease", Self::ExertIncrease);
            m.insert("FireCanFreeze", Self::FireCanFreeze);
            m.insert("FireCanShock", Self::FireCanShock);
            m.insert("FireDamage", Self::FireDamage);
            m.insert("FireDamageGainAsChaos", Self::FireDamageGainAsChaos);
            m.insert("FireDamageTaken", Self::FireDamageTaken);
            m.insert("FireDotMultiplier", Self::FireDotMultiplier);
            m.insert("FireExposureChance", Self::FireExposureChance);
            m.insert("FireMax", Self::FireMax);
            m.insert("FireMin", Self::FireMin);
            m.insert("FirePenetration", Self::FirePenetration);
            m.insert("FireResist", Self::FireResist);
            m.insert("FireResistMax", Self::FireResistMax);
            m.insert("FlaskEffect", Self::FlaskEffect);
            m.insert("FlaskManaRecovery", Self::FlaskManaRecovery);
            m.insert("ForkCountMax", Self::ForkCountMax);
            m.insert("ForkOnce", Self::ForkOnce);
            m.insert("ForkTwice", Self::ForkTwice);
            m.insert("FortifyDuration", Self::FortifyDuration);
            m.insert("FreezeAsThoughDealing", Self::FreezeAsThoughDealing);
            m.insert("FrenzyChargeCount", Self::FrenzyChargeCount);
            m.insert("FrenzyChargesMax", Self::FrenzyChargesMax);
            m.insert(
                "HybridManaAndLifeCost_Life",
                Self::HybridManaAndLifeCost_Life,
            );
            m.insert("IgniteBurnFaster", Self::IgniteBurnFaster);
            m.insert("ImpaleChance", Self::ImpaleChance);
            m.insert("ImpaleEffect", Self::ImpaleEffect);
            m.insert(
                "ImprovedMinionDamageAppliesToPlayer",
                Self::ImprovedMinionDamageAppliesToPlayer,
            );
            m.insert(
                "ImprovedSpellDamageAppliesToAttacks",
                Self::ImprovedSpellDamageAppliesToAttacks,
            );
            m.insert("Intelligence", Self::Intelligence);
            m.insert("IronGrip", Self::IronGrip);
            m.insert("IronWill", Self::IronWill);
            m.insert("IsLeeching", Self::IsLeeching);
            m.insert("KnockbackImmune", Self::KnockbackImmune);
            m.insert("Life", Self::Life);
            m.insert("LifeCost", Self::LifeCost);
            m.insert("LifeCostGainAsReservation", Self::LifeCostGainAsReservation);
            m.insert("LifeGainOnHit", Self::LifeGainOnHit);
            m.insert("LifeOnHit", Self::LifeOnHit);
            m.insert("LifeRegeneration", Self::LifeRegeneration);
            m.insert("LifeReservationEfficiency", Self::LifeReservationEfficiency);
            m.insert("LifeReserved", Self::LifeReserved);
            m.insert("LightningCanFreeze", Self::LightningCanFreeze);
            m.insert("LightningCanIgnite", Self::LightningCanIgnite);
            m.insert("LightningCannotShock", Self::LightningCannotShock);
            m.insert("LightningDamage", Self::LightningDamage);
            m.insert(
                "LightningDamageGainAsChaos",
                Self::LightningDamageGainAsChaos,
            );
            m.insert("LightningDamageTaken", Self::LightningDamageTaken);
            m.insert("LightningExposureChance", Self::LightningExposureChance);
            m.insert("LightningMax", Self::LightningMax);
            m.insert("LightningMin", Self::LightningMin);
            m.insert("LightningPenetration", Self::LightningPenetration);
            m.insert("LightningResist", Self::LightningResist);
            m.insert("LightningResistMax", Self::LightningResistMax);
            m.insert("LinkEffectOnSelf", Self::LinkEffectOnSelf);
            m.insert("LootQuantity", Self::LootQuantity);
            m.insert("LootRarity", Self::LootRarity);
            m.insert("LuckyHitsChance", Self::LuckyHitsChance);
            m.insert("Mana", Self::Mana);
            m.insert("ManaCostGainAsReservation", Self::ManaCostGainAsReservation);
            m.insert("ManaCostNoMult", Self::ManaCostNoMult);
            m.insert("ManaGainOnHit", Self::ManaGainOnHit);
            m.insert("ManaOnHit", Self::ManaOnHit);
            m.insert("ManaRegeneration", Self::ManaRegeneration);
            m.insert("ManaReservationEfficiency", Self::ManaReservationEfficiency);
            m.insert("ManaReserved", Self::ManaReserved);
            m.insert("MaxDoom", Self::MaxDoom);
            m.insert("MaxEnergyShieldLeechRate", Self::MaxEnergyShieldLeechRate);
            m.insert("MaxLifeLeechRate", Self::MaxLifeLeechRate);
            m.insert("MeleeWeaponRange", Self::MeleeWeaponRange);
            m.insert(
                "MineDetonationAreaOfEffect",
                Self::MineDetonationAreaOfEffect,
            );
            m.insert("MineDuration", Self::MineDuration);
            m.insert("MineLayingSpeed", Self::MineLayingSpeed);
            m.insert("MineThrowCount", Self::MineThrowCount);
            m.insert(
                "MinionDamageAppliesToPlayer",
                Self::MinionDamageAppliesToPlayer,
            );
            m.insert("MinionModifier", Self::MinionModifier);
            m.insert("MinionPerCastCount", Self::MinionPerCastCount);
            m.insert("MovementSpeed", Self::MovementSpeed);
            m.insert(
                "NightbladeElusiveCritMultiplier",
                Self::NightbladeElusiveCritMultiplier,
            );
            m.insert("NoAdditionalChains", Self::NoAdditionalChains);
            m.insert("NoAdditionalProjectiles", Self::NoAdditionalProjectiles);
            m.insert(
                "NoCooldownRecoveryInDuration",
                Self::NoCooldownRecoveryInDuration,
            );
            m.insert("NoCritMultiplier", Self::NoCritMultiplier);
            m.insert("NoRepeatBonuses", Self::NoRepeatBonuses);
            m.insert("PhysicalCanFreeze", Self::PhysicalCanFreeze);
            m.insert("PhysicalCanIgnite", Self::PhysicalCanIgnite);
            m.insert("PhysicalCanShock", Self::PhysicalCanShock);
            m.insert("PhysicalDamage", Self::PhysicalDamage);
            m.insert(
                "PhysicalDamageConvertToChaos",
                Self::PhysicalDamageConvertToChaos,
            );
            m.insert(
                "PhysicalDamageConvertToCold",
                Self::PhysicalDamageConvertToCold,
            );
            m.insert(
                "PhysicalDamageConvertToFire",
                Self::PhysicalDamageConvertToFire,
            );
            m.insert(
                "PhysicalDamageConvertToLightning",
                Self::PhysicalDamageConvertToLightning,
            );
            m.insert(
                "PhysicalDamageConvertToRandom",
                Self::PhysicalDamageConvertToRandom,
            );
            m.insert("PhysicalDamageGainAsChaos", Self::PhysicalDamageGainAsChaos);
            m.insert("PhysicalDamageGainAsCold", Self::PhysicalDamageGainAsCold);
            m.insert("PhysicalDamageGainAsFire", Self::PhysicalDamageGainAsFire);
            m.insert(
                "PhysicalDamageGainAsLightning",
                Self::PhysicalDamageGainAsLightning,
            );
            m.insert("PhysicalDamageTaken", Self::PhysicalDamageTaken);
            m.insert(
                "PhysicalDamageTakenOverTime",
                Self::PhysicalDamageTakenOverTime,
            );
            m.insert("PhysicalMax", Self::PhysicalMax);
            m.insert("PhysicalMin", Self::PhysicalMin);
            m.insert("PierceAllTargets", Self::PierceAllTargets);
            m.insert("PierceChance", Self::PierceChance);
            m.insert("PierceCount", Self::PierceCount);
            m.insert("PoisonChance", Self::PoisonChance);
            m.insert("PoisonFaster", Self::PoisonFaster);
            m.insert("PoisonStackLimit", Self::PoisonStackLimit);
            m.insert("PowerChargeCount", Self::PowerChargeCount);
            m.insert("PowerChargesMax", Self::PowerChargesMax);
            m.insert("ProjectileCount", Self::ProjectileCount);
            m.insert("ProjectileDamageTaken", Self::ProjectileDamageTaken);
            m.insert("ProjectileSpeed", Self::ProjectileSpeed);
            m.insert("PvpDamageMultiplier", Self::PvpDamageMultiplier);
            m.insert("RepeatCount", Self::RepeatCount);
            m.insert("ReservationEfficiency", Self::ReservationEfficiency);
            m.insert("Reserved", Self::Reserved);
            m.insert("SecondaryDuration", Self::SecondaryDuration);
            m.insert("SelfBleedChance", Self::SelfBleedChance);
            m.insert("SelfDamageLifeLeech", Self::SelfDamageLifeLeech);
            m.insert("SelfDamageManaLeech", Self::SelfDamageManaLeech);
            m.insert("SelfExtraCritChance", Self::SelfExtraCritChance);
            m.insert("SelfFreezeChance", Self::SelfFreezeChance);
            m.insert("SelfIgniteChance", Self::SelfIgniteChance);
            m.insert("SelfKnockbackChance", Self::SelfKnockbackChance);
            m.insert("SelfLifeOnHit", Self::SelfLifeOnHit);
            m.insert("SelfLifeOnKill", Self::SelfLifeOnKill);
            m.insert("SelfManaOnHit", Self::SelfManaOnHit);
            m.insert("SelfManaOnKill", Self::SelfManaOnKill);
            m.insert("SelfPierceChance", Self::SelfPierceChance);
            m.insert("SelfShockChance", Self::SelfShockChance);
            m.insert("ShamblingUndeadLimit", Self::ShamblingUndeadLimit);
            m.insert("ShockAsThoughDealing", Self::ShockAsThoughDealing);
            m.insert("ShockMax", Self::ShockMax);
            m.insert("ShockMinimum", Self::ShockMinimum);
            m.insert("SkillAttackTime", Self::SkillAttackTime);
            m.insert(
                "SkillColdDamageConvertToChaos",
                Self::SkillColdDamageConvertToChaos,
            );
            m.insert(
                "SkillColdDamageConvertToFire",
                Self::SkillColdDamageConvertToFire,
            );
            m.insert("SkillData", Self::SkillData);
            m.insert(
                "SkillFireDamageConvertToChaos",
                Self::SkillFireDamageConvertToChaos,
            );
            m.insert(
                "SkillLightningDamageConvertToChaos",
                Self::SkillLightningDamageConvertToChaos,
            );
            m.insert(
                "SkillLightningDamageConvertToCold",
                Self::SkillLightningDamageConvertToCold,
            );
            m.insert(
                "SkillLightningDamageConvertToFire",
                Self::SkillLightningDamageConvertToFire,
            );
            m.insert("SkillMineThrowingTime", Self::SkillMineThrowingTime);
            m.insert(
                "SkillPhysicalDamageConvertToChaos",
                Self::SkillPhysicalDamageConvertToChaos,
            );
            m.insert(
                "SkillPhysicalDamageConvertToCold",
                Self::SkillPhysicalDamageConvertToCold,
            );
            m.insert(
                "SkillPhysicalDamageConvertToFire",
                Self::SkillPhysicalDamageConvertToFire,
            );
            m.insert(
                "SkillPhysicalDamageConvertToLightning",
                Self::SkillPhysicalDamageConvertToLightning,
            );
            m.insert("SkillTrapThrowingTime", Self::SkillTrapThrowingTime);
            m.insert("Speed", Self::Speed);
            m.insert("SpellBlockChance", Self::SpellBlockChance);
            m.insert(
                "SpellCastTimeAddedToCooldownIfTriggered",
                Self::SpellCastTimeAddedToCooldownIfTriggered,
            );
            m.insert(
                "SpellDamageAppliesToAttacks",
                Self::SpellDamageAppliesToAttacks,
            );
            m.insert("SpellDodgeChance", Self::SpellDodgeChance);
            m.insert("SplitCount", Self::SplitCount);
            m.insert("Strength", Self::Strength);
            m.insert("StunImmune", Self::StunImmune);
            m.insert("StunRecovery", Self::StunRecovery);
            m.insert("StunThreshold", Self::StunThreshold);
            m.insert("SupportedGemProperty", Self::SupportedGemProperty);
            m.insert("TotalDamageTaken", Self::TotalDamageTaken);
            m.insert("TotemDuration", Self::TotemDuration);
            m.insert("TotemLife", Self::TotemLife);
            m.insert("TotemPlacementSpeed", Self::TotemPlacementSpeed);
            m.insert("TrapDuration", Self::TrapDuration);
            m.insert("TrapThrowCount", Self::TrapThrowCount);
            m.insert("TrapThrowingSpeed", Self::TrapThrowingSpeed);
            m.insert("TrapTriggerAreaOfEffect", Self::TrapTriggerAreaOfEffect);
            m.insert("TraumaSelfDamageTakenLife", Self::TraumaSelfDamageTakenLife);
            m.insert("TriggeredDamage", Self::TriggeredDamage);
            m.insert("TripleDamageChance", Self::TripleDamageChance);
            m.insert("UnarmedRange", Self::UnarmedRange);
            m.insert("UnlimitedBrandDuration", Self::UnlimitedBrandDuration);
            m.insert("UsesWarcryPower", Self::UsesWarcryPower);
            m.insert("WarcrySpeed", Self::WarcrySpeed);
            m.insert("snipeAilmentMulti", Self::snipeAilmentMulti);
            m.insert("snipeHitMulti", Self::snipeHitMulti);
            // Added: missing from original generation
            m.insert("BuffExpireFaster", Self::BuffExpireFaster);
            m.insert("ChaosDamageLifeLeech", Self::ChaosDamageLifeLeech);
            m.insert("ColdDamageLeech", Self::ColdDamageLeech);
            m.insert("DamageEnergyShieldLeech", Self::DamageEnergyShieldLeech);
            m.insert("DamageLifeLeech", Self::DamageLifeLeech);
            m.insert("DamageManaLeech", Self::DamageManaLeech);
            m.insert("ElementalDamageManaLeech", Self::ElementalDamageManaLeech);
            m.insert("ElementalResistMax", Self::ElementalResistMax);
            m.insert(
                "EnemyImpalePhysicalDamageReduction",
                Self::EnemyImpalePhysicalDamageReduction,
            );
            m.insert(
                "EnemyPhysicalDamageReduction",
                Self::EnemyPhysicalDamageReduction,
            );
            m.insert("EnemyStunThreshold", Self::EnemyStunThreshold);
            m.insert("FireDamageLeech", Self::FireDamageLeech);
            m.insert("FireDegen", Self::FireDegen);
            m.insert("HitsInvertEleResChance", Self::HitsInvertEleResChance);
            m.insert("LifeRegen", Self::LifeRegen);
            m.insert("LifeRegenPercent", Self::LifeRegenPercent);
            m.insert("LightningDamageLeech", Self::LightningDamageLeech);
            m.insert("ManaCost", Self::ManaCost);
            m.insert("ManaRegen", Self::ManaRegen);
            m.insert("PhysicalDamageLifeLeech", Self::PhysicalDamageLifeLeech);
            m.insert("PhysicalDamageReduction", Self::PhysicalDamageReduction);
            m.insert("PhysicalDegen", Self::PhysicalDegen);
            m.insert("PrimaryDuration", Self::PrimaryDuration);
            m.insert("SelfCritMultiplier", Self::SelfCritMultiplier);
            m.insert("SelfFreezeDuration", Self::SelfFreezeDuration);
            m.insert("SelfIgniteDuration", Self::SelfIgniteDuration);
            m.insert("SelfShockDuration", Self::SelfShockDuration);
            // Phase 3.14: gem stat coverage expansion
            m.insert("ActionSpeed", Self::ActionSpeed);
            m.insert("BlindChance", Self::BlindChance);
            m.insert("CharacterSize", Self::CharacterSize);
            m.insert("ChillChance", Self::ChillChance);
            m.insert("FreezeChance", Self::FreezeChance);
            m.insert("IgniteChance", Self::IgniteChance);
            m.insert("KnockbackDistance", Self::KnockbackDistance);
            m.insert("ShockChance", Self::ShockChance);
            m.insert("Stealth", Self::Stealth);
            // Per-stat tag runtime values
            m.insert("Ward", Self::Ward);
            m.insert("EnergyShieldOnWeapon2", Self::EnergyShieldOnWeapon2);
            m.insert("EvasionOnWeapon2", Self::EvasionOnWeapon2);
            m.insert("ArmourOnWeapon2", Self::ArmourOnWeapon2);
            m.insert("Chain", Self::Chain);
            m.insert("ChainRemaining", Self::ChainRemaining);
            m.insert("PiercedCount", Self::PiercedCount);
            // Phase 5: CalcSetup base values
            m.insert("ChaosResistMax", Self::ChaosResistMax);
            m.insert("BlockChanceMax", Self::BlockChanceMax);
            m.insert("SpellBlockChanceMax", Self::SpellBlockChanceMax);
            m.insert("MaxManaLeechRate", Self::MaxManaLeechRate);
            m.insert("MaxLifeLeechInstance", Self::MaxLifeLeechInstance);
            m.insert("MaxManaLeechInstance", Self::MaxManaLeechInstance);
            m.insert(
                "MaxEnergyShieldLeechInstance",
                Self::MaxEnergyShieldLeechInstance,
            );
            m.insert("DamageReductionMax", Self::DamageReductionMax);
            // Phase 5.5: Defence calc stats
            m.insert("ChaosInoculation", Self::ChaosInoculation);
            m.insert("IronReflexes", Self::IronReflexes);
            m.insert("ZealotsOath", Self::ZealotsOath);
            m.insert("SpellSuppressionChance", Self::SpellSuppressionChance);
            m.insert("SpellSuppressionEffect", Self::SpellSuppressionEffect);
            m.insert("LifeRecoveryRate", Self::LifeRecoveryRate);
            m.insert("ManaRecoveryRate", Self::ManaRecoveryRate);
            m.insert("EnergyShieldRecoveryRate", Self::EnergyShieldRecoveryRate);
            m.insert(
                "EnergyShieldRechargeFaster",
                Self::EnergyShieldRechargeFaster,
            );
            m.insert("SpellDodgeChanceMax", Self::SpellDodgeChanceMax);
            m.insert("ManaRegenPercent", Self::ManaRegenPercent);
            m.insert("ManaRegenPercent", Self::ManaRegenPercent);
            m.insert("ManaRegenerationPercent", Self::ManaRegenPercent);
            m.insert("EnergyShieldRegenPercent", Self::EnergyShieldRegenPercent);
            m.insert("EnergyShieldRegen", Self::EnergyShieldRegenPercent);
            m.insert(
                "LightningDamageConvertToCold",
                Self::LightningDamageConvertToCold,
            );
            m.insert(
                "LightningDamageConvertToFire",
                Self::LightningDamageConvertToFire,
            );
            m.insert(
                "LightningDamageConvertToChaos",
                Self::LightningDamageConvertToChaos,
            );
            m.insert("ColdDamageConvertToFire", Self::ColdDamageConvertToFire);
            m.insert("ColdDamageConvertToChaos", Self::ColdDamageConvertToChaos);
            m.insert("FireDamageConvertToChaos", Self::FireDamageConvertToChaos);
            m
        })
        .get(name)
        .copied()
    }
}
