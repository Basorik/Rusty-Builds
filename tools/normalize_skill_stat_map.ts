/**
 * normalize_skill_stat_map.ts
 *
 * Transforms PoB's SkillStatMap.json from its irregular format into a
 * normalized structure that Rust can deserialize with zero special-casing.
 *
 * Input:  src-tauri/data/pob/SkillStatMap.json       (PoB original)
 * Output: src-tauri/data/pob/SkillStatMap.norm.json   (normalized)
 *
 * Changes:
 *   1. Every entry becomes an array (bare objects with numbered keys → array)
 *   2. Empty arrays are dropped entirely (14 entries)
 *   3. Bare objects without name/type are dropped (1 entry: skill_can_fire_arrows)
 *   4. Condition/multiplier tags move from numbered keys ("1", "2", "3") into a
 *      proper "tags" array on each mapping entry — ALL tag fields preserved
 *   5. Outer-level "div"/"value" on bare objects are injected into each inner entry
 *   6. All entries have a consistent shape:
 *      { name, type, flags, keywordFlags, tags: [...], div?, value? }
 *
 * Usage:
 *   bun run tools/normalize_skill_stat_map.ts
 *   bun run tool:normalize-ssm
 */

import { resolve } from "node:path";

const INPUT = resolve("src-tauri/data/pob/SkillStatMap.json");
const OUTPUT = resolve("src-tauri/data/pob/SkillStatMap.norm.json");

// ── Types ───────────────────────────────────────────────────────────

/** Normalized tag — preserves ALL original fields verbatim. */
export type NormalizedTag = Record<string, unknown> & { type: string };

export interface NormalizedEntry {
    name: string;
    type: string;
    flags: number;
    keywordFlags: number;
    tags: NormalizedTag[];
    div?: number;
    value?: unknown;
}

// ── Helpers ─────────────────────────────────────────────────────────

/**
 * Fields in tags that Lua stores as 1/0 but should be proper JSON booleans.
 * PoB's Lua-to-JSON export writes `true` as `1` for some fields.
 */
const BOOLEAN_TAG_FIELDS = new Set(["neg", "percent"]);

/**
 * Extract numbered tags ("1", "2", "3") from an entry object.
 * Copies ALL fields from each tag object verbatim, except Lua-style
 * integer booleans (neg, percent) are coerced to proper booleans.
 */
function extractTags(entry: Record<string, unknown>): NormalizedTag[] {
    const tags: NormalizedTag[] = [];
    for (const key of ["1", "2", "3"]) {
        const raw = entry[key];
        if (raw && typeof raw === "object" && !Array.isArray(raw)) {
            // Clone the entire tag object — preserves every field
            const tag = { ...(raw as Record<string, unknown>) } as NormalizedTag;
            // Coerce Lua integer booleans → JSON booleans
            for (const field of BOOLEAN_TAG_FIELDS) {
                if (field in tag && typeof tag[field] === "number") {
                    tag[field] = tag[field] !== 0;
                }
            }
            tags.push(tag);
        }
    }
    return tags;
}

/**
 * Normalize a single mapping entry (must have "name" and "type" fields)
 * into the standard shape. Returns null if name/type are missing.
 */
function normalizeEntry(
    raw: Record<string, unknown>,
    outerDiv?: number,
    outerValue?: unknown,
): NormalizedEntry | null {
    const name = raw.name;
    const type = raw.type;
    if (typeof name !== "string" || typeof type !== "string") return null;

    const flags = typeof raw.flags === "number" ? raw.flags : 0;
    const keywordFlags =
        typeof raw.keywordFlags === "number" ? raw.keywordFlags : 0;

    const tags = extractTags(raw);

    const entry: NormalizedEntry = { name, type, flags, keywordFlags, tags };

    // div: entry-level takes precedence over outer-level
    const div = raw.div ?? outerDiv;
    if (div !== undefined && div !== null) entry.div = div as number;

    // value: entry-level takes precedence over outer-level
    const value = raw.value ?? outerValue;
    if (value !== undefined && value !== null) entry.value = value;

    return entry;
}

