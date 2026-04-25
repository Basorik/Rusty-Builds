# Rusty Builds — Development Plan

**Goal**: Rebuild [Path of Building](https://github.com/PathOfBuildingCommunity/PathOfBuilding) as a Tauri desktop app using Rust and Svelte, targeting **Path of Exile 1** with full calculation parity.

> This document is the implementation roadmap. Each phase builds on the last and produces a testable milestone.

---

## Table of Contents

- [Design Decisions](#design-decisions)
- [Data Sources](#data-sources)
- [Completed Work (Phases 1–2)](#completed-work-phases-12)
- [Phase 3: Gem & Skill System (RePoE)](#phase-3-gem--skill-system-repoe)
- [Phase 4: Item System](#phase-4-item-system)
- [Phase 5: Full Calculation Engine](#phase-5-full-calculation-engine)
- [Phase 6: Build Management & Import/Export](#phase-6-build-management--importexport)
- [Phase 7: Configuration, Party, Jewels & Advanced Features](#phase-7-configuration-party-jewels--advanced-features)
- [Phase 8: Polish, Performance & Trade](#phase-8-polish-performance--trade)
- [Reference: PoB Source Map](#reference-pob-source-map)

---

## Design Decisions

These are locked in. Don't revisit them until Phase 8.

| Decision | Choice | Rationale |
|---|---|---|
| Game version | **PoE 1 only** | Stable, mature data; simpler scope |
| Calc engine | **Full parity with PoB** | Core value of the app |
| Data source (everything except uniques) | **RePoE** | Raw stat IDs, structured JSON, no text parsing needed |
| Data source (unique items) | **PoB text + stat_translations inversion** | RePoE `uniques.json` has no stat data |
| Tree model | **Two-tree** | GGG tree for frontend rendering (positions, connections, orbits); RePoE tree for calculations (raw stat IDs) |
| Stat resolution | **`parser::resolve()` → `stat_table.rs` O(1) lookup** | Single entry point for all stat ID → Modifier conversion |
| Stat table | **Codegen from SkillStatMap** | 931-line generated file; compile-time validation; zero runtime JSON loading |
| Stat identification | **`StatId` enum (u16)** | No string hashing in hot paths; compile-time exhaustiveness checks |
| Hash maps | **`FxHashMap`** via `rustc-hash` | Faster non-cryptographic hashing for small keys |
| Modifier queries | **`CalcContext` parameter** on all `ModDB` queries | Conditional mods ("while Leeching") need context without API changes later |
| Modifier layout | **Compact struct**: `SourceId(u32)`, `SmallVec<[ModTag; 2]>` | No heap Strings in the hot path |
| ModDB architecture | **Layered composition** (tree/class/gems/items/config) | Targeted cache invalidation per source |
| Display text parsing | **`parse_display_text()` for PoB unique items only** | All other sources provide raw stat IDs via RePoE |
| Frontend split | **Selection in PixiJS, calculations in Rust** | 60fps responsiveness for tree interactions |
| Build code format | **PoB-compatible** | Import/export share codes with the real Path of Building app |
| Internal errors | **Typed `thiserror` enums; `String` only at IPC boundary** | Pattern matching, actionable error handling |

**Excluded from scope**: PoE 2 support, mobile/responsive design, auto-updating game data from API.

---

## Data Sources

All game data comes from two sources. Understanding the split is critical.

### RePoE (`data/repoe/`)

Source: [repoe-fork/repoe](https://github.com/repoe-fork/repoe) — community-maintained structured export of PoE game data.

| File | What It Contains | Used By |
|---|---|---|
| `passive_skill_trees/Default.json` | Raw stat IDs per passive node (`stats: {stat_id: value}`) | `rebuild_tree()` → `resolve()` |
| `gems.json` | All gems: static stats with IDs, per-level values, stat_conversions, support matching, quality stats | Phase 3 gem system |
| `mods.json` | All item mods: stat IDs with min/max ranges, spawn weights, domains, groups | Phase 4 item mods |
| `base_items.json` | Base item types: defences, requirements, implicits, tags | Phase 4 base items |
| `stat_translations.json` | Stat ID → display text templates (`"{0}% increased maximum Life"`) | Phase 4: invert for unique text parsing |
| `stats.json` | Stat metadata: `is_local`, `is_aliased`, `alias` | Phase 4: local mod classification |
| `uniques.json` | **Metadata only**: name, item_class, visual_identity — **NO stats** | Not used for calculations |

### PoB (`data/pob/`)

Source: [repoe-fork/pob-data](https://repoe-fork.github.io/pob-data/poe1/) — PoB-formatted game data.

| File | What It Contains | Used By |
|---|---|---|
| `SkillStatMap.json` | Stat ID → calc variable name + modifier type mapping (707 entries) | `stat_table.rs` codegen via `gen_ssm.ts` |
| `Uniques/*.json` | Raw text strings for every unique item (display text with stat values) | Phase 4: unique item parsing (the ONLY source for unique stats) |
| `Gems.json` | ~~Gem metadata~~ **Replaced by RePoE `gems.json`** | Legacy — to be removed in Phase 3 |
| `Skills/*.json` | ~~Granted effect data~~ **Replaced by RePoE `gems.json`** | Legacy — to be removed in Phase 3 |

### GGG Tree (`data/tree/`)

Source: [grindinggear/skilltree-export](https://github.com/grindinggear/skilltree-export) — official passive tree JSON.

| File | What It Contains | Used By |
|---|---|---|
| `<version>/data.json` | Node positions, connections, groups, orbits, class start nodes, mastery effects, display text, class base stats | Frontend rendering (`SkillTree.svelte`) and class-level data (`rebuild_class()`) |

Current version: `3.27.0g`

---

## Completed Work (Phases 1–2)

### Phase 1: Foundation & Data Layer ✅

- GGG tree loaded from versioned `data/tree/3.27.0g/data.json`, served to frontend via `get_tree_json`
- RePoE tree loaded from `data/repoe/passive_skill_trees/Default.json` for calculations
- `PassiveTree`, `PassiveNode`, `ClassData`, `AscendancyData`, `BloodlineData` types in `data/tree.rs`
- `RePoETree`, `RePoEPassive` types in `data/repoe_tree.rs` with `get_passive(hash)` O(1) lookup
- `GameData` managed as `Arc<RwLock<GameData>>` with `load_from_dir()`
- `DataLoader` trait, `DataError` enum, `SourceId(u32)` newtype in `data/mod.rs`
- `StatId` enum: 320 variants (`#[repr(u16)]`) generated from `SkillStatMap.json`, `from_name()` via `OnceLock<FxHashMap>`
- `DEFAULT_TREE_VERSION` constant in `lib.rs`; `tools/fetch_data.ts` for data downloading
- Frontend: PixiJS tree rendering, pan/zoom, BFS-validated node selection, spatial grid, viewport culling

### Phase 2: Modifier System & Passive Tree Stats ✅

- **`stat_table.rs`** (931 lines, codegen): 692 SSM entries + ~80 manual entries, 9 helper functions (`def`, `flagged_def`, `kw_def`, `div_def`, `tagged_def`, etc.), `StatDef` struct with `apply()`, `OnceLock<FxHashMap>`
- **`parser::resolve(stat_id, value, source) → Vec<Modifier>`**: Single public entry point for all stat resolution, O(1) table lookup, no fallback
- **`parse_display_text(text, source) → Vec<Modifier>`**: Template-based text parser (~30 entries), for future PoB unique items only
- **`ModDB`**: `FxHashMap<StatId, Vec<Modifier>>` with `sum_base`, `sum_inc`, `product_more`, `has_flag`, `get_override`, `get_max`, `get_min`, `tabulate`, `sum_base_multi`, `calculate`, `merge`, `iter_all`
- **`ModDBLayers`**: `tree` (working — RePoE stats via `resolve()`), `class` (working — base Str/Dex/Int from `ClassData`), `gems` (placeholder — empty)
- **Types**: `Modifier`, `ModType` (9 variants: Base/Inc/More/Flag/Override/List/Max/Min/Chance), `ModFlag` (12 flags), `KeywordFlag` (5 flags), `ModTag` (10 variants with interned strings), `CalcContext`
- **`tools/gen_ssm.ts`**: Codegen from `SkillStatMap.norm.json` → `stat_table.rs` with manual section preservation
- 25 modifier tests passing, all call `resolve()` directly
- `StatAccumulator` and `stats.rs` deleted

### Current File Structure (Rust)

```
src-tauri/src/
├── main.rs                    # Binary entry → lib::run()
├── lib.rs                     # App setup, commands, BuildInfo, BuildStats, Class enum
├── modifier/
│   ├── mod.rs                 # Re-exports ModDB, ModDBLayers, all types
│   ├── types.rs               # Modifier, ModType, ModFlag, KeywordFlag, ModTag, CalcContext, intern()
│   ├── stat_table.rs          # GENERATED: stat ID → StatDef table (931 lines)
│   ├── parser.rs              # resolve(), parse_display_text(), helper constructors
│   └── mod_db.rs              # ModDB (queries), ModDBLayers (tree/class/gems layers)
├── data/
│   ├── mod.rs                 # GameData, DataLoader, DataError, SourceId
│   ├── stat_id.rs             # StatId enum (320 variants)
│   ├── tree.rs                # PassiveTree, PassiveNode, ClassData (GGG tree)
│   ├── repoe_tree.rs          # RePoETree, RePoEPassive (RePoE tree)
│   ├── gems.rs                # RePoEGem + subtypes, GemSummary/GemColor (IPC), compute_gem_stats(), apply_stat_conversions()
│   ├── skills.rs              # GemInstance (with always_active), SkillGroup, GemRef, SupportCompatEntry
│   │                          #   Also contains: GrantedEffect (PoB legacy — TO BE DELETED in step 3.8)
│   ├── bases.rs               # Empty stub
│   ├── mods.rs                # Empty stub
│   └── uniques.rs             # Empty stub
├── client/                    # PoeClient skeleton (empty)
├── storage/                   # StorageManager stub, FileCache (rkyv)
└── models.rs                  # LiteNode (legacy, unused)
```

---

## Phase 3: Gem & Skill System (RePoE)

### Goal

Replace the PoB-based gem/skill data (`data/pob/Gems.json` + `data/pob/Skills/*.json`) with RePoE's `gems.json`, which provides everything in one file with raw stat IDs. Wire gem stats into `ModDBLayers.rebuild_gems()` via `parser::resolve()`.

### Why RePoE Instead of PoB

| Aspect | PoB (`Gems.json` + `Skills/*.json`) | RePoE (`gems.json`) |
|---|---|---|
| **Stat format** | Positional arrays + `statMap` with nested modifier templates (complex, untyped `serde_json::Value`) | Raw stat IDs in `static.stats[].id` + per-level values — feeds directly into `resolve()` |
| **File count** | 11 files (1 Gems + 10 Skills/*.json) | 1 file |
| **Type safety** | Numeric SkillType IDs → postfix boolean expressions (stack machine) | String-based `allowed_types` / `excluded_types` (simple array matching) |
| **Stat conversions** | Merged from `statMap` + `SkillStatMap.json` at runtime | Directly in `active_skill.stat_conversions` per gem |
| **Deserialization** | Complex: `#[serde(flatten)]`, custom visitors, `deserialize_with` | Simple: every field has a clear type |

### RePoE `gems.json` Format

Top-level: flat object keyed by gem name (e.g., `"Fireball"`, `"SupportAddedFireDamage"`).

Each entry has this structure:

```
gem_name → {
    active_skill:    (null for supports) skill metadata + stat_conversions + types
    base_item:       id, display_name, max_level, release_state
    cast_time:       milliseconds (null for supports)
    color:           "b" | "r" | "g"
    display_name:    user-facing name
    is_support:      bool
    per_level:       { "1": level_data, "2": level_data, ... }
    static:          non-level-dependent stats (stat IDs + types defined here)
    tags:            ["fire", "spell", "projectile", ...]
    support_gem:     (null for actives) allowed_types, excluded_types, added_types
}
```

**Critical relationship between `static` and `per_level`**:

- `static.stats[i]` defines the stat slot: `{ id: "spell_minimum_base_fire_damage", type: "float", value: null }`
- `per_level["1"].stats[i]` provides the level-specific value: `{ value: 9, id: null, type: null }`
- The arrays are **positionally aligned** — index `i` in per_level corresponds to the stat defined at index `i` in static

**Stat type meanings**:
- `"float"` — scales per level (damage, radius, etc.)
- `"constant"` — fixed across all levels (e.g., 25% ignite chance)
- `"additional"` — modifier applied to supported skills
- `"implicit"` — internal flags (e.g., `base_is_projectile: 1`)

**Stat conversions** (on `active_skill`):
```json
"stat_conversions": {
    "fireball_ignite_chance_%": "base_chance_to_ignite_%",
    "fireball_damage_+%": "damage_+%"
}
```
Maps skill-specific stat IDs to generic IDs that appear in SkillStatMap. Apply these **before** calling `resolve()`.

**Quality stats** (on `static`):
```json
"quality_stats": [{
    "stats": { "fire_damage_+%": 500 }
}]
```
Value is per 1% quality, in internal units. Divide by 1000 before use: `stat_value = raw_value / 1000.0 * quality`.

**Support matching** (on `support_gem`):
```json
"support_gem": {
    "allowed_types": ["Damage", "Attack"],
    "excluded_types": null,
    "supports_gems_only": false,
    "added_types": ["Fire"]
}
```
Simple: does the active skill's `types` array contain any of `allowed_types`? If so, compatible. Does it contain any of `excluded_types`? If so, excluded.

### Steps

#### 3.1 — Define RePoE Gem Types (`data/gems.rs`) ✅

PoB-based `GemItem` replaced with `RePoEGem` and subtypes. `GemSummary` and `GemColor` kept for frontend IPC.

**Key deviations from original plan:**
- `per_level` keys are `u32` (not `String`) — serde auto-parses JSON string keys as integers
- `base_item: Option<GemItem>` (not `GemBaseItem`) — ~388 internal/triggered gems have `null` base_item
- `display_name: Option<String>` — same gems have null display_name
- `color: GemColor` (enum with `#[serde(alias)]` for `"r"`/`"g"`/`"b"`) instead of `String`
- `damage_effectiveness: Option<i32>` (not `u32`) — can be negative (as low as -97)
- Custom `null_as_empty_vec` deserializer for Vec fields that may be explicit JSON `null`
- `SupportGemInfo` → renamed `SupportSkill` in implementation
- `GemStatic` → named `GemStaticProps`; `GemStatEntry` → named `GemStat`
- `GemQualityStat` (was `QualityStat` in plan)

Actual struct signatures in `data/gems.rs`:

```rust
pub struct RePoEGem {
    pub active_skill: Option<ActiveSkill>,
    pub base_item: Option<GemItem>,       // Option — null for internal gems
    pub cast_time: Option<u32>,
    pub color: GemColor,                  // enum, not String
    pub display_name: Option<String>,     // Option — null for internal gems
    pub is_support: bool,
    pub per_level: FxHashMap<u32, GemLevel>,  // u32 keys, not String
    pub stat_translation_file: Option<String>,
    #[serde(rename = "static")]
    pub static_data: GemStaticProps,
    pub tags: Vec<String>,
    pub support_gem: Option<SupportSkill>,
    pub secondary_granted_effect: Option<serde_json::Value>,
}

/// Active skill metadata — present only on non-support gems.
#[derive(Debug, Clone, Deserialize)]
pub struct ActiveSkill {
    pub display_name: String,
    pub id: String,
    pub is_manually_casted: bool,
    pub is_skill_totem: bool,
    /// Maps skill-specific stat IDs to generic stat IDs.
    /// e.g. "fireball_ignite_chance_%" → "base_chance_to_ignite_%"
    /// Apply before calling resolve().
    #[serde(default)]
    pub stat_conversions: FxHashMap<String, String>,
    pub types: Vec<String>,
    #[serde(default)]
    pub weapon_restrictions: Vec<String>,
    pub skill_totem_life_multiplier: Option<f64>,
    pub minion_types: Option<Vec<String>>,
}

/// Gem base item identity.
#[derive(Debug, Clone, Deserialize)]
pub struct GemBaseItem {
    pub display_name: String,
    pub id: String,
    pub max_level: u32,
    pub release_state: String,
}

/// Per-level data. Stats are positionally aligned with static.stats.
#[derive(Debug, Clone, Deserialize)]
pub struct GemLevel {
    pub costs: Option<FxHashMap<String, Option<f64>>>,
    pub required_level: Option<u32>,
    pub stats: Vec<Option<GemStatEntry>>,
    pub damage_effectiveness: Option<u32>,
    pub cooldown: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub stored_uses: Option<u32>,
    pub attack_speed_multiplier: Option<f64>,
    pub damage_multiplier: Option<f64>,
}

/// A single stat value. In static: id and type are populated. In per_level: only value.
#[derive(Debug, Clone, Deserialize)]
pub struct GemStatEntry {
    pub value: Option<f64>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub stat_type: Option<String>,
}

/// Non-level-dependent gem properties.
#[derive(Debug, Clone, Deserialize)]
pub struct GemStatic {
    pub crit_chance: Option<u32>,
    pub damage_effectiveness: Option<u32>,
    #[serde(default)]
    pub quality_stats: Vec<QualityStat>,
    pub stats: Vec<Option<GemStatEntry>>,
    pub cooldown: Option<u32>,
    pub stored_uses: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub attack_speed_multiplier: Option<f64>,
}

/// Quality scaling: per 1% quality, add this to the stat. Divide value by 1000.
#[derive(Debug, Clone, Deserialize)]
pub struct QualityStat {
    pub stats: FxHashMap<String, f64>,
}

/// Support gem matching rules.
#[derive(Debug, Clone, Deserialize)]
pub struct SupportGemInfo {
    pub allowed_types: Option<Vec<String>>,
    pub excluded_types: Option<Vec<String>>,
    pub letter: String,
    pub supports_gems_only: bool,
    pub added_types: Option<Vec<String>>,
    pub added_minion_types: Option<Vec<String>>,
}
```

Keep `GemSummary` and `GemColor` unchanged — they're the frontend IPC types.

#### 3.2 — Load RePoE `gems.json` at Startup ✅

`GameData` in `data/mod.rs` updated:

```rust
pub struct GameData {
    pub tree: PassiveTree,
    pub repoe_tree: RePoETree,
    pub gems: FxHashMap<String, RePoEGem>,  // was: FxHashMap<String, GemItem>
    pub source_names: Vec<String>,
}
```

`load_from_dir()` loads `data/repoe/gems.json`. The PoB `data/pob/Gems.json` + `data/pob/Skills/*.json` are no longer loaded at startup (still on disk for potential future reference).

#### 3.3 — Implement Gem Stat Computation ✅

`compute_gem_stats(gem, level, quality) -> Vec<(String, f64)>` implemented in `data/gems.rs`.
Uses `u32` key to look up `per_level` directly (no string conversion). Skips `"implicit"` stat types.
Falls back to `static_data` value when per-level entry has `None`.

```rust
pub fn compute_gem_stats(
    gem: &RePoEGem,
    level: u32,
    quality: u32,
) -> Vec<(String, f64)> {
    let mut result = Vec::new();

    let level_data = match gem.per_level.get(&level) {  // u32 key, not String
        Some(ld) => ld,
        None => return result,
    };

    // 1. Per-level stats — aligned positionally with static.stats
    for (i, static_slot) in gem.static_data.stats.iter().enumerate() {
        let Some(stat_def) = static_slot.as_ref() else { continue };
        let Some(stat_id) = stat_def.id.as_ref() else { continue };

        // Skip implicit-type stats (internal engine flags, not modifiers)
        if stat_def.stat_type.as_deref() == Some("implicit") {
            continue;
        }

        // Get value from per_level (falls back to static value)
        let value = level_data.stats
            .get(i)
            .and_then(|s| s.as_ref())
            .and_then(|s| s.value)
            .or(stat_def.value)
            .unwrap_or(0.0);

        result.push((stat_id.clone(), value));
    }

    // 2. Quality stats — divide by 1000, multiply by quality
    if quality > 0 {
        for qs in &gem.static_data.quality_stats {
            for (stat_id, raw_value) in &qs.stats {
                let value = (raw_value / 1000.0) * quality as f64;
                result.push((stat_id.clone(), value));
            }
        }
    }

    result
}
```

**Test cases**:
- Fireball level 1: `spell_minimum_base_fire_damage = 9`, `spell_maximum_base_fire_damage = 14`, `base_chance_to_ignite_% = 25`, `active_skill_base_area_of_effect_radius = 9`
- Fireball level 20: damage values scale up, ignite chance stays 25 (constant type)
- Added Fire Damage Support level 20: `physical_damage_%_to_add_as_fire = 39`
- Quality 20 on Added Fire Damage: `fire_damage_+% = 500/1000 * 20 = 10`

#### 3.4 — Apply Stat Conversions ✅

`apply_stat_conversions(stats, conversions)` implemented in `data/gems.rs` (public, not private).
Called in `mod_db.rs` `add_gem_stats()` before `resolve()` for active skill gems.

```rust
/// Apply stat_conversions to a list of (stat_id, value) pairs.
/// Renames skill-specific stat IDs to generic ones.
pub fn apply_stat_conversions(
    stats: &mut Vec<(String, f64)>,
    conversions: &FxHashMap<String, String>,
) {
    for (stat_id, _) in stats.iter_mut() {
        if let Some(converted) = conversions.get(stat_id.as_str()) {
            *stat_id = converted.clone();
        }
    }
}
```

**Example**: Fireball with `"fireball_ignite_chance_%": "base_chance_to_ignite_%"`:
- Input: `("fireball_ignite_chance_%", 25.0)`
- After conversion: `("base_chance_to_ignite_%", 25.0)`
- Then `resolve("base_chance_to_ignite_%", 25.0, source)` looks up the stat table

#### 3.5 — Active Gem Framework ✅

**Design pivot from original plan**: Instead of selecting an entire skill group (`active_group_id: Option<u32>`), the system selects a specific gem within a group, and supports an `always_active` flag for persistent-effect gems (auras, heralds, warcries).

This correctly models PoE's two calculation modes:
- **Main skill** — one user-selected active gem + its compatible supports → drives DPS calculations
- **Always-active** — any active gem with `always_active = true` across any group (e.g., Clarity, Grace) → always contributes to stats

**`GemRef` struct** added to `data/skills.rs`:
```rust
pub struct GemRef {
    pub group_id: u32,    // which SkillGroup
    pub gem_index: u32,   // index within SkillGroup.gems
}
```

**`GemInstance.always_active`** added to `data/skills.rs`:
```rust
pub struct GemInstance {
    // ... existing fields ...
    pub always_active: bool,  // auras/heralds/warcries — contribute regardless of active skill
}
```

**`BuildInfo.active_gem`** in `lib.rs` (replaced `active_skill: Option<u32>`):
```rust
pub struct BuildInfo {
    pub skill_groups: Vec<SkillGroup>,
    pub active_gem: Option<GemRef>,  // None = no main skill selected
    // ...
}
```

**Commands** in `lib.rs`:
- `set_active_gem(gem_ref: Option<GemRef>)` — selects the main skill, triggers `rebuild_gems`. Currently stubs `Err("Not yet implemented")` for BuildStats return — wired in step 3.10.
- `set_gem_always_active(group_id, gem_index, always_active)` — toggles flag, triggers `rebuild_gems`, returns updated `SkillGroup`.
- `delete_skill_group` — clears `active_gem` if the deleted group contained it; clones `skill_groups` before calling `rebuild_gems` to avoid borrow conflicts.

#### 3.6 — Wire into `ModDBLayers.rebuild_gems()` ✅

The placeholder in `mod_db.rs` replaced with a two-pass implementation matching the `active_gem` + `always_active` design from step 3.5.

**Actual signature** (takes all groups + a gem reference, not a single group):
```rust
pub fn rebuild_gems(
    &mut self,
    skill_groups: &[SkillGroup],
    active_gem: Option<&GemRef>,
    game_data: &GameData,
)
```

**Two-pass logic:**
- **Pass 1**: Find the group/gem pointed to by `active_gem`; call `add_gem_with_supports()` for it
- **Pass 2**: Scan all groups for enabled, non-support gems with `always_active = true`; call `add_gem_with_supports()` for each (skipping if it's already the main skill from pass 1)

**`add_gem_with_supports(active, group, game_data)`** — adds the active gem's stats + all enabled supports in the same group. Support compatibility filtering is NOT yet applied (TODO 3.7).

**`add_gem_stats(gem_inst, game_data)`** — calls `compute_gem_stats()`, applies `apply_stat_conversions()` for active skills, generates a stable `SourceId` via FNV hash of the gem_id string, then calls `parser::resolve()` for each stat. This mirrors the tree stat path exactly — `resolve()` is the single entry point.

#### 3.7 — Support Matching

RePoE uses simple string-based type matching instead of PoB's postfix boolean expressions.

```rust
/// Check if a support gem can support an active gem.
pub fn can_support(support: &RePoEGem, active: &RePoEGem) -> bool {
    let support_info = match &support.support_gem {
        Some(info) => info,
        None => return false, // not a support gem
    };
    let active_skill = match &active.active_skill {
        Some(skill) => skill,
        None => return false, // not an active gem
    };

    // supports_gems_only check: handled at a higher level (item-granted skills)

    // Exclude check: if active has any excluded type, reject
    if let Some(excluded) = &support_info.excluded_types {
        if active_skill.types.iter().any(|t| excluded.contains(t)) {
            return false;
        }
    }

    // Require check: if allowed_types is set, active must have at least one
    match &support_info.allowed_types {
        Some(allowed) if !allowed.is_empty() => {
            active_skill.types.iter().any(|t| allowed.contains(t))
        }
        _ => true, // no restrictions = supports everything
    }
}

/// Resolve which supports apply to an active gem within a skill group.
/// Two-pass logic: supports can add types that enable other supports.
pub fn resolve_supports(
    active: &RePoEGem,
    supports: &[&RePoEGem],
    gems: &FxHashMap<String, RePoEGem>,
) -> Vec<usize> {
    let active_skill = match &active.active_skill {
        Some(skill) => skill,
        None => return Vec::new(),
    };

    // Build effective type set starting from active skill's types
    let mut effective_types: Vec<String> = active_skill.types.clone();
    let mut compatible = vec![false; supports.len()];
    let mut added_new = true;

    // Pass 1: iteratively add types from compatible supports
    while added_new {
        added_new = false;
        for (i, support) in supports.iter().enumerate() {
            if compatible[i] { continue; }
            let Some(info) = &support.support_gem else { continue };

            // Check exclude
            if let Some(excluded) = &info.excluded_types {
                if effective_types.iter().any(|t| excluded.contains(t)) {
                    continue;
                }
            }
            // Check require
            let matches = match &info.allowed_types {
                Some(allowed) if !allowed.is_empty() => {
                    effective_types.iter().any(|t| allowed.contains(t))
                }
                _ => true,
            };
            if matches {
                compatible[i] = true;
                if let Some(added) = &info.added_types {
                    for t in added {
                        if !effective_types.contains(t) {
                            effective_types.push(t.clone());
                            added_new = true;
                        }
                    }
                }
            }
        }
    }

    // Return indices of compatible supports
    compatible.iter().enumerate()
        .filter(|(_, &c)| c)
        .map(|(i, _)| i)
        .collect()
}
```

**Test cases**:
- Added Fire Damage Support (`allowed_types: ["Damage", "Attack"]`) supports Fireball (`types` includes "Damage") → `true`
- Added Fire Damage Support does NOT support a pure aura with no "Damage" or "Attack" type → `false`
- Spell Totem (`added_types: ["Totem"]`) adds "Totem" to effective types, enabling a totem-requiring support

#### 3.8 — Simplify Runtime Gem State

**Partially done**: `always_active: bool` was added to `GemInstance` in step 3.5. The remaining cleanup is to remove legacy PoB-style fields that are no longer needed since stats are computed on-demand.

**Still to remove** from `data/skills.rs`:
- `GemInstance.stats: BTreeMap<String, f64>`, `mana_cost`, `crit_chance`, `damage_effectiveness`, `mana_multiplier`, `cooldown`, `attack_speed_multiplier`
- `SkillGroup.compatibility: Vec<SupportCompatEntry>`
- `SupportCompatEntry` struct
- `GrantedEffect`, `GrantedEffectLevel`, `QualityStats` structs
- Dead deserializer functions: `deserialize_skill_types`, `map_or_empty_seq`, `optional_map_or_seq`, `deserialize_levels`

**Target shape** once cleanup is done:

```rust
pub struct GemInstance {
    pub gem_id: String,
    pub name: String,
    pub is_support: bool,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
    pub always_active: bool,
}

/// A socket group — one active skill plus its supports.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct SkillGroup {
    pub id: u32,
    pub label: String,
    pub gems: Vec<GemInstance>,
    pub enabled: bool,
}
```

Remove `SupportCompatEntry`, `QualityStats` struct, `GrantedEffect`, `GrantedEffectLevel`, `deserialize_skill_types`, `map_or_empty_seq`, and all the PoB-specific deserialization complexity from `data/skills.rs`.

#### 3.9 — Update `get_gem_list` Command ✅

`get_gem_list` updated to work with `RePoEGem`. Filters to gems that have both `display_name` and `base_item` (excludes ~388 internal/triggered gems with `null` in those fields). Color comes directly from the `GemColor` enum field rather than a tag-lookup helper.

```rust
fn get_gem_list(...) -> Result<Vec<GemSummary>, String> {
    let game = game_data.read()...;
    let mut list: Vec<GemSummary> = game.gems.iter()
        .filter(|(_, gem)| gem.display_name.is_some() && gem.base_item.is_some())
        .map(|(id, gem)| GemSummary {
            id: id.clone(),
            name: gem.display_name.clone().unwrap(),
            tag_string: gem.tags.join(""),
            is_support: gem.is_support,
            color: gem.color,
            description: gem.base_item.as_ref().map(|b| b.display_name.clone()),
        })
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(list)
}

#### 3.10 — Recalculation Flow

Two distinct triggers. `set_active_gem` currently stubs `Err("Not yet implemented")` — this step wires it to return real `BuildStats`.

```
── Gem change (add/remove/level/quality) ──────────────────────────────────
User edits gem → Tauri command →
    mutate BuildInfo.skill_groups →
    if group contains the active gem OR group has always_active gems:
        call rebuild_gems(skill_groups, active_gem, &game_data)
    return updated SkillGroup to frontend

── Active gem selection ──────────────────────────────────────────────────
User clicks gem → set_active_gem(Some(GemRef { group_id, gem_index })) →
    build.active_gem = Some(gem_ref) →
    skill_groups.clone() → rebuild_gems(groups, Some(&gem_ref), &game_data) →
        Pass 1: compute active gem + supports → resolver → gems layer
        Pass 2: all always_active gems + their supports → resolver → gems layer
    merged = mod_db_layers.merged() →
    BuildStats computed from merged ModDB →
    return BuildStats to frontend

── Always-active toggle ──────────────────────────────────────────────────
User toggles always_active → set_gem_always_active(group_id, gem_index, bool) →
    group.gems[idx].always_active = bool →
    rebuild_gems(groups, active_gem, &game_data) →   (already implemented)
    return updated SkillGroup
```

The recalc path is identical to the tree stat flow — `resolve()` is the single entry point for both.

#### 3.11 — Frontend `SkillsTab.svelte`

Create `src/components/SkillsTab.svelte`:

- **Skill group list**: Add/remove groups; clicking a gem within a group calls `set_active_gem({ group_id, gem_index })` which rebuilds the gems layer and returns updated `BuildStats` — the sidebar updates immediately
- **Active gem indicator**: Mark the currently selected main skill gem (the one powering DPS calculations)
- **Always-active toggle**: Per-gem toggle for auras/heralds; calls `set_gem_always_active`
- **Gem selector**: Searchable dropdown filtered by `GemSummary.tag_string` and name
- **Per-gem controls**: Level slider (1–21+), quality slider (0–23), enable/disable toggle
- **Auto-resolved supports**: Show which supports are compatible with the active gem (requires 3.7)
- **Display**: Show gem name, support letter, computed stat summary

Add tab navigation to `src/routes/skilltree/+page.svelte` with Tree and Skills tabs functional.

#### 3.12 — Delete PoB-Based Code

After RePoE gem loading is working and tested:

1. In `data/gems.rs`: Delete old `GemItem` struct (replace with `RePoEGem`)
2. In `data/skills.rs`: Delete `GrantedEffect`, `GrantedEffectLevel`, `QualityStats`, `deserialize_skill_types`, `map_or_empty_seq`, `does_type_expression_match`, `skill_type` constants — all replaced by RePoE format
3. In `data/mod.rs`: Remove `use crate::data::skills::GrantedEffect` and `skills: FxHashMap<String, GrantedEffect>` field
4. In `GameData::load_from_dir()` and `load_all_json()`: Remove loading of `data/pob/Gems.json` and `data/pob/Skills/*.json`

The files `data/pob/Gems.json` and `data/pob/Skills/*.json` can stay on disk (other tools may reference them) but are no longer loaded at startup.

#### 3.13 — Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn load_gems() -> FxHashMap<String, RePoEGem> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/repoe/gems.json");
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_load_all_gems() {
        let gems = load_gems();
        assert!(gems.len() > 400, "Expected 400+ gems, got {}", gems.len());
    }

    #[test]
    fn test_fireball_level_1_stats() {
        let gems = load_gems();
        let fb = &gems["Fireball"];
        let stats = compute_gem_stats(fb, 1, 0);
        // fire min damage, fire max damage, ignite chance, AoE radius
        assert!(stats.iter().any(|(id, v)| id == "spell_minimum_base_fire_damage" && *v == 9.0));
        assert!(stats.iter().any(|(id, v)| id == "spell_maximum_base_fire_damage" && *v == 14.0));
        assert!(stats.iter().any(|(id, v)| id == "base_chance_to_ignite_%" && *v == 25.0));
    }

    #[test]
    fn test_fireball_level_20_stats() {
        let gems = load_gems();
        let fb = &gems["Fireball"];
        let stats = compute_gem_stats(fb, 20, 0);
        assert!(stats.iter().any(|(id, v)| id == "spell_minimum_base_fire_damage" && *v == 1640.0));
        assert!(stats.iter().any(|(id, v)| id == "spell_maximum_base_fire_damage" && *v == 2460.0));
        // ignite chance stays 25 (constant type)
        assert!(stats.iter().any(|(id, v)| id == "base_chance_to_ignite_%" && *v == 25.0));
    }

    #[test]
    fn test_stat_conversions() {
        let gems = load_gems();
        let fb = &gems["Fireball"];
        let mut stats = compute_gem_stats(fb, 1, 0);
        let conversions = &fb.active_skill.as_ref().unwrap().stat_conversions;
        apply_stat_conversions(&mut stats, conversions);
        // "fireball_ignite_chance_%" should have been converted to "base_chance_to_ignite_%"
        // (it already IS "base_chance_to_ignite_%" from static, but verify the mechanism works)
    }

    #[test]
    fn test_quality_stats() {
        let gems = load_gems();
        let added_fire = &gems["SupportAddedFireDamage"];
        let stats = compute_gem_stats(added_fire, 20, 20);
        // Quality 20: fire_damage_+% = 500/1000 * 20 = 10
        assert!(stats.iter().any(|(id, v)| id == "fire_damage_+%" && (*v - 10.0).abs() < 0.01));
    }

    #[test]
    fn test_support_matching() {
        let gems = load_gems();
        let fireball = &gems["Fireball"];
        let added_fire = &gems["SupportAddedFireDamage"];
        assert!(can_support(added_fire, fireball));
    }

    #[test]
    fn test_support_not_matching() {
        let gems = load_gems();
        // Find a pure aura gem and verify Added Fire doesn't support it
        let clarity = &gems["Clarity"];
        let added_fire = &gems["SupportAddedFireDamage"];
        assert!(!can_support(added_fire, clarity));
    }

    #[test]
    fn test_resolve_integration() {
        // Verify gem stats flow through resolve() into Modifiers
        let gems = load_gems();
        let added_fire = &gems["SupportAddedFireDamage"];
        let stats = compute_gem_stats(added_fire, 20, 0);
        let source = SourceId(999);
        let mut mod_count = 0;
        for (stat_id, value) in &stats {
            let mods = crate::modifier::parser::resolve(stat_id, *value, source);
            mod_count += mods.len();
        }
        assert!(mod_count > 0, "Should produce at least one Modifier from gem stats");
    }
}
```

#### 3.14 — Gem Stat Coverage Expansion

**Problem**: `resolve()` via `stat_table.rs` covers only ~445 of ~1,808 unique gem stat IDs (25% coverage). The remaining 75% are silently dropped — `resolve()` returns an empty `Vec<Modifier>`, so most gem effects never enter `ModDB`. Any build using gem stats (i.e., every build) produces incorrect calculations.

**Root cause**: `stat_table.rs` is generated from `SkillStatMap.json` which maps ~700 tree/item stat IDs. Gem-specific stat IDs use skill-prefixed names (e.g., `support_melee_physical_damage_+%_final`, `ice_nova_area_of_effect_+%`) that aren't in SSM because PoB handles them through a separate per-gem `statMap` mechanism — which the RePoE migration eliminated.

**Coverage gap analysis**:

| Category | Missing Unique IDs | Example |
|---|---|---|
| `*_+%_final` (More) | 222 | `support_melee_physical_damage_+%_final` |
| `*_+%` (Inc) | 78 | `ice_nova_area_of_effect_+%` |
| `*_+` (Base flat) | 19 | `supported_active_skill_gem_level_+` |
| `*_%` (various) | 116 | `base_chance_to_ignite_%` |
| `*_ms` (duration) | 89 | `active_skill_withered_base_duration_ms` |
| Other | 796 | `base_is_projectile`, `spell_minimum_base_fire_damage` |
| **Total missing** | **1,363** | |

**Display text source coverage** (the key data for the solution):

| Source | Gem Stat Coverage | Notes |
|---|---|---|
| RePoE `stat_translations.json` | ~24% of gem stat occurrences | Generic/shared stats only — missing gem-specific |
| PoB `StatDescriptions/*.json` (22 files) | ~71% of gem stat occurrences | **Includes gem-specific stats** |
| Combined (either source) | ~71% | Very little unique to RePoE |
| Neither source | ~29% (~299 unique IDs) | Mostly internal flags (`base_is_projectile`, `is_area_damage`) |

**Solution: Codegen-expanded `stat_table.rs` using display text as input**

The display text from PoB StatDescriptions contains all the semantic information needed to build stat_table entries. For example: *"Supported Skills deal {0}% **more** **Melee Physical Damage**"* → `ModType::More`, `ModFlag::MELEE | ModFlag::ATTACK`, `StatId::PhysDamage`.

A codegen tool will:
1. Load all gem stat IDs from `gems.json`
2. Look up display text from combined PoB StatDescriptions + RePoE stat_translations
3. Parse display text patterns to extract `(StatId, ModType, ModFlag, KeywordFlag)`
4. Generate a new `GEM STATS` section in `stat_table.rs`
5. Use a manual override file for edge cases and internal flags

This keeps the runtime architecture unchanged — `resolve()` stays O(1) via `stat_table` lookup. The codegen tool runs at development time, like `gen_ssm.ts`. Build command: `bun run tool:gen-gem-stats`.

##### 3.14a — Build Combined Stat Translations Loader

Create `tools/stat_translations.ts` — a shared module that reads all 22 PoB `StatDescriptions/*.json` files plus RePoE `stat_translations.json` and produces a unified `Map<string, string>` of `stat_id → display_text_template`.

**PoB StatDescriptions format** (e.g., `gem_stat_descriptions.json`):
```
Top section: flat index { stat_id: numeric_key }
    e.g. "support_melee_physical_damage_+%_final": 2

Bottom section: entries keyed by numeric string
    "2": {
      "stats": ["support_melee_physical_damage_+%_final"],
      "1": [{
        "limit": [[1, "#"]],
        "text": "Supported Skills deal {0}% more Melee Physical Damage"
      }, {
        "limit": [["#", -1]],
        "text": "Supported Skills deal {0}% less Melee Physical Damage",
        "1": { "k": "negate", "v": 1 }
      }]
    }
```

**RePoE stat_translations.json format**:
```
Array of 11,075 entries:
    {
      "ids": ["fire_damage_+%"],
      "English": [{
        "condition": [{ "min": 1, "max": null }],
        "string": "{0}% increased Fire Damage",
        "index_handlers": [[]]
      }, {
        "condition": [{ "min": null, "max": -1 }],
        "string": "{0}% reduced Fire Damage",
        "index_handlers": [["negate"]]
      }]
    }
```

The loader selects the **positive variant** for each stat ID:
- PoB: entry with `limit: [[1, "#"]]` (positive value)
- RePoE: entry with `condition: [{ min: 1 }]` (positive value)

Priority: PoB StatDescriptions first (better gem coverage), then RePoE for anything PoB doesn't have.

**Index handlers / value transforms** are recorded but not used by the codegen tool — the semantic parsing is purely from the English text pattern.

##### 3.14b — Create `gen_gem_stat_table.ts` Codegen Tool

Create `tools/gen_gem_stat_table.ts` that:

1. **Extracts** all unique stat IDs from `data/repoe/gems.json` — both `static.stats[].id` and quality_stats keys
2. **Filters out** stat IDs already in SkillStatMap (already covered by `gen_ssm.ts`)
3. **Looks up** display text for each stat ID via the combined translations loader (3.14a)
4. **Parses** display text patterns to determine modifier properties
5. **Loads** manual overrides from `tools/gem_stat_overrides.json` for edge cases
6. **Generates** Rust entries in the stat_table.rs helper format
7. **Writes** to a new `// BEGIN GEM STATS` / `// END GEM STATS` section in `stat_table.rs`
8. **Reports** any stat IDs that couldn't be resolved (should be 0 after overrides)

**Display text pattern → modifier mapping rules**:

| Display Text Pattern | ModType | Example |
|---|---|---|
| `"X% more Y"` | `More` | "Supported Skills deal {0}% more Melee Physical Damage" |
| `"X% less Y"` | `More` | (negative variant, handled by negate) |
| `"X% increased Y"` | `Inc` | "{0}% increased Area of Effect" |
| `"X% reduced Y"` | `Inc` | (negative variant) |
| `"Adds X to Y Z Damage"` | `Base` | "Adds {0} to {1} Fire Damage to Attacks" |
| `"+X to Y"` | `Base` | "+{0} to maximum Life" |
| `"X additional Y"` | `Base` | "{0} additional Projectile" |
| `"Gain X"` / presence/boolean | `Flag` | "Gain Unholy Might" |

**Domain keyword extraction from display text**:

| Keyword in Text | Maps To |
|---|---|
| "Melee" | `ModFlag::MELEE` |
| "Attack" / "Attacks" | `ModFlag::ATTACK` |
| "Spell" / "Spells" | `ModFlag::SPELL` |
| "Projectile" | `ModFlag::PROJECTILE` |
| "with Bows" | `ModFlag::BOW` |
| "Physical" | `KeywordFlag::PHYSICAL` |
| "Fire" | `KeywordFlag::FIRE` |
| "Cold" | `KeywordFlag::COLD` |
| "Lightning" | `KeywordFlag::LIGHTNING` |
| "Chaos" | `KeywordFlag::CHAOS` |

**StatId selection heuristic from display text**:

| Noun Phrase | StatId |
|---|---|
| "Physical Damage" | `PhysDamage` |
| "Fire Damage" | `FireDamage` |
| "Cold Damage" | `ColdDamage` |
| "Lightning Damage" | `LightningDamage` |
| "Chaos Damage" | `ChaosDamage` |
| "Damage" (generic) | `Damage` |
| "Attack Speed" | `Speed` (+ ATTACK flag) |
| "Cast Speed" | `Speed` (+ SPELL flag) |
| "Critical Strike Chance" | `CritChance` |
| "Critical Strike Multiplier" | `CritMulti` |
| "Area of Effect" | `AreaOfEffect` |
| "Mana Cost" | `ManaCost` |
| "Duration" | `Duration` |
| "Life" / "maximum Life" | `Life` |
| "Mana" / "maximum Mana" | `Mana` |
| "Energy Shield" | `EnergyShield` |

The noun-phrase table is extensible — the codegen tool should print any unrecognized noun phrases so they can be added.

**Generated output format** (Rust entries):
```rust
// BEGIN GEM STATS — auto-generated by gen_gem_stat_table.ts. Do not edit.
("support_melee_physical_damage_+%_final",
    flagged_def(StatId::PhysDamage, ModType::More, ModFlag::MELEE | ModFlag::ATTACK)),
("support_melee_physical_damage_attack_speed_+%_final",
    flagged_def(StatId::Speed, ModType::More, ModFlag::MELEE | ModFlag::ATTACK)),
("ice_nova_area_of_effect_+%",
    def(StatId::AreaOfEffect, ModType::Inc)),
// ... (~1000+ entries)
// END GEM STATS
```

##### 3.14c — Expand `StatId` Enum

Audit all gem display text patterns for stat concepts not yet in the 320-variant `StatId` enum. Likely additions:

| New StatId | Reason |
|---|---|
| `AreaOfEffect` | Common gem stat, not in SSM |
| `Duration` | Skill effect duration |
| `Damage` | Generic/untyped damage (not element-specific) |
| `ProjectileSpeed` | If not already present |
| `ManaCost` | Mana cost reduction/increase |
| `Cooldown` | Cooldown recovery |
| `MinionDamage` | Minion-specific damage |
| `MinionLife` | Minion-specific life |
| `TrapDamage` | Trap-specific |
| `MineDamage` | Mine-specific |
| `TotemDamage` | Totem-specific |

Run `gen_gem_stat_table.ts` in dry-run/audit mode first to identify exact gaps, then add missing `StatId` variants to `stat_id.rs` or expand the generation pass.

##### 3.14d — Manual Override File

Create `tools/gem_stat_overrides.json` for the ~299 stat IDs that have no display text in any translation source. Most fall into clear categories:

```json
{
  "_comment": "Stat IDs with no display text — classified manually",

  "base_is_projectile": { "stat": "IsProjectile", "type": "FLAG" },
  "is_area_damage": { "stat": "IsAreaDamage", "type": "FLAG" },
  "base_is_attack": { "stat": "IsAttack", "type": "FLAG" },

  "spell_minimum_base_fire_damage": { "stat": "FireDamageMin", "type": "BASE", "flags": ["SPELL"] },
  "spell_maximum_base_fire_damage": { "stat": "FireDamageMax", "type": "BASE", "flags": ["SPELL"] },
  "attack_minimum_base_fire_damage": { "stat": "FireDamageMin", "type": "BASE", "flags": ["ATTACK"] },
  "attack_maximum_base_fire_damage": { "stat": "FireDamageMax", "type": "BASE", "flags": ["ATTACK"] },

  "active_skill_withered_base_duration_ms": {
    "stat": "WitherDuration", "type": "BASE", "div": 1000
  }
}
```

**Categories of manual overrides**:
1. **Internal engine flags** (`base_is_projectile`, `is_area_damage`) → `ModType::Flag`
2. **Flat base damage** (min/max patterns) → `ModType::Base` with attack/spell flag, derivable from stat ID naming
3. **Duration in milliseconds** → `ModType::Base` with `div: 1000`
4. **Implicit stats** (`skill_can_fire_arrows`) → `ModType::Flag` or skipped (engine-internal)

The codegen tool applies overrides as a final fallback after display text parsing.

##### 3.14e — Pre-Resolution Cache (Optimization)

After `stat_table.rs` covers all gem stat IDs, add a load-time cache on `RePoEGem` to eliminate runtime string lookups from the calculation path:

```rust
/// A gem stat with its stat_table definitions pre-resolved at load time.
pub struct ResolvedGemStat {
    pub stat_id: String,                  // raw stat ID (for display/debugging)
    pub defs: SmallVec<[StatDef; 1]>,     // pre-resolved from stat_table
}

pub struct RePoEGem {
    // ... existing fields ...
    /// Pre-resolved stat definitions, populated at GameData load time.
    /// Positionally aligned with static.stats (same indexing).
    #[serde(skip)]
    pub resolved_stats: Vec<Option<ResolvedGemStat>>,
}
```

After loading `gems.json` in `GameData::load_from_dir()`, iterate all gems' `static.stats` and look up each stat ID in `stat_table()`, storing the `StatDef`s. At calc time, `add_gem_stats()` skips the string lookup entirely.

**Stat conversions interaction**: `apply_stat_conversions()` maps gem-specific IDs to generic IDs (e.g., `fireball_ignite_chance_%` → `base_chance_to_ignite_%`). With the expanded stat_table, both the pre-conversion and post-conversion IDs will have entries. Pre-resolution should apply `stat_conversions` first (converting the stat ID), then look up the converted ID. This preserves correctness while benefiting from the cache.

##### 3.14f — Simplify `add_gem_stats()`

With pre-resolved stats cached on the gem, `add_gem_stats()` becomes:

```rust
fn add_gem_stats(&mut self, gem_inst: &GemInstance, game_data: &GameData) {
    let Some(gem) = game_data.gems.get(&gem_inst.gem_id) else { return };
    let level_data = match gem.per_level.get(&gem_inst.level) {
        Some(ld) => ld,
        None => return,
    };
    let source = make_source_id(&gem_inst.gem_id);

    for (i, resolved) in gem.resolved_stats.iter().enumerate() {
        let Some(resolved) = resolved.as_ref() else { continue };

        // Get value from per_level (falls back to static)
        let value = level_data.stats
            .get(i)
            .and_then(|s| s.as_ref())
            .and_then(|s| s.value)
            .unwrap_or(0.0);

        // Quality bonus (if applicable)
        // ... (separate pass for quality_stats, same pattern)

        for def in &resolved.defs {
            self.gems.add_mod(def.apply(value / def.div, source));
        }
    }
}
```

No string lookups, no runtime `resolve()` call, no per-stat HashMap probing. Just iterate pre-resolved defs and apply values.

##### 3.14g — Coverage Validation & Testing

1. **Codegen coverage report**: After running the tool, print:
   - Total unique gem stat IDs processed
   - Auto-parsed from display text: N
   - From manual overrides: N
   - Already in SSM: N
   - **STILL MISSING: N** (target: 0)

2. **Integration test**: Load `gems.json`, run every gem's static stats through `stat_table()` lookup, assert 100% of non-implicit stat IDs resolve to at least one `StatDef`:
   ```rust
   #[test]
   fn test_all_gem_stats_resolve() {
       let gems = load_gems();
       let table = stat_table();
       let mut missing = Vec::new();
       for (name, gem) in &gems {
           for stat_slot in &gem.static_data.stats {
               let Some(stat) = stat_slot.as_ref() else { continue };
               let Some(id) = stat.id.as_ref() else { continue };
               if stat.stat_type.as_deref() == Some("implicit") { continue; }
               if !table.contains_key(id.as_str()) {
                   missing.push((name.clone(), id.clone()));
               }
           }
       }
       assert!(missing.is_empty(), "Unresolved gem stat IDs: {:?}", &missing[..missing.len().min(20)]);
   }
   ```

3. **Regression test**: Compare Fireball / Added Fire Damage Support / Melee Physical Damage Support stats before and after — same `Modifier`s should be produced for the previously-working stats, plus the previously-missing ones now resolve.

4. **Modifier count test**: With the expanded table, the `test_resolve_integration` test from step 3.13 should produce significantly more modifiers (currently most stats silently drop).

### How to Verify Phase 3 is Complete

- [x] RePoE `gems.json` loaded into `GameData.gems` at startup (PoB `Gems.json` + `Skills/*.json` no longer loaded)
- [x] `compute_gem_stats()` produces correct values for Fireball levels 1 and 20
- [x] `apply_stat_conversions()` correctly renames skill-specific stat IDs
- [x] Quality stats computed correctly (divide by 1000, multiply by quality)
- [x] `GemRef` + `always_active` framework in place; `set_active_gem` + `set_gem_always_active` commands registered
- [x] `rebuild_gems()` two-pass implementation: active gem (pass 1) + always-active gems (pass 2)
- [ ] `can_support()` correctly filters support compatibility (TODO 3.7)
- [ ] `resolve_supports()` handles multi-pass type addition (TODO 3.7)
- [ ] `set_active_gem` returns real `BuildStats` (TODO 3.10)
- [ ] `GemInstance` legacy fields removed (`stats: BTreeMap`, `mana_cost`, `SupportCompatEntry`, etc.) (TODO 3.8)
- [ ] Changing a gem level/quality triggers recalculation with updated stats (TODO 3.10)
- [ ] Frontend SkillsTab: add gem, change level/quality, see stats update (TODO 3.11)
- [ ] All PoB-based gem types deleted (`GrantedEffect`, etc.) (TODO 3.12)
- [ ] All tests pass including resolve() integration test (TODO 3.13)
- [ ] `gen_gem_stat_table.ts` codegen tool generates GEM STATS section in `stat_table.rs` (TODO 3.14b)
- [ ] 100% of non-implicit gem stat IDs resolve to at least one `StatDef` (TODO 3.14g)
- [ ] Pre-resolved stats cached on `RePoEGem` at load time, no runtime string lookups (TODO 3.14e)
- [ ] Coverage report shows 0 missing stat IDs after overrides (TODO 3.14g)

---

## Phase 4: Item System

### Goal

Load base items, item mods, and stat metadata from RePoE; load unique items from PoB text; define the full `Item` type system; compute local item stats (weapon DPS, armour values); build item mods into the layered `ModDB`; and expose an `ItemsTab` UI.

### Data Sources

| File | Source | What It Provides |
|---|---|---|
| `data/repoe/base_items.json` | RePoE | 5,052 base items — defences, requirements, implicit mod IDs, tags |
| `data/repoe/mods.json` | RePoE | 39,291 mods — raw stat IDs + min/max + spawn weights + domains |
| `data/repoe/stats.json` | RePoE | 22,757 stat metadata entries — `is_local` classification |
| `data/repoe/stat_translations.json` | RePoE | 11,075 display templates — INVERTED for unique text parsing |
| `data/pob/Uniques/*.json` | PoB | ~1,268 unique items as raw display text strings |

### Data Pipeline

```
RePoE base_items.json  →  BaseItem structs (defences, requirements, implicit mod IDs, tags)
RePoE mods.json        →  ModPool (stat IDs + min/max + spawn weights); also resolves base implicit mod IDs
RePoE stats.json       →  is_local classification for local mod extraction
RePoE stat_translations.json  →  INVERTED into display_text_template → stat_id lookup
PoB Uniques/*.json     →  Raw display text → inverted stat_translations → resolve()
```

### Steps

#### 4.1 — Load RePoE `base_items.json` (`data/bases.rs`)

RePoE `base_items.json` is a flat object keyed by metadata path (e.g. `"Metadata/Items/Armours/BodyArmours/BodyStr1"`). Each entry has a universal flat structure — weapon, armour, and flask fields all exist on every item but are `null` when not applicable.

**Critical data shape notes** (verified against actual data):
- `armour`, `evasion`, `energy_shield` are `{min, max}` objects (NOT flat numbers)
- `tags` is `Vec<String>` (NOT a hashmap)
- `implicits` is `Vec<String>` containing **mod IDs** that reference `mods.json` (NOT display text, NOT nested stat objects)
- `requirements` uses `strength`/`dexterity`/`intelligence`/`level` (NOT `str`/`dex`/`int`)
- `movement_speed` is negative when it's a penalty (e.g. `-3` for body armour)
- `block` is a flat integer (shields only)
- `critical_strike_chance` is stored ×100 (e.g. `650` = 6.5%)
- No `influenceTags`, `socketLimit`, or `subType` fields — these are PoB-only

```rust
/// Deserialized from RePoE base_items.json.
/// All weapon/armour/flask fields exist on every item but are null when N/A.
#[derive(Debug, Deserialize)]
pub struct RePoEBaseItem {
    pub name: String,
    pub item_class: String,
    pub drop_level: u32,
    pub tags: Vec<String>,
    pub implicits: Vec<String>,          // Mod IDs → resolve via mods.json
    pub release_state: String,           // "released", "unreleased", etc.
    pub domain: String,                  // "item", "flask", "jewel", etc.
    pub inventory_width: Option<u32>,
    pub inventory_height: Option<u32>,

    // Requirements (all items)
    #[serde(default)]
    pub requirements: BaseRequirements,

    // Defence fields (null on non-armour)
    pub armour: Option<MinMax>,
    pub evasion: Option<MinMax>,
    pub energy_shield: Option<MinMax>,
    pub ward: Option<MinMax>,
    pub block: Option<u32>,              // Shields only
    pub movement_speed: Option<i32>,     // Negative = penalty

    // Weapon fields (null on non-weapons)
    pub physical_damage_min: Option<f64>,
    pub physical_damage_max: Option<f64>,
    pub attack_time: Option<u32>,        // Milliseconds
    pub critical_strike_chance: Option<u32>,  // ×100 (650 = 6.5%)
    pub range: Option<u32>,

    // Flask fields (null on non-flasks)
    pub charges_max: Option<u32>,
    pub charges_per_use: Option<u32>,
    pub duration: Option<u32>,           // Milliseconds
    pub life_per_use: Option<f64>,
    pub mana_per_use: Option<f64>,
    pub flask_type: Option<String>,      // null, "Life", "Mana", "Hybrid", "Utility"

    // Misc
    pub grants_buff: Option<GrantsBuff>,
}

#[derive(Debug, Deserialize, Default)]
pub struct BaseRequirements {
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub strength: u32,
    #[serde(default)]
    pub dexterity: u32,
    #[serde(default)]
    pub intelligence: u32,
}

#[derive(Debug, Deserialize)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Deserialize)]
pub struct GrantsBuff {
    pub id: String,
    pub stats: FxHashMap<String, f64>,
}
```

**Filtering**: Only load equippable item classes (~1,077 items). Skip `StackableCurrency`, `DivinationCard`, `Map`, `QuestItem`, `Active Skill Gem`, `Support Skill Gem`, etc. Define an `EQUIPPABLE_CLASSES` const with the ~25 relevant classes: `Body Armour`, `Helmet`, `Gloves`, `Boots`, `Shield`, `Bow`, `Claw`, `Dagger`, `Rune Dagger`, `One Hand Sword`, `Thrusting One Hand Sword`, `One Hand Axe`, `One Hand Mace`, `Sceptre`, `Two Hand Sword`, `Two Hand Axe`, `Two Hand Mace`, `Staff`, `Warstaff`, `Wand`, `Amulet`, `Ring`, `Belt`, `Quiver`, `LifeFlask`, `ManaFlask`, `HybridFlask`, `UtilityFlask`, `Jewel`, `AbyssJewel`, `Tincture`, `FishingRod`.

**Socket limit derivation** (not in RePoE — derive from `inventory_height`):
- `height >= 4` → 6 sockets (body armour, 2H weapons)
- `height == 3` → 4 sockets (boots, gloves, helmets)
- `height <= 2` → 3 sockets (1H weapons, shields)
- Jewels/Amulets/Rings/Belts → 0 sockets

Store as `GameData.bases: FxHashMap<String, RePoEBaseItem>` keyed by `name` (not metadata path).

#### 4.2 — Load RePoE `mods.json` (`data/mods.rs`)

39,291 total mods across 35 domains. Each mod has raw stat IDs with min/max ranges.

**Domain filtering**: Only load domains relevant to items. The key domains and their counts:
- `item` (24,334) — regular item affixes
- `crafted` (1,630) — crafting bench mods
- `flask` (437) — flask-specific mods
- `abyss_jewel` (548) — abyss jewel mods
- `unveiled` (257) — unveiled mods
- `tincture` (89) — tincture mods
- `veiled` (20) — veiled mods
- `misc` (768) — some are implicitly referenced by base items

Skip `monster`, `area`, `map_relic`, `watchstone`, `heist_*`, `delve_area`, `atlas`, `synthesis_*`, etc.

```rust
/// Deserialized from RePoE mods.json.
#[derive(Debug, Deserialize)]
pub struct RePoEMod {
    pub domain: String,
    pub generation_type: String,         // "prefix", "suffix", "unique", "corrupted", etc.
    pub groups: Vec<String>,             // Mutual exclusion groups
    pub implicit_tags: Vec<String>,
    pub is_essence_only: bool,
    pub name: String,
    pub required_level: u32,
    pub spawn_weights: Vec<SpawnWeight>,
    pub stats: Vec<ModStat>,             // Raw stat IDs + ranges
    #[serde(default)]
    pub adds_tags: Vec<String>,
    #[serde(default)]
    pub grants_effects: Vec<serde_json::Value>,  // Complex; defer full parsing
}

#[derive(Debug, Deserialize)]
pub struct SpawnWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Debug, Deserialize)]
pub struct ModStat {
    pub id: String,
    pub min: f64,
    pub max: f64,
}
```

Store as `GameData.mods: FxHashMap<String, RePoEMod>` keyed by mod ID.

**How base implicits connect to mods**: A base item's `implicits: ["AccuracyPercentImplicitSword1"]` references a key in `mods.json`. That mod entry contains `stats: [{id: "accuracy_rating_+%", min: 40, max: 40}]`. At equip time, look up each implicit mod ID in the mod registry, extract its stats, and call `resolve(stat_id, value, source)` on each.

#### 4.3 — Load RePoE `stats.json` (`data/stats.rs` or extend `data/mods.rs`)

22,757 entries mapping raw stat IDs to metadata. The only field needed for Phase 4 is `is_local`.

```rust
/// Minimal deserialization — only the fields we need.
#[derive(Debug, Deserialize)]
pub struct StatMeta {
    pub is_local: bool,
}
```

Store as `GameData.stat_meta: FxHashMap<String, StatMeta>`.

Usage: when separating local vs global mods on an equipped item, check `stat_meta[stat_id].is_local`.

#### 4.4 — Item Type System (`item/types.rs`)

Define all types **before** writing any parser — the parser needs a target type to populate.

**Note on `RePoEBaseItem.properties`**: all weapon/armour/flask stats sit inside a nested `properties` object in `base_items.json`, not at the top level. Access via `base.properties.armour`, `base.properties.attack_time`, etc.

##### `Rarity`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}
```

Needs `specta::Type` because it crosses IPC. Add `from_str(s: &str) -> Option<Rarity>` for parsing `"Rarity: Unique"` lines.

##### `ItemType`

Derived from `item_class` string at parse time. Never serialized — the `item_class: String` on `Item` is what crosses IPC.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    // Armour
    BodyArmour, Helmet, Gloves, Boots, Shield,
    // Weapons (1H)
    Claw, Dagger, RuneDagger, OneHandSword, ThrustingOneHandSword,
    OneHandAxe, OneHandMace, Sceptre, Wand,
    // Weapons (2H)
    Bow, TwoHandSword, TwoHandAxe, TwoHandMace, Staff, Warstaff,
    // Accessories
    Amulet, Ring, Belt, Quiver,
    // Flasks
    LifeFlask, ManaFlask, HybridFlask, UtilityFlask,
    // Jewels
    Jewel, AbyssJewel,
    // Other
    Tincture, FishingRod,
}

impl ItemType {
    pub fn from_class(class: &str) -> Option<Self> { /* match class → variant */ }
    pub fn is_weapon(self) -> bool { /* 1H and 2H */ }
    pub fn is_one_handed(self) -> bool { /* Claw, Dagger, RuneDagger, 1HSword, etc. */ }
    pub fn is_two_handed(self) -> bool { /* Bow, 2HSword, etc. */ }
    pub fn is_armour(self) -> bool { /* BodyArmour, Helmet, Gloves, Boots, Shield */ }
    pub fn is_flask(self) -> bool { /* Life/Mana/Hybrid/Utility */ }
    pub fn is_jewel(self) -> bool { /* Jewel, AbyssJewel */ }
    pub fn is_accessory(self) -> bool { /* Amulet, Ring, Belt, Quiver */ }
}
```

Used by: slot validation (`is_compatible`), local mod classification, DPS computation.

##### `InfluenceSet`

```rust
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct InfluenceSet: u8 {
        const SHAPER   = 1 << 0;
        const ELDER    = 1 << 1;
        const CRUSADER = 1 << 2;
        const HUNTER   = 1 << 3;
        const REDEEMER = 1 << 4;
        const WARLORD  = 1 << 5;
    }
}
```

Same `bitflags` pattern as `ModFlag` / `KeywordFlag` in `modifier/types.rs`. Detected from lines like `"Shaper Item"` and `"Elder Item"` in PoE clipboard text; or from `tags` for crafting mod weight lookup (Phase 5+).

##### `Socket` and `SocketColor`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketColor { Red, Green, Blue, White, Abyss }

#[derive(Debug, Clone, Copy)]
pub struct Socket {
    pub colour: SocketColor,
    pub group: u8,   // 0-based linked group; sockets sharing a group are linked
}
```

Parsed from `"Sockets: R-R-G-B R"` — dashes = same group, spaces = new group.

##### `ModLineSource` and `ModLine`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModLineSource { Implicit, Explicit, Crafted, Enchant, Fractured }

#[derive(Debug, Clone)]
pub struct ModLine {
    pub text: String,             // Original display text (for tooltip rendering)
    pub modifiers: Vec<Modifier>, // Parsed modifiers from this one line
    pub is_local: bool,           // Set during Step 4.7 local extraction
    pub source: ModLineSource,
}
```

##### `ItemRequirements`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct ItemRequirements {
    pub level: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
}
```

Copied from `base.requirements` at parse time; can be raised by mods (e.g. "Requires Level 80"). Crosses IPC so needs serde + specta.

##### `WeaponData`

Computed during Step 4.7. All values are post–local-mod, final numbers.

```rust
#[derive(Debug, Clone, Default)]
pub struct WeaponData {
    // From base (properties.physical_damage_min/max, attack_time, critical_strike_chance, range)
    pub phys_min: f64,
    pub phys_max: f64,
    pub attack_time_ms: u32,
    pub range: u32,

    // Computed (after local mods + quality)
    pub attacks_per_second: f64,   // 1000.0 / attack_time_ms * (1 + local_speed_inc/100)
    pub crit_chance: f64,          // properties.critical_strike_chance/100 * (1 + local_crit_inc/100)
    pub phys_dps: f64,
    pub ele_dps: f64,              // fire + cold + lightning combined
    pub chaos_dps: f64,
    pub total_dps: f64,
}
```

##### `ArmourData`

```rust
#[derive(Debug, Clone, Default)]
pub struct ArmourData {
    // Computed (after local mods + quality); null fields stay 0.0
    pub armour: f64,
    pub evasion: f64,
    pub energy_shield: f64,
    pub ward: f64,
    pub block: u32,
    pub movement_speed_penalty: i32,
}
```

Formula per stat: `base.properties.X.max * (1 + quality/100 + inc_pct/100)` — quality and %increased are **additive**, not multiplicative.

##### `FlaskData`

```rust
#[derive(Debug, Clone, Default)]
pub struct FlaskData {
    pub charges_max: u32,
    pub charges_per_use: u32,
    pub duration_ms: u32,
    pub life_per_use: f64,
    pub mana_per_use: f64,
}
```

Copied from `base.properties.*` then modified by local flask mods.

##### `Item` (full struct)

```rust
pub struct Item {
    // Identity
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub item_type: ItemType,
    pub rarity: Rarity,

    // Flags
    pub corrupted: bool,
    pub mirrored: bool,
    pub fractured_count: u32,
    pub synthesised: bool,
    pub influences: InfluenceSet,

    // Level / quality
    pub item_level: u32,
    pub quality: u32,

    // Requirements
    pub requirements: ItemRequirements,

    // Sockets
    pub sockets: Vec<Socket>,

    // Mod lines (parsed from text or resolved from mod IDs)
    pub implicit_lines: Vec<ModLine>,
    pub explicit_lines: Vec<ModLine>,
    pub crafted_lines: Vec<ModLine>,
    pub enchant_lines: Vec<ModLine>,

    // PoB variant support (uniques only)
    pub variant_list: Vec<String>,
    pub selected_variant: usize,

    // Computed local stats (None when not applicable to item type)
    pub weapon_data: Option<WeaponData>,
    pub armour_data: Option<ArmourData>,
    pub flask_data: Option<FlaskData>,

    // Final output: global-only modifiers that enter the items ModDB layer
    // Local mods are removed from this list during Step 4.7
    pub mod_list: Vec<Modifier>,
}
```

##### IPC Summary Types

Only lightweight types cross IPC (avoid sending `Vec<Modifier>` over IPC boundary):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ItemSummary {
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub rarity: Rarity,
    pub corrupted: bool,
    pub item_level: u32,
    pub quality: u32,
    pub influences: u8,          // InfluenceSet bits — u8 is specta-compatible
    pub total_dps: Option<f64>,  // Some for weapons
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
}
```

`get_equipped_items()` returns `FxHashMap<String, ItemSummary>` (slot name → summary). Only fetch the full `Item` (via `equip_item` / parse path) when equipping.



#### 4.5 — Load `stats.json` and Build Inverted `stat_translations` Map

Two startup tasks: (1) load `stats.json` for `is_local` metadata; (2) build the inverted translation map from `stat_translations.json` so that display text lines from PoB unique items can be resolved back to raw stat IDs and values.

---

##### 4.5a — Load `stats.json` → `GameData.stat_meta`

**Data shape**: `src-tauri/data/repoe/stats.json` — 22,757 entries, a JSON object keyed by stat ID string (e.g. `"base_maximum_life"`). Each value has `{ alias, is_aliased, is_local }`.

```rust
#[derive(Debug, Deserialize)]
pub struct StatMeta {
    pub is_local: bool,
    pub is_aliased: bool,
}
```

Add to `GameData`:
```rust
pub stat_meta: FxHashMap<String, StatMeta>,   // keyed by raw stat ID string
```

Load in `DataLoader::load()`:
```rust
let stat_meta = serde_json::from_str::<FxHashMap<String, StatMeta>>(&stat_meta_json)?;
```

**Usage**: during unique item parsing (step 4.6), after resolving a display line to `(stat_id, value)` pairs, look up each stat_id in `stat_meta`. If **any** of the stat IDs for a `ModLine` have `is_local: true`, set `ModLine.is_local = true`. This drives the local stat computation in step 4.7.

---

##### 4.5b — Build `InvertedTranslations` from `stat_translations.json`

**Data shape**: `src-tauri/data/repoe/stat_translations.json` — 11,075 entries, a JSON array where each entry is:

```json
{
  "ids": ["base_maximum_life"],
  "English": [
    {
      "condition": [{ "min": 1, "max": null, "negated": null }],
      "format": ["#"],
      "index_handlers": [[]],
      "string": "+{0} to maximum Life",
      "reminder_text": null,
      "is_markup": null
    }
  ],
  "trade_stats": [...],
  "hidden": null,
  "French": [...], "German": [...], ...
}
```

- **`ids`**: 1–4 raw stat ID strings. 271 entries have 2+ IDs (multi-stat lines).
- **`English[N].string`**: Display template with `{0}`, `{1}`, ... placeholders.
- **`English[N].format`**: Per-placeholder display format: `"#"` (number), `"ignore"` (non-numeric lookup), `"d"` (integer).
- **`English[N].index_handlers`**: Per-placeholder array of handler names. Handlers transform the raw internal value before display. To recover the raw value you apply the inverse.
- **`English[N].condition`**: Per-stat value range (`min`/`max`) that this English variant is valid for (used to select correct variant when multiple exist). 4,511 entries have 2+ English variants.

**Structs** (goes in `item/unique_stats.rs`, already scaffolded):

```rust
pub struct InvertedTranslations {
    /// normalized template string → list of candidates (ambiguous templates possible)
    lookup: FxHashMap<String, Vec<TranslationEntry>>,
}

pub struct TranslationEntry {
    pub stat_ids: Vec<String>,            // 1–4 raw stat ID strings
    pub index_handlers: Vec<Vec<String>>, // per-placeholder handler list
    pub format: Vec<String>,              // per-placeholder: "#", "ignore", "d"
    pub condition: Vec<StatCondition>,    // per-stat condition for variant selection
}

pub struct StatCondition {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub negated: Option<bool>,
}
```

**`InvertedTranslations::build(raw: &str) -> Result<Self>`**:

```
1. Deserialize raw JSON → Vec<RawTranslationEntry>  (private deserialization type)
2. Allocate lookup: FxHashMap::with_capacity(11075 * 2)   // ~2 variants per entry avg
3. For each entry:
   a. For each English variant `t`:
      - Normalize t.string: replace {0}, {1}, ... with "#" using regex `\{[0-9]+\}`
      - Lowercase the normalized string
      - Insert normalized_key → TranslationEntry into lookup
        (push to the Vec if key already exists — handles genuine duplicates)
4. Return InvertedTranslations { lookup }
```

**Important**: store all keys lowercase. The lookup function must also lowercase the PoB line
before lookup. `stat_translations.json` uses mixed case (`"supported"` vs PoB's `"Supported"`)
so case-insensitive matching is required — validated to increase match rate from 89.5% → 91.6%.



Add to `GameData`:
```rust
pub translations: InvertedTranslations,
```

Built once, stored in `GameData` — no rebuild needed between items.

---

##### 4.5c — `InvertedTranslations::resolve_line()`

**Signature**:
```rust
pub fn resolve_line(
    &self,
    line: &str,
    stat_meta: &FxHashMap<String, StatMeta>,
) -> Option<ResolvedLine>

pub struct ResolvedLine {
    pub stats: Vec<(String, f64)>,  // (raw_stat_id, raw_value)
    pub is_local: bool,
}
```

**Algorithm**:
```
Pre-processing (PoB line normalization):
  a. Strip {tag} prefixes: remove everything matching \{[^}]+\}
  b. Trim whitespace
  c. Strip leading "+" (PoB prefixes positive numbers with "+", translations don't)
  d. Normalize em-dashes: replace "–" (U+2013) and "—" (U+2014) with "-"
  e. Lowercase the entire line
  f. Replace parenthesized ranges "(X-Y)" with "#" (regex: \(-?[0-9.]+-[0-9.]+\))
  g. Replace "+N" and "-N" standalone numbers with "#" (regex: [+-][0-9]+(?:\.[0-9]+)?)
  h. Replace remaining plain numbers with "#" (regex: [0-9]+(?:\.[0-9]+)?)

1. Extract numeric values from `line` BEFORE normalization:
   - For parenthesized ranges "(X-Y)": take average (X+Y)/2.0
   - For "+N" or "-N": preserve sign
   - Collect as Vec<f64> in encounter order (positional index matches placeholder index)
2. Normalize line via steps above → lookup key
3. Skip certain line types before lookup:
   - Lines ending in ":" (multi-mod headers like "Every 10 seconds:")
   - Lines starting with "(The " or "(This " (reminder text)
4. Look up lowercase normalized key in lookup; if not found → return None
5. For each candidate TranslationEntry:
   a. For each placeholder i where format[i] == "ignore": skip this position
      (non-numeric display text — passive names, gem names, item class names)
   b. For each numeric placeholder i:
      - recovered_value = extracted_numbers[numeric_index]  (positional among NUMERIC placeholders)
      - Apply inverse handler (see table below) to get raw_value
   c. For each stat_id (positional): check condition[i] against raw_value
      - If condition.min is set and raw_value < condition.min → skip this candidate
      - If condition.max is set and raw_value > condition.max → skip this candidate
   d. If all conditions pass → emit stats: Vec<(stat_id.clone(), raw_value)>
       and is_local = stat_ids.iter().any(|id| stat_meta.get(id).map_or(false, |m| m.is_local))
6. Return the first passing candidate (or None)
```

**Validated coverage** (tested against all 6,379 mod lines across 1,268 PoB uniques):

| Normalization rules applied | Match rate |
|---|---|
| Raw string only | 66.5% |
| + strip leading `+` | 89.5% |
| + case-insensitive | 91.6% |
| + em-dash normalization | ~91.9% |

Remaining 8.1% breakdown:
- **6.9% genuine misses** (442 lines, 251 uniques affected) — these are lines not present in `stat_translations.json` at all. The affected uniques fall into three categories:
  - **Timeless jewels** (Militant Faith, Brutal Restraint, Elegant Hubris, Glorious Vanity, Lethal Pride): 4–19 misses each. Use `passive_hash` handler — the text describes which passives are transformed, not a numeric stat. **Untranslatable regardless; Phase 7 concern.**
  - **Skill grant lines** (`Grants Level 20 Summon Bestial Rhoa Skill`, `50% chance to Cast a Socketed Lightning Spell on Hit`): skill/trigger mechanics. **Phase 3/5 concern.**
  - **A small number of calculatable misses** (`Maximum Endurance, Frenzy and Power Charges is 0`, flat damage-per-stat-per-level lines, etc.) that need a hardcoded override table (< 20 items, < 50 lines). Add as `UNIQUE_OVERRIDES: &[(&str, &str, f64)]` in `item/parser.rs`.
- **1.2% flag mods** (74 lines): zero-number lines (`Your Critical Strikes have Culling Strike`, `Lightning Damage from Enemies Hitting you is Lucky`). Not in `stat_translations`. Set as FLAG modifiers via a separate flag-lookup table in the parser.
- **0.5% skippable** (header/reminder text, em-dash, flat negative values)



**Index handler inverse table** (49 handlers total):

| Handler | Display formula | Inverse (raw → recovered) |
|---|---|---|
| *(none)* | value as-is | value |
| `negate` | `-value` | `×(-1)` |
| `divide_by_one_hundred` | `value/100` | `×100` |
| `divide_by_one_hundred_2dp` | `value/100` | `×100` |
| `divide_by_one_hundred_2dp_if_required` | `value/100` if fractional | `×100` |
| `divide_by_one_hundred_and_negate` | `-value/100` | `×(-100)` |
| `divide_by_ten_0dp` | `value/10` | `×10` |
| `divide_by_ten_1dp` | `value/10` | `×10` |
| `divide_by_ten_1dp_if_required` | `value/10` if fractional | `×10` |
| `divide_by_five` | `value/5` | `×5` |
| `divide_by_four` | `value/4` | `×4` |
| `divide_by_three` | `value/3` | `×3` |
| `divide_by_two_0dp` | `value/2` | `×2` |
| `divide_by_six` | `value/6` | `×6` |
| `divide_by_twelve` | `value/12` | `×12` |
| `divide_by_fifteen_0dp` | `value/15` | `×15` |
| `divide_by_twenty` | `value/20` | `×20` |
| `divide_by_one_thousand` | `value/1000` | `×1000` |
| `double` | `value×2` | `÷2` |
| `negate_and_double` | `-value×2` | `÷(-2)` |
| `milliseconds_to_seconds` | `value/1000` | `×1000` |
| `milliseconds_to_seconds_0dp` | `value/1000` | `×1000` |
| `milliseconds_to_seconds_1dp` | `value/1000` | `×1000` |
| `milliseconds_to_seconds_2dp` | `value/1000` | `×1000` |
| `milliseconds_to_seconds_2dp_if_required` | `value/1000` | `×1000` |
| `deciseconds_to_seconds` | `value/10` | `×10` |
| `per_minute_to_per_second` | `value/60` | `×60` |
| `per_minute_to_per_second_0dp` | `value/60` | `×60` |
| `per_minute_to_per_second_1dp` | `value/60` | `×60` |
| `per_minute_to_per_second_2dp` | `value/60` | `×60` |
| `per_minute_to_per_second_2dp_if_required` | `value/60` | `×60` |
| `permyriad_per_minute_to_%_per_second` | `value/6000` | `×6000` |
| `times_twenty` | `value×20` | `÷20` |
| `times_one_point_five` | `value×1.5` | `÷1.5` |
| `30%_of_value` | `value×0.3` | `÷0.3` |
| `60%_of_value` | `value×0.6` | `÷0.6` |
| `plus_two_hundred` | `value+200` | `-200` |
| `multiplicative_damage_modifier` | `(value-100)` displayed | `+100` |
| `old_leech_percent` | `value/5` | `×5` |
| `old_leach_permyriad` | `value/50` | `×50` |
| `locations_to_metres` | `value/10` | `×10` |
| `divide_by_twenty_then_double_0dp` | `(value/20)×2` | `×10` |
| **Skip (display-only — no number to invert):** |||
| `passive_hash` | jewel passive ID lookup | skip placeholder |
| `tree_expansion_jewel_passive` | jewel expansion passive | skip placeholder |
| `mod_value_to_item_class` | item class name lookup | skip placeholder |
| `canonical_stat` | canonical stat name | skip placeholder |
| `display_indexable_skill` | skill name lookup | skip placeholder |
| `display_indexable_support` | support gem name lookup | skip placeholder |
| `weapon_tree_unique_base_type_name` | base type name lookup | skip placeholder |
| `affliction_reward_type` | affliction name lookup | skip placeholder |

Skip-placeholder handlers match lines where `format[i] = "ignore"`. These placeholders contain a string index rather than a number; they don't affect numeric stat values and can be ignored for PoB item parsing.

---

##### 4.5d — `"ignore"` format placeholders

When a placeholder has `format = "ignore"` the display string substitutes a lookup string (e.g. a charge type name, an item class name, a passive skill name) rather than a number. These lines still appear in PoB unique text but the non-numeric slots should be skipped during number extraction. The numeric placeholder positions still work normally; only the non-numeric positions need to be excluded from `extracted_numbers`.

Example: `"Bathed in the blood of {1} sacrificed in the name of Xibaqua\nPassives in radius are Conquered by the Vaal"` — placeholder `{1}` has `format = "ignore"`, placeholder `{0}`, `{2}`, `{3}` have `format = "ignore"` too; only one placeholder in this variant is numeric. The text between the literal `\n` and the number tokens is what you match.

---

##### 4.5e — Multi-stat lines

271 entries have 2–4 stat IDs. The extracted values are assigned positionally: `extracted_numbers[0] → stat_ids[0]`, etc. Each stat ID has its own `condition[i]` and `index_handlers[i]`. Emit one `(stat_id, raw_value)` per ID, all into the same `ResolvedLine.stats`.

---

##### 4.5f — Disambiguation (multiple English variants)

4,511 entries have 2+ English variants. Example — same stat ID, different variants for increased vs reduced:
```
Variant A: "Socketed Gems have {0}% increased Reservation Efficiency"
  → condition[0].max = -1, handler = ["negate"]
Variant B: "Socketed Gems have {0}% reduced Reservation Efficiency"
  → condition[0].min = 1, handler = []
```

Both normalize to `"Socketed Gems have #% increased Reservation Efficiency"` and `"Socketed Gems have #% reduced Reservation Efficiency"` — **different** normalized strings, so they are stored under different keys. The disambiguation in step 4.5c is only needed when two variants produce the **same** normalized key (e.g. `"# to maximum Life"` matching a stat where `min = 1` vs `min = -1`). In that case, use `condition` to pick the matching candidate.

#### 4.6 — Parse PoB Unique Text (`data/uniques.rs` + `item/parser.rs`)

PoB's `Uniques/*.json` are arrays of raw text strings with literal `\n` as line delimiters inside each JSON string. 22 files (graft.json is an empty array), ~1,268 uniques total.

**Verified text format** (from data investigation):

```
Wings of Entropy\n                       ← line[0]: unique name (always plain)
{variant:1,2,3,4}Sundering Axe\n        ← line[1..M]: one or more base type lines
{variant:5}Ezomyte Axe\n                    (may have {variant:N,M} prefix; plain if single base)
Variant: Pre 1.3.0\n                     ← metadata lines (any order, any count)
Variant: Current\n
Implicits: 0\n                           ← Implicits boundary — exactly this many mod lines follow
{tags:physical,attack}(250-300)% increased Physical Damage\n  ← mod lines start here
{variant:2,3,5}(8-12)% increased Attack Speed\n
```

Multi-base example (Bloodgrip amulet):
```
Bloodgrip\n
{variant:1}Coral Amulet\n
{variant:2,3}Marble Amulet\n
Variant: Pre 3.0.0\n
Variant: Pre 3.12.0\n
Variant: Current\n
Requires Level 74\n
Implicits: 2\n
(implicit 1)\n
(implicit 2)\n
(explicit mods follow)\n
```

**Key structural rules** (verified from full dataset):
- `Implicits: N` separates header from mod region — **no `--------` separator exists anywhere**
- After `Implicits: N`, the next N lines are implicits; all subsequent lines are explicits
- `Implicits: 0` items have 0 implicit lines — explicit mods begin immediately after
- Line[0] is always the plain unique name (no tags)
- Lines 1..M (before any non-`{variant:}` content) are base type lines — stop at first line that does NOT start with `{variant:`
- `{variant:N,...}` prefix on metadata lines (e.g. `{variant:1,2,3}LevelReq: 24`) — check variant before using
- All `{...}` tags appear only at the very start of a line, never mid-line (Source: lines can contain `{` mid-text but these are not PoB tags)

**Prefix tag types on mod lines**:
| Tag | Meaning |
|---|---|
| `{variant:1,2}` | Only apply when `selected_variant` is in the list |
| `{tags:attack,physical}` | Mod tags for damage breakdown (carry through as `ModTag`) |
| `{crafted}` | Crafted mod (display teal) |
| `{fractured}` | Fractured mod |

**Complete metadata skip list** (lines to ignore, after stripping any leading `{variant:N}` prefix):
- `Variant: …` — variant description labels
- `League: …` — league restriction
- `Source: …` — drop source note
- `Requires Level …`, `LevelReq: …` — requirements
- `Requires …` (Strength/Dexterity/Intelligence requirements)
- `Implicits: N` — handled as boundary, not skipped mid-flow
- `Has Alt Variant: …`, `Has Alt Variant Two: …`, `Has Alt Variant Three: …`
- `Selected Variant: …`, `Selected Alt Variant: …`, `Selected Alt Variant Two: …`
- `Has no Sockets`
- `Sockets: …`
- `Radius: …`
- `Talisman Tier: …`
- `Limited to: …`
- `Upgrade: …`
- `Notable` (skill tree note)
- Class-specific lines: `Duelist`, `Marauder`, `Ranger`, `Shadow`, `Witch`, `Templar`, `Scion`
- Influence lines (plain text): `Shaper Item`, `Elder Item`, `Crusader Item`, `Hunter Item`, `Redeemer Item`, `Warlord Item`, `Searing Exarch Item`, `Eater of Worlds Item`, `Synthesised Item`, `Fractured Item`, `Corrupted Item`, `Mirrored Item`

**Algorithm** (`parse_unique_item(raw, selected_variant, game_data) → Item`):

```
1. Split raw on '\n', collect non-empty lines
2. name = lines[0]
3. base_name = pick base type line for selected_variant:
     i = 1
     while i < len && lines[i] starts with '{':
         strip leading {variant:N,...} prefix
         if variant matches selected_variant (or no variant tag) → this is the base
         i++
4. Scan remaining lines for metadata, collecting:
     - variant_count / names from 'Variant: X' lines
     - influences: InfluenceSet from 'Shaper Item' / 'Elder Item' / etc.
     - talisman_tier from 'Talisman Tier: N'
     - upgrade_target from 'Upgrade: X'
5. Find 'Implicits: N' line → implicit_count = N
6. mod_lines = lines after 'Implicits:' line; first implicit_count are implicits
7. For each mod line:
     a. Extract all leading {tag} blocks (stop at first non-{} char)
     b. If has {variant:N,...}: skip if selected_variant not in list
     c. strip_tags → raw_text
     d. call InvertedTranslations::resolve_line(raw_text, &game_data.stat_meta)
     e. If None → try parse_display_text() fallback → if still None, log+skip
     f. Map resolved stats to Modifier structs with correct ModType and SourceId
```

**Lazy parse**: At startup (`data/uniques.rs`), only extract lightweight metadata:

```rust
pub struct UniqueItemDef {
    pub name: String,
    pub base_names: Vec<(Vec<usize>, String)>,  // (variant_indices, base_name)
    pub league: Option<String>,
    pub variant_labels: Vec<String>,             // e.g. ["Pre 3.0.0", "Current"]
    pub has_alt_variant: bool,
    pub talisman_tier: Option<u32>,
    pub upgrade_target: Option<String>,
    pub influences: u8,                          // InfluenceSet bitflags
    pub raw_text: String,                        // Original text for on-demand full parse
    pub file_source: String,                     // e.g. "amulet.json"
}
```

`UniqueItemDef::variant_count()` returns `variant_labels.len().max(1)`.
`UniqueItemDef::base_for_variant(v)` returns the matching base name for variant index `v` (1-based).

Store as `GameData.uniques: Vec<UniqueItemDef>`.

#### 4.7 — Local Mod Extraction & Computed Stats

After building an item's modifier list, separate local mods from global mods:
- **Local** mods (weapon attack speed, armour defences, flask duration) modify the item's computed stats
- **Global** mods go into the `items` ModDB layer

**Classification**: For each modifier's source stat ID, check `GameData.stat_meta[stat_id].is_local`. Also check item context — some stats are local only when on specific item types (e.g. `attack_speed_+%` is local on weapons but global on rings). PoB's `calcLocal()` logic is the reference.

**Weapon DPS calculation**:
1. Base phys = `physical_damage_min/max` from `RePoEBaseItem`
2. Add flat phys from local mods
3. Apply `(1 + quality/100 + inc_phys%/100)` — quality and %inc are **additive** with each other
4. Attack 0 rate = `1000 / attack_time * (1 + local_speed_inc%/100)`
5. Crit = `critical_strike_chance / 100 * (1 + local_crit_inc%/100)`
6. Physical DPS = `(phys_min + phys_max) / 2 * attack_rate`
7. Elemental/chaos damage: flat adds only (no quality bonus), same DPS formula

**Armour/evasion/ES calculation**: `base_max * (1 + quality/100 + inc%/100)` — quality and %inc additive.

**After computation**: `Vec::retain()` to remove local mods from `item.mod_list`. Only global mods enter the ModDB.

#### 4.8 — Equipment Manager & ModDB Layer

**`ItemSlot` enum** — add to `item/types.rs` with all derives needed for use as a HashMap key and for IPC:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, specta::Type)]
#[repr(u8)]
pub enum ItemSlot {
    Weapon1 = 0, Weapon2 = 1,
    Helmet = 2, BodyArmour = 3, Gloves = 4, Boots = 5,
    Amulet = 6, Ring1 = 7, Ring2 = 8, Belt = 9,
    Flask1 = 10, Flask2 = 11, Flask3 = 12, Flask4 = 13, Flask5 = 14,
}
```

Weapon swap slots deferred to Phase 7.

**Equipment state in `BuildInfo`** — add alongside `mod_db_layers`:

```rust
#[serde(skip)]
#[specta(skip)]
pub equipped: FxHashMap<ItemSlot, Item>,
```

`Item` is a large runtime type; it is not serialized to the frontend. Summaries are sent via the `get_equipped_items` command instead.

**`ModDBLayers`** — uncomment and add the `items` layer, update `merged()`:

```rust
pub items: ModDB,   // Phase 4 — replace "// Phase 4+:" comment

fn rebuild_items(&mut self, equipped: &FxHashMap<ItemSlot, Item>) {
    self.items = ModDB::new();
    for item in equipped.values() {
        for modifier in &item.mod_list {
            self.items.add_mod(modifier.clone());
        }
    }
}
```

`merged()` must also include `combined.merge(&self.items)`.

**Slot compatibility** (`is_compatible(slot, item, weapon1_class: Option<&str>) → bool`):

| Slot | Accepted `item_class` values |
|---|---|
| `Weapon1` | all weapon classes (1H + 2H) |
| `Weapon2` | Shield, Quiver, or any 1H weapon class (only when `Weapon1` is 1H) |
| `Helmet` | Helmet |
| `BodyArmour` | Body Armour |
| `Gloves` | Gloves |
| `Boots` | Boots |
| `Amulet` | Amulet |
| `Ring1`, `Ring2` | Ring |
| `Belt` | Belt |
| `Flask1`–`Flask5` | LifeFlask, ManaFlask, HybridFlask, UtilityFlask |

1H weapon classes: Claw, Dagger, Rune Dagger, One Hand Sword, Thrusting One Hand Sword, One Hand Axe, One Hand Mace, Sceptre, Wand.
2H weapon classes: Bow, Two Hand Sword, Two Hand Axe, Two Hand Mace, Staff, Warstaff, FishingRod.

When `Weapon1` is a 2H weapon (or a Bow), `Weapon2` slot is locked (reject any equip attempt with an error).

#### 4.9 — Tauri Commands & Frontend

**IPC types** — add to `lib.rs` with `Serialize + Deserialize + Type`:

```rust
/// Lightweight unique info for the search results list.
pub struct UniqueSearchResult {
    pub name: String,
    pub base_name: String,   // base for variant 1
    pub league: Option<String>,
    pub variant_count: usize,
}

/// Per-slot summary sent to the frontend — avoids serializing the full Item.
pub struct EquippedItemSummary {
    pub slot: item::types::ItemSlot,
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub total_dps: Option<f64>,     // weapons only
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
    pub ward: Option<f64>,
    pub selected_variant: usize,
    pub variant_count: usize,
    pub influences: u8,             // InfluenceSet bits
}
```

The full `mod_list` for tooltip display is not sent; a separate `get_item_mods(slot)` command can be added in Phase 9 if needed.

**Commands** (all in `lib.rs`, registered in `collect_commands![]`):

| Command | Signature | Notes |
|---|---|---|
| `equip_item` | `(slot: ItemSlot, unique_name: String, variant: usize) → Result<BuildStats, String>` | Looks up `game_data.uniques` by name, validates slot, parses on-demand, stores in `build.equipped`, calls `rebuild_items`, returns updated stats |
| `unequip_item` | `(slot: ItemSlot) → Result<BuildStats, String>` | Removes from map, rebuilds items layer, returns stats |
| `get_equipped_items` | `() → Result<Vec<EquippedItemSummary>, String>` | Returns summaries for all currently occupied slots |
| `search_uniques` | `(query: String) → Result<Vec<UniqueSearchResult>, String>` | Case-insensitive substring match on `def.name`; returns up to 50 results |
| `get_base_items` | `(item_class: Option<String>) → Result<Vec<BaseItemSummary>, String>` | Filters `game_data.bases` by item_class if provided |

`equip_item` also updates `DebugStatsResponse` — add `items_mods: Vec<DebugModEntry>` to that struct so the debug page shows item modifiers.

**`BaseItemSummary`** (for `get_base_items`):
```rust
pub struct BaseItemSummary {
    pub name: String,
    pub item_class: String,
    pub drop_level: u32,
}
```

**Frontend — `/items` route** (`src/routes/items/+page.svelte`):

Uses the same `Header` + `Sidebar` shell as `/skills`. Two-column body:

- **Left** — equipment slot grid matching PoE's layout (helmet top-center, weapon slots flanking body armour, ring/amulet/belt row, flask bar). Each slot is a clickable button showing the equipped item summary (name, DPS/defence value) or an empty-slot placeholder. Click an occupied slot → call `unequipItem(slot)`.

- **Right** — unique search panel: text input → `searchUniques(query)` on input → results list. Click a result → show a variant selector (if `variant_count > 1`) then call `equipItem(selectedSlot, name, variant)`.

**`buildState.svelte.ts` additions**:
```ts
let equippedItems = $state<EquippedItemSummary[]>([]);
let selectedEquipSlot = $state<ItemSlot | null>(null);
```

Navigation: add a `goto('/items')` button in the Sidebar alongside the existing Menu button.

### How to Verify Phase 4 is Complete

- [ ] RePoE `base_items.json` loaded and filtered to ~1,077 equippable items
- [ ] RePoE `mods.json` loaded with domain filtering (~27K relevant mods)
- [ ] RePoE `stats.json` loaded for `is_local` classification
- [ ] Inverted `stat_translations` map built at startup (11,075 entries)
- [ ] PoB unique text parsed into `Item` structs with correct modifiers
- [ ] Local mod extraction computes correct weapon DPS / armour values
- [ ] Equipping items adds global mods to the `items` ModDB layer
- [ ] Frontend ItemsTab functional end-to-end
- [ ] Test: equip a weapon → verify DPS matches PoB calculation
- [ ] Test: equip a unique → verify stat totals change correctly

---

## Phase 5: Full Calculation Engine

### Goal

Port PoB's ~10,000+ lines of calculation Lua (CalcPerform, CalcSetup, CalcDefence, CalcOffence) into Rust. This is the largest and most complex phase. It aggregates all modifier sources (tree, class, gems, items) and computes every character stat: life, mana, ES, resistances, armour, evasion, DPS, ailments, and more.

The target is full calculation parity with Path of Building — given the same tree/gear/gems, we produce identical numbers (within rounding tolerance of ±1).

### Architecture Overview

**New module**: `src-tauri/src/calc/`

```
calc/
├── mod.rs          # Re-exports, CalcResult type
├── context.rs      # CalcContext expansion, context builder
├── setup.rs        # Base value seeding (initModDB equivalent), layer merge
├── perform.rs      # Orchestrator — 8-step calculation flow
├── attributes.rs   # Attribute calc, bonuses, conditions
├── defence.rs      # Life/mana/ES/armour/evasion/resist/block/regen/leech
├── offence.rs      # Damage pipeline: base → conversion → inc/more → crit → DPS
└── conversion.rs   # Damage conversion chain (Phys → Lightning → Cold → Fire → Chaos)
```

**Data flow**:
```
ModDBLayers.merged()  ──►  CalcSetup (seed base values)
                             │
                             ▼
                          CalcPerform (orchestrator)
                             │
                    ┌────────┼────────┐
                    ▼        ▼        ▼
               Attributes  Defence  Offence
                    │        │        │
                    └────────┼────────┘
                             ▼
                         CalcResult
                             │
                       IPC → Frontend
```

**Key principle**: The `ModDB` and all query methods already exist from Phase 2. Phase 5 does NOT modify their signatures. It fills in the `matches_context()` stub and orchestrates the queries in the correct order.

**PoB Reference**: The calculation flow mirrors PoB's `CalcPerform.lua` → `calcs.perform()`, which documents a 10-step pipeline. We simplify steps 2–3 (minion skills → Phase 7), step 4 (flasks → Phase 7), and steps 9–10 (partial buff processing → Phase 7).

### Steps

#### 5.1 — `CalcContext` Expansion (`calc/context.rs`)

This must come first because every subsequent calculation depends on context-aware modifier queries. Currently `CalcContext` has only `flags: ModFlag` and `key_flags: KeywordFlag`, and `matches_context()` always returns `true`. This is the critical gap.

**Expanded `CalcContext`**:

```rust
pub struct CalcContext {
    /// ModFlag bits — ATTACK, SPELL, HIT, DOT, MELEE, RANGED, etc.
    pub flags: ModFlag,
    /// KeywordFlag bits — PHYSICAL, FIRE, COLD, LIGHTNING, CHAOS
    pub key_flags: KeywordFlag,
    /// Boolean conditions: "LowLife", "FullLife", "Leeching", "UsingShield", etc.
    pub conditions: FxHashMap<&'static str, bool>,
    /// Numeric multipliers: "PowerCharge" → count, "EnduranceCharge" → count, etc.
    pub multipliers: FxHashMap<&'static str, f64>,
    /// Pre-computed stat values for PerStat/StatThreshold/PercentStat tags
    pub stat_values: FxHashMap<StatId, f64>,
}
```

**`matches_context()` implementation** — replace the `true` stub in `mod_db.rs`. This evaluates whether a modifier applies given the current context by checking its `flags`, `keywords`, and `tags`:

```rust
fn matches_context(&self, modifier: &Modifier, ctx: &CalcContext) -> bool {
    // 1. Flag matching: if modifier has flags, ctx must have ALL of them
    if !modifier.flags.is_empty() && !ctx.flags.contains(modifier.flags) {
        return false;
    }
    // 2. Keyword matching: if modifier has keywords, ctx must have ALL of them
    if !modifier.keywords.is_empty() && !ctx.key_flags.contains(modifier.keywords) {
        return false;
    }
    // 3. Tag matching: ALL tags must pass
    for tag in &modifier.tags {
        match tag {
            ModTag::Condition(var) => {
                if !ctx.conditions.get(var).copied().unwrap_or(false) {
                    return false;
                }
            }
            ModTag::ModFlagOr(flags) => {
                let flags = ModFlag::from_bits_truncate(*flags);
                if !ctx.flags.intersects(flags) {
                    return false;
                }
            }
            ModTag::SkillType(skill_type_id) => {
                // Phase 7: check against active skill's type flags
                // For now, skip (always passes)
            }
            // Tags that SCALE value rather than gatekeep are handled in
            // effective_value(), not here. Condition is the only gatekeep tag.
            _ => {}
        }
    }
    true
}
```

**Value scaling** — Some `ModTag` variants don't gatekeep (pass/fail), they scale the modifier's value. These are evaluated in a separate `effective_value()` helper:

```rust
fn effective_value(&self, modifier: &Modifier, ctx: &CalcContext) -> f64 {
    let mut value = modifier.value;
    for tag in &modifier.tags {
        match tag {
            ModTag::Multiplier(var) => {
                let mult = ctx.multipliers.get(var).copied().unwrap_or(0.0);
                value *= mult;
            }
            ModTag::MultiplierThreshold { var, threshold } => {
                let mult = ctx.multipliers.get(var).copied().unwrap_or(0.0);
                if mult < *threshold { return 0.0; }
                // gate only — value unchanged
            }
            ModTag::PerStat { stat, div } => {
                let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                value *= (stat_val / div).floor();
            }
            ModTag::PercentStat { stat } => {
                let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                value = stat_val * value / 100.0;
            }
            ModTag::StatThreshold { stat, threshold } => {
                let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                if stat_val < *threshold as f64 {
                    return 0.0; // Below threshold — no effect
                }
            }
            // Condition, ModFlagOr, SkillType handled in matches_context()
            // GlobalEffect, SlotName, ActorCondition — Phase 7
            _ => {}
        }
    }
    value
}
```

**Integration**: All `ModDB` query methods (`sum_base`, `sum_inc`, `product_more`, etc.) must call `matches_context()` for gating and `effective_value()` for the actual value. Currently they use `modifier.value` directly — they need to switch to `self.effective_value(modifier, ctx)`.

**Context builders** — helper functions to create contexts for different calculation scopes:

```rust
impl CalcContext {
    /// For defence calculations (no attack/spell flags)
    pub fn defence() -> Self { Self::empty() }

    /// For attack offence calculations
    pub fn attack(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::ATTACK | ModFlag::HIT,
            key_flags: keywords,
            ..Self::empty()
        }
    }

    /// For spell offence calculations
    pub fn spell(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::SPELL | ModFlag::HIT,
            key_flags: keywords,
            ..Self::empty()
        }
    }

    /// For DoT calculations
    pub fn dot(keywords: KeywordFlag) -> Self {
        Self {
            flags: ModFlag::DOT,
            key_flags: keywords,
            ..Self::empty()
        }
    }
}
```

**PoB equivalent**: PoB passes `cfg` (a table with `flags`, `keywordFlags`, `slotName`, etc.) to every `ModDB:Sum()`, `ModDB:More()`, `ModDB:Flag()` call. Our `CalcContext` is the Rust equivalent.

#### 5.2 — `CalcSetup` (`calc/setup.rs`)

Before any calculations begin, the merged `ModDB` needs base values seeded — resist caps, charge caps, leech rates, resistance penalties, and per-level stat contributions. This is the Rust equivalent of PoB's `calcs.initModDB()` in `CalcSetup.lua`.

**Currently**: `ModDBLayers::merged()` just combines the 4 layers (tree, class, gems, items). It does NOT add any base game constants.

**What needs to happen**:

```rust
pub fn setup_moddb(layers: &ModDBLayers, level: u32) -> ModDB {
    let mut db = layers.merged();
    seed_base_values(&mut db, level);
    db
}
```

**`seed_base_values()` — inject PoE game constants** (from PoB's `initModDB`):

```rust
fn seed_base_values(db: &mut ModDB, level: u32) {
    let src = SourceId(0); // "Base"

    // --- Resource pool bases ---
    // Life: 38 base + 12 per level (PoB: initModDB)
    db.add_mod(Modifier::new(StatId::Life, ModType::Base, 38.0 + 12.0 * level as f64, src));
    // Mana: 34 base + 6 per level (PoB: initModDB, base=34)
    db.add_mod(Modifier::new(StatId::Mana, ModType::Base, 34.0 + 6.0 * level as f64, src));

    // --- Resistance caps (default 75%) ---
    db.add_mod(Modifier::new(StatId::FireResistMax, ModType::Base, 75.0, src));
    db.add_mod(Modifier::new(StatId::ColdResistMax, ModType::Base, 75.0, src));
    db.add_mod(Modifier::new(StatId::LightningResistMax, ModType::Base, 75.0, src));
    db.add_mod(Modifier::new(StatId::ChaosResistMax, ModType::Base, 75.0, src));

    // --- Resistance penalties (Merciless = -60 to each, except Chaos = -20 before 3.15) ---
    // PoE 3.15+ changed Chaos penalty to -60. Use -60 for all per PoB.
    db.add_mod(Modifier::new(StatId::FireResist, ModType::Base, -60.0, src));
    db.add_mod(Modifier::new(StatId::ColdResist, ModType::Base, -60.0, src));
    db.add_mod(Modifier::new(StatId::LightningResist, ModType::Base, -60.0, src));
    db.add_mod(Modifier::new(StatId::ChaosResist, ModType::Base, -60.0, src));

    // --- Block caps ---
    db.add_mod(Modifier::new(StatId::BlockChanceMax, ModType::Base, 75.0, src));
    db.add_mod(Modifier::new(StatId::SpellBlockChanceMax, ModType::Base, 75.0, src));

    // --- Charge maximums ---
    db.add_mod(Modifier::new(StatId::PowerChargesMax, ModType::Base, 3.0, src));
    db.add_mod(Modifier::new(StatId::FrenzyChargesMax, ModType::Base, 3.0, src));
    db.add_mod(Modifier::new(StatId::EnduranceChargesMax, ModType::Base, 3.0, src));

    // --- Leech caps ---
    db.add_mod(Modifier::new(StatId::MaxLifeLeechRate, ModType::Base, 20.0, src));   // % of max life/sec
    db.add_mod(Modifier::new(StatId::MaxManaLeechRate, ModType::Base, 20.0, src));
    db.add_mod(Modifier::new(StatId::MaxLifeLeechInstance, ModType::Base, 10.0, src)); // % per instance
    db.add_mod(Modifier::new(StatId::MaxManaLeechInstance, ModType::Base, 10.0, src));
    db.add_mod(Modifier::new(StatId::MaxEnergyShieldLeechRate, ModType::Base, 10.0, src));
    db.add_mod(Modifier::new(StatId::MaxEnergyShieldLeechInstance, ModType::Base, 10.0, src));

    // --- Damage reduction cap ---
    db.add_mod(Modifier::new(StatId::DamageReductionMax, ModType::Base, 90.0, src));

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
```

**New `StatId` variants needed**: Some of these stats may not yet exist in the `StatId` enum. The following should be added if missing:
- `FireResistMax`, `ColdResistMax`, `LightningResistMax`, `ChaosResistMax`
- `BlockChanceMax`, `SpellBlockChanceMax`
- `PowerChargesMax`, `FrenzyChargesMax`, `EnduranceChargesMax`
- `MaxLifeLeechRate`, `MaxManaLeechRate`, `MaxLifeLeechInstance`, `MaxManaLeechInstance`
- `MaxEnergyShieldLeechRate`, `MaxEnergyShieldLeechInstance`
- `DamageReductionMax`
- `EnergyShieldRecharge` (for ES recharge rate)
- `Ward`

Check `stat_id.rs`for existing coverage before adding. New variants get appended at the end of the enum.

**Bug fix**: The correct PoB value is **34** base mana (`base=34` in `initModDB`, not 40). The inline formula in `update_selected_nodes` will be replaced by the calc module, which uses `seed_base_values` with the correct constant.

**PoB equivalent**: `calcs.initModDB()` in `CalcSetup.lua` (~lines 1-150). We omit flask, minion-specific, and party seeding (Phase 7).

#### 5.3 — `CalcPerform` — The Orchestrator (`calc/perform.rs`)

This is the heart of the engine. It calls all other calc modules in the correct order.

**PoB's `calcs.perform()` documents a 10-step pipeline** (from the comment block in `CalcPerform.lua`):

1. Merge keystone modifiers
2. Initialise minion skills
3. Initialise main skill's minion
4. Merge flask effects
5. Set conditions and calculate attributes (`doActorAttribsConditions`)
6. Calculate life and mana (`doActorLifeMana`)
7. Calculate reservations
8. Set life/mana reservation (`doActorLifeManaReservation`)
9. Process buffs and debuffs
10. Process charges and misc buffs (`doActorCharges`, `doActorMisc`)

Then: `calcs.defence()`, `calcs.triggers()`, `calcs.offence()`

**Our Phase 5 adaptation** (steps 2–4, 7–10 are simplified or deferred):

```rust
pub fn calculate(
    layers: &ModDBLayers,
    level: u32,
    class: &Class,
    active_gem: Option<&GemRef>,
    skill_groups: &[SkillGroup],
    equipped: &FxHashMap<ItemSlot, Item>,
    game_data: &GameData,
) -> CalcResult {
    // Step 1: Merge all layers + seed base values
    let mut db = setup::setup_moddb(layers, level);

    // Step 2: Calculate attributes (two-pass for circular deps)
    let attrs = attributes::calc_attributes(&db, level, class);

    // Step 3: Inject attribute bonuses into the ModDB
    attributes::inject_attribute_bonuses(&mut db, &attrs);

    // Step 4: Set conditions (weapon-based, attribute comparisons, etc.)
    let conditions = attributes::determine_conditions(&db, &attrs, equipped);

    // Step 5: Build base CalcContext for defence
    let mut ctx = CalcContext::empty();
    ctx.conditions = conditions;

    // Step 6: Process charges and inject multipliers
    let charges = charges::process_charges(&db, &ctx);
    ctx.multipliers = charges.multipliers;

    // Step 7: Calculate defence (life, mana, ES, armour, evasion, resists, etc.)
    let defence = defence::calc_defence(&db, level, &ctx);

    // Step 8: Update context stat_values with computed pool sizes
    ctx.stat_values.insert(StatId::Life, defence.life as f64);
    ctx.stat_values.insert(StatId::Mana, defence.mana as f64);
    ctx.stat_values.insert(StatId::EnergyShield, defence.energy_shield as f64);
    ctx.stat_values.insert(StatId::Str, attrs.strength as f64);
    ctx.stat_values.insert(StatId::Dex, attrs.dexterity as f64);
    ctx.stat_values.insert(StatId::Int, attrs.intelligence as f64);

    // Step 9: Calculate offence (DPS, crit, speed, etc.)
    let offence = if let Some(gem_ref) = active_gem {
        offence::calc_offence(&db, gem_ref, skill_groups, game_data, &ctx, &attrs, equipped)
    } else {
        OffenceResult::default()
    };

    CalcResult {
        defence,
        offence,
        attributes: attrs,
    }
}
```

**`CalcResult`** — replaces the currently minimal `BuildStats` (6 fields):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CalcResult {
    pub defence: DefenceResult,
    pub offence: OffenceResult,
    pub attributes: AttributeResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AttributeResult {
    pub strength: i32,
    pub dexterity: i32,
    pub intelligence: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct DefenceResult {
    pub life: u32,
    pub mana: u32,
    pub energy_shield: u32,
    pub armour: u32,
    pub evasion: u32,
    pub ward: u32,
    pub fire_resist: i32,          // Can be negative
    pub fire_resist_cap: i32,
    pub fire_resist_overcap: i32,
    pub cold_resist: i32,
    pub cold_resist_cap: i32,
    pub cold_resist_overcap: i32,
    pub lightning_resist: i32,
    pub lightning_resist_cap: i32,
    pub lightning_resist_overcap: i32,
    pub chaos_resist: i32,
    pub chaos_resist_cap: i32,
    pub chaos_resist_overcap: i32,
    pub block_chance: f64,         // Attack block %
    pub spell_block_chance: f64,
    pub spell_suppression: f64,
    pub attack_dodge: f64,
    pub spell_dodge: f64,
    pub life_regen: f64,           // Per second
    pub mana_regen: f64,
    pub es_regen: f64,
    pub es_recharge: f64,          // Per second (when recharging)
    pub es_recharge_delay: f64,    // Seconds before recharge starts
    pub life_leech_rate_max: f64,
    pub mana_leech_rate_max: f64,
    pub movement_speed_mod: f64,   // 1.0 = 100% base
    pub mana_unreserved: u32,
    pub life_unreserved: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
pub struct OffenceResult {
    pub total_dps: f64,
    pub hit_dps: f64,
    pub average_hit: f64,
    pub crit_chance: f64,          // Effective crit chance %
    pub crit_multiplier: f64,      // e.g. 1.5 = 150%
    pub hit_chance: f64,           // Chance to hit % (attacks only)
    pub attack_speed: f64,         // Attacks/casts per second
    pub cast_speed: f64,
    /// Damage breakdown per element (after all modifiers)
    pub phys_dps: f64,
    pub fire_dps: f64,
    pub cold_dps: f64,
    pub lightning_dps: f64,
    pub chaos_dps: f64,
    /// DoT
    pub dot_dps: f64,
    pub bleed_dps: f64,
    pub poison_dps: f64,
    pub ignite_dps: f64,
    /// Speed info
    pub speed: f64,                // Active attacks/casts per second
    pub is_attack: bool,
}
```

**IPC update**: Replace `BuildStats` with `CalcResult` as the return type of `update_selected_nodes`. The frontend receives the full calculation result instead of just 6 fields.

**PoB equivalent**: `calcs.perform()` in `CalcPerform.lua` (~900 lines).

#### 5.4 — Attribute Calculation (`calc/attributes.rs`)

Attributes must be calculated first because they feed into everything: life (STR), mana (INT), ES (INT), accuracy (DEX), evasion (DEX).

**PoB reference**: `doActorAttribsConditions` in `CalcPerform.lua`.

**Step 1: Calculate raw attributes**

```rust
pub fn calc_attributes(db: &ModDB, level: u32, class: &Class) -> AttributeResult {
    let ctx = CalcContext::empty();

    // Base attributes already in ModDB from class layer (rebuild_class adds base_str/dex/int)
    let str_base = db.sum_base(StatId::Str, &ctx);
    let dex_base = db.sum_base(StatId::Dex, &ctx);
    let int_base = db.sum_base(StatId::Int, &ctx);

    // Apply inc/more to attributes
    // PoB: output.Str = m_max(round(strBase * strMult), 0)
    // where strMult = calcLib.mod(modDB, nil, "Str", "All")
    let str_inc = db.sum_inc(StatId::Str, &ctx); // + "All" if we add a combined stat
    let str_more = db.product_more(StatId::Str, &ctx);
    let strength = (str_base * (1.0 + str_inc / 100.0) * str_more).round().max(0.0) as i32;

    let dex_inc = db.sum_inc(StatId::Dex, &ctx);
    let dex_more = db.product_more(StatId::Dex, &ctx);
    let dexterity = (dex_base * (1.0 + dex_inc / 100.0) * dex_more).round().max(0.0) as i32;

    let int_inc = db.sum_inc(StatId::Int, &ctx);
    let int_more = db.product_more(StatId::Int, &ctx);
    let intelligence = (int_base * (1.0 + int_inc / 100.0) * int_more).round().max(0.0) as i32;

    AttributeResult { strength, dexterity, intelligence }
}
```

**Step 2: Inject attribute bonuses** — After computing final attributes, inject the derived bonuses back into the ModDB as BASE/INC modifiers (this is why PoB does a two-pass):

```rust
pub fn inject_attribute_bonuses(db: &mut ModDB, attrs: &AttributeResult) {
    let src = SourceId(0);

    // Strength bonuses:
    //   +floor(STR/2) Base Life (PoB: NewMod("Life", "BASE", m_floor(Str/2)))
    //   +X% increased Melee Physical Damage (PoB: Str * 0.2 INC MeleePhysicalDamage)
    db.add_mod(Modifier::new(StatId::Life, ModType::Base, (attrs.strength / 2) as f64, src));

    // Dexterity bonuses:
    //   +floor(DEX/2) Base Accuracy (PoB: NewMod("Accuracy", "BASE", m_floor(Dex/2)))
    //   +floor(DEX/5) INC Evasion (PoB: NewMod("Evasion", "INC", m_floor(Dex/5)))
    db.add_mod(Modifier::new(StatId::Accuracy, ModType::Base, (attrs.dexterity / 2) as f64, src));
    db.add_mod(Modifier::new(StatId::Evasion, ModType::Inc, (attrs.dexterity / 5) as f64, src));

    // Intelligence bonuses:
    //   +floor(INT/2) Base Mana (PoB: NewMod("Mana", "BASE", m_floor(Int/2)))
    //   +floor(INT/5) INC Energy Shield (PoB: NewMod("EnergyShield", "INC", m_floor(Int/5)))
    db.add_mod(Modifier::new(StatId::Mana, ModType::Base, (attrs.intelligence / 2) as f64, src));
    db.add_mod(Modifier::new(StatId::EnergyShield, ModType::Inc, (attrs.intelligence / 5) as f64, src));
}
```

**Critical correctness note**: In the current `lib.rs::update_selected_nodes()`, STR/2 is added AFTER life scaling (`life = base * inc * more + str/2`). This is **wrong**. PoB adds STR/2 as a BASE modifier to Life, meaning it IS multiplied by inc/more. The calc module fixes this by injecting attribute bonuses via `inject_attribute_bonuses()` before the defence calc runs.

**Step 3: Determine conditions** — Set boolean conditions based on computed state:

```rust
pub fn determine_conditions(
    _db: &ModDB,
    attrs: &AttributeResult,
    equipped: &FxHashMap<ItemSlot, Item>,
) -> FxHashMap<&'static str, bool> {
    let mut conds: FxHashMap<&'static str, bool> = FxHashMap::default();

    let weapon1 = equipped.get(&ItemSlot::Weapon1).map(|i| i.item_type);
    let weapon2 = equipped.get(&ItemSlot::Weapon2).map(|i| i.item_type);
    let has_gloves = equipped.contains_key(&ItemSlot::Gloves);

    // ── Off-hand / shield / quiver ───────────────────────────────────────────
    // PoB CalcPerform.lua: doActorAttribsConditions
    match weapon2 {
        Some(ItemType::Shield) => { conds.insert(intern("UsingShield"), true); }
        Some(ItemType::Quiver) => { conds.insert(intern("UsingQuiver"), true); }
        None                   => { conds.insert(intern("OffHandIsEmpty"), true); }
        _ => {}
    }

    // ── Unarmed ──────────────────────────────────────────────────────────────
    if weapon1.is_none() {
        conds.insert(intern("Unarmed"), true);
        if weapon2.is_none() && !has_gloves {
            conds.insert(intern("Unencumbered"), true);
        }
    }

    // ── Weapon-type conditions from each hand ────────────────────────────────
    // weapon_flag() maps ItemType → PoB flag ("Claw","Dagger","Sword","Axe","Mace","Wand","Bow","Staff")
    // PoB sets Using<Flag> from both weapon slots, not just main-hand.
    if let Some(w1) = weapon1 { set_weapon_conditions(&mut conds, w1); }
    if let Some(w2) = weapon2 { if is_weapon(w2) { set_weapon_conditions(&mut conds, w2); } }

    // ── Dual-wield conditions ────────────────────────────────────────────────
    // DualWielding: both slots contain a weapon (not shield/quiver)
    // DualWieldingClaws: both are Claw
    // DualWieldingDaggers: both are Dagger or RuneDagger
    // WieldingDifferentWeaponTypes: different weapon flag AND both 1H (PoB parity)
    let w1_is_weapon = weapon1.map_or(false, is_weapon);
    let w2_is_weapon = weapon2.map_or(false, is_weapon);
    if w1_is_weapon && w2_is_weapon {
        conds.insert(intern("DualWielding"), true);
        let (w1, w2) = (weapon1.unwrap(), weapon2.unwrap());
        if matches!(w1, ItemType::Claw) && matches!(w2, ItemType::Claw) {
            conds.insert(intern("DualWieldingClaws"), true);
        }
        if matches!(w1, ItemType::Dagger | ItemType::RuneDagger)
            && matches!(w2, ItemType::Dagger | ItemType::RuneDagger)
        {
            conds.insert(intern("DualWieldingDaggers"), true);
        }
        if weapon_flag(w1) != weapon_flag(w2) && is_one_handed(w1) && is_one_handed(w2) {
            conds.insert(intern("WieldingDifferentWeaponTypes"), true);
        }
    }

    // ── Attribute comparison conditions ──────────────────────────────────────
    // PoB: doActorAttribsConditions → calculateAttributes()
    let (str, dex, int) = (attrs.strength, attrs.dexterity, attrs.intelligence);
    conds.insert(intern("StrHigherThanDex"), str > dex);
    conds.insert(intern("StrHigherThanInt"), str > int);
    conds.insert(intern("DexHigherThanStr"), dex > str);
    conds.insert(intern("DexHigherThanInt"), dex > int);
    conds.insert(intern("IntHigherThanStr"), int > str);
    conds.insert(intern("IntHigherThanDex"), int > dex);
    conds.insert(intern("StrHighestAttribute"), str >= dex && str >= int);
    conds.insert(intern("DexHighestAttribute"), dex >= str && dex >= int);
    conds.insert(intern("IntHighestAttribute"), int >= str && int >= dex);
    let mut sorted = [str, dex, int];
    sorted.sort();
    conds.insert(intern("TwoHighestAttributesEqual"), sorted[1] == sorted[2]);

    conds
}

// Helper functions (private — used only by determine_conditions):

/// PoB `weaponTypeInfo[type].flag` — suffix for `Using<Flag>` conditions.
fn weapon_flag(t: ItemType) -> Option<&'static str> {
    match t {
        ItemType::Claw => Some("Claw"),
        ItemType::Dagger | ItemType::RuneDagger => Some("Dagger"),
        ItemType::OneHandSword | ItemType::ThrustingOneHandSword | ItemType::TwoHandSword => Some("Sword"),
        ItemType::OneHandAxe | ItemType::TwoHandAxe => Some("Axe"),
        ItemType::OneHandMace | ItemType::TwoHandMace | ItemType::Sceptre => Some("Mace"),
        ItemType::Wand => Some("Wand"),
        ItemType::Bow  => Some("Bow"),
        ItemType::Staff | ItemType::Warstaff => Some("Staff"),
        _ => None,
    }
}
fn is_one_handed(t: ItemType) -> bool {
    matches!(t, ItemType::Claw | ItemType::Dagger | ItemType::RuneDagger
                | ItemType::OneHandSword | ItemType::ThrustingOneHandSword
                | ItemType::OneHandAxe | ItemType::OneHandMace | ItemType::Sceptre | ItemType::Wand)
}
fn is_two_handed(t: ItemType) -> bool {
    matches!(t, ItemType::Bow | ItemType::TwoHandSword | ItemType::TwoHandAxe
                | ItemType::TwoHandMace | ItemType::Staff | ItemType::Warstaff)
}
fn is_weapon(t: ItemType) -> bool { is_one_handed(t) || is_two_handed(t) }
fn is_melee_weapon(t: ItemType) -> bool {
    matches!(t, ItemType::Claw | ItemType::Dagger | ItemType::RuneDagger
                | ItemType::OneHandSword | ItemType::ThrustingOneHandSword
                | ItemType::OneHandAxe | ItemType::OneHandMace | ItemType::Sceptre
                | ItemType::TwoHandSword | ItemType::TwoHandAxe | ItemType::TwoHandMace
                | ItemType::Staff | ItemType::Warstaff)
}
fn set_weapon_conditions(conds: &mut FxHashMap<&'static str, bool>, t: ItemType) {
    if let Some(flag) = weapon_flag(t) { conds.insert(intern(&format!("Using{flag}")), true); }
    if is_melee_weapon(t) { conds.insert(intern("UsingMeleeWeapon"), true); }
    if is_one_handed(t)   { conds.insert(intern("UsingOneHandedWeapon"), true); }
    else if is_two_handed(t) { conds.insert(intern("UsingTwoHandedWeapon"), true); }
}
```

**Two-pass note**: PoB does a two-pass attribute calculation because some keystones create circular dependencies (e.g., "Iron Will" makes STR affect spell damage, which can feed back). For Phase 5, a single pass is sufficient — the two-pass optimization is needed only when specific keystones/items create feedback loops, which can be added later if parity testing reveals issues.

#### 5.5 — `CalcDefence` (`calc/defence.rs`)

Calculate all defensive stats using the merged ModDB. All queries use `CalcContext` from step 5.1.

**PoB reference**: `calcs.defence()` in `CalcDefence.lua` (~2,500 lines). We implement the core formulas; EHP/max-hit estimation is Phase 8.

**5.5.1 — Life**

```rust
// PoB formula (doActorLifeMana):
//   base = sum_base("Life") + sum_base("LifePerLevel") * level
//   life = max(round(base * (1 + sum_inc("Life")/100) * product_more("Life")), 1)
//
// Chaos Inoculation special case: life = 1
// Note: STR/2 is already in the ModDB as a Life BASE mod from inject_attribute_bonuses()
// Note: 38 + 12*level is already seeded from seed_base_values()

let base = db.sum_base(StatId::Life, &ctx);
let inc = db.sum_inc(StatId::Life, &ctx);
let more = db.product_more(StatId::Life, &ctx);

let life = if db.has_flag(StatId::ChaosInoculation, &ctx) {
    1
} else {
    (base * (1.0 + inc / 100.0) * more).round().max(1.0) as u32
};
```

**5.5.2 — Mana**

```rust
// PoB: base = sum_base("Mana") + sum_base("ManaPerLevel") * level
// Note: 34 + 6*level seeded, INT/2 injected as BASE

let base = db.sum_base(StatId::Mana, &ctx);
let inc = db.sum_inc(StatId::Mana, &ctx);
let more = db.product_more(StatId::Mana, &ctx);
let mana = (base * (1.0 + inc / 100.0) * more).round().max(0.0) as u32;
```

**5.5.3 — Energy Shield**

```rust
// ES comes from two sources: gear slots and global
// Per slot: es_from_slot = armourData.EnergyShield * calcLib.mod(modDB, slotCfg, "EnergyShield", "Defences")
// Global: es_base = sum_base("EnergyShield"); es_global = es_base * mod("EnergyShield", "Defences")
// INT/10 INC is already in ModDB from inject_attribute_bonuses()
//
// Total ES = sum(per_slot_es) + global_es
// Then: ES = override("EnergyShield") or max(round(total), 0)

// For Phase 5: simplified — sum all ES as global
let es_base = db.sum_base(StatId::EnergyShield, &ctx);
let es_inc = db.sum_inc(StatId::EnergyShield, &ctx);
let es_more = db.product_more(StatId::EnergyShield, &ctx);
let energy_shield = (es_base * (1.0 + es_inc / 100.0) * es_more).round().max(0.0) as u32;
```

**5.5.4 — Armour**

```rust
// Similar per-slot + global structure to ES
// Iron Reflexes: all evasion is added to armour instead
let armour_base = db.sum_base(StatId::Armour, &ctx);
let armour_inc = db.sum_inc(StatId::Armour, &ctx);
let armour_more = db.product_more(StatId::Armour, &ctx);
let mut armour = (armour_base * (1.0 + armour_inc / 100.0) * armour_more).round().max(0.0) as u32;

if db.has_flag(StatId::IronReflexes, &ctx) {
    // Add evasion to armour
    armour += evasion;
    evasion = 0;
}
```

**5.5.5 — Evasion**

```rust
// DEX/5 INC already in ModDB
let evasion_base = db.sum_base(StatId::Evasion, &ctx);
let evasion_inc = db.sum_inc(StatId::Evasion, &ctx);
let evasion_more = db.product_more(StatId::Evasion, &ctx);
let evasion = (evasion_base * (1.0 + evasion_inc / 100.0) * evasion_more).round().max(0.0) as u32;
```

**5.5.6 — Resistances**

Each resistance follows the same pattern. PoB truncates fractional resistances.

```rust
// For each element (Fire, Cold, Lightning, Chaos):
//   total = override("FireResist") or base * inc
//   max = override("FireResistMax") or min(MaxResistCap, sum_base("FireResistMax"))
//   final = clamp(total, ResistFloor, max)
//   overcap = max(0, total - max)

fn calc_resistance(db: &ModDB, ctx: &CalcContext, stat: StatId, max_stat: StatId) -> (i32, i32, i32) {
    let max_resist_cap = 90; // PoB: data.misc.MaxResistCap
    let resist_floor = -200; // PoB: data.misc.ResistFloor

    let max = db.get_override(max_stat, ctx)
        .unwrap_or_else(|| (db.sum_base(max_stat, ctx) + db.sum_base(StatId::ElementalResistMax, ctx)).min(max_resist_cap as f64));

    let total = db.get_override(stat, ctx)
        .unwrap_or_else(|| {
            let base = db.sum_base(stat, ctx);
            // PoB: total = base * calcLib.mod(modDB, nil, elem.."Resist", "ElementalResist")
            // calcLib.mod = (1 + Sum("INC")/100) * More()
            let inc = db.sum_inc(stat, ctx) + db.sum_inc(StatId::ElementalResist, ctx);
            let more = db.product_more(stat, ctx) * db.product_more(StatId::ElementalResist, ctx);
            base * (1.0 + inc / 100.0) * more
        });

    let total_trunc = total.floor() as i32; // PoB: fractional resistances are truncated
    let max_trunc = max as i32;
    let final_resist = total_trunc.clamp(resist_floor, max_trunc);
    let overcap = (total_trunc - max_trunc).max(0);

    (final_resist, max_trunc, overcap)
}
```

Apply for all 4 elements:
```rust
let (fire_resist, fire_cap, fire_overcap) = calc_resistance(&db, &ctx, StatId::FireResist, StatId::FireResistMax);
let (cold_resist, cold_cap, cold_overcap) = calc_resistance(&db, &ctx, StatId::ColdResist, StatId::ColdResistMax);
let (lightning_resist, lightning_cap, lightning_overcap) = calc_resistance(&db, &ctx, StatId::LightningResist, StatId::LightningResistMax);
let (chaos_resist, chaos_cap, chaos_overcap) = calc_resistance(&db, &ctx, StatId::ChaosResist, StatId::ChaosResistMax);
```

**5.5.7 — Block**

```rust
// Attack block: base from shield + mods, capped at BlockChanceMax (default 75%)
let block_max = db.sum_base(StatId::BlockChanceMax, &ctx);
let block_base = db.sum_base(StatId::BlockChance, &ctx);
let block_inc = db.sum_inc(StatId::BlockChance, &ctx);
let block_more = db.product_more(StatId::BlockChance, &ctx);
let block = (block_base * (1.0 + block_inc / 100.0) * block_more).floor().min(block_max);

// Spell block: separate stat, separate cap
let spell_block_max = db.sum_base(StatId::SpellBlockChanceMax, &ctx);
let spell_block_inc = db.sum_inc(StatId::SpellBlockChance, &ctx);
let spell_block_more = db.product_more(StatId::SpellBlockChance, &ctx);
let spell_block = (db.sum_base(StatId::SpellBlockChance, &ctx)
    * (1.0 + spell_block_inc / 100.0) * spell_block_more).floor().min(spell_block_max);
```

**5.5.8 — Spell Suppression**

```rust
// PoB: spellSuppressionChance = sum_base("SpellSuppressionChance")
// Suppression effect reduces spell damage taken by 50% (base)
let suppression = db.sum_base(StatId::SpellSuppressionChance, &ctx).min(100.0);
```

**5.5.9 — Regeneration**

```rust
// Life regen = (flat_regen + pool * regen_percent/100) * (1 + inc/100) * more * recovery_rate
let life_regen_base = db.sum_base(StatId::LifeRegen, &ctx)
    + life as f64 * db.sum_base(StatId::LifeRegenPercent, &ctx) / 100.0;
let life_regen_inc = db.sum_inc(StatId::LifeRegen, &ctx);
let life_regen_more = db.product_more(StatId::LifeRegen, &ctx);
let life_regen = life_regen_base * (1.0 + life_regen_inc / 100.0) * life_regen_more;
// Apply recovery rate modifier: (1 + inc/100) * more
let life_recovery_rate = (1.0 + db.sum_inc(StatId::LifeRecoveryRate, &ctx) / 100.0)
    * db.product_more(StatId::LifeRecoveryRate, &ctx);
let life_regen = (life_regen * life_recovery_rate * 10.0).round() / 10.0; // Round to 1dp
```

Similar for mana regen and ES regen. Also handle:
- **Zealot's Oath**: Life regen applies to ES instead
- **ES Recharge**: `es_recharge_rate = ES * base_rate(20%) * (1+inc/100) * more`
- **ES Recharge Delay**: `delay = base(2.0s) / (1 + faster_start/100)`

**5.5.10 — Leech Caps**

```rust
// Max leech rate = pool * MaxLeechRate% / 100
let max_life_leech = life as f64 * db.sum_base(StatId::MaxLifeLeechRate, &ctx) / 100.0;
let max_mana_leech = mana as f64 * db.sum_base(StatId::MaxManaLeechRate, &ctx) / 100.0;
```

**5.5.11 — Movement Speed**

```rust
// PoB: override("MovementSpeed") or calcLib.mod(modDB, nil, "MovementSpeed")
let move_speed = db.get_override(StatId::MovementSpeed, &ctx)
    .unwrap_or_else(|| {
        let inc = db.sum_inc(StatId::MovementSpeed, &ctx);
        let more = db.product_more(StatId::MovementSpeed, &ctx);
        (1.0 + inc / 100.0) * more
    });
```

**5.5.12 — Armour Damage Reduction Formula**

Used for EHP calculations and displayed in the UI:

```rust
/// PoB: calcs.armourReductionF(armour, raw_damage)
/// Returns damage reduction percentage from armour against raw_damage
fn armour_reduction(armour: f64, raw_damage: f64) -> f64 {
    if armour == 0.0 && raw_damage == 0.0 { return 0.0; }
    armour / (armour + raw_damage * 5.0) * 100.0
}
```

**5.5.13 — Hit Chance (Evasion-based)**

```rust
/// PoB: calcs.hitChance(evasion, accuracy)
/// Returns hit chance percentage (clamped 5-100%)
fn hit_chance(evasion: f64, accuracy: f64) -> f64 {
    if accuracy < 0.0 { return 5.0; }
    let raw = accuracy / (accuracy + (evasion / 5.0).powf(0.9)) * 125.0;
    raw.round().clamp(5.0, 100.0)
}
```

#### 5.6 — Damage Conversion Chain (`calc/conversion.rs`)

PoE's damage conversion follows a strict one-directional chain:

```
Physical → Lightning → Cold → Fire → Chaos
```

Conversion can only flow left-to-right. You cannot convert Fire back to Physical. Multiple conversion sources stack but are capped at 100% per source type.

**Conversion types**:
- **"X% of Physical Damage Converted to Fire Damage"** — moves damage from one type to another. Capped at 100% total conversion from each source type.
- **"Gain X% of Physical Damage as Extra Fire Damage"** — adds damage without removing it from the source. NOT capped.
- **Avatar of Fire** — 50% of non-fire damage converted to fire, deal no non-fire damage.

**Two-layer conversion priority (exactly as PoB `buildConversionTable`)**:

PoB distinguishes two layers:
- **Skill conversion** (`SkillPhysicalDamageConvertTo*` etc.) — built-in to the gem (e.g., Incinerate converts lightning to fire inherently). Keyed with `Skill` prefix.
- **Global conversion** (`PhysicalDamageConvertTo*` etc.) — from passive tree, gear, buffs. Also checks `ElementalDamageConvertTo*` shortcuts for elemental sources, and `NonChaosDamageConvertTo*` for all non-chaos sources.
- **Priority**: Skill conversion is applied first. If `skill_total > 100%`, scale skill down and discard global entirely (skill owns all 100%). If `global_total + skill_total > 100%`, scale global down proportionally to fill the remaining space. Final `total = min(skill + global, 100%)`.
- **`remaining_mult`**: Each source element tracks `1.0 - total_converted_away` as `remaining_mult`. This value multiplies the source damage after conversion is applied and is needed by ailment calculations (bleed uses physical *after* conversion losses).

**StatId reference — full conversion chain:**

> **⚠️ Missing global elemental conversion stats**: `stat_id.rs` currently only has global conversion for Physical→X. Global conversions for Lightning→X, Cold→X, Fire→X (e.g., from uniques like Pyre) need variants added to `stat_id.rs` at Phase 5.6 time. These are named per PoB's mod system: `"LightningDamageConvertToCold"`, `"LightningDamageConvertToFire"`, `"LightningDamageConvertToChaos"`, `"ColdDamageConvertToFire"`, `"ColdDamageConvertToChaos"`, `"FireDamageConvertToChaos"`. Also add `"ElementalDamageConvertTo{Fire/Cold/Chaos}"` and `"NonChaosDamageConvertTo{...}"` shorthand variants. Until Phase 4 uniques are implemented these will only return 0, so they are safe to add as stubs.

| Conversion leg | Global StatIds | Skill StatIds |
|---|---|---|
| Physical → Lightning | `PhysicalDamageConvertToLightning` (235) | `SkillPhysicalDamageConvertToLightning` (290) |
| Physical → Cold | `PhysicalDamageConvertToCold` (233) | `SkillPhysicalDamageConvertToCold` (288) |
| Physical → Fire | `PhysicalDamageConvertToFire` (234) | `SkillPhysicalDamageConvertToFire` (289) |
| Physical → Chaos | `PhysicalDamageConvertToChaos` (232) | `SkillPhysicalDamageConvertToChaos` (287) |
| Lightning → Cold | `LightningDamageConvertToCold` ⚠️ add | `SkillLightningDamageConvertToCold` (284) |
| Lightning → Fire | `LightningDamageConvertToFire` ⚠️ add | `SkillLightningDamageConvertToFire` (285) |
| Lightning → Chaos | `LightningDamageConvertToChaos` ⚠️ add | `SkillLightningDamageConvertToChaos` (283) |
| Cold → Fire | `ColdDamageConvertToFire` ⚠️ add | `SkillColdDamageConvertToFire` (280) |
| Cold → Chaos | `ColdDamageConvertToChaos` ⚠️ add | `SkillColdDamageConvertToChaos` (279) |
| Fire → Chaos | `FireDamageConvertToChaos` ⚠️ add | `SkillFireDamageConvertToChaos` (282) |

**StatId reference — "Gain as Extra" chain:**

| Gain-as leg | StatId |
|---|---|
| Physical → Lightning | `PhysicalDamageGainAsLightning` (240) |
| Physical → Cold | `PhysicalDamageGainAsCold` (238) |
| Physical → Fire | `PhysicalDamageGainAsFire` (239) |
| Physical → Chaos | `PhysicalDamageGainAsChaos` (237) |
| Lightning → Chaos | `LightningDamageGainAsChaos` (190) |
| Cold → Fire | `ColdDamageGainAsFire` (74) |
| Fire → Chaos | `FireDamageGainAsChaos` (149) |

> Elemental/non-chaos shorthand "gain as" variants (e.g., `"ElementalDamageGainAsFire"`) also need to be added to `stat_id.rs` if not already present.

**Implementation**:

```rust
/// Damage values per element, used throughout the conversion pipeline.
/// Used for both min and max (tracked separately for ailment calculations).
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

/// Per-source-type conversion data: amounts → each dest type, plus remaining fraction.
pub struct SourceConv {
    /// Fraction of this source type that converts to each destination (0.0–1.0).
    /// conv[dst_index] where dst_index: 0=Phys, 1=Light, 2=Cold, 3=Fire, 4=Chaos.
    pub conv: [f64; 5],
    /// `1.0 - sum(conv)` — fraction of source remaining unconverted.
    /// Used by ailment calculations (bleed uses physical_remaining).
    pub remaining_mult: f64,
    /// "Gain as extra" fraction to each destination (additive, uncapped).
    pub gain: [f64; 5],
}

/// Full conversion table for one skill cast.
pub struct ConversionTable {
    /// Indexed by source element: [Phys, Light, Cold, Fire, Chaos].
    pub src: [SourceConv; 5],
}

/// Build the conversion table from ModDB — exact mirror of PoB `buildConversionTable`.
///
/// Priority rules (from PoB):
///   1. Skill conversion (SkillXDamageConvertToY) — gem-internal, summed first.
///   2. Global conversion (XDamageConvertToY) — passive/gear, summed second.
///   3. If skill_total > 100: scale skill entries down, discard global entirely.
///   4. If global_total + skill_total > 100: scale global to fit remaining space.
///   5. remaining_mult = 1.0 - min(skill_total + global_total, 1.0).
pub fn build_conversion_table(db: &ModDB, ctx: &CalcContext) -> ConversionTable {
    // Indexed: Physical=0, Lightning=1, Cold=2, Fire=3, Chaos=4
    // Chaos cannot be converted (only first 4 sources have outbound conversion)
    // ...
}

/// Apply conversion: each source type's damage is split according to ConversionTable.
/// "Gain as extra" is INCLUDED here (adds to destination without subtracting from source).
///
/// Processing order matches dmgTypeList: Physical first, then Lightning, Cold, Fire, Chaos.
/// This is important because converted damage from earlier types accumulates into later types
/// BEFORE those later types are processed — the accumulated damage is then also subject
/// to conversion/modifiers at the destination element.
///
/// PoB's `calcDamage()` is recursive: to compute final Fire damage, it also calls
/// calcDamage(Physical) and multiplies by convMult from Phys→Fire. This means
/// converted damage goes through the destination element's inc/more modifiers,
/// not the source element's. Our implementation must mirror this behavior.
pub fn apply_conversion(base: &DamageSet, table: &ConversionTable) -> DamageSet {
    // For each destination element, accumulate:
    //   1. Source's own remaining_mult * source_base
    //   2. Contributions from all earlier sources via their conv[dst] * that_source_base
    //   3. Gain-as-extra contributions from all sources via gain[dst] * source_base
    // ...
}
```

**⚠️ Important implementation note — PoB's recursive calcDamage()**: PoB does NOT process conversion with a simple flat `source × conv_pct` pass. Instead `calcDamage(damageType)` recursively calls `calcDamage(otherType)` for each earlier type, then multiplies by `convMult`. This means **converted damage goes through the destination element's inc/more pipeline**, which is the correct PoE behaviour. Implement this recursively or via a topological sort over the chain order.

**Avatar of Fire**: Handled as a special `FLAG` modifier in the ModDB. When active:
1. Add 50% conversion from each of Physical/Lightning/Cold/Chaos → Fire (as global conversion)
2. Set `DealNoNonFireDamage` flag — after full conversion pipeline, zero all non-fire output
3. PoB sets `NonChaosDamageConvertToFire BASE 50` and `DealNoLightning/Cold/Physical/ChaosDamage FLAG` flags for this keystoneach of which apply after normal conversions resolve.

#### 5.7 — `CalcOffence` (`calc/offense.rs`)

The offence calculation is the most complex single module. It computes DPS for the active skill.

**PoB reference**: `CalcOffence.lua` (dev branch, ~4,000 lines). Phase 5 implements the core pipeline; trigger-based skills, totem/trap/mine modifiers, dual-wield, and minion DPS are Phase 7.

**⚠️ Setup prerequisite — base crit multiplier**: `setup.rs::seed_base_values()` must inject `CritMultiplier BASE 50` into the ModDB (50% extra damage = 150% total). This matches PoB's `initModDB` which seeds `{ "CritMultiplier", "BASE", 50, "Base" }`. Without this, all crit multiplier calculations will undercount by 150%.

**⚠️ WeaponData gap — per-element damage**: `WeaponData` currently only tracks `phys_min`/`phys_max` and aggregate `ele_dps`. It does NOT have `fire_min/max`, `cold_min/max`, `lightning_min/max`. Phase 5.7 requires these fields to compute correct per-element base damage for attacks. **Add these fields to `WeaponData` in `item/types.rs`** and populate them in `item/local_stats.rs` from local mod stats `"local_minimum_added_{fire,cold,lightning}_damage"` and the base min/max from `base_items.json`.

**5.7.1 — Active Skill Context**

Before calculating DPS, determine what kind of skill we're computing. Type detection uses `RePoEGem.active_skill.types` (a `Vec<String>` of PoE skill type names like `"Attack"`, `"Spell"`, `"Projectile"`, `"Area"`, `"Melee"` etc.) **not** any `tags` field:

```rust
/// Actual file: src-tauri/src/calc/offense.rs
pub fn calc_offence(
    db: &ModDB,
    gem_ref: &GemRef,
    skill_groups: &[SkillGroup],
    game_data: &GameData,
    ctx: &CalcContext,
    attrs: &AttributeResult,
    equipped: &FxHashMap<ItemSlot, Item>,
) -> OffenceResult {
    // 1. Resolve the GemInstance from gem_ref
    let group = skill_groups.iter().find(|g| g.id == gem_ref.group_id)?;
    let gem_instance = group.gems.get(gem_ref.gem_index as usize)?;

    // 2. Look up the RePoEGem definition (has cast_time and active_skill.types)
    let gem_def = game_data.gems.get(&gem_instance.gem_id)?;

    // 3. Determine attack vs spell from active_skill.types
    let active_types = gem_def.active_skill.as_ref()
        .map(|a| a.types.as_slice())
        .unwrap_or(&[]);
    let is_attack = active_types.contains(&"Attack".to_string());
    let is_spell  = active_types.contains(&"Spell".to_string());
    let is_melee  = active_types.contains(&"Melee".to_string());
    let is_projectile = active_types.contains(&"Projectile".to_string());

    // 4. Build offence-specific CalcContext (add HIT + attack/spell flags)
    let mut off_ctx = ctx.clone();
    off_ctx.flags |= ModFlag::HIT;
    if is_attack { off_ctx.flags |= ModFlag::ATTACK; }
    if is_spell  { off_ctx.flags |= ModFlag::SPELL; }
    if is_melee  { off_ctx.flags |= ModFlag::MELEE; }
    // Weapon-type flags (MainHand, Bow, Claw, Dagger, etc.) — derive from equipped main hand
    // For Phase 5: add ModFlag::WEAPON_MELEE / WEAPON_RANGED based on weapon type

    // 5. Run the DPS pipeline (see sub-sections below)
    calc_hit_dps(db, gem_def, &off_ctx, is_attack, is_spell, attrs, equipped)
}
```

**5.7.2 — Base Damage**

PoB's base damage for each element comes from three sources, combined as:
`base_X = (source[XMin] + source[XBonusMin]) * baseMultiplier + addedXMin * damageEffectiveness`

For Phase 5, `baseMultiplier = 1.0` and `damageEffectiveness = 1.0` (from gem level data — add later when per-level data is consumed in Phase 3).

The `source` for attacks is the weapon data (`weapon.PhysicalMin`, `weapon.FireMin`, etc.). For spells the source min/max are 0 (all damage comes from gem/ModDB added damage). PoB uses `"PhysicalMin"` / `"PhysicalMax"` as the ModDB stat names for **added** flat physical damage (matches `StatId::PhysicalMin = 244` and `StatId::PhysicalMax = 243`). Similarly for other elements.

```rust
let mut base_dmg_min = DamageSet::default();
let mut base_dmg_max = DamageSet::default();

if is_attack {
    if let Some(wd) = equipped.get(&ItemSlot::MainHand).and_then(|w| w.weapon_data.as_ref()) {
        // Weapon base physical damage (after local mods — already computed in WeaponData)
        base_dmg_min.physical = wd.phys_min;
        base_dmg_max.physical = wd.phys_max;
        // Weapon base elemental damage (requires fire_min/max etc. on WeaponData — add these)
        base_dmg_min.fire      = wd.fire_min;    // ⚠️ field to be added to WeaponData
        base_dmg_max.fire      = wd.fire_max;
        base_dmg_min.cold      = wd.cold_min;
        base_dmg_max.cold      = wd.cold_max;
        base_dmg_min.lightning = wd.lightning_min;
        base_dmg_max.lightning = wd.lightning_max;
        // No per-element chaos weapon base in standard PoE 1
    }
}

// Add flat damage from ModDB (gem stats + gear/passive added damage)
// These use the stat names "PhysicalMin"/"PhysicalMax" etc. as per PoB's CalcOffence.lua:
//   addedMin = skillModList:Sum("BASE", cfg, damageTypeMin) + enemyDB:Sum("BASE", cfg, "Self"..damageTypeMin)
base_dmg_min.physical += db.sum_base(StatId::PhysicalMin, &off_ctx);
base_dmg_max.physical += db.sum_base(StatId::PhysicalMax, &off_ctx);
base_dmg_min.fire      += db.sum_base(StatId::FireMin, &off_ctx);   // Stat: "FireMin" — verify exists
base_dmg_max.fire      += db.sum_base(StatId::FireMax, &off_ctx);
// ... cold, lightning, chaos similarly

// Average min+max for DPS calculation (PoB works with min/max internally but
// we simplify to average for Phase 5. Track min/max separately for Phase 5.8+ damage ranges.)
let base_damage = DamageSet {
    physical: (base_dmg_min.physical + base_dmg_max.physical) / 2.0,
    fire:      (base_dmg_min.fire      + base_dmg_max.fire)      / 2.0,
    cold:      (base_dmg_min.cold      + base_dmg_max.cold)      / 2.0,
    lightning: (base_dmg_min.lightning + base_dmg_max.lightning) / 2.0,
    chaos:     (base_dmg_min.chaos     + base_dmg_max.chaos)     / 2.0,
};
```

**⚠️ Added flat damage StatId notes**: Check `stat_id.rs` for `FireMin`, `FireMax`, `ColdMin`, `ColdMax`, `LightningMin`, `LightningMax`, `ChaosMin`, `ChaosMax`. PoB mod names are `"FireMin"`, `"FireMax"`, etc. If missing variants, add them (they will be present in `SkillStatMap.json`).

**5.7.3 — Apply Conversion Chain**

Build the conversion table from the ModDB and apply it. `apply_conversion()` does the full topological pass: for each source type, it scales the base damage by the source element's inc/more, then distributes converted fractions to downstream types (which accumulate their own inc/more on the incoming converted portion). Gain-as-extra is folded in here too.

> **inc/more is handled inside `apply_conversion()`**: the function internally calls `element_scale()` per source type, so the returned `DamageSet` already includes each element's increase/more modifiers applied correctly. Do NOT apply inc/more again after calling it — that would double-count.

```rust
// Build conversion table (see 5.6)
let conv_table = conversion::build_conversion_table(db, &off_ctx);

// apply_conversion() processes the full chain including gain-as-extra.
// Returns damage per element AFTER all conversion AND inc/more have been applied.
// The returned DamageSet is ready for crit/speed/hit-chance multiplication.
let final_damage = conversion::apply_conversion(&base_damage, &conv_table, db, &off_ctx);
```

**5.7.4 — ~~Apply Increase/More Per Element~~** *(removed — handled inside `apply_conversion()`)*

> **This step no longer exists as a separate pass.** `apply_conversion()` (implemented in Phase 5.6) already applies each source element's inc/more multipliers as part of the topological conversion pass (via the private `element_scale()` helper). The `final_damage` returned from 5.7.3 is the fully scaled per-element damage ready for crit and speed multiplication.
>
> **Consequence for code in 5.7.5–5.7.8**: use `final_damage` directly; do not re-apply `(1 + inc/100) * more` for damage.

**5.7.5 — Critical Strikes**

```rust
// PoB crit chain (from CalcOffence.lua):
//   baseCrit = source.CritChance  (from weapon or gem — for attacks: wd.crit_chance, for spells: gem_def.crit_chance_override)
//   crit_from_mods = skillModList:Sum("BASE", cfg, "CritChance")  (flat added crit %)
//   inc = skillModList:Sum("INC", cfg, "CritChance")
//   more = skillModList:More(cfg, "CritChance")
//   final_crit_pct = (baseCrit + crit_from_mods) * (1 + inc/100) * more
//   capped at 100% (or skillModList:Sum("BASE", cfg, "CritChanceCap") if overridden)

let crit_base_pct = if is_attack {
    // Weapon crit chance is post-local-mod and stored as a percentage (e.g. 6.5 = 6.5%)
    equipped.get(&ItemSlot::Weapon1)
        .and_then(|w| w.weapon_data.as_ref())
        .map(|wd| wd.crit_chance)   // e.g. 6.5 for 6.5%
        .unwrap_or(0.0)
} else {
    // Spell base crit from gem static_data.crit_chance — units: hundredths-of-percent (600 = 6.00%).
    // ALL spells have a non-zero base crit (e.g. Fireball = 600 = 6.00%).
    gem_def.static_data.crit_chance
        .map(|c| c as f64 / 100.0)
        .unwrap_or(0.0)
};

let crit_flat   = db.sum_base(StatId::CritChance, &off_ctx);  // "+X% base crit" passives
let crit_inc    = db.sum_inc(StatId::CritChance, &off_ctx);
let crit_more   = db.product_more(StatId::CritChance, &off_ctx);
let crit_pct    = ((crit_base_pct + crit_flat) * (1.0 + crit_inc / 100.0) * crit_more)
                      .clamp(0.0, 100.0);
let crit_chance = crit_pct / 100.0;  // 0.0–1.0

// Crit multiplier:
//   PoB injects "CritMultiplier BASE 50" in initModDB as the base 50% extra (= 150% total).
//   setup.rs::seed_base_values() must do the same — see ⚠️ note above.
//   All additional "+X% increased Crit Multi" items add to this BASE sum.
//   extra_damage = sum("BASE", "CritMultiplier") / 100   → at minimum 0.5 from base 50
//   crit_multiplier = 1.0 + max(0.0, extra_damage)       → at minimum 1.5
let crit_multi_pct = db.sum_base(StatId::CritMultiplier, &off_ctx); // Includes base 50
let crit_multiplier = 1.0 + (crit_multi_pct / 100.0).max(0.0);     // e.g. 1.5

// Effective damage multiplier from crits (weighted average):
//   1.0 at 0% crit, crit_multiplier at 100% crit
let effective_crit = 1.0 + crit_chance * (crit_multiplier - 1.0);
```

**5.7.6 — Speed**

PoB uses the stat name `"Speed"` for **both** attack speed and cast speed mods. The `ModFlag::ATTACK` or `ModFlag::SPELL` context already set on `off_ctx` means only the correct mods apply. `StatId::Speed = 292`.

> **PoB speed rounding** (`CalcOffence.lua`): The combined speed multiplier `(1 + inc/100) * more` is **rounded to 2 decimal places** before dividing into base time: `speed = 1 / (baseTime / round((1 + inc/100) * more, 2))`. Apply `f64::round()` equivalently in Rust for matching output.

```rust
// Attack speed: base = weapon.attacks_per_second (field name on WeaponData)
// Cast speed:   base = 1 / (gem_def.cast_time_ms / 1000.0)
//               gem_def.cast_time is Option<u32> in MILLISECONDS
//
// PoB formula (from CalcOffence.lua):
//   baseTime = 1 / source.AttackRate   (attacks: from weapon)
//   baseTime = grantedEffect.castTime  (spells: in seconds)
//   speed = 1 / (baseTime / round((1 + inc/100) * more, 2))
//
// Note: "Speed" INC and MORE both use StatId::Speed (292).
// The ModFlag context (ATTACK vs SPELL) handles filtering.

let speed = if is_attack {
    let base_rate = equipped.get(&ItemSlot::Weapon1)
        .and_then(|w| w.weapon_data.as_ref())
        .map(|wd| wd.attacks_per_second)  // Correct field name on WeaponData
        .unwrap_or(1.2);                  // Default unarmed attack rate

    let inc  = db.sum_inc(StatId::Speed, &off_ctx);
    let more = db.product_more(StatId::Speed, &off_ctx);
    let effective_time = (1.0 / base_rate) / ((1.0 + inc / 100.0) * more).max(0.01);
    1.0 / effective_time
} else {
    // cast_time is Option<u32> in MILLISECONDS — convert to seconds
    let cast_time_s = gem_def.cast_time
        .map(|ms| ms as f64 / 1000.0)
        .unwrap_or(1.0);
    let base_speed = 1.0 / cast_time_s;

    let inc  = db.sum_inc(StatId::Speed, &off_ctx);
    let more = db.product_more(StatId::Speed, &off_ctx);
    base_speed * (1.0 + inc / 100.0) * more
};
```

**5.7.7 — Accuracy / Hit Chance (attacks only)**

```rust
// PoB: output.Accuracy = max(0, floor(base * (1 + inc/100) * more))
// DEX/2 flat accuracy bonus is injected by inject_attribute_bonuses() in perform.rs.
// StatId::Accuracy = 0.

let hit_chance_pct = if is_attack {
    let acc_base = db.sum_base(StatId::Accuracy, &off_ctx);
    let acc_inc  = db.sum_inc(StatId::Accuracy, &off_ctx);
    let acc_more = db.product_more(StatId::Accuracy, &off_ctx);
    let accuracy = (acc_base * (1.0 + acc_inc / 100.0) * acc_more).max(0.0).floor();

    // Enemy evasion — placeholder. Phase 7: configurable via ConfigTab.
    // PoB default level 83 monster has ~7,500 evasion for mapping reference.
    let enemy_evasion = 1000.0_f64;
    hit_chance(enemy_evasion, accuracy) // returns %, 5–100
} else {
    100.0 // Spells always hit
};
```

**5.7.8 — Final DPS**

PoB formula: `AverageDamage = AverageHit * HitChance/100`, `TotalDPS = AverageDamage * Speed * dpsMultiplier`.
`AverageHit = totalHitAvg * (1 - CritChance/100) + totalCritAvg * CritChance/100`.
For Phase 5, `totalCritAvg = totalHitAvg * crit_multiplier` (crits roll the same average damage scaled by crit_multi), so `AverageHit = totalHitAvg * (1 + crit_chance * (crit_multiplier - 1))` = `totalHitAvg * effective_crit`.

```rust
let avg_hit = final_damage.total(); // Sum of all elements
let hit_chance_f = hit_chance_pct / 100.0;

// Per-element DPS (for the result struct breakdown fields)
let elem_mult = speed * hit_chance_f * effective_crit;
let phys_dps  = final_damage.physical  * elem_mult;
let fire_dps  = final_damage.fire      * elem_mult;
let cold_dps  = final_damage.cold      * elem_mult;
let light_dps = final_damage.lightning * elem_mult;
let chaos_dps = final_damage.chaos     * elem_mult;

let hit_dps = avg_hit * elem_mult;

OffenceResult {
    total_dps:      hit_dps + dot_dps,
    hit_dps,
    average_hit:    avg_hit * effective_crit,   // Per-hit damage including crit weighting
    crit_chance:    crit_pct,                   // % (0–100)
    crit_multiplier: crit_multiplier,           // e.g. 1.5
    hit_chance:     hit_chance_pct,             // % (0–100), 100.0 for spells
    attack_speed:   if is_attack { speed } else { 0.0 },
    cast_speed:     if !is_attack { speed } else { 0.0 },
    speed,
    is_attack,
    phys_dps, fire_dps, cold_dps, lightning_dps: light_dps, chaos_dps,
    dot_dps, bleed_dps, poison_dps, ignite_dps,
}
```

**5.7.9 — DoT DPS** (separate path)

DoT damage is **NOT** affected by: accuracy/hit chance, crit (crit has a separate DoT path via Perfect Agony that is Phase 7), attack/cast speed. It IS affected by: DoT-specific inc/more mods, element-specific DoT multipliers.

```rust
// DoT uses a separate CalcContext with ModFlag::DOT instead of HIT.
// Per PoB: keywordFlags = KeywordFlag::PhysicalDot / ChaosDot / FireDot etc.
//
// For Phase 5, implement generic skill DoT from skillData[damageTypeDot]:
//   base = skillData["PhysicalDot"] or 0
//   inc  = db.sum_inc(PhysicalDamage, &dot_ctx) + db.sum_inc(Damage, &dot_ctx)
//   more = db.product_more(PhysicalDamage, &dot_ctx) * db.product_more(Damage, &dot_ctx)
//   dot_mult = 1.0 + db.sum_base(StatId::DotMultiplier, &dot_ctx) / 100.0   // StatId::DotMultiplier = 108
//   total = base * (1 + inc/100) * more * dot_mult
//
// ⚠️ StatId::DotMultiplier = 108. Do NOT use a non-existent "DamageOverTimeMultiplier".
// Element-specific multipliers: StatId::PhysicalDotMultiplier (add if missing),
//   StatId::FireDotMultiplier = 151, StatId::ColdDotMultiplier = 76,
//   StatId::ChaosDotMultiplier = 65.
```

**5.7.10 — Ailment DPS** (bleed, poison, ignite)

Ailments are computed from post-conversion hit damage (the `final_damage` values returned by `apply_conversion()` in 5.7.3):

```
Bleed:  source = physical remaining after conversion × remaining_mult (from ConversionTable)
Poison: source = (physical + chaos) after conversion  
Ignite: source = fire after conversion
```

**Base rates from PoB `data.misc`** (hard-coded game constants):

| Ailment | Base rate per second | PoB constant |
|---|---|---|
| Bleed | **70%** of physical hit | `data.misc.BleedPercentBase = 70` |
| Poison | **30%** of (phys+chaos) hit | `data.misc.PoisonPercentBase = 30` |
| Ignite | **90%** of fire hit | `data.misc.IgnitePercentBase = 90` ← NOT 50% |

Bleed's 70% is per second (base duration 5 seconds = 350% total). While moving the target takes double bleed damage, but for Phase 5 assume stationary (50% effective, as PoB's default).
Ignite defaults to 4-second duration. Poison defaults to 2-second duration.

```rust
// PoB ailment multipliers use dotCfg flags:
//   Bleed:  ModFlag::DOT | ModFlag::AILMENT | weapon flags, KeywordFlag::BLEED | AILMENT | PHYSICAL_DOT
//   Poison: ModFlag::DOT | ModFlag::AILMENT | weapon flags, KeywordFlag::POISON | AILMENT | CHAOS_DOT
//   Ignite: ModFlag::DOT | ModFlag::AILMENT | weapon flags, KeywordFlag::IGNITE | AILMENT | FIRE_DOT

// Bleed (physical attacks only; "CannotBleed" flag disables it)
let bleed_base = final_damage.physical              // After conversion + inc/more
    * conv_table.src[0].remaining_mult;             // remaining_mult: fraction of phys not converted away
let bleed_dps = if is_attack && bleed_base > 0.0 {
    let dot_multi = 1.0 + db.sum_base(StatId::DotMultiplier, &bleed_ctx) / 100.0
        // + PhysicalDotMultiplier if it exists
        ;
    let effective_bleed_mult = calcLib_mod(bleed_ctx, db); // inc/more for bleed
    bleed_base * 0.70 * effective_bleed_mult * dot_multi
    // Multiply by active stacks and duration for total DPS accounting is Phase 7.
} else { 0.0 };

// Poison (all hits; source = physical + chaos BEFORE inc/more in final_damage)
// ⚠️ PoB uses source damage from calcAilmentSourceDamage(), which calls calcDamage()
//    for Physical and Chaos independently and uses conversionTable[damageType].mult.
//    For Phase 5 simplify: use final_damage.physical + final_damage.chaos as source.
let poison_source = final_damage.physical + final_damage.chaos;
let poison_dps = if poison_source > 0.0 {
    let dot_multi = 1.0 + db.sum_base(StatId::DotMultiplier, &poison_ctx) / 100.0
        + db.sum_base(StatId::ChaosDotMultiplier, &poison_ctx) / 100.0;
    poison_source * 0.30 * dot_multi  // 30% per second
} else { 0.0 };

// Ignite (fire hits; source = fire damage through calcAilmentSourceDamage())
let ignite_source = final_damage.fire;
let ignite_dps = if ignite_source > 0.0 {
    let dot_multi = 1.0 + db.sum_base(StatId::DotMultiplier, &ignite_ctx) / 100.0
        + db.sum_base(StatId::FireDotMultiplier, &ignite_ctx) / 100.0;
    ignite_source * 0.90 * dot_multi  // 90% per second (NOT 50%)
} else { 0.0 };
```

Advanced ailment mechanics deferred to Phase 7: Crimson Dance (bleed stacking), ailment proliferation, Poison stacking and duration, Ignite chance thresholds, avg vs max roll averaging, crit ailment interactions (Perfect Agony).

#### 5.8 — Charges & Buffs (`calc/perform.rs`, inline)

Charges are processed in `CalcPerform` before defence/offence. Each charge type provides stat bonuses that scale with the charge count.

**PoB reference**: `doActorCharges()` in CalcPerform.lua.

**Charge processing**:

```rust
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

    ChargeState { power: power_max, frenzy: frenzy_max, endurance: endurance_max, multipliers }
}
```

**Buff conditions** (Phase 5 subset — full buff processing in Phase 7):

```rust
// Set basic self-conditions that can be computed without config tab:
// "LowLife" = life ≤ 35% of maximum (PoB: data.misc.LowPoolThreshold = 0.35)
// "FullLife" = life = 100% (assume true if no degen)
// "Leeching" = has any leech active (config-driven, Phase 7)
// "CritRecently" = config-driven (Phase 7)

ctx.conditions.insert(intern("LowLife"), (life_unreserved as f64) <= life as f64 * 0.35);
ctx.conditions.insert(intern("FullLife"), life_unreserved == life);
```

#### 5.9 — Frontend Integration

**5.9.1 — Event-Driven Recalculation**

Any state change (tree selection, gear change, gem change) triggers a full recalc. The result is returned directly from the Tauri command (not via events — that would add unnecessary complexity):

```rust
#[tauri::command]
#[specta::specta]
pub fn update_selected_nodes(
    node_ids: Vec<u32>,
    // ... other params
) -> Result<CalcResult, String> {
    // ... rebuild layers as before
    let result = calc::perform::calculate(&layers, level, &class, active_gem, ...);
    Ok(result)
}
```

The frontend receives `CalcResult` directly from the command response — no event listener needed. The debounced 50ms call pattern already exists in `SkillTree.svelte`.

**5.9.2 — CalcsTab Component**

Add a new tab/panel in the sidebar or as a separate page to display the full `CalcResult`:

```svelte
<!-- src/components/CalcsTab.svelte -->
<script lang="ts">
    let { result }: { result: CalcResult } = $props();
</script>

<!-- Defence Section -->
<section>
    <h3>Defence</h3>
    <div>Life: {result.defense.life}</div>
    <div>Mana: {result.defense.mana} ({result.defense.mana_unreserved} unreserved)</div>
    <div>Energy Shield: {result.defense.energy_shield}</div>
    <div>Armour: {result.defense.armour}</div>
    <div>Evasion: {result.defense.evasion}</div>
    <!-- Resistances with color coding -->
    <div>Fire Resist: {result.defense.fire_resist}% / {result.defense.fire_resist_cap}%</div>
    <!-- ... -->
</section>

<!-- Offence Section -->
<section>
    <h3>Offence</h3>
    <div>Total DPS: {formatNumber(result.offence.total_dps)}</div>
    <div>Average Hit: {formatNumber(result.offence.average_hit)}</div>
    <div>Crit Chance: {result.offence.crit_chance.toFixed(1)}%</div>
    <div>Speed: {result.offence.speed.toFixed(2)}/s</div>
    <!-- ... -->
</section>
```

**5.9.3 — Sidebar Update**

Update `Sidebar.svelte` to display the expanded `CalcResult` instead of the current minimal `BuildStats` (6 fields).

#### 5.10 — Benchmarks & Testing

**5.10.1 — Criterion Benchmarks**

Implemented in `src-tauri/benches/`.

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "calc_bench"
harness = false

[[bench]]
name = "moddb_bench"
harness = false
```

**Running benchmarks:**

```bash
# Run all benchmarks (generates HTML reports in target/criterion/)
cd src-tauri && cargo bench

# Run only one benchmark suite
cargo bench --bench calc_bench
cargo bench --bench moddb_bench

# Run a specific benchmark by name pattern
cargo bench --bench calc_bench -- "full_calc/no_gem"

# Baseline comparison (save a baseline, then compare after changes)
cargo bench --bench calc_bench -- --save-baseline main
# ... make changes ...
cargo bench --bench calc_bench -- --baseline main

# Flamegraph profiling (requires cargo-flamegraph)
cargo flamegraph --bench calc_bench -- --bench
```

**Benchmark suites:**

`benches/calc_bench.rs` — end-to-end `calculate()` pipeline:
- `full_calc/no_gem/0..300` — full recalculation at different node counts, no active gem
- `full_calc/with_gem_100_nodes` — full recalculation including the offence pipeline

`benches/moddb_bench.rs` — individual ModDB layer operations (fine-grained hot paths):
- `moddb/rebuild_tree/50..600` — tree layer rebuild at different node counts
- `moddb/rebuild_class` — class layer rebuild (triggered on class/ascendancy change)
- `moddb/rebuild_items_empty` — item layer rebuild baseline
- `moddb/merge_100_nodes` — cost of flattening all layers into a single ModDB

**Requirements for bench compilation:**

The internal modules must be public so the bench crates (separate compilation units) can
access `calc::calculate`, `modifier::ModDBLayers`, `data::GameData`, etc.
`lib.rs` exposes: `pub mod calc`, `pub mod data`, `pub mod item`, `pub mod modifier`.

**In-app performance overlay:**

`BuildStats.calc_time_us: u64` — pure Rust `calculate()` time, measured via
`std::time::Instant`, serialized with every IPC response.

`buildState.lastIpcMs: number` — full IPC round-trip time measured by the frontend
with `performance.now()` around each `commands.updateSelectedNodes()` call.

The Sidebar displays these when `lastIpcMs > 0`:
- **Rust calc**: time inside `calc::calculate()` only
- **IPC round-trip**: total wall time from `performance.now()` pair
- **IPC overhead**: round-trip minus Rust time (serialization + channel + deserialization)

**Target**: Full recalculation < 50ms. PoB averages ~10-30ms in Lua; Rust should be faster. Profile with `cargo flamegraph` if needed.

**5.10.2 — Test Strategy**

Unit tests per module:

```rust
#[cfg(test)]
mod tests {
    // Test resistance calculation with caps
    #[test]
    fn test_resistance_cap() {
        // 135% fire resist, 75% cap → final=75%, overcap=60%
    }

    // Test life formula matches PoB
    #[test]
    fn test_life_calculation() {
        // Level 90 Marauder with 200% inc life, 50 STR
        // Expected: (38 + 90*12 + 0 + 25) * (1 + 200/100) * 1.0 = 3243
        // (base=38, per_lvl=1080, str_bonus=25, inc=200%)
    }

    // Test damage conversion chain
    #[test]
    fn test_phys_to_fire_conversion() {
        // 100 phys, 50% converted to fire → 50 phys + 50 fire
    }

    // Test Avatar of Fire
    #[test]
    fn test_avatar_of_fire() {
        // All non-fire damage → 50% to fire, no non-fire dealt
    }

    // Test critical strike calculation
    #[test]
    fn test_crit_effective_multiplier() {
        // 50% crit, 200% multiplier → effective = 1 + 0.5 * (2.0 - 1.0) = 1.5
    }

    // Test attribute bonuses
    #[test]
    fn test_str_life_bonus_is_base() {
        // 100 STR → +50 BASE life (multiplied by inc/more, not added after)
    }
}
```

**5.10.3 — Integration Tests (PoB Comparison)**

Create reference builds in PoB and compare numbers:

```rust
#[test]
fn test_pob_parity_marauder_rt() {
    // Load a known Marauder build (tree + gear + gems)
    // Compare life, mana, DPS, resists against PoB's numbers
    // Tolerance: ±1 for integer stats, ±0.1% for percentages
}
```

Save reference builds as JSON fixtures in `src-tauri/tests/fixtures/`.

### Implementation Order

The steps above should be implemented in this order to minimize rework:

1. **5.1 CalcContext** — Foundation for everything else
2. **5.2 CalcSetup** — Base values needed by all calcs
3. **5.4 Attributes** — Needed before life/mana/ES
4. **5.5 CalcDefence** — Life/mana/ES/resists (most impactful user-visible stats)
5. **5.8 Charges** — Affects both defence and offence
6. **5.6 Damage Conversion** — Needed before offence
7. **5.7 CalcOffence** — DPS pipeline (most complex, do last)
8. **5.3 CalcPerform** — Wire everything together
9. **5.9 Frontend** — Display results
10. **5.10 Benchmarks** — Verify performance

### How to Verify Phase 5 is Complete

- [ ] `calc/` module compiles with all submodules
- [ ] `CalcContext::matches_context()` correctly gates modifiers by flags, keywords, and conditions
- [ ] `CalcContext::effective_value()` correctly scales values by multipliers, per-stat, and thresholds
- [ ] Life/Mana/ES calculated correctly with attribute bonuses applied as BASE (not post-scaling)
- [ ] Life formula: `(38 + 12*level + flat + floor(str/2)) * (1 + inc/100) * more` matches PoB
- [ ] Mana formula: `(34 + 6*level + flat + floor(int/2)) * (1 + inc/100) * more` matches PoB
- [ ] Resistances calculated with -60 penalty, correct caps, overcap reporting
- [ ] Armour/Evasion with inc/more, Iron Reflexes converts evasion to armour when active
- [ ] Block chance (attack + spell) calculated and capped correctly
- [ ] Regen (life/mana/ES) with flat + percent + inc/more + recovery rate
- [ ] Damage conversion chain works: Physical → Lightning → Cold → Fire → Chaos
- [ ] Avatar of Fire: 50% of non-fire to fire, zero non-fire output
- [ ] "Gain as Extra" damage stacks correctly (not capped)
- [ ] Crit chance/multiplier/effective crit computed correctly
- [ ] Attack speed from weapon base * inc * more
- [ ] Cast speed from gem base cast time * inc * more
- [ ] Accuracy from DEX/2 + flat + inc + more; hit chance formula matches PoB
- [ ] Per-element DPS breakdown (phys/fire/cold/lightning/chaos)
- [ ] DoT DPS calculated separately (no accuracy, no crit)
- [ ] Ailments: bleed (70% phys), poison (20% phys+chaos), ignite (50% fire)
- [ ] Charge processing sets multipliers in CalcContext
- [ ] Full DPS number displays for a main skill
- [ ] Changing gear/tree/gems triggers recalculation and returns updated CalcResult
- [ ] CalcsTab shows full stat breakdown (defence + offence)
- [ ] At least one full build matches PoB numbers within ±1 (integers) / ±0.1% (percentages)
- [ ] Benchmark: full recalculation < 50ms
- [ ] `BuildStats` replaced with `CalcResult` across all IPC commands

---

## Phase 6: Build Management & Import/Export

### Goal

Save/load builds to disk, import/export PoB-compatible share codes, import characters from the PoE API, add undo/redo, and wire up the home page build list.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│ Frontend (Svelte 5)                                                │
│                                                                    │
│  +page.svelte ──► commands.listBuilds()                            │
│                ──► commands.saveBuild(name)                         │
│                ──► commands.loadBuild(id) ──► resetBuildState()     │
│                ──► commands.deleteBuild(id)                         │
│                ──► commands.renameBuild(id, name)                   │
│                                                                    │
│  Sidebar/Header ──► commands.exportPobCode() ──► clipboard         │
│                 ──► commands.importPobCode(code) ──► rebuild UI     │
│                                                                    │
│  ImportTab ──► commands.startPoeAuth() ──► opens browser            │
│            ──► commands.listPoeCharacters(token)                    │
│            ──► commands.importPoeCharacter(token, name)             │
│                                                                    │
│  All tabs ──► commands.undo() / commands.redo()                    │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ IPC (tauri-specta)
┌──────────────────────────▼──────────────────────────────────────────┐
│ Backend (Rust)                                                     │
│                                                                    │
│  storage/manager.rs ─── SavedBuild JSON files ──► app_data_dir/    │
│  build/codec.rs     ─── XML ↔ zlib ↔ base64url (PoB compat)       │
│  build/import.rs    ─── PoE API Character → Build conversion       │
│  build/undo.rs      ─── UndoStack<T> with undo[]/redo[] stacks    │
│                                                                    │
│  On load: deserialize → rebuild items (reparse) → rebuild ModDB    │
│           → recalculate stats                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

### Steps

#### 6.1 — Serialization Prerequisites

Before builds can be saved/loaded, every field in `BuildInfo` that isn't `#[serde(skip)]` must be fully (de)serializable. Currently `BuildInfo` skips `mod_db_layers`, `equipped`, `inventory`, and `next_item_id` — these are reconstructed on load. But for native save files we need to persist items too.

**Decision: Two serialization paths.**
- **Native save/load**: Store items as structured JSON (typed Rust structs with serde). This gives lossless round-trip and fast load.
- **PoB share codes**: Store items as mod-text lines in XML (same as PoB). This gives interoperability.

For the native path, `Item` and its transitive dependencies must derive `Serialize + Deserialize`. The computed fields (`mod_list` on `Item`, `modifiers` on `ModLine`) can be `#[serde(skip)]` and recomputed on load since they are derived from the text representation.

##### 6.1.1 — Add serde to Item type chain

Current state: `Item` derives only `Debug, Clone`. Its type chain blocks serialization at multiple points.

**Types needing `Serialize, Deserialize` added:**

| Type | File | Current Derives | Blocking Fields | Strategy |
|---|---|---|---|---|
| `ItemType` | `item/types.rs` | `Debug, Clone, Copy, PartialEq, Eq` | None (simple enum) | Add `Serialize, Deserialize` directly |
| `ModLineSource` | `item/types.rs` | `Debug, Clone, Copy, PartialEq, Eq` | None (simple enum) | Add `Serialize, Deserialize` directly |
| `BasePropertyOverride` | `item/types.rs` | `Debug, Clone, Default` | None (all primitive fields) | Add `Serialize, Deserialize` directly |
| `WeaponData` | `item/types.rs` | `Debug, Clone, Default` | None (all primitive fields) | Add `Serialize, Deserialize` directly |
| `ArmourData` | `item/types.rs` | `Debug, Clone, Default` | None (all primitive fields) | Add `Serialize, Deserialize` directly |
| `FlaskData` | `item/types.rs` | `Debug, Clone, Default` | None (all primitive fields) | Add `Serialize, Deserialize` directly |
| `ModLine` | `item/types.rs` | `Debug, Clone` | `modifiers: Vec<Modifier>` | Add `Serialize, Deserialize` with `#[serde(skip)]` on `modifiers` — recomputed on load |
| `Item` | `item/types.rs` | `Debug, Clone` | `mod_list: Vec<Modifier>` | Add `Serialize, Deserialize` with `#[serde(skip)]` on `mod_list` — recomputed on load |

**Why `#[serde(skip)]` on computed modifier data**: The `Modifier` struct contains `ModFlag`/`KeywordFlag` (bitflags), `SourceId`, `StatId`, `SmallVec<[ModTag; 2]>` with `&'static str` interned pointers, and `ModTag` — none of which have serde. Adding serde to the entire `Modifier` chain would require:
- `serde` feature on `smallvec` crate
- Serde impls for `ModTag` (which contains `&'static str` — requires custom deserializer to re-intern)
- Serde for `ModFlag`/`KeywordFlag` bitflags
- Serde for `SourceId`, `StatId`

This is ~15 types for data that is **fully reconstructable** from the item's text lines via `parser::resolve()`. PoB takes the same approach: items are saved as text, reparsed on load. So we skip serializing the computed modifier data and reparse on load.

**No changes needed:**
- `ItemRequirements` — already has `Serialize, Deserialize, Type`
- `Rarity` — already has `Serialize, Deserialize, Type`
- `InfluenceSet` — already has `Serialize, Deserialize` (via bitflags serde)
- `ItemSlot` — already has `Serialize, Deserialize, Type`
- `ItemSummary` — already has `Serialize, Deserialize, Type`
- `GemInstance`, `SkillGroup`, `GemRef` — already fully serializable

##### 6.1.2 — Update `BuildInfo` for save/load

Currently `BuildInfo` has `#[serde(skip)]` on `equipped`, `inventory`, and `next_item_id`. For native saves, these must be persisted. Two options:

**Option A (chosen)**: Create a separate `SavedBuildData` struct that wraps all persistable data:

```rust
#[derive(Serialize, Deserialize)]
pub struct SavedBuildData {
    pub name: String,
    pub level: u32,
    pub class: Class,
    pub bloodline: Bloodline,
    pub selected_nodes: BuildSelection,
    pub skill_groups: Vec<SkillGroup>,
    pub active_gem: Option<GemRef>,
    pub next_group_id: u32,
    // Item data (new for save/load)
    pub equipped: FxHashMap<ItemSlot, Item>,
    pub inventory: Vec<Item>,
    pub next_item_id: u32,
    // Metadata
    pub tree_version: String,
    pub save_version: u32,
}
```

This struct is what gets written to/read from JSON. On load, we reconstruct:
1. Deserialize `SavedBuildData` from JSON
2. For each `Item`: call `parser::resolve()` on each `ModLine` to rebuild `modifiers`, rebuild `mod_list`
3. Populate `BuildInfo` fields from `SavedBuildData`
4. Call `rebuild_items()` → `rebuild_tree()` → `rebuild_class()` → `rebuild_gems()` on `ModDBLayers`
5. Run `perform_calc()` to regenerate `BuildStats`

**Option B (rejected)**: Add serde to `BuildInfo` directly and remove the skip attributes. Rejected because `BuildInfo` also contains `ModDBLayers` (no serde, no clone) and `BuildStats` (computed, not stored).

##### 6.1.3 — Implementation checklist

```
src-tauri/src/item/types.rs:
  - [ ] Add Serialize, Deserialize to: ItemType, ModLineSource, BasePropertyOverride,
        WeaponData, ArmourData, FlaskData
  - [ ] Add Serialize, Deserialize to ModLine with #[serde(skip, default)] on `modifiers`
  - [ ] Add Serialize, Deserialize to Item with #[serde(skip, default)] on `mod_list`

src-tauri/src/lib.rs:
  - [ ] Define SavedBuildData struct (or in a new build/ module)
  - [ ] Add to_saved_data() and from_saved_data() conversion methods
  - [ ] from_saved_data() calls parser::resolve() on each ModLine to rebuild modifiers
  - [ ] from_saved_data() calls ModDBLayers::rebuild_items() etc. to rebuild calc state
```

---

#### 6.2 — Build Save/Load (`storage/manager.rs`)

Replace the current no-op `StorageManager` stub with a real implementation that manages JSON build files.

##### 6.2.1 — Storage location

Builds are stored as individual JSON files under the Tauri app data directory:

```
{app_data_dir}/builds/
├── {uuid}.json         # SavedBuildData serialized as JSON
├── {uuid}.json
└── ...
```

- **App identifier**: `com.user.rusty-builds` → app data dir is platform-specific:
  - Windows: `C:\Users\<user>\AppData\Roaming\com.user.rusty-builds\builds\`
  - macOS: `~/Library/Application Support/com.user.rusty-builds/builds/`
  - Linux: `~/.config/com.user.rusty-builds/builds/` (or `$XDG_CONFIG_HOME`)
- Use `app.path().app_data_dir()` to get the base path.
- File names are UUIDs (no user-controlled file names → prevents path traversal).
- No filesystem plugin needed — use Rust `std::fs` directly (backend-side only).

##### 6.2.2 — `StorageManager` struct

```rust
pub struct StorageManager {
    builds_dir: PathBuf,
}

impl StorageManager {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, StorageError> {
        let builds_dir = app.path().app_data_dir()
            .map_err(|_| StorageError::PathResolution)?
            .join("builds");
        std::fs::create_dir_all(&builds_dir)?;
        Ok(Self { builds_dir })
    }

    pub fn list_builds(&self) -> Result<Vec<BuildSummary>, StorageError>;
    pub fn save_build(&self, data: &SavedBuildData) -> Result<String, StorageError>;  // returns UUID
    pub fn load_build(&self, id: &str) -> Result<SavedBuildData, StorageError>;
    pub fn delete_build(&self, id: &str) -> Result<(), StorageError>;
    pub fn rename_build(&self, id: &str, new_name: &str) -> Result<(), StorageError>;
}
```

##### 6.2.3 — `BuildSummary` (for list display)

```rust
#[derive(Serialize, Deserialize, Type)]
pub struct BuildSummary {
    pub id: String,              // UUID filename without .json
    pub name: String,
    pub class: String,           // Display name: "Marauder", "Juggernaut", etc.
    pub level: u32,
    pub node_count: u32,
    pub last_modified: String,   // ISO 8601 timestamp
}
```

`list_builds()` reads each JSON file, deserializes just enough to extract the summary fields (name, class, level), and gets `last_modified` from file metadata. For efficiency, consider storing a separate `builds_index.json` manifest — but for v1, scanning the directory (expected <100 files) is fine.

##### 6.2.4 — Atomic writes

Use `tempfile::NamedTempFile` (already a dependency) for crash-safe saves:

```rust
pub fn save_build(&self, data: &SavedBuildData) -> Result<String, StorageError> {
    let id = data.name.is_empty()
        .then(|| uuid::new_v4().to_string())  // new build
        .unwrap_or_else(|| existing_id);       // overwrite
    let path = self.builds_dir.join(format!("{}.json", id));

    let json = serde_json::to_string_pretty(data)?;
    let mut tmp = tempfile::NamedTempFile::new_in(&self.builds_dir)?;
    tmp.write_all(json.as_bytes())?;
    tmp.persist(&path)?;  // atomic rename

    Ok(id)
}
```

**Note**: Add `uuid` crate to Cargo.toml (features: `v4`).

##### 6.2.5 — Tauri commands

```rust
#[tauri::command]
#[specta::specta]
fn list_builds(storage: State<StorageManager>) -> Result<Vec<BuildSummary>, String>;

#[tauri::command]
#[specta::specta]
fn save_build(
    name: String,
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
    storage: State<StorageManager>,
) -> Result<String, String>;  // returns build ID

#[tauri::command]
#[specta::specta]
fn load_build(
    id: String,
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
    storage: State<StorageManager>,
) -> Result<BuildStats, String>;

#[tauri::command]
#[specta::specta]
fn delete_build(id: String, storage: State<StorageManager>) -> Result<(), String>;

#[tauri::command]
#[specta::specta]
fn rename_build(id: String, new_name: String, storage: State<StorageManager>) -> Result<(), String>;
```

**`save_build` flow:**
1. Lock `BuildInfo`
2. Extract `SavedBuildData` from current state (including items, nodes, skills)
3. Set `name` field
4. Call `storage.save_build(&data)`
5. Return the build ID

**`load_build` flow:**
1. Call `storage.load_build(&id)` → `SavedBuildData`
2. Validate `tree_version` — if different from current, load correct tree version
3. Call `from_saved_data()` to reconstruct `BuildInfo`:
   - For each `Item`: reparse mod lines via `parser::resolve()` to rebuild `modifiers`
   - Rebuild `mod_list` on each item
   - Rebuild `ModDBLayers` (tree, class, items, gems)
   - Recalculate `BuildStats`
4. Replace the managed `Mutex<BuildInfo>` state
5. Return the new `BuildStats`

##### 6.2.6 — Error types

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Failed to resolve app data path")]
    PathResolution,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Build not found: {0}")]
    NotFound(String),
    #[error("Invalid build ID: {0}")]
    InvalidId(String),
}
```

##### 6.2.7 — Input validation

- Build IDs must match `^[0-9a-f-]+$` (UUID format) — reject anything else to prevent path traversal.
- Build names are stored inside the JSON, not used as filenames.
- On delete, verify the path is inside `builds_dir` before removing.

##### 6.2.8 — Implementation checklist

```
src-tauri/Cargo.toml:
  - [ ] Add uuid = { version = "1", features = ["v4"] }

src-tauri/src/storage/manager.rs:
  - [ ] Replace stub with real StorageManager
  - [ ] Implement list_builds(), save_build(), load_build(), delete_build(), rename_build()
  - [ ] Atomic writes via tempfile
  - [ ] UUID-based filenames (no user input in paths)

src-tauri/src/storage/mod.rs:
  - [ ] Remove FileCache re-export (dead code)
  - [ ] Export StorageManager and BuildSummary

src-tauri/src/storage/file_system.rs:
  - [ ] Delete (dead legacy code — LiteNode/rkyv caching is unused)

src-tauri/src/lib.rs:
  - [ ] Add save_build, load_build, list_builds, delete_build, rename_build commands
  - [ ] Register in tauri_specta::collect_commands![]
  - [ ] Update StorageManager initialization in Builder::setup (pass app handle, handle error)

src-tauri/src/lib.rs (or build/mod.rs):
  - [ ] SavedBuildData struct with Serialize/Deserialize
  - [ ] to_saved_data(&BuildInfo, tree_version) -> SavedBuildData
  - [ ] from_saved_data(SavedBuildData, &GameData) -> BuildInfo (with item reparse + ModDB rebuild)
```

---

#### 6.3 — PoB Share Codes (`build/codec.rs`)

Implement bidirectional encoding of builds in PoB's share code format for interoperability. Users should be able to:
1. Export a Rusty Builds build → paste the code in PoB → it loads correctly
2. Copy a PoB share code → import in Rusty Builds → build displays correctly

##### 6.3.1 — Format specification

PoB share codes use this pipeline:

```
Build State ──► XML string ──► zlib deflate ──► base64url encode ──► share code string
                                                   (+→-, /→_)

share code string ──► base64url decode ──► zlib inflate ──► XML string ──► Build State
                         (-→+, _→/)
```

**Base64url variant**: PoB uses standard base64 but with URL-safe character substitution: `+` → `-`, `/` → `_`. No padding `=` characters. This is NOT the standard base64url alphabet — it's a custom substitution applied after encoding.

**Zlib**: Raw deflate compression (zlib format with header), not gzip. Use `flate2` with `ZlibEncoder`/`ZlibDecoder`.

##### 6.3.2 — PoB XML schema

The XML root element is `<PathOfBuilding>` with these child sections:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<PathOfBuilding>
    <Build
        level="95"
        targetVersion="3_0"
        bandit="None"
        className="Marauder"
        ascendClassName="Juggernaut"
        mainSocketGroup="1"
        viewMode="TREE"
        pantheonMajorGod="None"
        pantheonMinorGod="None"
        characterLevelAutoMode="true">
        <PlayerStat stat="Life" value="5234"/>
        <PlayerStat stat="Str" value="412"/>
        <!-- ... computed stats for display only -->
    </Build>
    <Import lastAccountHash="" lastRealm="" lastCharacterHash=""/>
    <Calcs/>
    <Skills defaultGemLevel="20" defaultGemQuality="0" sortGemsByDPS="true"
            showSupportGemTypes="ALL" showAltQualityGems="false">
        <SkillSet id="1" title="Default">
            <Skill mainActiveSkillCalcs="1" mainActiveSkill="1"
                   label="" enabled="true" slot="" source="">
                <Gem level="20" quality="0" skillId="FireballOfMagma"
                     nameSpec="Fireball of Magma" enabled="true"/>
                <Gem level="20" quality="0" skillId="SpellEcho"
                     nameSpec="Spell Echo" enabled="true"/>
            </Skill>
        </SkillSet>
    </Skills>
    <Tree activeSpec="1">
        <Spec title="Default" treeVersion="3_27"
              ascendClassName="Juggernaut" classId="1"
              nodes="10...,20...,30..."
              masteryEffects="{}">
            <!-- Optional: <Socket nodeId="..." itemId="..."/> for jewel sockets -->
        </Spec>
    </Tree>
    <Notes>User notes text here</Notes>
    <Party/>
    <Items activeItemSet="1" useSecondWeaponSet="nil">
        <ItemSet useSecondWeaponSet="nil" id="1" title="Default">
            <Slot name="Weapon 1" itemId="1"/>
            <Slot name="Body Armour" itemId="2"/>
        </ItemSet>
        <Item id="1">
            Rarity: UNIQUE
            Atziri's Disfavour
            Vaal Axe
            ...item text lines...
        </Item>
        <Item id="2">
            Rarity: RARE
            Custom Plate
            Glorious Plate
            ...mod lines...
        </Item>
    </Items>
    <Config>
        <Input name="enemyLevel" number="84"/>
        <Input name="conditionStationary" boolean="true"/>
    </Config>
</PathOfBuilding>
```

**Key observations from PoB's `Build.lua`:**
- `<Build>` section stores `level`, `className`, `ascendClassName`, `bandit`, `mainSocketGroup`, `viewMode`, `pantheonMajorGod`, `pantheonMinorGod`, plus `<PlayerStat>` children (display-only, recalculated on load)
- `<Tree>` has `<Spec>` children with `nodes` as comma-separated u32 node IDs, `classId` as integer
- `<Items>` stores items as **text blocks** (not structured data) — each `<Item>` contains the same text you'd paste into PoB's item editor. Items are referenced by numeric `id` attributes
- `<Skills>` contains `<Skill>` groups with `<Gem>` children, each having `skillId`, `level`, `quality`, `enabled`
- `<Config>` stores key-value pairs for configuration options
- `<Notes>`, `<Party>`, `<Calcs>`, `<Import>` are optional/empty sections

##### 6.3.3 — Dependencies

```toml
# Cargo.toml additions
flate2 = "1"
base64 = "0.22"
quick-xml = { version = "0.37", features = ["serialize"] }
```

##### 6.3.4 — Encoding implementation

```rust
// build/codec.rs

pub fn encode_share_code(build: &BuildInfo, game: &GameData) -> Result<String, CodecError> {
    // 1. Build XML string from current state
    let xml = build_to_pob_xml(build, game)?;

    // 2. Zlib compress
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(xml.as_bytes())?;
    let compressed = encoder.finish()?;

    // 3. Base64 encode with PoB's URL-safe substitution
    let encoded = base64::engine::general_purpose::STANDARD.encode(&compressed);
    let url_safe = encoded.replace('+', "-").replace('/', "_");

    Ok(url_safe)
}

pub fn decode_share_code(code: &str) -> Result<ImportedBuild, CodecError> {
    // 1. Reverse URL-safe substitution
    let standard = code.replace('-', "+").replace('_', "/");

    // 2. Base64 decode
    let compressed = base64::engine::general_purpose::STANDARD.decode(&standard)?;

    // 3. Zlib decompress
    let mut decoder = flate2::read::ZlibDecoder::new(&compressed[..]);
    let mut xml = String::new();
    decoder.read_to_string(&mut xml)?;

    // 4. Parse XML into ImportedBuild
    pob_xml_to_build(&xml)
}
```

##### 6.3.5 — `ImportedBuild` intermediate struct

When importing a PoB code, we parse into an intermediate struct before converting to our internal types:

```rust
pub struct ImportedBuild {
    pub level: u32,
    pub class_name: String,
    pub ascend_class_name: String,
    pub bandit: String,
    pub selected_nodes: Vec<u32>,
    pub mastery_effects: Vec<(u32, u32)>,  // (node_id, effect_id) pairs
    pub items: Vec<ImportedItem>,           // text-based item definitions
    pub item_slots: Vec<(String, u32)>,     // (slot_name, item_id)
    pub skill_groups: Vec<ImportedSkillGroup>,
    pub config: Vec<(String, ConfigValue)>,
    pub notes: String,
    pub tree_version: String,
}

pub struct ImportedItem {
    pub id: u32,
    pub text: String,  // Raw item text (PoB format)
}

pub struct ImportedSkillGroup {
    pub label: String,
    pub enabled: bool,
    pub slot: String,
    pub gems: Vec<ImportedGem>,
}

pub struct ImportedGem {
    pub skill_id: String,
    pub name_spec: String,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
}
```

##### 6.3.6 — XML generation (`build_to_pob_xml`)

Map our internal types to PoB's XML format:

| Our Type | PoB XML | Mapping |
|---|---|---|
| `Class::Marauder(Some(Juggernaut))` | `className="Marauder" ascendClassName="Juggernaut"` | Enum variant → string |
| `BuildInfo.level` | `level="95"` | Direct |
| `BuildSelection.node_ids` | `<Spec nodes="10,20,30...">` | Set → comma-separated |
| `Item` | `<Item id="1">\nRarity: UNIQUE\nName\nBase\n...mods...</Item>` | Serialize as PoB text format |
| `SkillGroup` | `<Skill><Gem skillId="..." .../></Skill>` | Map gem IDs to PoB skill IDs |

**Item text format** (PoB convention):
```
Rarity: UNIQUE
Atziri's Disfavour
Vaal Axe
Quality: +20%
Sockets: R-R-R-R-R-R
Item Level: 80
+2 to Level of Socketed Support Gems
Adds 12 to 26 Physical Damage
25% chance to cause Bleeding on Hit
{crafted}Can have up to 3 Crafted Modifiers
```

Lines prefixed with `{crafted}`, `{fractured}`, `{enchant}` indicate mod sources.

##### 6.3.7 — Item text serialization/deserialization

For export: convert our `Item` struct to PoB text format:
- First line: `Rarity: NORMAL|MAGIC|RARE|UNIQUE`
- Second line: item name (for unique/rare) or empty
- Third line: base type name
- Then property lines (Quality, Sockets, Item Level, Requirements)
- Then implicit mods (with `{implicit}` prefix if needed)
- Then explicit mods (prefixed by source: `{crafted}`, `{fractured}`, `{enchant}`)

For import: parse PoB text format back into an `Item`. This reuses the same item text parsing that PoB uses, which we'll need to implement. Reference: PoB's `src/Classes/Item.lua` `ParseRaw()` function.

**Scope note**: Full PoB item text parsing is complex (~2000 lines in PoB). For Phase 6 MVP, support:
1. Export: items we created can be exported (we control the format)
2. Import: parse basic item text (rarity, name, base, explicit mods). Sockets, quality, requirements can be parsed but may not affect calcs in Phase 6.

##### 6.3.8 — Tauri commands

```rust
#[tauri::command]
#[specta::specta]
fn export_pob_code(
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
) -> Result<String, String>;

#[tauri::command]
#[specta::specta]
fn import_pob_code(
    code: String,
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
) -> Result<BuildStats, String>;
```

##### 6.3.9 — Implementation checklist

```
src-tauri/Cargo.toml:
  - [ ] Add flate2 = "1"
  - [ ] Add base64 = "0.22"
  - [ ] Add quick-xml = { version = "0.37", features = ["serialize"] }

src-tauri/src/build/mod.rs:
  - [ ] Create module: pub mod codec;

src-tauri/src/build/codec.rs:
  - [ ] encode_share_code(build, game) -> Result<String>
  - [ ] decode_share_code(code) -> Result<ImportedBuild>
  - [ ] build_to_pob_xml(build, game) -> Result<String>
  - [ ] pob_xml_to_build(xml) -> Result<ImportedBuild>
  - [ ] ImportedBuild, ImportedItem, ImportedSkillGroup, ImportedGem structs
  - [ ] item_to_pob_text(item) -> String
  - [ ] pob_text_to_item(text) -> Result<Item> (basic parser)
  - [ ] CodecError enum (thiserror)
  - [ ] Tests: round-trip encode→decode, known PoB code decode

src-tauri/src/lib.rs:
  - [ ] Add export_pob_code, import_pob_code commands
  - [ ] Register in tauri_specta::collect_commands![]
  - [ ] import_pob_code: decode → convert ImportedBuild to BuildInfo → rebuild ModDB → recalculate
```

---

#### 6.4 — PoE Character Import (`build/import.rs`)

Import a character directly from the PoE API, including passive tree allocations, equipped items, and skill gems.

##### 6.4.1 — PoE API Overview

**Base URL**: `https://api.pathofexile.com`  
**Auth**: OAuth 2.1 with PKCE (Public Client flow)  
**Relevant endpoints**:
- `GET /character` — List all characters for the authenticated account
- `GET /character/{name}` — Full character details including equipment, passives, jewels

**Rate limiting**: The API has rate limits communicated via response headers. Use the `governor` crate (already a dependency) for client-side rate limiting.

##### 6.4.2 — OAuth 2.1 Public Client with PKCE

Desktop apps are "Public Clients" in PoE's OAuth system. The flow:

```
1. App generates:
   - code_verifier: 43-128 char random string (A-Z, a-z, 0-9, -, ., _, ~)
   - code_challenge: base64url(SHA256(code_verifier))  (S256 method)

2. App starts local HTTP server on 127.0.0.1:{random_port}
   - Redirect URI: http://127.0.0.1:{port}/callback

3. App opens browser to PoE authorization URL:
   https://www.pathofexile.com/oauth/authorize?
     client_id={CLIENT_ID}
     &response_type=code
     &scope=account:characters
     &state={random_state}
     &redirect_uri=http://127.0.0.1:{port}/callback
     &code_challenge={code_challenge}
     &code_challenge_method=S256

4. User logs in and authorizes in browser

5. PoE redirects to: http://127.0.0.1:{port}/callback?code={auth_code}&state={state}
   - App's local server captures the authorization code
   - Validates state parameter matches
   - Serves a "success, you can close this tab" page

6. App exchanges auth code for tokens:
   POST https://www.pathofexile.com/oauth/token
   Content-Type: application/x-www-form-urlencoded
   Body: client_id={CLIENT_ID}&grant_type=authorization_code&code={auth_code}
         &redirect_uri=http://127.0.0.1:{port}/callback
         &code_verifier={code_verifier}

7. Response: { access_token, token_type, expires_in (36000s = 10h),
               refresh_token (7 day lifetime), scope }
```

**Client ID**: Must be registered at `https://www.pathofexile.com/developer/apps`. The app will need a registered OAuth client with:
- Client type: "Public (PKCE)"
- Redirect URI pattern: `http://127.0.0.1`

**Scope needed**: `account:characters` — provides read access to character list and details.

**Token storage**: Access tokens (10h lifetime) and refresh tokens (7d lifetime) should be stored securely. Options:
- Store in memory only (user re-authenticates each session) — simplest, chosen for v1
- Store encrypted in app data dir — future enhancement

##### 6.4.3 — Character API response format

`GET /character/{name}` returns:

```json
{
    "id": "abc123",
    "name": "MyCharacter",
    "class": "Juggernaut",
    "level": 95,
    "experience": 12345678,
    "league": "Standard",
    "equipment": [
        {
            "id": "item-hash",
            "name": "<<set:MS>><<set:M>><<set:S>>Atziri's Disfavour",
            "typeLine": "Vaal Axe",
            "baseType": "Vaal Axe",
            "rarity": 3,
            "ilvl": 80,
            "frameType": 3,
            "implicitMods": ["..."],
            "explicitMods": ["..."],
            "craftedMods": ["..."],
            "fracturedMods": ["..."],
            "enchantMods": ["..."],
            "sockets": [...],
            "socketedItems": [...],
            "properties": [...],
            "requirements": [...]
        }
    ],
    "inventory": [...],
    "jewels": [...],
    "passives": {
        "hashes": [10, 20, 30, ...],
        "hashes_ex": [40, 50, ...],
        "mastery_effects": {"12345": 67890},
        "bandit_choice": "None",
        "pantheon_major": "...",
        "pantheon_minor": "...",
        "alternate_ascendancy": 0
    }
}
```

**PoE API Item format** (key fields):
- `frameType`: 0=Normal, 1=Magic, 2=Rare, 3=Unique
- `implicitMods`, `explicitMods`, `craftedMods`, `fracturedMods`, `enchantMods`: arrays of mod text strings
- `sockets`: array of `{group: number, attr: "S"|"D"|"I"|"G"|"A"|"DV"}` objects
- `socketedItems`: array of socketed gem items
- `properties`: array of `{name, values: [[value, type]], ...}` for weapon/armour base stats
- `influences`: `{elder: bool, shaper: bool, ...}`

##### 6.4.4 — Conversion: PoE API → Internal types

| PoE API Field | Internal Type | Conversion |
|---|---|---|
| `class` | `Class` enum | String → enum variant with ascendancy |
| `level` | `u32` | Direct |
| `passives.hashes` | `BuildSelection.node_ids` | Vec<u32> → HashSet<u32> |
| `passives.hashes_ex` | Cluster jewel nodes | Vec<u32> → add to node_ids |
| `passives.alternate_ascendancy` | `Bloodline` | Integer → enum variant |
| `equipment[].explicitMods` | `ModLine.text` | Mod strings → parse via `parser::resolve()` |
| `equipment[].frameType` | `Rarity` | 0→Normal, 1→Magic, 2→Rare, 3→Unique |
| `equipment[].baseType` | Base type lookup | String → find in `GameData` base items |
| `equipment[].socketedItems` | Gems in skill groups | Extract gem data from socketed items |

**Item slot mapping** (PoE API inventory position → our `ItemSlot`):

| PoE `inventoryId` | Our `ItemSlot` |
|---|---|
| `"Weapon"` | `Weapon1` |
| `"Offhand"` | `Weapon2` |
| `"Helm"` | `Helmet` |
| `"BodyArmour"` | `BodyArmour` |
| `"Gloves"` | `Gloves` |
| `"Boots"` | `Boots` |
| `"Amulet"` | `Amulet` |
| `"Ring"` | `Ring1` |
| `"Ring2"` | `Ring2` |
| `"Belt"` | `Belt` |
| `"Flask"` | `Flask1`-`Flask5` (by `x` position) |

##### 6.4.5 — Implementation approach

```rust
// build/import.rs

pub struct PoeClient {
    http: reqwest::Client,
    // No persistent token storage for v1 — tokens passed per-request
}

impl PoeClient {
    pub async fn list_characters(&self, token: &str)
        -> Result<Vec<PoeCharacterSummary>, ImportError>;

    pub async fn get_character(&self, token: &str, name: &str)
        -> Result<PoeCharacter, ImportError>;
}

pub fn convert_poe_character(
    character: PoeCharacter,
    game: &GameData,
) -> Result<BuildInfo, ImportError>;
```

##### 6.4.6 — OAuth flow implementation

The OAuth flow requires a temporary local HTTP server. Options:
- Use `tokio::net::TcpListener` to bind `127.0.0.1:0` (OS picks port)
- Serve a minimal HTML page on the callback route
- Wait for the callback, extract code + state, shut down server

```rust
pub async fn start_oauth_flow() -> Result<OAuthTokens, ImportError> {
    // 1. Generate PKCE verifier + challenge
    let verifier = generate_code_verifier();  // 128 random chars
    let challenge = base64url_encode(sha256(verifier));

    // 2. Bind local server
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);

    // 3. Open browser (via tauri-plugin-opener)
    let auth_url = format!(
        "https://www.pathofexile.com/oauth/authorize?\
         client_id={}&response_type=code&scope=account:characters\
         &state={}&redirect_uri={}&code_challenge={}&code_challenge_method=S256",
        CLIENT_ID, state, redirect_uri, challenge
    );

    // 4. Wait for callback (with timeout)
    // 5. Exchange code for token
    // 6. Return tokens
}
```

**Security considerations:**
- Validate `state` parameter matches to prevent CSRF
- Use PKCE (S256) to prevent authorization code interception
- Bind to `127.0.0.1` only (not `0.0.0.0`) to prevent network access
- Token in memory only, not persisted to disk
- Server shuts down immediately after receiving callback

##### 6.4.7 — Tauri commands

```rust
#[tauri::command]
#[specta::specta]
async fn start_poe_auth(app: AppHandle) -> Result<String, String>;
// Opens browser, starts OAuth flow, returns access token

#[tauri::command]
#[specta::specta]
async fn list_poe_characters(token: String) -> Result<Vec<PoeCharacterSummary>, String>;

#[tauri::command]
#[specta::specta]
async fn import_poe_character(
    token: String,
    name: String,
    build: State<'_, Mutex<BuildInfo>>,
    game: State<'_, Arc<RwLock<GameData>>>,
) -> Result<BuildStats, String>;
```

##### 6.4.8 — MVP scope

For Phase 6, character import covers:
- ✅ OAuth flow and token acquisition
- ✅ Character list and selection
- ✅ Passive tree node import (`hashes` → `selected_nodes`)
- ✅ Basic equipped item import (name, base, rarity, mods as text)
- ✅ Class and level import
- ⬜ Gem import from socketed items (Phase 6 stretch or Phase 7)
- ⬜ Jewel import with socket assignments (Phase 7)
- ⬜ Cluster jewel expansion (Phase 7)

##### 6.4.9 — Implementation checklist

```
src-tauri/Cargo.toml:
  - [ ] Add sha2 = "0.10" (for PKCE S256 challenge)
  - [ ] reqwest already present — verify "json" feature enabled

src-tauri/src/build/import.rs:
  - [ ] PoeClient struct with list_characters(), get_character()
  - [ ] OAuth PKCE flow: generate verifier, challenge, local server, token exchange
  - [ ] PoeCharacter, PoeItem, PoePassives deserialization structs
  - [ ] convert_poe_character() → BuildInfo
  - [ ] Item slot mapping (PoE inventoryId → ItemSlot)
  - [ ] Mod text extraction (implicitMods, explicitMods, etc. → ModLine)
  - [ ] ImportError enum (thiserror)
  - [ ] Rate limiting via governor

src-tauri/src/lib.rs:
  - [ ] Add start_poe_auth, list_poe_characters, import_poe_character commands
  - [ ] Register in tauri_specta::collect_commands![]
  - [ ] Configure CLIENT_ID (environment variable or config file)

src-tauri/src/client/poe.rs:
  - [ ] Move PoeClient implementation here (currently empty skeleton)
```

**Note**: The PoE API client ID must be registered. This requires creating an application at `https://www.pathofexile.com/developer/apps`. The client ID should be stored as a constant or environment variable, NOT as a secret (public clients have no secret).

---

#### 6.5 — Undo/Redo (`build/undo.rs`)

Implement undo/redo for user actions. PoB uses a simple approach: snapshot entire state, push to stack.

##### 6.5.1 — Design

PoB's `UndoHandler.lua` (1550 bytes) uses:
- `undo[]` stack — array of saved states
- `redo[]` stack — array of saved states  
- Max 101 states in undo stack
- `AddUndoState()` — push current state to undo, clear redo
- `Undo()` — pop from undo, push current to redo, restore
- `Redo()` — pop from redo, push current to undo, restore

We'll implement separate undo stacks for **tree**, **items**, and **skills** (like PoB has separate undo handlers per tab). This prevents a tree undo from accidentally reverting an item change.

##### 6.5.2 — Generic `UndoStack<T>`

```rust
pub struct UndoStack<T: Clone> {
    undo: Vec<T>,
    redo: Vec<T>,
    max_size: usize,
}

impl<T: Clone> UndoStack<T> {
    pub fn new(max_size: usize) -> Self;
    pub fn push(&mut self, state: T);     // Push state, clear redo
    pub fn undo(&mut self, current: T) -> Option<T>;  // Pop undo, push to redo
    pub fn redo(&mut self, current: T) -> Option<T>;  // Pop redo, push to undo
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn clear(&mut self);
}
```

##### 6.5.3 — State snapshots

What gets snapshotted for each domain:

| Domain | Snapshot Type | Data |
|---|---|---|
| Tree | `TreeSnapshot` | `selected_nodes: HashSet<u32>`, `selected_asc_nodes: HashSet<u32>` |
| Items | `ItemSnapshot` | `equipped: FxHashMap<ItemSlot, Item>`, `inventory: Vec<Item>` |
| Skills | `SkillSnapshot` | `skill_groups: Vec<SkillGroup>`, `active_gem: Option<GemRef>` |

All snapshot types must derive `Clone`.

##### 6.5.4 — Integration with `BuildInfo`

Add undo stacks to `BuildInfo` (they are `#[serde(skip)]` since they're ephemeral session state):

```rust
pub struct BuildInfo {
    // ... existing fields ...
    #[serde(skip)]
    #[specta(skip)]
    pub tree_undo: UndoStack<TreeSnapshot>,
    #[serde(skip)]
    #[specta(skip)]
    pub item_undo: UndoStack<ItemSnapshot>,
    #[serde(skip)]
    #[specta(skip)]
    pub skill_undo: UndoStack<SkillSnapshot>,
}
```

##### 6.5.5 — When to push undo states

- **Tree**: After each `update_selected_nodes()` call (debounced from frontend)
- **Items**: After `equip_item()`, `unequip_item()`, `add_crafted_item()`, `equip_from_inventory()`, `remove_inventory_item()`
- **Skills**: After `create_skill_group()`, `delete_skill_group()`, `add_gem_to_group()`, `remove_gem_from_group()`, `update_gem_level_quality()`

##### 6.5.6 — Tauri commands

```rust
#[tauri::command]
#[specta::specta]
fn undo(
    domain: String,  // "tree" | "items" | "skills"
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
) -> Result<UndoResult, String>;

#[tauri::command]
#[specta::specta]
fn redo(
    domain: String,
    build: State<Mutex<BuildInfo>>,
    game: State<Arc<RwLock<GameData>>>,
) -> Result<UndoResult, String>;
```

`UndoResult` includes the new `BuildStats` plus flags indicating what changed (so the frontend knows which parts to refresh).

##### 6.5.7 — Frontend integration

Keyboard shortcuts: `Ctrl+Z` → undo active tab's domain, `Ctrl+Shift+Z` / `Ctrl+Y` → redo.

The frontend determines which domain to undo based on the current view:
- Skill tree tab → `"tree"`
- Items tab → `"items"` 
- Skills tab → `"skills"`

##### 6.5.8 — Implementation checklist

```
src-tauri/src/build/undo.rs:
  - [ ] UndoStack<T: Clone> with push(), undo(), redo(), can_undo(), can_redo(), clear()
  - [ ] Max 100 undo states
  - [ ] TreeSnapshot, ItemSnapshot, SkillSnapshot structs (derive Clone)

src-tauri/src/lib.rs:
  - [ ] Add tree_undo, item_undo, skill_undo fields to BuildInfo (#[serde(skip)])
  - [ ] Push undo states in existing commands (update_selected_nodes, equip_item, etc.)
  - [ ] Add undo, redo commands
  - [ ] Register in tauri_specta::collect_commands![]
  - [ ] UndoResult struct (Serialize, Deserialize, Type)

src/components/Header.svelte (or layout):
  - [ ] Add Ctrl+Z / Ctrl+Shift+Z keyboard listener
  - [ ] Determine domain from current route/tab
  - [ ] Call commands.undo(domain) / commands.redo(domain)
  - [ ] Refresh appropriate UI state from result
```

---

#### 6.6 — Wire Up Home Page

Connect the `+page.svelte` home page to real Tauri commands for build management.

##### 6.6.1 — Current state

The home page (`src/routes/+page.svelte`) has:
- `SavedBuild` interface with `id`, `name`, `class`, `lastModified`, `nodeCount`
- Empty `savedBuilds` array (placeholder data commented out)
- `newBuild()` stub — resets state and navigates to `/skilltree`
- `loadBuild(id)` stub — just navigates (no actual load)
- `deleteBuild(id)` stub — filters local array only

##### 6.6.2 — Changes needed

```svelte
<!-- +page.svelte changes -->
<script lang="ts">
    import { commands } from '../bindings';
    import { goto } from '$app/navigation';
    import { getBuildState, resetBuildState } from '$lib/buildState.svelte';

    let savedBuilds = $state<BuildSummary[]>([]);
    let loading = $state(true);

    // Load build list on mount
    $effect(() => {
        commands.listBuilds()
            .then(builds => { savedBuilds = builds; loading = false; })
            .catch(err => { console.error(err); loading = false; });
    });

    async function newBuild() {
        resetBuildState();
        await goto('/skilltree');
    }

    async function loadBuild(id: string) {
        const stats = await commands.loadBuild(id);
        // Update buildState from loaded data
        // Navigate to skill tree
        await goto('/skilltree');
    }

    async function deleteBuild(id: string) {
        await commands.deleteBuild(id);
        savedBuilds = savedBuilds.filter(b => b.id !== id);
    }

    async function saveBuild() {
        const id = await commands.saveBuild(buildState.name || "Unnamed Build");
        // Refresh list
        savedBuilds = await commands.listBuilds();
    }
</script>
```

##### 6.6.3 — Import/Export UI

Add import/export controls (can be on the home page or a dedicated import tab):

- **Export button** (in Header or Sidebar): calls `commands.exportPobCode()`, copies result to clipboard
- **Import text field**: paste PoB share code, calls `commands.importPobCode(code)`
- **PoE Account import button**: calls `commands.startPoeAuth()`, then shows character picker

##### 6.6.4 — Post-load state sync

After `loadBuild()`, the frontend needs to refresh all reactive state from the backend:
1. Call `commands.getEquippedItems()` → update `buildState.equippedItems`
2. Call `commands.getInventoryItems()` → update `buildState.inventoryItems` 
3. Call `commands.getSkillGroups()` → update `buildState.skillGroups`
4. Call `commands.getTreeJson()` → update `buildState.treeData` (if tree version changed)
5. `BuildStats` comes back from the `loadBuild` response directly
6. Class, level, ascendancy, bloodline — either returned in `loadBuild` response or queried separately

**Consider**: Extend `loadBuild` response to return all needed state in a single IPC call (a `LoadedBuildState` struct) to avoid N+1 round trips.

##### 6.6.5 — Auto-save

For v1, builds are explicitly saved (user clicks "Save"). Future enhancement: auto-save on a timer or on significant changes.

##### 6.6.6 — Implementation checklist

```
src/routes/+page.svelte:
  - [ ] Replace SavedBuild interface with BuildSummary from bindings.ts
  - [ ] Load build list on mount via commands.listBuilds()
  - [ ] Wire newBuild() to resetBuildState() + navigate
  - [ ] Wire loadBuild(id) to commands.loadBuild(id) + sync frontend state + navigate
  - [ ] Wire deleteBuild(id) to commands.deleteBuild(id) + refresh list
  - [ ] Add save button (name input + commands.saveBuild())
  - [ ] Add import PoB code input + button
  - [ ] Add export PoB code button
  - [ ] Loading spinner while fetching build list

src/routes/skilltree/+page.svelte (or Header.svelte):
  - [ ] Add "Save" button calling commands.saveBuild()
  - [ ] Add "Export Code" button calling commands.exportPobCode()
  - [ ] Add "Import Code" input calling commands.importPobCode()

src/lib/buildState.svelte.ts:
  - [ ] Add function to bulk-update state from loaded build response
  - [ ] Ensure resetBuildState() also clears undo stacks (via backend call)
```

---

### New Dependencies Summary

| Crate | Version | Purpose | Phase Step |
|---|---|---|---|
| `flate2` | `1` | zlib compress/decompress for PoB share codes | 6.3 |
| `base64` | `0.22` | Base64 encode/decode for PoB share codes | 6.3 |
| `quick-xml` | `0.37` (features: `serialize`) | XML parse/generate for PoB format | 6.3 |
| `uuid` | `1` (features: `v4`) | Generate unique build file IDs | 6.2 |
| `sha2` | `0.10` | SHA-256 for PKCE code challenge | 6.4 |

**Already present**: `reqwest` (HTTP), `governor` (rate limiting), `tempfile` (atomic writes), `serde`/`serde_json`, `tokio` (async), `thiserror`.

### New File Structure

```
src-tauri/src/
├── build/
│   ├── mod.rs          # Re-exports
│   ├── codec.rs        # PoB share code encode/decode (XML ↔ zlib ↔ base64url)
│   ├── import.rs       # PoE API character import + OAuth flow
│   ├── undo.rs         # UndoStack<T> generic undo/redo
│   └── saved.rs        # SavedBuildData struct + to/from conversion
├── storage/
│   ├── mod.rs          # Re-exports StorageManager, BuildSummary
│   └── manager.rs      # Real StorageManager (replaces stub)
│   (file_system.rs removed — dead legacy code)
├── client/
│   ├── mod.rs          # Re-exports
│   └── poe.rs          # PoeClient (currently empty → real OAuth + API calls)
└── ...
```

### Implementation Order

The steps should be implemented in this order due to dependencies:

1. **6.1 — Serialization Prerequisites** (no external deps, enables everything else)
2. **6.2 — Build Save/Load** (depends on 6.1, adds `uuid`)
3. **6.5 — Undo/Redo** (independent, can parallel with 6.3)
4. **6.3 — PoB Share Codes** (depends on 6.1 for item serialization, adds `flate2`/`base64`/`quick-xml`)
5. **6.4 — PoE Character Import** (depends on 6.1 for item types, adds `sha2`)
6. **6.6 — Wire Up Home Page** (depends on 6.2 at minimum, benefits from 6.3-6.5)

### How to Verify Phase 6 is Complete

- [ ] Save → close → reopen → load → identical state (all nodes, items, gems, class, level preserved)
- [ ] Save → load → `BuildStats` matches pre-save values exactly
- [ ] Generate share code → paste in real PoB → build loads with correct tree, items, skills
- [ ] Import a real PoB share code → build displays correctly (tree nodes allocated, items equipped)
- [ ] Round-trip: export from Rusty Builds → import back → identical state
- [ ] PoE character import: OAuth flow completes, character list shows, import loads tree + items
- [ ] Undo/redo tracks tree, item, and skill changes (separate stacks per domain)
- [ ] Ctrl+Z undoes last action, Ctrl+Shift+Z redoes
- [ ] Home page lists all saved builds with name, class, level, date
- [ ] Delete build removes from list and disk
- [ ] Builds directory uses UUID filenames (no path traversal possible)
- [ ] Atomic file writes — no corrupted saves on crash

---

## Phase 7: Configuration, Party, Jewels & Advanced Features

### Steps

#### 7.1 — Configuration System (`config/`)

Mirror PoB's `ConfigOptions.lua`:
- Enemy level, boss type, enemy resistances
- Charge counts (power/frenzy/endurance)
- Buff toggles (onslaught, fortify, unholy might, etc.)
- Conditions (low life, full life, leeching, etc.)

Each option feeds mods into the `config` ModDB layer during setup. Add `config: ModDB` to `ModDBLayers`.

#### 7.2 — Jewel System

- **Regular jewels**: Parse mods → add to tree layer (like items but in tree sockets)
- **Cluster jewels**: Expand tree with additional notable nodes (from `ClusterJewels.json`)
- **Timeless jewels**: Seed-based node stat replacement (from `TimelessJewelData/`)

#### 7.3 — Party Support

Import party member builds (share codes), calculate party-wide auras and curses, feed into config layer.

#### 7.4 — Frontend Tabs

- `ConfigTab.svelte` — Generated from config option metadata
- `NotesTab.svelte` — Simple textarea saved with build
- `PartyTab.svelte` — Party member import + aura toggle

### How to Verify Phase 7 is Complete

- [ ] Power Charge: 3 → crit chance increases correctly
- [ ] Enemy resistance config changes DPS numbers
- [ ] Regular jewel equip adds stats
- [ ] Config tab shows all options from PoB

---

## Phase 8: Polish, Performance & Trade

### Steps

#### 8.1 — Trade Query Generation

Generate trade site queries weighted by DPS impact per mod. Open in browser via `tauri-plugin-opener`.

#### 8.2 — Node Power Overlay

For each unallocated adjacent node, calculate DPS/Life delta. Color-code nodes by impact (green = gain, red = loss). PoB's most loved feature.

#### 8.3 — Performance Optimization

Many optimizations already baked in:
- ✅ `StatId` enum — no string hashing in modifier lookups
- ✅ `FxHashMap` — fast hashing everywhere
- ✅ `CalcContext` — conditional evaluation without API changes
- ✅ Compact `Modifier` — `SourceId(u32)`, `SmallVec<[ModTag; 2]>`
- ✅ Layered `ModDB` — targeted cache invalidation
- ✅ `stat_table.rs` codegen — zero runtime JSON loading
- ✅ Raw JSON string for tree IPC — no double serialization

**Remaining**: Profile with `cargo flamegraph`, consider `rayon` for parallel per-skill DPS, optimize `Modifier` struct layout for cache lines. Target: full recalc < 20ms.

#### 8.4 — Release Profile

```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

#### 8.5 — CI/CD

GitHub Actions: `cargo test`, `cargo clippy`, `cargo fmt --check`, `bun run check`, Tauri build, upload artifacts.

### How to Verify Phase 8 is Complete

- [ ] Full calc benchmark < 20ms
- [ ] Node power overlay matches PoB recommendations
- [ ] Trade query opens correct URL
- [ ] CI pipeline green on push

---

## Reference: PoB Source Map

When implementing a feature, look at the corresponding PoB Lua source. The repo is at [github.com/PathOfBuildingCommunity/PathOfBuilding](https://github.com/PathOfBuildingCommunity/PathOfBuilding) (branch: `dev`).

| Our Module | PoB Reference | Lines | Phase |
|---|---|---|---|
| `data/gems.rs` (RePoE) | `CalcTools.lua`, `CalcActiveSkill.lua` | ~1500 | 3 |
| `modifier/parser.rs` (`resolve()`) | `ModParser.lua` | ~2500 | ✅ 2 |
| `modifier/mod_db.rs` | `ModDB.lua` + `ModStore.lua` | ~900 | ✅ 2 |
| `item/parser.rs` | `Item.lua` (`ParseRaw()`) | ~2000 | 4 |
| `item/local_mods.rs` | `Item.lua` (`calcLocal()`) | ~400 | 4 |
| `calc/offence.rs` | `CalcOffence.lua` | ~4000 | 5 |
| `calc/defence.rs` | `CalcDefence.lua` | ~2500 | 5 |
| `calc/active.rs` | `CalcActiveSkill.lua` | ~1500 | 5 |
| `calc/perform.rs` | `CalcPerform.lua` | ~900 | 5 |
| `calc/setup.rs` | `CalcSetup.lua` | ~1200 | 5 |
| `calc/triggers.rs` | `CalcTriggers.lua` | ~800 | 5 |
| `build/codec.rs` | `Build.lua` (encode/decode) | ~500 | 6 |
| `config/options.rs` | `ConfigOptions.lua` | ~1200 | 7 |

**Total PoB calc engine**: ~15,000+ lines of Lua.
**Core formula**: `Override check → base × (1 + sum_of_increases%) × product_of_mores`
**PoB ModDB mod types**: BASE, INC, MORE, FLAG, OVERRIDE, LIST, MAX, MIN
**Key query methods**: `Sum()`, `More()`, `Flag()`, `Override()`, `List()`, `Tabulate()`, `Max()`, `Min()`
