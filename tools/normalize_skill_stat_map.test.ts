/**
 * normalize_skill_stat_map.test.ts
 *
 * Tests for the SkillStatMap normalizer. Verifies:
 *   1. Unit tests — each JSON shape is handled correctly
 *   2. Integration tests against the real SkillStatMap.json — nothing is lost
 *
 * Run: bun test tools/normalize_skill_stat_map.test.ts
 */

import { describe, test, expect } from "bun:test";
import { resolve } from "node:path";
import {
    normalizeSkillStatMap,
    type NormalizedEntry,
    type NormalizeResult,
} from "./normalize_skill_stat_map";

// ── Helpers ─────────────────────────────────────────────────────────

function normalize(input: Record<string, unknown>): NormalizeResult {
    return normalizeSkillStatMap(input);
}

function entries(result: NormalizeResult, key: string): NormalizedEntry[] {
    return result.normalized[key] ?? [];
}

// ═══════════════════════════════════════════════════════════════════
// Unit tests — synthetic inputs
// ═══════════════════════════════════════════════════════════════════

describe("Shape handling", () => {
    test("Shape 1: standard array entry", () => {
        const result = normalize({
            "attack_speed_+%": [
                {
                    name: "Speed",
                    type: "INC",
                    flags: 1,
                    keywordFlags: 0,
                },
            ],
        });

        expect(result.totalOut).toBe(1);
        const e = entries(result, "attack_speed_+%");
        expect(e).toHaveLength(1);
        expect(e[0].name).toBe("Speed");
        expect(e[0].type).toBe("INC");
        expect(e[0].flags).toBe(1);
        expect(e[0].keywordFlags).toBe(0);
        expect(e[0].tags).toEqual([]);
        expect(e[0].div).toBeUndefined();
        expect(e[0].value).toBeUndefined();
    });

    test("Shape 1: multi-entry array", () => {
        const result = normalize({
            some_stat: [
                { name: "A", type: "FLAG", flags: 0, keywordFlags: 0, value: true },
                { name: "B", type: "MAX", flags: 0, keywordFlags: 0 },
            ],
        });

        const e = entries(result, "some_stat");
        expect(e).toHaveLength(2);
        expect(e[0].name).toBe("A");
        expect(e[0].value).toBe(true);
        expect(e[1].name).toBe("B");
    });

    test("Shape 2: bare object with numbered keys", () => {
        const result = normalize({
            some_permyriad_stat: {
                div: 100,
                "1": {
                    name: "CritChance",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                },
            },
        });

        expect(result.totalOut).toBe(1);
        const e = entries(result, "some_permyriad_stat");
        expect(e).toHaveLength(1);
        expect(e[0].name).toBe("CritChance");
        expect(e[0].div).toBe(100);
    });

    test("Shape 2: bare object with multiple numbered keys", () => {
        const result = normalize({
            multi_stat: {
                div: 60,
                "1": { name: "A", type: "BASE", flags: 0, keywordFlags: 0 },
                "2": { name: "B", type: "INC", flags: 0, keywordFlags: 0 },
                "3": { name: "C", type: "MORE", flags: 0, keywordFlags: 0 },
            },
        });

        const e = entries(result, "multi_stat");
        expect(e).toHaveLength(3);
        // All three should inherit the outer div
        expect(e[0].div).toBe(60);
        expect(e[1].div).toBe(60);
        expect(e[2].div).toBe(60);
        expect(e[0].name).toBe("A");
        expect(e[1].name).toBe("B");
        expect(e[2].name).toBe("C");
    });

    test("Shape 2: inner entry div overrides outer div", () => {
        const result = normalize({
            mixed_div: {
                div: 100,
                "1": {
                    name: "X",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    div: 50,
                },
            },
        });

        const e = entries(result, "mixed_div");
        expect(e[0].div).toBe(50); // Entry-level wins
    });

    test("Shape 2: outer value injected", () => {
        const result = normalize({
            with_value: {
                value: true,
                "1": {
                    name: "SomeFlag",
                    type: "FLAG",
                    flags: 0,
                    keywordFlags: 0,
                },
            },
        });

        const e = entries(result, "with_value");
        expect(e[0].value).toBe(true);
    });

    test("Shape 3: empty array is dropped", () => {
        const result = normalize({
            empty_stat: [],
        });

        expect(result.totalOut).toBe(0);
        expect(result.droppedEmpty).toBe(1);
        expect(result.normalized).not.toHaveProperty("empty_stat");
    });

    test("Malformed: bare object without name/type is dropped", () => {
        const result = normalize({
            skill_can_fire_arrows: { skillFlag: "arrow" },
        });

        expect(result.totalOut).toBe(0);
        expect(result.droppedMalformed).toBe(1);
    });
});

