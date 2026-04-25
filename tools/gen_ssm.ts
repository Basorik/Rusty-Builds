/**
 * gen_ssm.ts — Generate src-tauri/src/modifier/stat_table.rs from SkillStatMap.norm.json
 *
 * Usage:  bun run tool:gen-ssm
 *
 * Reads the normalized SkillStatMap (output of fetch_data.ts) and emits a Rust source file
 * containing a static FxHashMap that maps every stat ID string to one or more StatDef entries.
 * The generated file is committed as source; re-run this to refresh after game data updates.
 *
 * The "Manual additions" section at the bottom of the generated build() function is NOT
 * overwritten by this tool — keep custom entries there.
 */

import { readFileSync, writeFileSync, existsSync } from "fs";
import { resolve, join } from "path";

const REPO_ROOT = resolve(import.meta.dir, "..");
const SSM_PATH = join(REPO_ROOT, "src-tauri/data/pob/SkillStatMap.norm.json");
const OUT_PATH = join(REPO_ROOT, "src-tauri/src/modifier/stat_table.rs");
const MANUAL_MARKER = "    // ── Manual additions ────────────────────────────────────────────────";

// ── JSON types (mirror SkillStatMap.norm.json shape) ────────────────────────

type Tag = {
    type: string;
    var?: string;
    varList?: string[];
    stat?: string;
    div?: number;
    limit?: number;
    neg?: boolean;
    actor?: string;
    skillType?: number;
    threshold?: number;
    effectName?: string;
    effectType?: string;
    percent?: boolean;
    modFlags?: number;
    ramp?: number[][];
};

type Mapping = {
    name: string;
    type: string;
    flags: number;
    keywordFlags: number;
    div?: number | null;
    value?: boolean | number | { key: string } | null;
    tags: Tag[];
};

type SSM = Record<string, Mapping[]>;

// ── Rust code emitters ───────────────────────────────────────────────────────

function emitModType(t: string): string {
    const map: Record<string, string> = {
        BASE: "ModType::Base",
        INC: "ModType::Inc",
        MORE: "ModType::More",
        FLAG: "ModType::Flag",
        MAX: "ModType::Max",
        MIN: "ModType::Min",
        LIST: "ModType::List",
        OVERRIDE: "ModType::Override",
        CHANCE: "ModType::Chance",
    };
    const r = map[t];
    if (!r) throw new Error(`Unknown mod type: ${t}`);
    return r;
}