// ── Core normalize function (exported for tests) ────────────────────

export interface NormalizeResult {
    normalized: Record<string, NormalizedEntry[]>;
    totalIn: number;
    totalOut: number;
    droppedEmpty: number;
    droppedMalformed: number;
}

export function normalizeSkillStatMap(
    raw: Record<string, unknown>,
): NormalizeResult {
    const normalized: Record<string, NormalizedEntry[]> = {};
    let totalIn = 0;
    let totalOut = 0;
    let droppedEmpty = 0;
    let droppedMalformed = 0;

    for (const [statId, val] of Object.entries(raw)) {
        totalIn++;

        if (Array.isArray(val)) {
            // Shape 1: array of mapping entries  OR  Shape 3: empty array
            if (val.length === 0) {
                droppedEmpty++;
                continue;
            }

            const entries: NormalizedEntry[] = [];
            for (const item of val) {
                if (
                    typeof item === "object" &&
                    item !== null &&
                    !Array.isArray(item)
                ) {
                    const norm = normalizeEntry(
                        item as Record<string, unknown>,
                    );
                    if (norm) entries.push(norm);
                    else droppedMalformed++;
                }
            }

            if (entries.length > 0) {
                normalized[statId] = entries;
                totalOut++;
            }
        } else if (typeof val === "object" && val !== null) {
            // Shape 2: bare object — outer div/value + numbered inner entries
            const obj = val as Record<string, unknown>;
            const outerDiv =
                typeof obj.div === "number" ? obj.div : undefined;
            const outerValue = obj.value;

            // Check if numbered keys exist (inner entries with "name")
            const hasNumberedEntries = ["1", "2", "3"].some((k) => {
                const inner = obj[k];
                return (
                    typeof inner === "object" &&
                    inner !== null &&
                    !Array.isArray(inner) &&
                    "name" in (inner as Record<string, unknown>)
                );
            });

            if (hasNumberedEntries) {
                const entries: NormalizedEntry[] = [];
                for (const key of ["1", "2", "3"]) {
                    const inner = obj[key];
                    if (
                        typeof inner === "object" &&
                        inner !== null &&
                        !Array.isArray(inner)
                    ) {
                        const norm = normalizeEntry(
                            inner as Record<string, unknown>,
                            outerDiv,
                            outerValue,
                        );
                        if (norm) entries.push(norm);
                        else droppedMalformed++;
                    }
                }
                if (entries.length > 0) {
                    normalized[statId] = entries;
                    totalOut++;
                }
            } else {
                // No numbered inner entries — try the object itself as an entry
                const norm = normalizeEntry(obj);
                if (norm) {
                    normalized[statId] = [norm];
                    totalOut++;
                } else {
                    // Object has no name/type — not a stat mapping (e.g. skillFlag)
                    droppedMalformed++;
                }
            }
        }
    }

    // Sort keys for deterministic output
    const sorted: Record<string, NormalizedEntry[]> = {};
    for (const key of Object.keys(normalized).sort()) {
        sorted[key] = normalized[key];
    }

    return {
        normalized: sorted,
        totalIn,
        totalOut,
        droppedEmpty,
        droppedMalformed,
    };
}

// ── Main (only runs when executed directly, not when imported) ──────

const isDirectRun = import.meta.main;

if (isDirectRun) {
    const inputJson = await Bun.file(INPUT).text();
    const raw: Record<string, unknown> = JSON.parse(inputJson);

    const result = normalizeSkillStatMap(raw);

    await Bun.write(
        OUTPUT,
        JSON.stringify(result.normalized, null, 2) + "\n",
    );

    console.log("\n✅ Normalized SkillStatMap");
    console.log(`   Input:  ${result.totalIn} stat IDs from ${INPUT}`);
    console.log(`   Output: ${result.totalOut} stat IDs to ${OUTPUT}`);
    console.log(
        `   Dropped: ${result.droppedEmpty} empty, ${result.droppedMalformed} malformed`,
    );
    console.log(
        `   Every entry is now a uniform array with a "tags" array.\n`,
    );
}