describe("Tag extraction", () => {
    test("Condition tag is moved into tags array", () => {
        const result = normalize({
            some_stat: [
                {
                    name: "Accuracy",
                    type: "INC",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "Condition", var: "LowLife" },
                },
            ],
        });

        const e = entries(result, "some_stat");
        expect(e[0].tags).toHaveLength(1);
        expect(e[0].tags[0]).toEqual({ type: "Condition", var: "LowLife" });
    });

    test("Multiple tags extracted in order", () => {
        const result = normalize({
            dual_tag: [
                {
                    name: "X",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "Condition", var: "A" },
                    "2": { type: "Multiplier", var: "B" },
                },
            ],
        });

        const e = entries(result, "dual_tag");
        expect(e[0].tags).toHaveLength(2);
        expect(e[0].tags[0].type).toBe("Condition");
        expect(e[0].tags[0].var).toBe("A");
        expect(e[0].tags[1].type).toBe("Multiplier");
        expect(e[0].tags[1].var).toBe("B");
    });

    test("ActorCondition tag preserved with actor field", () => {
        const result = normalize({
            vs_stunned: [
                {
                    name: "Damage",
                    type: "MORE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "ActorCondition",
                        actor: "enemy",
                        var: "Stunned",
                    },
                },
            ],
        });

        const tag = entries(result, "vs_stunned")[0].tags[0];
        expect(tag.type).toBe("ActorCondition");
        expect(tag.actor).toBe("enemy");
        expect(tag.var).toBe("Stunned");
    });

    test("PerStat tag preserved with stat and div", () => {
        const result = normalize({
            per_es: [
                {
                    name: "CritChance",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "PerStat",
                        stat: "EnergyShieldOnWeapon 2",
                        div: 10,
                    },
                },
            ],
        });

        const tag = entries(result, "per_es")[0].tags[0];
        expect(tag.type).toBe("PerStat");
        expect(tag.stat).toBe("EnergyShieldOnWeapon 2");
        expect(tag.div).toBe(10);
    });

    test("MultiplierThreshold tag preserved", () => {
        const result = normalize({
            rage_stat: [
                {
                    name: "Speed",
                    type: "INC",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "MultiplierThreshold",
                        var: "Rage",
                        threshold: 20,
                    },
                },
            ],
        });

        const tag = entries(result, "rage_stat")[0].tags[0];
        expect(tag.type).toBe("MultiplierThreshold");
        expect(tag.var).toBe("Rage");
        expect(tag.threshold).toBe(20);
    });

    test("SkillType tag preserved", () => {
        const result = normalize({
            ballista: [
                {
                    name: "ActiveTotemLimit",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "SkillType", skillType: 125 },
                },
            ],
        });

        const tag = entries(result, "ballista")[0].tags[0];
        expect(tag.type).toBe("SkillType");
        expect(tag.skillType).toBe(125);
    });

    test("GlobalEffect tag preserved", () => {
        const result = normalize({
            blind_eff: [
                {
                    name: "BlindEffect",
                    type: "INC",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "GlobalEffect",
                        effectType: "Debuff",
                        effectName: "Vaal Blade Flurry",
                    },
                },
            ],
        });

        const tag = entries(result, "blind_eff")[0].tags[0];
        expect(tag.type).toBe("GlobalEffect");
        expect(tag.effectType).toBe("Debuff");
        expect(tag.effectName).toBe("Vaal Blade Flurry");
    });

    test("DistanceRamp tag preserved with ramp array", () => {
        const result = normalize({
            charge_dmg: [
                {
                    name: "Damage",
                    type: "MORE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "DistanceRamp",
                        ramp: [
                            [0, 0],
                            [60, 1],
                        ],
                    },
                },
            ],
        });

        const tag = entries(result, "charge_dmg")[0].tags[0];
        expect(tag.type).toBe("DistanceRamp");
        expect(tag.ramp).toEqual([
            [0, 0],
            [60, 1],
        ]);
    });

    test("varList tag preserved", () => {
        const result = normalize({
            multi_cond: [
                {
                    name: "X",
                    type: "FLAG",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "Condition",
                        varList: ["Elusive", "ElusiveElusive"],
                    },
                },
            ],
        });

        const tag = entries(result, "multi_cond")[0].tags[0];
        expect(tag.varList).toEqual(["Elusive", "ElusiveElusive"]);
    });

    test("neg and limit fields preserved on tags", () => {
        const result = normalize({
            neg_stat: [
                {
                    name: "X",
                    type: "INC",
                    flags: 0,
                    keywordFlags: 0,
                    "1": {
                        type: "Multiplier",
                        var: "FrenzyCharge",
                        limit: 10,
                        neg: true,
                    },
                },
            ],
        });

        const tag = entries(result, "neg_stat")[0].tags[0];
        expect(tag.neg).toBe(true);
        expect(tag.limit).toBe(10);
    });

    test("Tags on bare object inner entries are extracted", () => {
        const result = normalize({
            bare_with_tags: {
                div: 100,
                "1": {
                    name: "CritChance",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "Condition", var: "Elusive" },
                },
            },
        });

        const e = entries(result, "bare_with_tags");
        expect(e[0].div).toBe(100);
        expect(e[0].tags).toHaveLength(1);
        expect(e[0].tags[0].var).toBe("Elusive");
    });

    test("Lua integer booleans (1/0) are coerced to proper booleans on tags", () => {
        const result = normalize({
            lua_bool_stat: [
                {
                    name: "SomePercent",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "PercentStat", stat: "Mana", percent: 1 },
                },
            ],
            neg_int_stat: [
                {
                    name: "SomeNeg",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "Multiplier", var: "Power", neg: 1 },
                },
            ],
            zero_neg_stat: [
                {
                    name: "SomeZero",
                    type: "BASE",
                    flags: 0,
                    keywordFlags: 0,
                    "1": { type: "Multiplier", var: "Frenzy", neg: 0 },
                },
            ],
        });

        // percent: 1 → true
        const percentTag = entries(result, "lua_bool_stat")[0].tags[0];
        expect(percentTag.percent).toBe(true);
        expect(typeof percentTag.percent).toBe("boolean");

        // neg: 1 → true
        const negTag = entries(result, "neg_int_stat")[0].tags[0];
        expect(negTag.neg).toBe(true);
        expect(typeof negTag.neg).toBe("boolean");

        // neg: 0 → false
        const zeroTag = entries(result, "zero_neg_stat")[0].tags[0];
        expect(zeroTag.neg).toBe(false);
        expect(typeof zeroTag.neg).toBe("boolean");
    });
});

