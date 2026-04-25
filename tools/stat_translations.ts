/**
 * stat_translations.ts — Combined stat translations loader.
 *
 * Reads all 22 PoB StatDescriptions/*.json files plus RePoE stat_translations.json
 * and produces a unified Map<string, string> of stat_id → display_text_template.
 *
 * Selects the "positive" variant for each stat ID (e.g. "more" not "less",
 * "increased" not "reduced").
 *
 * Usage:
 *   import { loadAllStatTranslations } from "./stat_translations";
 *   const map = loadAllStatTranslations();
 *   map.get("support_melee_physical_damage_+%_final")
 *     // → "Supported Skills deal {0}% more Melee Physical Damage"
 */

import { readFileSync, readdirSync } from "fs";
import { resolve, join } from "path";

const REPO_ROOT = resolve(import.meta.dir, "..");
const POB_DESC_DIR = join(REPO_ROOT, "src-tauri/data/pob/StatDescriptions");
const REPOE_TRANS_PATH = join(
    REPO_ROOT,
    "src-tauri/data/repoe/stat_translations.json"
);

// ── PoB StatDescriptions format ──────────────────────────────────────────────
//
// Each file is a JSON object with two sections:
//   1. Flat index:  { "stat_id": numeric_key, ... }
//   2. Entries:     { "numeric_key": { stats: [...], "1": [variants...] } }
//
// A variant has:
//   { limit: [[min, max], ...], text: "...", "1"?: { k: "negate", v: 1 } }
//
// We pick the first variant whose limit implies a positive value.

interface PobVariant {
    limit: (number | string)[][];
    text: string;
    [key: string]: unknown; // value transform handlers like "1": { k: "negate", v: 1 }
}

interface PobEntry {
    stats: string[];
    [key: string]: unknown; // numbered keys like "1", "2" holding variant arrays
}

function loadPobStatDescriptions(): Map<string, string> {
    const result = new Map<string, string>();

    const files = readdirSync(POB_DESC_DIR).filter((f) => f.endsWith(".json"));
    for (const file of files) {
        const raw = JSON.parse(
            readFileSync(join(POB_DESC_DIR, file), "utf-8")
        ) as Record<string, unknown>;

        // Process numbered entries (the actual translation data)
        for (const [key, val] of Object.entries(raw)) {
            // Skip the flat index entries (stat_id → number mappings)
            if (typeof val !== "object" || val === null) continue;
            const entry = val as PobEntry;
            if (!entry.stats || !Array.isArray(entry.stats)) continue;

            // Find variant arrays — they're in numbered string keys like "1", "2"
            // The key "1" contains variants for the first stat in stats[]
            // For single-stat entries there's one variant array at key "1"
            const variantKey = "1";
            const variants = entry[variantKey];
            if (!Array.isArray(variants)) continue;

            // Pick the positive-value variant (first variant where limit implies positive)
            let bestText: string | null = null;
            for (const variant of variants as PobVariant[]) {
                if (!variant.text) continue;

                // Check if this variant has a negate handler (indicates "reduced"/"less" form)
                let hasNegate = false;
                for (const handlerKey of Object.keys(variant)) {
                    if (handlerKey === "limit" || handlerKey === "text") continue;
                    const handler = variant[handlerKey];
                    if (
                        typeof handler === "object" &&
                        handler !== null &&
                        (handler as Record<string, unknown>).k === "negate"
                    ) {
                        hasNegate = true;
                        break;
                    }
                }

                // Prefer non-negated variant (the "more"/"increased" form)
                if (!hasNegate) {
                    bestText = variant.text;
                    break;
                }
                // Fall back to negated if it's the only one
                if (!bestText) bestText = variant.text;
            }

            if (bestText) {
                // Map each stat_id in this entry to the display text
                for (const statId of entry.stats) {
                    if (!result.has(statId)) {
                        result.set(statId, bestText);
                    }
                }
            }
        }
    }

    return result;
}

// ── RePoE stat_translations.json format ──────────────────────────────────────
//
// Array of entries:
//   { ids: [stat_id, ...], English: [{ condition, string, index_handlers }] }
//
// condition: [{ min, max, negated }]
// We pick the entry where min >= 1 (positive values).

interface RePoECondition {
    min: number | null;
    max: number | null;
    negated: boolean | null;
}

interface RePoETranslationVariant {
    condition: RePoECondition[];
    string: string;
    format: string[];
    index_handlers: string[][];
    reminder_text?: string | null;
    is_markup?: boolean | null;
}

interface RePoETranslationEntry {
    ids: string[];
    English: RePoETranslationVariant[] | null;
    hidden?: boolean | null;
    [key: string]: unknown;
}

function loadRePoEStatTranslations(): Map<string, string> {
    const result = new Map<string, string>();

    const raw = JSON.parse(
        readFileSync(REPOE_TRANS_PATH, "utf-8")
    ) as RePoETranslationEntry[];

    for (const entry of raw) {
        if (!entry.English || entry.English.length === 0) continue;
        if (entry.hidden) continue;

        // Pick the positive variant
        let bestText: string | null = null;
        for (const variant of entry.English) {
            if (!variant.string) continue;

            // Check if this is the positive variant (min >= 1 on first condition)
            const isPositive =
                variant.condition.length > 0 &&
                variant.condition[0].min !== null &&
                variant.condition[0].min >= 1;

            // Check for negate handler
            const hasNegate =
                variant.index_handlers.length > 0 &&
                variant.index_handlers.some((h) => h.includes("negate"));

            if (isPositive && !hasNegate) {
                bestText = variant.string;
                break;
            }
            if (!bestText) bestText = variant.string;
        }

        if (bestText) {
            for (const statId of entry.ids) {
                if (!result.has(statId)) {
                    result.set(statId, bestText);
                }
            }
        }
    }

    return result;
}

// ── Public API ───────────────────────────────────────────────────────────────

/**
 * Load all stat translations from both PoB StatDescriptions and RePoE,
 * returning a Map<stat_id, display_text_template>.
 *
 * PoB takes priority for gem-specific stats (better coverage).
 * RePoE fills in anything PoB doesn't have.
 */
export function loadAllStatTranslations(): Map<string, string> {
    const pobMap = loadPobStatDescriptions();
    const repoeMap = loadRePoEStatTranslations();

    // PoB first, RePoE as fallback
    const combined = new Map(pobMap);
    for (const [id, text] of repoeMap) {
        if (!combined.has(id)) {
            combined.set(id, text);
        }
    }

    return combined;
}

// ── CLI: run standalone for diagnostics ──────────────────────────────────────
if (import.meta.main) {
    const map = loadAllStatTranslations();
    console.log(`Total stat translations loaded: ${map.size}`);

    // Show some gem stat samples
    const samples = [
        "support_melee_physical_damage_+%_final",
        "ice_nova_area_of_effect_+%",
        "fire_damage_+%",
        "attack_speed_+%",
        "spell_minimum_base_fire_damage",
        "base_chance_to_ignite_%",
        "support_melee_physical_damage_attack_speed_+%_final",
    ];
    console.log("\nSample lookups:");
    for (const id of samples) {
        const text = map.get(id);
        console.log(`  ${id} → ${text ?? "(NOT FOUND)"}`);
    }
}