// SkillData LIST key → StatId::Sd* variant name
const SKILL_DATA_KEY_MAP: Record<string, string> = {
    AreaOfEffect: "SdAreaOfEffect",
    ChaosMax: "SdChaosMax",
    ChaosMin: "SdChaosMin",
    ColdMax: "SdColdMax",
    ColdMin: "SdColdMin",
    CritMultiplier: "SdCritMultiplier",
    Damage: "SdDamage",
    FireMax: "SdFireMax",
    FireMin: "SdFireMin",
    LightningMax: "SdLightningMax",
    LightningMin: "SdLightningMin",
    PhysicalMax: "SdPhysicalMax",
    PhysicalMin: "SdPhysicalMin",
    arrowSpeedAppliesToAreaOfEffect: "SdArrowSpeedAppliesToAreaOfEffect",
    bleedDurationIsSkillDuration: "SdBleedDurationIsSkillDuration",
    cannotBeEvaded: "SdCannotBeEvaded",
    castTimeOverridesAttackTime: "SdCastTimeOverridesAttackTime",
    chanceToTriggerCounterAttackOnHit: "SdChanceToTriggerCounterAttackOnHit",
    chanceToTriggerCounterattackOnBlock: "SdChanceToTriggerCounterattackOnBlock",
    chanceToTriggerCurseOnCurse: "SdChanceToTriggerCurseOnCurse",
    chanceToTriggerOnCrit: "SdChanceToTriggerOnCrit",
    chanceToTriggerOnMeleeKill: "SdChanceToTriggerOnMeleeKill",
    chanceToTriggerOnParentAttack: "SdChanceToTriggerOnParentAttack",
    chanceToTriggerOnStun: "SdChanceToTriggerOnStun",
    decay: "SdDecay",
    dotIsProjectile: "SdDotIsProjectile",
    dotIsSpell: "SdDotIsSpell",
    doubleHitsWhenDualWielding: "SdDoubleHitsWhenDualWielding",
    fixedCastTime: "SdFixedCastTime",
    mineDurationAppliesToSkill: "SdMineDurationAppliesToSkill",
    minionDamageEffectiveness: "SdMinionDamageEffectiveness",
    minionLevelIsEnemyLevel: "SdMinionLevelIsEnemyLevel",
    minionLevelIsPlayerLevel: "SdMinionLevelIsPlayerLevel",
    poisonDurationIsSkillDuration: "SdPoisonDurationIsSkillDuration",
    radius: "SdRadius",
    radiusExtra: "SdRadiusExtra",
    radiusSecondary: "SdRadiusSecondary",
    radiusTertiary: "SdRadiusTertiary",
    setOffHandAttackTime: "SdSetOffHandAttackTime",
    setOffHandColdMax: "SdSetOffHandColdMax",
    setOffHandColdMin: "SdSetOffHandColdMin",
    setOffHandFireMax: "SdSetOffHandFireMax",
    setOffHandFireMin: "SdSetOffHandFireMin",
    setOffHandPhysicalMax: "SdSetOffHandPhysicalMax",
    setOffHandPhysicalMin: "SdSetOffHandPhysicalMin",
    showAverage: "SdShowAverage",
    skillEffectAppliesToSoulGainPrevention: "SdSkillEffectAppliesToSoulGainPrevention",
    totemLevel: "SdTotemLevel",
    triggerMarkOnRareOrUnique: "SdTriggerMarkOnRareOrUnique",
    triggerOnCrit: "SdTriggerOnCrit",
    triggered: "SdTriggered",
    triggeredByAutoexertion: "SdTriggeredByAutoexertion",
    triggeredByAutomation: "SdTriggeredByAutomation",
    triggeredByCoc: "SdTriggeredByCoc",
    triggeredByCospris: "SdTriggeredByCospris",
    triggeredByCraft: "SdTriggeredByCraft",
    triggeredByDamageTaken: "SdTriggeredByDamageTaken",
    triggeredByFoulbornKitavaThirst: "SdTriggeredByFoulbornKitavaThirst",
    triggeredByGeneralsCry: "SdTriggeredByGeneralsCry",
    triggeredByKineticFlux: "SdTriggeredByKineticFlux",
    triggeredByKitavaThirst: "SdTriggeredByKitavaThirst",
    triggeredByMeleeKill: "SdTriggeredByMeleeKill",
    triggeredByMirageArcher: "SdTriggeredByMirageArcher",
    triggeredByMjolner: "SdTriggeredByMjolner",
    triggeredBySacredWisps: "SdTriggeredBySacredWisps",
    triggeredBySaviour: "SdTriggeredBySaviour",
    triggeredBySnipe: "SdTriggeredBySnipe",
    triggeredBySquirmingTerror: "SdTriggeredBySquirmingTerror",
    triggeredByTrapTrigger: "SdTriggeredByTrapTrigger",
    triggeredByUnique: "SdTriggeredByUnique",
    triggeredWhenHexEnds: "SdTriggeredWhenHexEnds",
    triggeredWhileChannelling: "SdTriggeredWhileChannelling",
};

function emitStatId(name: string): string {
    // "Condition:CanGainRage" → "StatId::ConditionCanGainRage"
    const normalized = name.replace(":", "");
    return `StatId::${normalized}`;
}