describe("Output shape consistency", () => {
    test("Every output entry has all required fields", () => {
        const result = normalize({
            a: [{ name: "X", type: "BASE", flags: 1, keywordFlags: 2 }],
            b: [
                {
                    name: "Y",
                    type: "FLAG",
                    flags: 0,
                    keywordFlags: 0,
                    value: true,
                },
            ],
            c: {
                div: 100,
                "1": { name: "Z", type: "INC", flags: 0, keywordFlags: 0 },
            },
        });

        for (const [_key, entryList] of Object.entries(result.normalized)) {
            expect(Array.isArray(entryList)).toBe(true);
            for (const entry of entryList) {
                expect(typeof entry.name).toBe("string");
                expect(typeof entry.type).toBe("string");
                expect(typeof entry.flags).toBe("number");
                expect(typeof entry.keywordFlags).toBe("number");
                expect(Array.isArray(entry.tags)).toBe(true);
            }
        }
    });

    test("flags default to 0 when missing in input", () => {
        const result = normalize({
            no_flags: [{ name: "X", type: "BASE" }],
        });

        const e = entries(result, "no_flags");
        expect(e[0].flags).toBe(0);
        expect(e[0].keywordFlags).toBe(0);
    });

    test("Output keys are sorted alphabetically", () => {
        const result = normalize({
            zzz: [{ name: "Z", type: "BASE", flags: 0, keywordFlags: 0 }],
            aaa: [{ name: "A", type: "BASE", flags: 0, keywordFlags: 0 }],
            mmm: [{ name: "M", type: "BASE", flags: 0, keywordFlags: 0 }],
        });

        const keys = Object.keys(result.normalized);
        expect(keys).toEqual(["aaa", "mmm", "zzz"]);
    });
});

// ═══════════════════════════════════════════════════════════════════
// Integration tests — real SkillStatMap.json
// ═══════════════════════════════════════════════════════════════════

