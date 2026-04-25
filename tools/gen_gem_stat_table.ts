/**
 * gen_gem_stat_table.ts — Generate gem stat entries for stat_table.rs
 *
 * Usage:  bun run tool:gen-gem-stats        (generate)
 *         bun run tool:gen-gem-stats --audit (dry-run: report coverage only)
 *
 * Reads gems.json, looks up display text from PoB StatDescriptions + RePoE
 * stat_translations, parses display text to determine (StatId, ModType, flags),
 * and writes a GEM STATS section into stat_table.rs.
 *
 * Stat IDs already covered by SkillStatMap (the main gen_ssm.ts output) are
 * skipped. Manual overrides from gem_stat_overrides.json are applied as a
 * final fallback for stat IDs without display text.
 */

import { readFileSync, writeFileSync, existsSync } from "fs";
import { resolve, join } from "path";
import { loadAllStatTranslations } from "./stat_translations";

const REPO_ROOT = resolve(import.meta.dir, "..");
const GEMS_PATH = join(REPO_ROOT, "src-tauri/data/repoe/gems.json");
const SSM_PATH = join(REPO_ROOT, "src-tauri/data/pob/SkillStatMap.norm.json");
const OVERRIDES_PATH = join(REPO_ROOT, "tools/gem_stat_overrides.json");
const STAT_TABLE_PATH = join(
    REPO_ROOT,
    "src-tauri/src/modifier/stat_table.rs"
);

const GEM_BEGIN = "    // ── BEGIN GEM STATS ──────────────────────────────────────────────────";
const GEM_END = "    // ── END GEM STATS ────────────────────────────────────────────────────";

// ── Types ────────────────────────────────────────────────────────────────────

interface Override {
    stat: string;
    type: string;
    flags?: string[];
    keywords?: string[];
    div?: number;
    skip?: boolean; // true = intentionally skip (display-only / internal)
}

interface ParsedStat {
    statId: string;
    modType: string;
    flags: string[];
    keywords: string[];
    div: number;
    source: "text" | "override" | "suffix";
}

// ── Noun phrase → StatId mapping ─────────────────────────────────────────────