function escStr(s: string): string {
    return `"${s.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

function emitTag(tag: Tag): string | null {
    switch (tag.type) {
        case "Condition":
            if (!tag.var) return null;
            return `ModTag::Condition(${escStr(tag.var)})`;
        case "ActorCondition": {
            if (!tag.var) return null;
            const actor = tag.actor ? `Some(${escStr(tag.actor)})` : "None";
            return `ModTag::ActorCondition { var: ${escStr(tag.var)}, actor: ${actor} }`;
        }
        case "Multiplier":
            if (!tag.var) return null;
            return `ModTag::Multiplier(${escStr(tag.var)})`;
        case "MultiplierThreshold": {
            if (!tag.var) return null;
            const thresh = tag.threshold ?? 0.0;
            return `ModTag::MultiplierThreshold { var: ${escStr(tag.var)}, threshold: ${thresh}_f64 }`;
        }
        case "PerStat": {
            if (!tag.stat) return null;
            const div = tag.div ?? 1.0;
            return `ModTag::PerStat { stat: ${escStr(tag.stat)}, div: ${div}_f64 }`;
        }
        case "PercentStat": {
            if (!tag.stat) return null;
            const pct = tag.percent ?? false;
            return `ModTag::PercentStat { stat: ${escStr(tag.stat)}, percent: ${pct} }`;
        }
        case "StatThreshold": {
            if (!tag.stat) return null;
            const thresh = tag.threshold ?? 0.0;
            return `ModTag::StatThreshold { stat: ${escStr(tag.stat)}, threshold: ${thresh}_f64 }`;
        }
        case "SkillType":
            if (tag.skillType == null) return null;
            return `ModTag::SkillType(${tag.skillType})`;
        case "GlobalEffect": {
            if (!tag.effectName) return null;
            const et = tag.effectType ? `Some(${escStr(tag.effectType)})` : "None";
            return `ModTag::GlobalEffect { effect_name: ${escStr(tag.effectName)}, effect_type: ${et} }`;
        }
        case "ModFlagOr":
            if (tag.modFlags == null) return null;
            return `ModTag::ModFlagOr(${tag.modFlags})`;
        case "DistanceRamp":
            // Assuming max/static distance — ramp not modelled, tag dropped
            return null;
        default:
            // Emit as comment — don't silently drop it
            return `/* TODO tag type "${tag.type}" */`;
    }
}

function emitStatDef(m: Mapping): string {
    // SkillData LIST entries: remap to dedicated SdX StatId with ModType::Base
    // so FireMin/FireMax/radius etc. each get a distinct StatId instead of all
    // colliding under StatId::SkillData.
    if (m.name === "SkillData" && m.type === "LIST" && typeof m.value === "object" && m.value !== null && "key" in m.value) {
        const sdVariant = SKILL_DATA_KEY_MAP[m.value.key];
        if (sdVariant) {
            return `def(StatId::${sdVariant}, ModType::Base)`;
        }
        // Unknown key — fall through to generic SkillData/List
    }
    const stat = emitStatId(m.name);
    const modType = emitModType(m.type);
    const hasFlags = m.flags != null && m.flags !== 0;
    const hasKw = m.keywordFlags != null && m.keywordFlags !== 0;
    const hasDev = m.div != null && m.div !== 1.0;

    const tagParts = (m.tags ?? []).map(emitTag).filter((t): t is string => t !== null);
    const hasTags = tagParts.length > 0;

    const flags = hasFlags ? `ModFlag::from_bits_truncate(${m.flags})` : null;
    const kw = hasKw ? `KeywordFlag::from_bits_truncate(${m.keywordFlags})` : null;
    const div = hasDev ? `${m.div}_f64` : null;
    const tagsStr = hasTags ? `smallvec![${tagParts.join(", ")}]` : null;

    if (!hasFlags && !hasKw && !hasDev && !hasTags)
        return `def(${stat}, ${modType})`;
    if (hasFlags && !hasKw && !hasDev && !hasTags)
        return `flagged_def(${stat}, ${modType}, ${flags})`;
    if (!hasFlags && hasKw && !hasDev && !hasTags)
        return `kw_def(${stat}, ${modType}, ${kw})`;
    if (hasFlags && hasKw && !hasDev && !hasTags)
        return `flagged_kw_def(${stat}, ${modType}, ${flags}, ${kw})`;
    if (!hasFlags && !hasKw && hasDev && !hasTags)
        return `div_def(${stat}, ${modType}, ${div})`;
    if (!hasFlags && hasKw && hasDev && !hasTags)
        return `kw_div_def(${stat}, ${modType}, ${kw}, ${div})`;
    if (!hasFlags && !hasKw && !hasDev && hasTags)
        return `tagged_def(${stat}, ${modType}, ${tagsStr})`;
    if (hasFlags && !hasKw && !hasDev && hasTags)
        return `flagged_tagged_def(${stat}, ${modType}, ${flags}, ${tagsStr})`;
    if (!hasFlags && hasKw && !hasDev && hasTags)
        return `kw_tagged_def(${stat}, ${modType}, ${kw}, ${tagsStr})`;
    // Fallback to full struct for unusual flag+kw+div combinations
    return `StatDef { stat: ${stat}, mod_type: ${modType}, flags: ${flags ?? "ModFlag::empty()"}, keywords: ${kw ?? "KeywordFlag::empty()"}, div: ${div ?? "1.0"}, tags: ${tagsStr ?? "smallvec![]"} }`;
}

// ── Main ─────────────────────────────────────────────────────────────────────

function main() {
    if (!existsSync(SSM_PATH)) {
        console.error(`SSM not found at ${SSM_PATH}`);
        console.error("Run 'bun run tool:fetch-data' first.");
        process.exit(1);
    }

    const ssm: SSM = JSON.parse(readFileSync(SSM_PATH, "utf-8"));
    const entries = Object.entries(ssm);

    // If the output file already exists, preserve the manual additions section.
    let manualSection = "";
    if (existsSync(OUT_PATH)) {
        const existing = readFileSync(OUT_PATH, "utf-8");
        const idx = existing.indexOf(MANUAL_MARKER);
        if (idx !== -1) {
            // Grab the manual section body only — stop at the utility section or
            // the first closing `}` that ends build_manual.
            const afterMarker = existing.slice(idx);
            // Stop at "\n}\n\n// ── Utility" which is the build_manual closing brace
            // followed by the utility comment, so we don't accidentally capture apply().
            const utilityBoundary = "\n}\n\n// ── Utility";
            const boundaryIdx = afterMarker.indexOf(utilityBoundary);
            if (boundaryIdx !== -1) {
                manualSection = afterMarker.slice(0, boundaryIdx);
            } else {
                // Fallback: first \n}\n (build_manual close)
                const endIdx = afterMarker.indexOf("\n}\n");
                if (endIdx !== -1) {
                    manualSection = afterMarker.slice(0, endIdx);
                }
            }
        }
    }

    const lines: string[] = [];

    lines.push(
        "// AUTO-GENERATED from SkillStatMap.norm.json — manual additions go in the section below.",
    );
    lines.push("// Re-run `bun run tool:gen-ssm` to refresh (manual additions are preserved).");
    lines.push("//");
    lines.push(
        "// Covers all 707 SkillStatMap entries: tree passives, gem stats, item mod IDs.",
    );
    lines.push(
        "// For stat IDs not in SkillStatMap, add entries in the manual section at the bottom.",
    );
    lines.push("");
    lines.push("use rustc_hash::FxHashMap;");
    lines.push("use smallvec::{smallvec, SmallVec};");
    lines.push("use std::sync::OnceLock;");
    lines.push("");
    lines.push("use crate::data::{SourceId, StatId};");
    lines.push("use super::types::{KeywordFlag, ModFlag, ModTag, ModType, Modifier};");
    lines.push("");
    lines.push(
        "// ── StatDef ─────────────────────────────────────────────────────────────────",
    );
    lines.push(
        "/// One resolved modifier definition from the stat table.",
    );
    lines.push(
        "/// Combined with a runtime `value` and `source` to produce a `Modifier`.",
    );
    lines.push("///");
    lines.push(
        "/// `tags` hold condition/multiplier context for Phase 5 (stored now, evaluated later).",
    );
    lines.push("#[derive(Debug, Clone)]");
    lines.push("pub struct StatDef {");
    lines.push("    pub stat: StatId,");
    lines.push("    pub mod_type: ModType,");
    lines.push("    pub flags: ModFlag,");
    lines.push("    pub keywords: KeywordFlag,");
    lines.push(
        "    /// Divide the raw stat value by this before creating the Modifier (usually 1.0).",
    );
    lines.push("    pub div: f64,");
    lines.push(
        "    /// Condition/multiplier tags — checked in Phase 5, stored unconditionally now.",
    );
    lines.push("    pub tags: SmallVec<[ModTag; 2]>,");
    lines.push("}");
    lines.push("");
    lines.push(
        "// ── Static table ─────────────────────────────────────────────────────────────",
    );
    lines.push(
        "static STAT_TABLE: OnceLock<FxHashMap<&'static str, SmallVec<[StatDef; 1]>>> =",
    );
    lines.push("    OnceLock::new();");
    lines.push("");
    lines.push(
        "/// Return the global stat ID → StatDef table, building it on first call.",
    );
    lines.push(
        "pub fn stat_table() -> &'static FxHashMap<&'static str, SmallVec<[StatDef; 1]>> {",
    );
    lines.push("    STAT_TABLE.get_or_init(build)");
    lines.push("}");
    lines.push("");
    lines.push("// ── Helpers ──────────────────────────────────────────────────────────────────");
    lines.push("fn def(stat: StatId, mod_type: ModType) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div: 1.0, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn flagged_def(stat: StatId, mod_type: ModType, flags: ModFlag) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags, keywords: KeywordFlag::empty(), div: 1.0, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn kw_def(stat: StatId, mod_type: ModType, keywords: KeywordFlag) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords, div: 1.0, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn flagged_kw_def(stat: StatId, mod_type: ModType, flags: ModFlag, keywords: KeywordFlag) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags, keywords, div: 1.0, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn div_def(stat: StatId, mod_type: ModType, div: f64) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn kw_div_def(stat: StatId, mod_type: ModType, keywords: KeywordFlag, div: f64) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords, div, tags: smallvec![] }");
    lines.push("}");
    lines.push("fn tagged_def(stat: StatId, mod_type: ModType, tags: SmallVec<[ModTag; 2]>) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div: 1.0, tags }");
    lines.push("}");
    lines.push("fn flagged_tagged_def(stat: StatId, mod_type: ModType, flags: ModFlag, tags: SmallVec<[ModTag; 2]>) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags, keywords: KeywordFlag::empty(), div: 1.0, tags }");
    lines.push("}");
    lines.push("fn kw_tagged_def(stat: StatId, mod_type: ModType, keywords: KeywordFlag, tags: SmallVec<[ModTag; 2]>) -> StatDef {");
    lines.push("    StatDef { stat, mod_type, flags: ModFlag::empty(), keywords, div: 1.0, tags }");
    lines.push("}");
    lines.push("");
    lines.push("fn build() -> FxHashMap<&'static str, SmallVec<[StatDef; 1]>> {");
    lines.push(`    let mut m: FxHashMap<&'static str, SmallVec<[StatDef; 1]>> =`);
    lines.push(`        FxHashMap::with_capacity_and_hasher(${entries.length + 700}, Default::default());`);
    lines.push("    build_ssm(&mut m);");
    lines.push("    build_gem_stats(&mut m);");
    lines.push("    build_manual(&mut m);");
    lines.push("    m");
    lines.push("}");
    lines.push("");
    lines.push("fn build_ssm(m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {");
    lines.push(

        "    // ── Generated entries (SkillStatMap) ─────────────────────────────────────",
    );

    let generated = 0;
    let skipped = 0;

    for (const [statKey, mappings] of entries) {
        if (!mappings || mappings.length === 0) {
            skipped++;
            continue;
        }

        try {
            const defs = mappings.map(emitStatDef);
            if (defs.length === 1) {
                lines.push(`    m.insert("${statKey}", smallvec![${defs[0]}]);`);
            } else {
                lines.push(`    m.insert("${statKey}", smallvec![`);
                for (const def of defs) {
                    lines.push(`        ${def},`);
                }
                lines.push(`    ]);`);
            }
            generated++;
        } catch (e) {
            lines.push(
                `    // SKIPPED "${statKey}": ${(e as Error).message}`,
            );
            skipped++;
        }
    }

    lines.push("");
    lines.push("}");
    lines.push("");
    lines.push("fn build_gem_stats(_m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {");
    lines.push("    // Auto-generated by gen_gem_stat_table.ts — do not edit here.");
    lines.push("    // Re-run `bun run tool:gen-gem-stats` to refresh.");
    lines.push("}");
    lines.push("");
    lines.push("fn build_manual(m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {");
    if (manualSection) {
        lines.push(manualSection);
    } else {
        lines.push(MANUAL_MARKER);
        lines.push(
            "    // Add entries here for stat IDs not in SkillStatMap.json.",
        );
        lines.push(
            "    // Format: m.insert(\"stat_id\", smallvec![StatDef { stat: StatId::X, mod_type: ModType::Y,",
        );
        lines.push(
            "    //     flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div: 1.0, tags: smallvec![] }]);",
        );
        lines.push("    //");
        lines.push("    // Compound stats that expand to multiple modifiers:");
        lines.push(
            "    // m.insert(\"base_strength_and_dexterity\", smallvec![",
        );
        lines.push(
            "    //     StatDef { stat: StatId::Strength, mod_type: ModType::Base, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div: 1.0, tags: smallvec![] },",
        );
        lines.push(
            "    //     StatDef { stat: StatId::Dexterity, mod_type: ModType::Base, flags: ModFlag::empty(), keywords: KeywordFlag::empty(), div: 1.0, tags: smallvec![] },",
        );
        lines.push("    // ]);");
    }

    lines.push("");
    lines.push("}");
    lines.push("");
    lines.push("// ── Utility ──────────────────────────────────────────────────────────────────");
    lines.push("");
    lines.push("/// Build a `Modifier` from a `StatDef` + runtime value + source.");
    lines.push(
        "pub fn apply(def: &StatDef, value: f64, source: SourceId) -> Modifier {",
    );
    lines.push("    Modifier {");
    lines.push("        stat: def.stat,");
    lines.push("        mod_type: def.mod_type,");
    lines.push("        value: value / def.div,");
    lines.push("        flags: def.flags,");
    lines.push("        keywords: def.keywords,");
    lines.push("        source,");
    lines.push("        tags: def.tags.clone(),");
    lines.push("    }");
    lines.push("}");
    lines.push("");

    writeFileSync(OUT_PATH, lines.join("\n"), "utf-8");
    console.log(
        `Generated ${OUT_PATH}`,
    );
    console.log(`  Entries: ${generated} generated, ${skipped} skipped`);
}

main();