describe("Integration: real SkillStatMap.json", () => {
    const INPUT = resolve("src-tauri/data/pob/SkillStatMap.json");

    let raw: Record<string, unknown>;
    let result: NormalizeResult;

    // Load once for all integration tests
    const inputJson = require("fs").readFileSync(INPUT, "utf-8");
    raw = JSON.parse(inputJson);
    result = normalizeSkillStatMap(raw);

    test("reads all 707 stat IDs from original", () => {
        expect(result.totalIn).toBe(707);
    });

    test("drops exactly 14 empty arrays", () => {
        expect(result.droppedEmpty).toBe(14);
    });

    test("drops exactly 1 malformed entry (skill_can_fire_arrows)", () => {
        expect(result.droppedMalformed).toBe(1);
        expect(result.normalized).not.toHaveProperty("skill_can_fire_arrows");
    });

    test("outputs exactly 692 stat IDs (707 - 14 empty - 1 malformed)", () => {
        expect(result.totalOut).toBe(692);
        expect(Object.keys(result.normalized)).toHaveLength(692);
    });

    test("every output entry is a non-empty array", () => {
        for (const [key, entryList] of Object.entries(result.normalized)) {
            expect(Array.isArray(entryList)).toBe(true);
            expect(entryList.length).toBeGreaterThan(0);
        }
    });

    test("every output entry has consistent shape", () => {
        for (const [_key, entryList] of Object.entries(result.normalized)) {
            for (const entry of entryList) {
                expect(typeof entry.name).toBe("string");
                expect(entry.name.length).toBeGreaterThan(0);
                expect(typeof entry.type).toBe("string");
                expect(entry.type.length).toBeGreaterThan(0);
                expect(typeof entry.flags).toBe("number");
                expect(typeof entry.keywordFlags).toBe("number");
                expect(Array.isArray(entry.tags)).toBe(true);
                // div is number or undefined
                if (entry.div !== undefined) {
                    expect(typeof entry.div).toBe("number");
                }
            }
        }
    });

    test("total mapping entries matches original (768 from non-empty sources)", () => {
        // Count all individual mapping entries in the original (excluding empty arrays
        // and the skillFlag object which has no name/type)
        let originalEntries = 0;
        for (const [_key, val] of Object.entries(raw)) {
            if (Array.isArray(val)) {
                originalEntries += val.length; // includes empty arrays (0 entries)
            } else if (typeof val === "object" && val !== null) {
                const obj = val as Record<string, unknown>;
                const numbered = ["1", "2", "3"].filter(
                    (k) =>
                        typeof obj[k] === "object" &&
                        obj[k] !== null &&
                        "name" in (obj[k] as Record<string, unknown>),
                );
                if (numbered.length > 0) {
                    originalEntries += numbered.length;
                } else if ("name" in obj) {
                    originalEntries += 1;
                }
                // skillFlag objects (no name) contribute 0
            }
        }

        let normalizedEntries = 0;
        for (const entryList of Object.values(result.normalized)) {
            normalizedEntries += entryList.length;
        }

        // 767 total entries from non-empty/non-malformed sources
        // (768 - 1 malformed skill_can_fire_arrows which has no name field)
        expect(originalEntries).toBe(767);
        expect(normalizedEntries).toBe(767);
    });

    test("every name from original is present in output", () => {
        // Collect all (statId, name) pairs from the original
        const originalPairs = new Set<string>();
        for (const [statId, val] of Object.entries(raw)) {
            if (Array.isArray(val)) {
                for (const entry of val) {
                    if (typeof entry === "object" && entry !== null && "name" in entry) {
                        originalPairs.add(`${statId}:${(entry as Record<string, unknown>).name}`);
                    }
                }
            } else if (typeof val === "object" && val !== null) {
                const obj = val as Record<string, unknown>;
                for (const k of ["1", "2", "3"]) {
                    const inner = obj[k];
                    if (typeof inner === "object" && inner !== null && "name" in (inner as Record<string, unknown>)) {
                        originalPairs.add(`${statId}:${(inner as Record<string, unknown>).name}`);
                    }
                }
            }
        }

        const normalizedPairs = new Set<string>();
        for (const [statId, entryList] of Object.entries(result.normalized)) {
            for (const entry of entryList) {
                normalizedPairs.add(`${statId}:${entry.name}`);
            }
        }

        // Every original pair must be in normalized
        for (const pair of originalPairs) {
            expect(normalizedPairs.has(pair)).toBe(true);
        }
        // And counts must match
        expect(normalizedPairs.size).toBe(originalPairs.size);
    });

    test("all tags are preserved (226 total from 181 entries)", () => {
        let totalTags = 0;
        let entriesWithTags = 0;
        for (const entryList of Object.values(result.normalized)) {
            for (const entry of entryList) {
                if (entry.tags.length > 0) {
                    entriesWithTags++;
                    totalTags += entry.tags.length;
                }
            }
        }
        expect(entriesWithTags).toBe(181);
        expect(totalTags).toBe(226);
    });

    test("every tag has a type field", () => {
        for (const entryList of Object.values(result.normalized)) {
            for (const entry of entryList) {
                for (const tag of entry.tags) {
                    expect(typeof tag.type).toBe("string");
                    expect(tag.type.length).toBeGreaterThan(0);
                }
            }
        }
    });

    test("all 11 tag types from original are represented", () => {
        const tagTypes = new Set<string>();
        for (const entryList of Object.values(result.normalized)) {
            for (const entry of entryList) {
                for (const tag of entry.tags) {
                    tagTypes.add(tag.type as string);
                }
            }
        }
        const expected = [
            "ActorCondition",
            "Condition",
            "DistanceRamp",
            "GlobalEffect",
            "ModFlagOr",
            "Multiplier",
            "MultiplierThreshold",
            "PerStat",
            "PercentStat",
            "SkillType",
            "StatThreshold",
        ];
        for (const t of expected) {
            expect(tagTypes.has(t)).toBe(true);
        }
    });

    test("div values are correctly propagated from bare objects", () => {
        // "additional_base_critical_strike_chance" is a bare object with div: 100
        const e = entries(result, "additional_base_critical_strike_chance");
        expect(e.length).toBeGreaterThan(0);
        expect(e[0].div).toBe(100);
    });

    test("all 9 mod types from original are represented", () => {
        const modTypes = new Set<string>();
        for (const entryList of Object.values(result.normalized)) {
            for (const entry of entryList) {
                modTypes.add(entry.type);
            }
        }
        const expected = [
            "BASE",
            "CHANCE",
            "FLAG",
            "INC",
            "LIST",
            "MAX",
            "MIN",
            "MORE",
            "OVERRIDE",
        ];
        for (const t of expected) {
            expect(modTypes.has(t)).toBe(true);
        }
    });

    test("specific known entries are correct", () => {
        // Simple case
        const speed = entries(result, "attack_speed_+%");
        expect(speed).toHaveLength(1);
        expect(speed[0].name).toBe("Speed");
        expect(speed[0].type).toBe("INC");
        expect(speed[0].flags).toBe(1); // ATTACK

        // Conditional case
        const lowLife = entries(result, "accuracy_rating_+%_when_on_low_life");
        expect(lowLife).toHaveLength(1);
        expect(lowLife[0].tags).toHaveLength(1);
        expect(lowLife[0].tags[0].type).toBe("Condition");
        expect(lowLife[0].tags[0].var).toBe("LowLife");

        // Multi-entry case (FLAG + MAX)
        const minionDmg = entries(
            result,
            "active_skill_additive_minion_damage_modifiers_apply_to_all_damage_at_%_value",
        );
        expect(minionDmg).toHaveLength(2);
        expect(minionDmg[0].type).toBe("FLAG");
        expect(minionDmg[0].value).toBe(true);
        expect(minionDmg[1].type).toBe("MAX");
    });

    test("bare object with 3 numbered entries is handled", () => {
        // This was identified as the only 3-numbered-key entry
        const e = entries(
            result,
            "minion_life_leech_from_elemental_damage_permyriad",
        );
        expect(e.length).toBe(3);
    });

    test("no numbered keys remain in output entries", () => {
        for (const [_key, entryList] of Object.entries(result.normalized)) {
            for (const entry of entryList) {
                // The entry itself should not have "1", "2", "3" keys
                expect(entry).not.toHaveProperty("1");
                expect(entry).not.toHaveProperty("2");
                expect(entry).not.toHaveProperty("3");
            }
        }
    });

    test("output is valid JSON that round-trips cleanly", () => {
        const json = JSON.stringify(result.normalized);
        const parsed = JSON.parse(json);
        expect(Object.keys(parsed)).toHaveLength(692);
    });
});