const NOUN_PHRASE_MAP: Record<string, { stat: string; flags?: string[]; keywords?: string[] }> = {
    // Generic damage
    "Damage": { stat: "Damage" },
    "damage": { stat: "Damage" },
    "Base Damage": { stat: "Damage" },

    // Damage by element
    "Physical Damage": { stat: "PhysicalDamage", keywords: ["PHYSICAL"] },
    "Fire Damage": { stat: "FireDamage", keywords: ["FIRE"] },
    "Cold Damage": { stat: "ColdDamage", keywords: ["COLD"] },
    "Lightning Damage": { stat: "LightningDamage", keywords: ["LIGHTNING"] },
    "Chaos Damage": { stat: "ChaosDamage", keywords: ["CHAOS"] },
    "Elemental Damage": { stat: "ElementalDamage" },

    // Damage sub-types
    "Area Damage": { stat: "Damage", flags: ["AREA"] },
    "Projectile Damage": { stat: "Damage", flags: ["PROJECTILE"] },
    "Melee Damage": { stat: "Damage", flags: ["MELEE"] },
    "Melee Physical Damage": { stat: "PhysicalDamage", flags: ["MELEE", "ATTACK"], keywords: ["PHYSICAL"] },
    "Attack Damage": { stat: "Damage", flags: ["ATTACK"] },
    "Spell Damage": { stat: "Damage", flags: ["SPELL"] },
    "Damage over Time": { stat: "Damage", flags: ["DOT"] },
    "Burning Damage": { stat: "FireDamage", flags: ["DOT"], keywords: ["FIRE"] },

    // Damage with qualifiers
    "Damage with Hits": { stat: "Damage", flags: ["HIT"] },
    "Damage with Hits and Ailments": { stat: "Damage" },
    "Damage with Ailments": { stat: "Damage" },
    "Hit Damage": { stat: "Damage", flags: ["HIT"] },
    "Damage with Poison": { stat: "ChaosDamage", flags: ["DOT"], keywords: ["CHAOS"] },
    "Damage with Ignite": { stat: "FireDamage", flags: ["DOT"], keywords: ["FIRE"] },
    "Damage with Bleeding": { stat: "PhysicalDamage", flags: ["DOT"], keywords: ["PHYSICAL"] },
    "Splash Damage to surrounding targets": { stat: "Damage" },

    // Speed
    "Attack Speed": { stat: "Speed", flags: ["ATTACK"] },
    "Melee Attack Speed": { stat: "Speed", flags: ["MELEE", "ATTACK"] },
    "Cast Speed": { stat: "Speed", flags: ["SPELL"] },
    "Movement Speed": { stat: "MovementSpeed" },

    // Critical
    "Critical Strike Chance": { stat: "CritChance" },
    "Critical Strike Multiplier": { stat: "CritMultiplier" },

    // Defences
    "Maximum Life": { stat: "Life" },
    "maximum Life": { stat: "Life" },
    "Life": { stat: "Life" },
    "maximum Mana": { stat: "Mana" },
    "Mana": { stat: "Mana" },
    "Energy Shield": { stat: "EnergyShield" },
    "Armour": { stat: "Armour" },
    "Evasion Rating": { stat: "Evasion" },

    // Area / projectile / duration
    "Area of Effect": { stat: "AreaOfEffect" },
    "Melee Splash Area of Effect": { stat: "AreaOfEffect", flags: ["MELEE"] },
    "Skill Effect Duration": { stat: "Duration" },
    "Projectile Speed": { stat: "ProjectileSpeed" },

    // Misc
    "Effect of Curse": { stat: "CurseEffect" },
    "Curse Duration": { stat: "Duration" },
    "Cooldown Recovery Rate": { stat: "CooldownRecovery" },
    "Mana Cost of Skills": { stat: "ManaCost" },
    "Mana Cost": { stat: "ManaCost" },
    "Character Size": { stat: "CharacterSize" },
    "Stealth": { stat: "Stealth" },
    "Action Speed": { stat: "ActionSpeed" },
    "Life Regeneration rate": { stat: "LifeRegen" },
    "Mana Regeneration rate": { stat: "ManaRegen" },

    // Minion/totem/trap/mine
    "Melee Damage while Totem is Active": { stat: "Damage", flags: ["MELEE"] },
    "Totem Life": { stat: "TotemLife" },
};

// ── Prefix phrase → extra flags ──────────────────────────────────────────────

const PREFIX_FLAGS: Record<string, { flags?: string[]; keywords?: string[] }> = {
    "Supported Skills deal": {},
    "Supported Skills have": {},
    "Supported Trigger Skills deal": {},
    "Supported Attack Skills deal": { flags: ["ATTACK"] },
    "Supported Spell Skills deal": { flags: ["SPELL"] },
};

// ── Display text → ParsedStat ────────────────────────────────────────────────

function parseDisplayText(statId: string, text: string): ParsedStat | null {
    // Normalize placeholders
    const norm = text.replace(/\{[0-9](?::\+?d?)?\}/g, "#");

    // 1. Try "more/less" pattern
    {
        const m = norm.match(/(?:#%?\s+)?(?:deal\s+|have\s+)?#?%?\s*(more|less)\s+(.+?)$/i);
        if (m) {
            const nounPhrase = m[2].trim();
            const mapping = findNounPhrase(nounPhrase);
            if (mapping) {
                return {
                    statId: mapping.stat,
                    modType: "More",
                    flags: [...(mapping.flags || [])],
                    keywords: [...(mapping.keywords || [])],
                    div: 1.0,
                    source: "text",
                };
            }
            // "more Damage" variants with extra qualifiers — just map to Damage
            if (nounPhrase.toLowerCase().includes("damage")) {
                return {
                    statId: "Damage",
                    modType: "More",
                    flags: extractFlagsFromText(nounPhrase),
                    keywords: extractKeywordsFromText(nounPhrase),
                    div: 1.0,
                    source: "text",
                };
            }
        }
    }

    // 2. Try "increased/reduced" pattern
    {
        const m = norm.match(/#%?\s*(increased|reduced)\s+(.+?)$/i);
        if (m) {
            const nounPhrase = m[2].trim();
            // Strip trailing qualifiers like "per stage", "up to X%", etc.
            const cleanPhrase = nounPhrase.replace(/\s*(?:per\s+.+|up\s+to\s+.+|while\s+.+|based\s+on\s+.+|,\s+.+|for\s+each\s+.+)$/i, "").trim();
            const mapping = findNounPhrase(cleanPhrase) || findNounPhrase(nounPhrase);
            if (mapping) {
                return {
                    statId: mapping.stat,
                    modType: "Inc",
                    flags: [...(mapping.flags || [])],
                    keywords: [...(mapping.keywords || [])],
                    div: 1.0,
                    source: "text",
                };
            }
            // Generic fallback for "increased Damage" variants
            if (cleanPhrase.toLowerCase().includes("damage")) {
                return {
                    statId: "Damage",
                    modType: "Inc",
                    flags: extractFlagsFromText(cleanPhrase),
                    keywords: extractKeywordsFromText(cleanPhrase),
                    div: 1.0,
                    source: "text",
                };
            }
        }
    }

    // 3. Try "Adds X to Y [Element] Damage" / "X to Y Added [Element] Damage"
    {
        const m = norm.match(/(?:Adds\s+)?#\s+to\s+#\s+(?:Added\s+)?(\w+(?:\s+\w+)?)\s+Damage/i);
        if (m) {
            const element = m[1].trim();
            const { stat, keywords } = elementToDamage(element);
            // This maps to both min and max — we can't determine which from display text alone.
            // Skip and let stat_id suffix heuristic handle min/max split
            return null;
        }
    }

    // 4. Try "Deals X to Y [Element] Damage"
    {
        const m = norm.match(/Deals\s+#\s+to\s+#\s+(\w+(?:\s+\w+)?)\s+Damage/i);
        if (m) {
            return null; // min/max handled by suffix heuristic
        }
    }

    // 5. Try "+X to [Stat]" patterns
    {
        const m = norm.match(/\+?#\s+(?:to\s+|additional\s+)?(.+?)$/i);
        if (m) {
            const nounPhrase = m[1].trim();
            const mapping = findNounPhrase(nounPhrase);
            if (mapping) {
                return {
                    statId: mapping.stat,
                    modType: "Base",
                    flags: [...(mapping.flags || [])],
                    keywords: [...(mapping.keywords || [])],
                    div: 1.0,
                    source: "text",
                };
            }
        }
    }

    // 6. "chance" pattern: "#% chance to X"
    {
        const m = norm.match(/#%?\s+chance\s+to\s+(.+?)$/i);
        if (m) {
            const action = m[1].trim().toLowerCase();
            if (action.includes("ignite")) return { statId: "IgniteChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("shock")) return { statId: "ShockChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("freeze")) return { statId: "FreezeChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("chill")) return { statId: "ChillChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("poison")) return { statId: "PoisonChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("bleed")) return { statId: "BleedChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("blind")) return { statId: "BlindChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("block")) return { statId: "BlockChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            if (action.includes("impale")) return { statId: "ImpaleChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "text" };
            // Generic chance — can't determine stat; skip for override
            return null;
        }
    }

    return null;
}

function findNounPhrase(phrase: string): { stat: string; flags?: string[]; keywords?: string[] } | null {
    // Direct match
    if (NOUN_PHRASE_MAP[phrase]) return NOUN_PHRASE_MAP[phrase];

    // Try without trailing noise
    const cleaned = phrase
        .replace(/\s*(?:per\s+.+|up\s+to\s+.+|while\s+.+|based\s+on\s+.+|,\s+.+|for\s+each\s+.+|to\s+(?:you|nearby|the).*)$/i, "")
        .trim();
    if (cleaned !== phrase && NOUN_PHRASE_MAP[cleaned]) return NOUN_PHRASE_MAP[cleaned];

    return null;
}

function elementToDamage(element: string): { stat: string; keywords: string[] } {
    const lower = element.toLowerCase();
    if (lower.includes("fire")) return { stat: "FireDamage", keywords: ["FIRE"] };
    if (lower.includes("cold")) return { stat: "ColdDamage", keywords: ["COLD"] };
    if (lower.includes("lightning")) return { stat: "LightningDamage", keywords: ["LIGHTNING"] };
    if (lower.includes("chaos")) return { stat: "ChaosDamage", keywords: ["CHAOS"] };
    if (lower.includes("physical")) return { stat: "PhysicalDamage", keywords: ["PHYSICAL"] };
    return { stat: "Damage", keywords: [] };
}

function extractFlagsFromText(text: string): string[] {
    const flags: string[] = [];
    const lower = text.toLowerCase();
    if (lower.includes("melee")) flags.push("MELEE");
    if (lower.includes("attack") && !lower.includes("attack speed")) flags.push("ATTACK");
    if (lower.includes("spell") && !lower.includes("spell damage")) flags.push("SPELL");
    if (lower.includes("projectile")) flags.push("PROJECTILE");
    if (lower.includes("area")) flags.push("AREA");
    if (lower.includes("hit")) flags.push("HIT");
    if (lower.includes("over time") || lower.includes("ailment")) flags.push("DOT");
    if (lower.includes("trap")) flags.push("TRAP");
    if (lower.includes("mine")) flags.push("MINE");
    if (lower.includes("totem")) flags.push("TOTEM");
    return flags;
}

function extractKeywordsFromText(text: string): string[] {
    const kw: string[] = [];
    const lower = text.toLowerCase();
    if (lower.includes("physical")) kw.push("PHYSICAL");
    if (lower.includes("fire")) kw.push("FIRE");
    if (lower.includes("cold")) kw.push("COLD");
    if (lower.includes("lightning")) kw.push("LIGHTNING");
    if (lower.includes("chaos")) kw.push("CHAOS");
    return kw;
}

// ── Suffix-based heuristic ───────────────────────────────────────────────────
// Many gem stat IDs follow a naming convention. Use suffixes as fallback when
// display text parsing fails or is unavailable.

/** Map stat ID substring → (StatId, flags, keywords) */
const STAT_ID_FRAGMENTS: {
    pattern: RegExp;
    stat: string;
    flags?: string[];
    keywords?: string[];
}[] = [
        // Damage by element
        { pattern: /physical_damage/, stat: "PhysicalDamage", keywords: ["PHYSICAL"] },
        { pattern: /fire_damage/, stat: "FireDamage", keywords: ["FIRE"] },
        { pattern: /cold_damage/, stat: "ColdDamage", keywords: ["COLD"] },
        { pattern: /lightning_damage/, stat: "LightningDamage", keywords: ["LIGHTNING"] },
        { pattern: /chaos_damage/, stat: "ChaosDamage", keywords: ["CHAOS"] },
        { pattern: /elemental_damage/, stat: "ElementalDamage" },
        { pattern: /projectile_damage/, stat: "Damage", flags: ["PROJECTILE"] },
        { pattern: /area_damage/, stat: "Damage", flags: ["AREA"] },
        { pattern: /melee_damage/, stat: "Damage", flags: ["MELEE"] },
        { pattern: /spell_damage/, stat: "Damage", flags: ["SPELL"] },
        { pattern: /attack_damage/, stat: "Damage", flags: ["ATTACK"] },
        { pattern: /trap_(?:and_mine_)?damage/, stat: "Damage", flags: ["TRAP"] },
        { pattern: /mine_damage/, stat: "Damage", flags: ["MINE"] },
        { pattern: /totem_damage/, stat: "Damage", flags: ["TOTEM"] },
        { pattern: /burning_damage/, stat: "FireDamage", flags: ["DOT"], keywords: ["FIRE"] },
        { pattern: /poison_damage/, stat: "ChaosDamage", flags: ["DOT"], keywords: ["CHAOS"] },
        { pattern: /bleeding_damage/, stat: "PhysicalDamage", flags: ["DOT"], keywords: ["PHYSICAL"] },
        { pattern: /(?:^|_)damage(?:_|$)/, stat: "Damage" },

        // Speed
        { pattern: /attack_speed/, stat: "Speed", flags: ["ATTACK"] },
        { pattern: /cast_speed/, stat: "Speed", flags: ["SPELL"] },
        { pattern: /movement_speed|movement_velocity/, stat: "MovementSpeed" },
        { pattern: /mine_laying_speed|mine_throwing_speed/, stat: "MineLayingSpeed" },
        { pattern: /trap_throwing_speed/, stat: "Speed", flags: ["TRAP"] },
        { pattern: /totem_placement_speed/, stat: "TotemPlacementSpeed" },

        // Critical
        { pattern: /critical_strike_chance/, stat: "CritChance" },
        { pattern: /critical_strike_multiplier/, stat: "CritMultiplier" },

        // Area / projectile / duration
        { pattern: /area_of_effect/, stat: "AreaOfEffect" },
        { pattern: /projectile_speed/, stat: "ProjectileSpeed" },
        { pattern: /skill_effect_duration|buff_duration|buff_effect_duration/, stat: "Duration" },
        { pattern: /cooldown/, stat: "CooldownRecovery" },

        // Defence
        { pattern: /armour/, stat: "Armour" },
        { pattern: /evasion/, stat: "Evasion" },
        { pattern: /energy_shield/, stat: "EnergyShield" },
        { pattern: /(?:maximum_)?life(?:_|$)/, stat: "Life" },
        { pattern: /(?:maximum_)?mana(?:_|$)/, stat: "Mana" },

        // Misc
        { pattern: /accuracy/, stat: "Accuracy" },
        { pattern: /stun/, stat: "EnemyStunDuration" },
        { pattern: /knockback/, stat: "KnockbackDistance" },
        { pattern: /curse_effect/, stat: "CurseEffect" },
        { pattern: /aura_effect/, stat: "AuraEffect" },
        { pattern: /buff_effect/, stat: "BuffEffect" },
        { pattern: /block/, stat: "BlockChance" },
    ];

function parseSuffix(statId: string): ParsedStat | null {
    // 1. Min/max damage patterns:
    //    spell_minimum_base_fire_damage, attack_maximum_added_cold_damage, etc.
    {
        const m = statId.match(/^(?:(.+?)_)?(minimum|maximum)_(?:base_|added_)?(\w+?)_damage(?:_.*)?$/);
        if (m) {
            const prefix = m[1] || ""; // "spell" | "attack" | skill name | ""
            const minmax = m[2]; // "minimum" | "maximum"
            const element = m[3]; // "fire" | "cold" | "lightning" | "chaos" | "physical"
            const { stat, keywords } = elementToDamage(element);
            if (stat !== "Damage") {
                const statName = minmax === "minimum"
                    ? stat.replace("Damage", "Min")
                    : stat.replace("Damage", "Max");
                const flags: string[] = [];
                if (prefix.includes("spell")) flags.push("SPELL");
                if (prefix.includes("attack")) flags.push("ATTACK");
                return {
                    statId: statName,
                    modType: "Base",
                    flags,
                    keywords,
                    div: 1.0,
                    source: "suffix",
                };
            }
        }
    }

    // 2. _+%_final suffix → More modifier
    if (statId.endsWith("_+%_final")) {
        const core = statId.replace(/_\+%_final$/, "");
        const found = matchFragment(core);
        if (found) {
            return { ...found, modType: "More", source: "suffix" };
        }
        // Fallback: generic More Damage
        return {
            statId: "Damage",
            modType: "More",
            flags: extractFlagsFromStatId(core),
            keywords: extractKeywordsFromStatId(core),
            div: 1.0,
            source: "suffix",
        };
    }

    // 3. _+% suffix → Inc modifier
    if (statId.endsWith("_+%") && !statId.endsWith("_+%_final")) {
        const core = statId.replace(/_\+%$/, "");
        const found = matchFragment(core);
        if (found) {
            return { ...found, modType: "Inc", source: "suffix" };
        }
    }

    // 4. _+ suffix → Base modifier
    if (statId.endsWith("_+") && !statId.endsWith("_+%")) {
        const core = statId.replace(/_\+$/, "");
        const found = matchFragment(core);
        if (found) {
            return { ...found, modType: "Base", source: "suffix" };
        }
    }

    // 5. _% suffix → Base modifier (chance-type or percentage)
    if (statId.endsWith("_%") && !statId.endsWith("_+%")) {
        const core = statId.replace(/_%$/, "");
        if (core.includes("chance_to_ignite")) return { statId: "IgniteChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_shock")) return { statId: "ShockChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_freeze")) return { statId: "FreezeChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_poison")) return { statId: "PoisonChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_bleed")) return { statId: "BleedChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_blind")) return { statId: "BlindChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
        if (core.includes("chance_to_impale")) return { statId: "ImpaleChance", modType: "Base", flags: [], keywords: [], div: 1.0, source: "suffix" };
    }

    // 6. active_skill_base_area_of_effect_radius-like patterns
    {
        const m = statId.match(/^(?:active_skill_)?base_(?:area_of_effect|radius)/);
        if (m) {
            return {
                statId: "AreaOfEffect",
                modType: "Base",
                flags: [],
                keywords: [],
                div: 1.0,
                source: "suffix",
            };
        }
    }

    return null;
}

function matchFragment(core: string): ParsedStat | null {
    for (const frag of STAT_ID_FRAGMENTS) {
        if (frag.pattern.test(core)) {
            return {
                statId: frag.stat,
                modType: "Base", // caller overrides
                flags: [...(frag.flags || [])],
                keywords: [...(frag.keywords || [])],
                div: 1.0,
                source: "suffix",
            };
        }
    }
    return null;
}

function extractFlagsFromStatId(core: string): string[] {
    const flags: string[] = [];
    if (core.includes("melee")) flags.push("MELEE");
    if (core.includes("attack") && !core.includes("attack_speed")) flags.push("ATTACK");
    if (core.includes("spell") && !core.includes("spell_damage")) flags.push("SPELL");
    if (core.includes("projectile")) flags.push("PROJECTILE");
    if (core.includes("area")) flags.push("AREA");
    if (core.includes("trap")) flags.push("TRAP");
    if (core.includes("mine")) flags.push("MINE");
    if (core.includes("totem")) flags.push("TOTEM");
    return flags;
}

function extractKeywordsFromStatId(core: string): string[] {
    const kw: string[] = [];
    if (core.includes("physical")) kw.push("PHYSICAL");
    if (core.includes("fire") && !core.includes("fired")) kw.push("FIRE");
    if (core.includes("cold")) kw.push("COLD");
    if (core.includes("lightning")) kw.push("LIGHTNING");
    if (core.includes("chaos")) kw.push("CHAOS");
    return kw;
}

// ── Rust codegen ─────────────────────────────────────────────────────────────

function emitRustEntry(rawStatId: string, parsed: ParsedStat): string {
    const modType = `ModType::${parsed.modType}`;
    const stat = `StatId::${parsed.statId}`;
    const hasFlags = parsed.flags.length > 0;
    const hasKw = parsed.keywords.length > 0;
    const hasDev = parsed.div !== 1.0;

    const flagsExpr = hasFlags
        ? parsed.flags.map(f => `ModFlag::${f}`).join(" | ")
        : null;
    const kwExpr = hasKw
        ? parsed.keywords.map(k => `KeywordFlag::${k}`).join(" | ")
        : null;

    let defCall: string;
    if (!hasFlags && !hasKw && !hasDev)
        defCall = `def(${stat}, ${modType})`;
    else if (hasFlags && !hasKw && !hasDev)
        defCall = `flagged_def(${stat}, ${modType}, ${flagsExpr})`;
    else if (!hasFlags && hasKw && !hasDev)
        defCall = `kw_def(${stat}, ${modType}, ${kwExpr})`;
    else if (hasFlags && hasKw && !hasDev)
        defCall = `flagged_kw_def(${stat}, ${modType}, ${flagsExpr}, ${kwExpr})`;
    else if (!hasFlags && !hasKw && hasDev)
        defCall = `div_def(${stat}, ${modType}, ${parsed.div}_f64)`;
    else
        defCall = `StatDef { stat: ${stat}, mod_type: ${modType}, flags: ${flagsExpr ?? "ModFlag::empty()"}, keywords: ${kwExpr ?? "KeywordFlag::empty()"}, div: ${parsed.div}_f64, tags: smallvec![] }`;

    return `    m.insert("${rawStatId}", smallvec![${defCall}]);`;
}

// ── Main ─────────────────────────────────────────────────────────────────────

function main() {
    const auditOnly = process.argv.includes("--audit");

    // Load data
    const gems: Record<string, any> = JSON.parse(readFileSync(GEMS_PATH, "utf-8"));
    const ssm: Record<string, any> = JSON.parse(readFileSync(SSM_PATH, "utf-8"));
    const translations = loadAllStatTranslations();
    const overrides: Record<string, Override> = existsSync(OVERRIDES_PATH)
        ? JSON.parse(readFileSync(OVERRIDES_PATH, "utf-8"))
        : {};

    // Collect all unique gem stat IDs not already in SSM
    const gemStatIds = new Set<string>();
    for (const gem of Object.values(gems) as any[]) {
        for (const s of gem.static?.stats || []) {
            if (!s?.id) continue;
            if (s.type === "implicit") continue;
            if (!ssm[s.id]) gemStatIds.add(s.id);
        }
        for (const qs of gem.static?.quality_stats || []) {
            for (const sid of Object.keys(qs.stats || {})) {
                if (!ssm[sid]) gemStatIds.add(sid);
            }
        }
    }

    // Also check stat_conversions targets — some converted IDs might not be in SSM
    for (const gem of Object.values(gems) as any[]) {
        if (gem.active_skill?.stat_conversions) {
            for (const target of Object.values(gem.active_skill.stat_conversions) as string[]) {
                if (!ssm[target]) gemStatIds.add(target);
            }
        }
    }

    console.log(`Total unique gem stat IDs (non-SSM, non-implicit): ${gemStatIds.size}`);

    // Process each stat ID
    const resolved = new Map<string, ParsedStat>();
    const unresolved: string[] = [];
    const skipped: string[] = [];

    for (const statId of [...gemStatIds].sort()) {
        // Check overrides first
        const ov = overrides[statId];
        if (ov) {
            if (ov.skip) {
                skipped.push(statId);
                continue;
            }
            resolved.set(statId, {
                statId: ov.stat,
                modType: ov.type,
                flags: ov.flags || [],
                keywords: ov.keywords || [],
                div: ov.div || 1.0,
                source: "override",
            });
            continue;
        }

        // Try display text parsing
        const text = translations.get(statId);
        if (text) {
            const parsed = parseDisplayText(statId, text);
            if (parsed) {
                resolved.set(statId, parsed);
                continue;
            }
        }

        // Try suffix heuristic
        const suffixed = parseSuffix(statId);
        if (suffixed) {
            resolved.set(statId, suffixed);
            continue;
        }

        unresolved.push(statId);
    }

    // Report
    const fromText = [...resolved.values()].filter(p => p.source === "text").length;
    const fromOverride = [...resolved.values()].filter(p => p.source === "override").length;
    const fromSuffix = [...resolved.values()].filter(p => p.source === "suffix").length;

    console.log(`\nResolution summary:`);
    console.log(`  From display text:  ${fromText}`);
    console.log(`  From suffix rules:  ${fromSuffix}`);
    console.log(`  From overrides:     ${fromOverride}`);
    console.log(`  Skipped (override): ${skipped.length}`);
    console.log(`  UNRESOLVED:         ${unresolved.length}`);
    console.log(`  Total resolved:     ${resolved.size} / ${gemStatIds.size}`);

    if (unresolved.length > 0) {
        console.log(`\nUnresolved stat IDs (need overrides):`);
        for (const id of unresolved.slice(0, 50)) {
            const text = translations.get(id);
            console.log(`  ${id}${text ? ` → "${text}"` : " (no display text)"}`);
        }
        if (unresolved.length > 50) {
            console.log(`  ... and ${unresolved.length - 50} more`);
        }
    }

    if (auditOnly) {
        console.log(`\nAudit mode — no files written.`);
        return;
    }

    // Generate Rust entries
    const rustLines: string[] = [];
    rustLines.push(GEM_BEGIN);
    rustLines.push("    // Auto-generated from gems.json display text. Do not edit manually.");
    rustLines.push(`    // Re-run \`bun run tool:gen-gem-stats\` to refresh.`);
    rustLines.push(`    // Entries: ${resolved.size} resolved, ${skipped.length} skipped, ${unresolved.length} unresolved`);
    rustLines.push("");

    for (const [statId, parsed] of [...resolved.entries()].sort()) {
        rustLines.push(emitRustEntry(statId, parsed));
    }

    rustLines.push("");
    rustLines.push(GEM_END);

    // Write to stat_table.rs
    const existing = readFileSync(STAT_TABLE_PATH, "utf-8");

    let output: string;
    const beginIdx = existing.indexOf(GEM_BEGIN);
    const endIdx = existing.indexOf(GEM_END);

    if (beginIdx !== -1 && endIdx !== -1) {
        // Replace existing section
        const afterEnd = existing.indexOf("\n", endIdx);
        output =
            existing.slice(0, beginIdx) +
            rustLines.join("\n") +
            existing.slice(afterEnd);
    } else {
        // Insert before build_manual function
        const gemStatsMarker = "fn build_gem_stats(";
        const gemStatsIdx = existing.indexOf(gemStatsMarker);
        if (gemStatsIdx === -1) {
            console.error("Cannot find build_gem_stats function in stat_table.rs");
            process.exit(1);
        }
        // Find the end of the empty build_gem_stats() stub to replace it entirely
        const stubEnd = existing.indexOf("}\n\nfn build_manual", gemStatsIdx);
        if (stubEnd === -1) {
            console.error("Cannot find end of build_gem_stats stub in stat_table.rs");
            process.exit(1);
        }
        output =
            existing.slice(0, gemStatsIdx) +
            `fn build_gem_stats(m: &mut FxHashMap<&'static str, SmallVec<[StatDef; 1]>>) {\n` +
            rustLines.join("\n") +
            "\n}" +
            existing.slice(stubEnd + 1); // skip the old closing `}`
    }

    writeFileSync(STAT_TABLE_PATH, output, "utf-8");
    console.log(`\nWrote ${resolved.size} gem stat entries to stat_table.rs`);
}

main();
