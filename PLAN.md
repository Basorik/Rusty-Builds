# Rusty Builds — Development Plan

**Goal**: Rebuild [Path of Building](https://github.com/PathOfBuildingCommunity/PathOfBuilding) as a Tauri desktop app using Rust and Svelte, targeting **Path of Exile 1** with full calculation parity — while learning Rust progressively.

> This document is your implementation roadmap. Each phase builds on the last, introduces new Rust concepts, and produces a testable milestone. Work through them in order.

---

## Table of Contents

- [Design Decisions](#design-decisions)
- [What Exists Today](#what-exists-today)
- [PoB Feature Inventory](#pob-feature-inventory)
- [Target Architecture](#target-architecture)
- [Phase 1: Foundation & Data Layer](#phase-1-foundation--data-layer)
- [Phase 2: Modifier System & Passive Tree Stats](#phase-2-modifier-system--passive-tree-stats)
- [Phase 3: Skill & Gem System](#phase-3-skill--gem-system)
- [Phase 4: Item System](#phase-4-item-system)
- [Phase 5: Full Calculation Engine](#phase-5-full-calculation-engine)
- [Phase 6: Build Management & Import/Export](#phase-6-build-management--importexport)
- [Phase 7: Configuration, Party, Jewels & Advanced Features](#phase-7-configuration-party-jewels--advanced-features)
- [Phase 8: Polish, Performance & Trade](#phase-8-polish-performance--trade)
- [Rust Learning Progression](#rust-learning-progression)
- [Reference: PoB Source Map](#reference-pob-source-map)

---

## Design Decisions

These are locked in. Don't revisit them until Phase 8.

| Decision | Choice | Rationale |
|---|---|---|
| Game version | **POE 1 only** | Simpler scope; POE 1 has mature, stable data |
| Calc engine | **Full parity with PoB** | This is the app's core value; no shortcuts |
| Data source | **Bundled static JSON** | Updated with app releases; no live API dependency for game data |
| Frontend split | **Balanced** | Selection validation stays in Pixi.js for 60fps responsiveness; all stat calculations, mod parsing, item logic, storage in Rust |
| Build code format | **PoB-compatible** | Share codes must import/export with the real Path of Building app |
| Learning approach | **Progressive** | Each phase introduces new Rust concepts; don't skip phases |

### Performance Decisions (Bake In From Day One)

These are architectural choices that would be extremely painful to retrofit later. Implement them from their first appearance — not as Phase 8 optimizations.

| Decision | Choice | Refactor Pain If Skipped | Rationale |
|---|---|---|---|
| Stat identification | **`StatId` enum** (not String) | Extreme — touches every modifier, every query, every calc function | Avoids string hashing/comparison on every modifier lookup; catches typos at compile time; enables `match` exhaustiveness checks |
| Hash maps | **`FxHashMap`** via `rustc-hash` | High — grep-replace every `HashMap` import | Faster non-cryptographic hashing; stat IDs are small integers, perfect for `Fx`; drop-in replacement API |
| Modifier queries | **`CalcContext` parameter** on all `ModDB` queries | Extreme — every call site needs a new argument | Conditional mods ("while Leeching", "per Power Charge") need calc context to evaluate; adding it later means touching every query call |
| Modifier layout | **Compact struct**: `SourceId(u32)`, `SmallVec` for tags | High — changes the core `Modifier` memory layout | No heap-allocated Strings in the hot path; `SourceId` is a u32 index into a name table; `SmallVec<[ModTag; 2]>` keeps 0-2 tags inline |
| App state | **`RwLock` split state** or **channel-based** architecture | High — rethink all state access patterns | Replace `Mutex<BuildInfo>`; calc reads don't block UI writes; separate `GameData` (read-only) from `BuildState` (read-write) |
| ModDB architecture | **Layered composition** | Moderate — restructure merge logic | Separate tree/item/gem/config layers; changing one layer doesn't rebuild all; enables targeted cache invalidation |
| Internal errors | **Typed `thiserror` enums** | Moderate — replace every `String` error | `String` errors only at IPC boundary; internal code uses typed enums for pattern matching and actionable error handling |

> **Rule**: If you see `HashMap` in this plan's code examples, read it as `FxHashMap`. If you see `stat: String`, read it as `stat: StatId`. The examples below use the correct types.

**New Cargo dependencies to add early** (Phase 1):
```toml
rustc-hash = "2"    # FxHashMap / FxHashSet
bitflags = "2"      # ModFlag, KeywordFlag (Phase 2, but add now)
smallvec = "1"      # Inline small collections in Modifier
```

**Excluded from scope**: POE 2 support, mobile/responsive design, auto-updating game data from API, custom league mechanics beyond what PoB supports.

---

## What Exists Today

Your starting point. Understand what's already built before changing anything.

### Rust Backend (`src-tauri/src/`)

| File | Status | What's There |
|---|---|---|
| `main.rs` | ✅ Complete | Binary entry point, delegates to `lib::run()` |
| `lib.rs` | ⚠️ Partial | 6 Tauri commands (`greet`, `update_build_info`, `update_selected_nodes`, `get_available_tree_versions`, `load_tree_version`, `get_tree_json`). Types: `BuildInfo`, `BuildSelection`, `BuildStats`, `Class` enum (7 classes + ascendancies), `Bloodline` enum. `DEFAULT_TREE_VERSION` constant. Setup: initializes `StorageManager`, manages `Mutex<BuildInfo>` and `Arc<RwLock<GameData>>`, exports TS bindings via tauri-specta. STR/DEX/INT computed from `PassiveNode.granted_*` fields; other stats accumulated via `StatAccumulator` in `stats.rs` |
| `stats.rs` | ✅ Complete | `StatAccumulator` — placeholder stat text parser. Extracts first number from stat strings, replaces numbers with `#` to form template keys, sums values. Will be replaced by `ModDB` + `ModParser` in Phase 2 |
| `data/mod.rs` | ✅ Complete | `DataLoader` trait, `DataError` enum (thiserror), `GameData` struct (holds `PassiveTree` + `source_names`), `SourceId(u32)` newtype with `intern_source()` |
| `data/tree.rs` | ✅ Complete | `PassiveTree`, `PassiveNode` (with `granted_strength/dexterity/intelligence`, `mastery_effects`, `out_connections`/`in_connections`, `is_ascendancy_start`, `class_start_index`), `PassiveGroup`, `ClassData` (with `base_str/base_dex/base_int`), `AscendancyData`, `BloodlineData`, `TreeConstants`, `TreePoints`, `MasteryEffect`. `get_node(skill_id)` helper. Unit tests with `include_str!` |
| `data/stat_id.rs` | ✅ Complete | 320-variant `StatId` enum (`#[repr(u16)]`) generated from `SkillStatMap.json`. `from_name()` lookup via `OnceLock<FxHashMap>` |
| `data/*.rs` (stubs) | ❌ Stubs | `bases.rs`, `gems.rs`, `mods.rs`, `skills.rs`, `uniques.rs` — empty, module structure only |
| `models.rs` | ⚠️ Legacy | `LiteNode` struct (id, x, y, icon) with rkyv serialization — unused, to be removed |
| `commands.rs` | ❌ Empty | Placeholder for future command extraction from `lib.rs` |
| `tree.rs` (root) | ❌ Empty | Placeholder, unused |
| `client/poe.rs` | ❌ Skeleton | `PoeClient` struct (empty), `PoeClientError` enum (only `Network` variant) |
| `storage/manager.rs` | ❌ Stub | `StorageManager` struct with no-op `new()` |
| `storage/file_system.rs` | ✅ Complete | `FileCache` with rkyv binary caching for `Vec<LiteNode>`, atomic writes via tempfile |

### Svelte Frontend (`src/`)

| File | Status | What's There |
|---|---|---|
| `components/SkillTree.svelte` | ✅ Complete | Pixi.js WebGL canvas: full tree rendering, pan/zoom, BFS-validated node selection, spatial grid hit detection, viewport culling, tooltips |
| `components/Header.svelte` | ✅ Complete | Level/class/ascendancy/bloodline dropdowns, syncs to Rust via `updateBuildInfo` |
| `components/Sidebar.svelte` | ⚠️ Partial | Shows node count only. "More stats coming soon" placeholder |
| `routes/+page.svelte` | ⚠️ Partial | Home page with "New Build" button, save/load/delete UI scaffolding (not wired) |
| `routes/skilltree/+page.svelte` | ✅ Complete | Composes Header + Sidebar + SkillTree. Fetches tree JSON from Rust via `commands.getTreeJson()` on mount, passes parsed data to SkillTree component |
| `bindings.ts` | ✅ Auto-gen | TypeScript types for all Rust commands/types via tauri-specta |
| `data.json` | ⚠️ Unused | POE 1 tree data (~160k lines). No longer imported — frontend fetches from Rust via `get_tree_json` command. Can be deleted |

### Key Problems to Fix

1. ~~**Wrong game data** — `data.json` is POE 2; you need POE 1~~ ✅ Fixed — POE 1 tree data fetched, versioned at `data/tree/3.27.0g/data.json`, loaded by Rust and served to frontend
2. ~~**Stats are all zeros**~~ ⚠️ Partial — STR/DEX/INT now computed from `PassiveNode.granted_*` fields; other stats use placeholder `StatAccumulator` (text parsing, not `ModDB`). Phase 2 will replace this with proper typed modifiers
3. **No persistence** — Builds exist only in RAM; lost on app close
4. **Storage never called** — `StorageManager` and `FileCache` are initialized but never used
5. **POE client empty** — No actual API methods implemented
6. **No modifier system** — The heart of PoB's engine doesn't exist yet. `StatAccumulator` is a stop-gap
7. **No items, skills, config, or calc engine** — Only the passive tree viewer + basic stat accumulation works

### Dependencies Already Installed

**Rust** (`Cargo.toml`): tauri 2, serde/serde_json, tokio, reqwest, governor, rkyv, tempfile, specta/tauri-specta/specta-typescript, thiserror, log/tauri-plugin-log, rustc-hash, regex

**To add in Phase 2**: `bitflags` (modifier flags), `smallvec` (compact modifier tags)

**Frontend** (`package.json`): @tauri-apps/api, pixi.js, svelte 5, sveltekit, vite, typescript

---

## PoB Feature Inventory

Everything the real Path of Building does — organized by system. This is what you're rebuilding.

### Calculation Engine (PoB: `src/Modules/Calc*.lua`)

This is ~60% of the total effort. PoB's calc engine is massive.

| PoB Module | What It Does | Your Rust Module |
|---|---|---|
| `CalcPerform.lua` | Orchestrates full character calculation | `calc/perform.rs` |
| `CalcSetup.lua` | Gathers all mod sources (tree, items, gems, config) | `calc/setup.rs` |
| `CalcOffence.lua` | Skill DPS, hit damage, DoT, crit, speed, projectiles | `calc/offence.rs` |
| `CalcDefence.lua` | Life, ES, armour, evasion, block, resistances, regen, leech | `calc/defence.rs` |
| `CalcActiveSkill.lua` | Resolves active skill + support gem linking | `calc/active.rs` |
| `CalcTriggers.lua` | CoC, CwDT, Mjolner trigger mechanics | `calc/triggers.rs` |
| `CalcMirages.lua` | General's Cry, clone skills | `calc/minions.rs` |
| `CalcBreakdown.lua` | Detailed stat breakdown for display | `calc/result.rs` |
| `CalcSections.lua` | Breakdown sections for UI | `calc/result.rs` |
| `CalcTools.lua` | Shared calc utilities | `calc/util.rs` |

### Modifier System (PoB: `src/Classes/Mod*.lua`)

The core data structure that everything else feeds into.

| PoB Module | What It Does | Your Rust Module |
|---|---|---|
| `ModDB.lua` | Modifier database — stores and queries all modifiers | `modifier/mod_db.rs` |
| `ModList.lua` | Ordered modifier list | `modifier/types.rs` |
| `ModStore.lua` | Modifier storage and lookup | `modifier/store.rs` |
| `ModParser.lua` | Parses mod text → structured modifier objects | `modifier/parser.rs` |
| `ModTools.lua` | Modifier manipulation utilities | `modifier/types.rs` |

### Data (PoB: `src/Data/`)

| PoB Data | What It Contains | Your Rust Module |
|---|---|---|
| `Gems.lua` | All skill/support gems with per-level stats | `data/gems.rs` |
| `Skills/` | Individual skill stat data | `data/skills.rs` |
| `Bases/` | Base item types (weapons, armour, jewellery) | `data/bases.rs` |
| `Uniques/` | Complete unique item database | `data/uniques.rs` |
| `Mod*.lua` (12+ files) | Item/jewel/flask/master/veiled/graft mods | `data/mods.rs` |
| `Minions.lua` | Minion base stats | `data/minions.rs` |
| `Spectres.lua` | Spectre data | `data/minions.rs` |
| `Pantheons.lua` | Pantheon bonuses | `data/pantheons.rs` |
| `ClusterJewels.lua` | Cluster jewel notables | `data/jewels.rs` |
| `TimelessJewelData/` | Timeless jewel seed/conversion tables | `data/jewels.rs` |
| `SkillStatMap.lua` | Stat IDs → calculation variable mapping | `data/skills.rs` |
| `Enchantment*.lua` | Helm/boots/gloves/weapon/flask enchants | `data/enchants.rs` |

### UI Tabs (PoB: `src/Classes/*Tab.lua`)

| Tab | PoB File | Your Svelte Component | Core Rust Support |
|---|---|---|---|
| Tree | `TreeTab.lua`, `PassiveTree.lua`, `PassiveTreeView.lua` | `SkillTree.svelte` (exists) | `data/tree.rs` |
| Skills | `SkillsTab.lua`, `SkillListControl.lua`, `GemSelectControl.lua` | `SkillsTab.svelte` | `skill/` module |
| Items | `ItemsTab.lua`, `Item.lua`, `ItemDBControl.lua`, `ItemSlotControl.lua` | `ItemsTab.svelte` | `item/` module |
| Calcs | `CalcsTab.lua`, `CalcBreakdownControl.lua`, `CalcSectionControl.lua` | `CalcsTab.svelte` | `calc/result.rs` |
| Config | `ConfigTab.lua`, `ConfigOptions.lua` | `ConfigTab.svelte` | `config/` module |
| Import | `ImportTab.lua` | `ImportTab.svelte` | `build/import.rs` |
| Notes | `NotesTab.lua` | `NotesTab.svelte` | (simple text storage) |
| Party | `PartyTab.lua` | `PartyTab.svelte` | `calc/` (party auras) |

### Build Management (PoB: `src/Classes/Build*.lua`, `src/Modules/Build*.lua`)

| Feature | PoB Source | Your Rust Module |
|---|---|---|
| Save/Load | `Build.lua`, `BuildList.lua` | `build/manager.rs` |
| Share codes | `Build.lua` (base64+zlib XML) | `build/codec.rs` |
| Character import | `ImportTab.lua` | `build/import.rs` |
| Undo/Redo | `UndoHandler.lua` | `build/undo.rs` |
| Passive spec | `PassiveSpec.lua` | `tree/spec.rs` |

### Trade (PoB: `src/Classes/Trade*.lua`)

| Feature | PoB Source | Your Rust Module |
|---|---|---|
| Query generation | `TradeQueryGenerator.lua` | `trade.rs` |
| Rate limiting | `TradeQueryRateLimiter.lua` | `trade.rs` (governor crate) |
| HTTP requests | `TradeQueryRequests.lua` | `trade.rs` (reqwest) |

---

## Target Architecture

```
┌───────────────────────────────────────────────────────────┐
│                      Tauri 2 Shell                        │
│                                                           │
│  ┌──────────────────┐  IPC (Commands)  ┌───────────────┐  │
│  │  Svelte Frontend │ ◄──────────────► │ Rust Backend  │  │
│  │                  │                  │               │  │
│  │  Tabs:           │                  │ data/         │  │
│  │  • Tree (Pixi)   │  ← CalcResult → │ ├ tree.rs     │  │
│  │  • Skills        │                  │ ├ gems.rs     │  │
│  │  • Items         │  ← Tooltip    → │ ├ bases.rs    │  │
│  │  • Calcs         │                  │ ├ uniques.rs  │  │
│  │  • Config        │  → UserAction → │ ├ mods.rs     │  │
│  │  • Import        │                  │               │  │
│  │  • Notes         │                  │ modifier/     │  │
│  │  • Party         │                  │ ├ mod_db.rs   │  │
│  │                  │                  │ ├ parser.rs   │  │
│  │  Node selection  │                  │ ├ types.rs    │  │
│  │  stays in Pixi   │                  │               │  │
│  │  for 60fps       │                  │ calc/         │  │
│  │                  │                  │ ├ perform.rs  │  │
│  │                  │                  │ ├ offence.rs  │  │
│  │                  │                  │ ├ defence.rs  │  │
│  │                  │                  │ ├ active.rs   │  │
│  │                  │                  │ ├ setup.rs    │  │
│  │                  │                  │               │  │
│  │                  │                  │ skill/        │  │
│  │                  │                  │ item/         │  │
│  │                  │                  │ build/        │  │
│  │                  │                  │ config/       │  │
│  └──────────────────┘                  └───────────────┘  │
└───────────────────────────────────────────────────────────┘
```

### Target File Tree (Rust)

```
src-tauri/
├── data/                          # Bundled game data (JSON files)
│   ├── tree/
│   │   └── <version>/data.json    # Versioned POE 1 passive tree (e.g., 3.27.0g/)
│   └── pob/                       # PoB game data from repoe-fork (123 files)
│       ├── SkillStatMap.json
│       ├── Gems.json
│       ├── Bases/
│       ├── Skills/
│       ├── StatDescriptions/
│       ├── Uniques/
│       └── ...
└── src/
    ├── main.rs                    # Binary entry (exists)
    ├── lib.rs                     # App setup, command registration, DEFAULT_TREE_VERSION (exists)
    ├── stats.rs                   # StatAccumulator — placeholder (exists, Phase 2 replaces)
    │
    ├── data/                      # Game data loading & types
    │   ├── mod.rs                 # DataLoader trait, DataError, GameData, SourceId (exists)
    │   ├── stat_id.rs             # StatId enum (u16, 320 variants) — all stat identifiers (exists)
    │   ├── tree.rs                # PassiveTree, PassiveNode, PassiveGroup, etc. (exists)
    │   ├── gems.rs                # GemData, GemLevel, GemTag, GemType (stub)
    │   ├── bases.rs               # BaseItem, ItemClass (stub)
    │   ├── uniques.rs             # UniqueItem, UniqueModRange (stub)
    │   ├── mods.rs                # ModDefinition, ModDomain (stub)
    │   ├── skills.rs              # SkillStatMap (stub)
    │   ├── minions.rs             # MinionData, SpectreData (future)
    │   └── jewels.rs              # ClusterJewel, TimelessJewelData (future)
    │
    ├── modifier/                  # Modifier database & parsing
    │   ├── mod.rs
    │   ├── types.rs               # Modifier, ModType, ModFlag, KeywordFlag, ModTag
    │   ├── mod_db.rs              # ModDB (store + query modifiers)
    │   ├── parser.rs              # ModParser (text → Modifier)
    │   └── store.rs               # ModStore trait
    │
    ├── calc/                      # Calculation engine
    │   ├── mod.rs
    │   ├── perform.rs             # CalcPerform — orchestrates full calc pass
    │   ├── setup.rs               # CalcSetup — gathers mod sources into ModDB
    │   ├── offence.rs             # CalcOffence — DPS, hit, DoT, crit, speed
    │   ├── defence.rs             # CalcDefence — life, ES, armour, resists, regen
    │   ├── active.rs              # CalcActiveSkill — resolve active + supports
    │   ├── triggers.rs            # Trigger mechanics (CoC, CwDT, etc.)
    │   ├── minions.rs             # Minion calculation
    │   ├── result.rs              # CalcResult, StatBreakdown
    │   └── util.rs                # Shared calc utilities
    │
    ├── skill/                     # Skill gem system
    │   ├── mod.rs
    │   ├── gem.rs                 # ActiveGem, SupportGem
    │   ├── group.rs               # SkillGroup (socket group)
    │   ├── resolved.rs            # ResolvedSkill (active + supports merged)
    │   └── skill_set.rs           # SkillSet (all groups for a build)
    │
    ├── item/                      # Item system
    │   ├── mod.rs
    │   ├── types.rs               # Item, ItemSlot, ItemMod, Rarity
    │   ├── parser.rs              # Parse item text from clipboard
    │   ├── tooltip.rs             # Display impl for item tooltips
    │   ├── crafting.rs            # Add/remove mods, validate affixes
    │   └── equip.rs               # Equipment (all equipped items → ModDB)
    │
    ├── build/                     # Build management
    │   ├── mod.rs
    │   ├── types.rs               # Build (aggregates all systems)
    │   ├── manager.rs             # CRUD for builds on disk
    │   ├── codec.rs               # PoB-compatible encode/decode (base64+zlib+XML)
    │   ├── import.rs              # Character import from PoE API
    │   └── undo.rs                # Generic UndoHandler<T>
    │
    ├── config/                    # Configuration system
    │   ├── mod.rs
    │   ├── options.rs             # All config options (enemy, charges, buffs, etc.)
    │   └── types.rs               # ConfigOption enum with UI metadata
    │
    ├── trade.rs                   # Trade query generation
    │
    ├── models.rs                  # LiteNode (exists, legacy — to be removed)
    ├── commands.rs                # Future: extracted commands (exists, empty)
    ├── client/                    # External API client (exists, skeleton)
    │   ├── mod.rs
    │   └── poe.rs
    └── storage/                   # File storage (exists)
        ├── mod.rs
        ├── manager.rs
        └── file_system.rs
```

---

## Phase 1: Foundation & Data Layer

### Rust Concepts You'll Learn

- **Structs** with `#[derive(Serialize, Deserialize, Clone, Debug)]`
- **Enums** with data (tagged unions for item types, gem types, node types)
- **Trait definition and implementation** (`Display`, `Default`, custom `DataLoader` trait)
- **`thiserror`** for typed error handling (internal enums, `String` only at IPC boundary)
- **`serde`** for JSON deserialization
- **Module organization** (`mod`, `pub`, `pub(crate)`, re-exports)
- **Pattern matching** (`match` on enums)
- **`FxHashMap`** (via `rustc-hash`) — faster non-cryptographic hashing for internal maps
- **`Vec`**, **`Option<T>`**, **`Result<T, E>`** idioms
- **Newtype pattern** (`struct StatId(u16)`) for zero-cost type safety
- **Unit testing** with `#[cfg(test)]`

### What You're Building

Load all POE 1 game data from bundled JSON files into typed Rust structs. Serve that data to the frontend via Tauri commands. Replace the current POE 2 data.json with POE 1 tree data.

### Steps

#### 1.1 — Get POE 1 Passive Tree Data

The current `src/data.json` contains POE 2 tree data. You need POE 1.

**Where to get it:**
- PoB's tree data lives in `src/TreeData/` in their repo — it's version-specific JSON
- The official API endpoint format: `https://www.pathofexile.com/passive-skill-tree` (returns JSON)
- PoB exports tree data during their release process

**What to do:**
1. Download the latest POE 1 tree JSON from PoB's `TreeData/` or the official API
2. Save it as `src-tauri/data/tree.json`
3. Keep `src/data.json` temporarily (frontend still uses it) — you'll switch in step 1.6

#### 1.2 — Create the `data` Module

Create `src-tauri/src/data/mod.rs`:

```rust
pub mod tree;
pub mod gems;
pub mod bases;
pub mod uniques;
pub mod mods;
pub mod skills;
pub mod stat_id;

// Re-export the main types
pub use tree::PassiveTree;
pub use stat_id::StatId;
```

Then create `src-tauri/src/data/stat_id.rs` — this is a **critical performance decision**:

```rust
/// Every stat in the game gets a unique integer ID.
/// This avoids string hashing and comparison in the hot path.
///
/// Start with the stats you need for Phase 1-2. Add more as you go.
/// The `#[repr(u16)]` ensures compact storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum StatId {
    // Core attributes
    Strength,
    Dexterity,
    Intelligence,
    // Defence
    Life,
    Mana,
    EnergyShield,
    Armour,
    Evasion,
    // Resistances
    FireResistance,
    ColdResistance,
    LightningResistance,
    ChaosResistance,
    // Offence
    AttackSpeed,
    CastSpeed,
    PhysicalDamage,
    SpellDamage,
    CriticalStrikeChance,
    CriticalStrikeMultiplier,
    // ... expand as needed. Aim for ~200 stats by Phase 5.
}

impl StatId {
    /// Lookup a StatId from the mod text token.
    /// Returns None for unrecognized stats (log a warning).
    pub fn from_text(text: &str) -> Option<Self> {
        // Build a static lookup map (FxHashMap<&str, StatId>) via OnceLock
        // or a match block. Match is fine to start with.
        match text {
            "Strength" => Some(Self::Strength),
            "Dexterity" => Some(Self::Dexterity),
            "Intelligence" => Some(Self::Intelligence),
            "maximum Life" | "Life" => Some(Self::Life),
            "maximum Mana" | "Mana" => Some(Self::Mana),
            "maximum Energy Shield" | "Energy Shield" => Some(Self::EnergyShield),
            "Attack Speed" => Some(Self::AttackSpeed),
            "Cast Speed" => Some(Self::CastSpeed),
            "Fire Resistance" => Some(Self::FireResistance),
            "Cold Resistance" => Some(Self::ColdResistance),
            "Lightning Resistance" => Some(Self::LightningResistance),
            "Chaos Resistance" => Some(Self::ChaosResistance),
            _ => None,
        }
    }
}
```

**Why this matters**: Every modifier lookup, every calc query, every stat aggregation will key on `StatId`. With a `u16` enum, hash lookups are essentially free (identity hash). With strings, you'd pay for hashing + heap allocation on every operation. This single decision affects ~80% of your hot paths.

#### 1.3 — Define Passive Tree Types (`data/tree.rs`)

Study the POE 1 tree JSON structure. Then create matching Rust types:

```rust
use serde::{Deserialize, Serialize};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveTree {
    pub nodes: FxHashMap<String, PassiveNode>,
    pub groups: FxHashMap<String, PassiveGroup>,
    pub classes: Vec<ClassData>,
    // ... other fields from the JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveNode {
    pub skill: u32,                        // node ID
    pub name: Option<String>,
    pub icon: Option<String>,
    #[serde(default)]
    pub stats: Vec<String>,               // mod text strings like "+10 to Strength"
    #[serde(rename = "isKeystone")]
    pub is_keystone: bool,
    #[serde(rename = "isNotable")]
    pub is_notable: bool,
    #[serde(rename = "isMastery")]
    pub is_mastery: bool,
    #[serde(rename = "isJewelSocket")]
    pub is_jewel_socket: bool,
    #[serde(rename = "ascendancyName")]
    pub ascendancy_name: Option<String>,
    #[serde(default)]
    pub out: Vec<String>,                  // connected node IDs
    #[serde(default)]
    pub r#in: Vec<String>,                 // incoming connections
    pub group: Option<u32>,
    pub orbit: Option<u32>,
    #[serde(rename = "orbitIndex")]
    pub orbit_index: Option<u32>,
    // ... map other fields you see in the JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveGroup {
    pub x: f64,
    pub y: f64,
    pub orbits: Vec<u32>,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassData {
    pub name: String,
    pub base_str: u32,
    pub base_dex: u32,
    pub base_int: u32,
    pub ascendancies: Vec<AscendancyData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AscendancyData {
    pub id: String,
    pub name: String,
}
```

**Key learning**: Use `serde(rename = "...")` for fields whose JSON keys don't match Rust naming conventions. Use `serde(default)` for optional arrays. Note we use `FxHashMap` instead of `HashMap` everywhere — it's a drop-in replacement with faster hashing for our use case (small keys).

#### 1.4 — Create a `DataLoader` Trait

```rust
// In data/mod.rs or a new data/loader.rs

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait DataLoader: Sized {
    fn load_from_file(path: &Path) -> Result<Self, DataError>;
    fn load_from_str(json: &str) -> Result<Self, DataError>;
}
```

Implement it for `PassiveTree`:

```rust
impl DataLoader for PassiveTree {
    fn load_from_file(path: &Path) -> Result<Self, DataError> {
        let contents = std::fs::read_to_string(path)?;
        Self::load_from_str(&contents)
    }

    fn load_from_str(json: &str) -> Result<Self, DataError> {
        Ok(serde_json::from_str(json)?)
    }
}
```

**Key learning**: This is your first custom trait and your first typed error enum. Notice `DataError` uses `thiserror` with `#[from]` for automatic conversion — this replaces `map_err` boilerplate. Internal code always uses `DataError`; you only convert to `String` at the IPC boundary (in Tauri command return types).

#### 1.5 — Create `GameData` Struct and Load on Startup

```rust
// In data/mod.rs

use std::sync::Arc;

pub struct GameData {
    pub tree: PassiveTree,
    /// Lookup table: SourceId(u32) → source name string.
    /// Modifiers store SourceId, not String, for compactness.
    pub source_names: Vec<String>,
    // pub gems: Vec<GemData>,      // Phase 3
    // pub bases: Vec<BaseItem>,     // Phase 4
    // pub uniques: Vec<UniqueItem>, // Phase 4
}

impl GameData {
    pub fn load() -> Result<Self, DataError> {
        // For now, embed the tree JSON at compile time
        let tree_json = include_str!("../../data/tree.json");
        let tree = PassiveTree::load_from_str(tree_json)?;

        Ok(GameData {
            tree,
            source_names: Vec::new(),
        })
    }

    /// Register a source name and get back a compact SourceId.
    pub fn intern_source(&mut self, name: &str) -> SourceId {
        // Check if already interned
        if let Some(pos) = self.source_names.iter().position(|s| s == name) {
            return SourceId(pos as u32);
        }
        let id = SourceId(self.source_names.len() as u32);
        self.source_names.push(name.to_owned());
        id
    }
}

/// Compact source identifier — index into GameData::source_names.
/// Stored in every Modifier instead of a String to save memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub u32);
```

Then in `lib.rs` setup:

```rust
.setup(move |app| {
    let game_data = data::GameData::load()
        .expect("Failed to load game data");
    app.manage(Arc::new(game_data));
    // ... rest of setup
})
```

**Key learning**: `Arc<T>` for shared ownership across threads, `include_str!` for compile-time file embedding, typed `DataError` instead of `Box<dyn Error>`. The `SourceId` / `intern_source` pattern is called *string interning* — it keeps the hot `Modifier` struct free of heap-allocated Strings.

#### 1.6 — Add Tauri Command to Serve Raw Tree JSON

> **Do not** clone `PassiveTree` across the IPC boundary. That would require adding `Serialize`
> back to all tree structs (undoing the cleanup), allocate a full second copy of the 5 MB tree, and
> re-serialize it — only for JS to parse it again. Instead, serve the raw file bytes directly.
> Rust just does one `fs::read_to_string`; JS does one `JSON.parse` — same cost as the static
> import, but without bundling the file and with version-switching support built in.

```rust
#[tauri::command]
#[specta::specta]
fn get_tree_json(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("data/tree")
        .join(DEFAULT_TREE_VERSION)
        .join("data.json");
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
```

Register it alongside the existing commands in the specta builder.

#### 1.7 — Update Frontend to Fetch Tree from Rust

In `src/routes/skilltree/+page.svelte`, replace the static import with a call on mount, then
delete `src/data.json` — it is no longer needed in the frontend bundle.

```typescript
import { commands } from '../../bindings';

let treeData = $state<any>(null);

onMount(async () => {
    const result = await commands.getTreeJson();
    if (result.status === 'ok') {
        treeData = JSON.parse(result.data);
    }
});
```

Pass `treeData` down to `<SkillTree>` as before; the component already guards on `treeData`
being truthy before calling `processGraph`, so no changes are needed inside `SkillTree.svelte`.

#### 1.8 — Stub Out Other Data Types

Create skeleton files for `gems.rs`, `bases.rs`, `uniques.rs`, `mods.rs`, `skills.rs` with empty structs and `todo!()` implementations. You'll fill these in later phases. The important thing is the module structure exists.

#### 1.9 — Write Unit Tests

Tests live at the bottom of `src-tauri/src/data/tree.rs` in a `#[cfg(test)]` block.
The `include_str!` path is relative to `tree.rs` (`src/data/tree.rs`), so `../../../data/tree/...`
resolves to `src-tauri/data/tree/...`. Use `DEFAULT_TREE_VERSION` from `lib.rs` to keep the
path in sync with the hardcoded default — or just hardcode the same version string directly.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const TREE_JSON: &str =
        include_str!("../../../data/tree/3.27.0g/data.json");

    #[test]
    fn test_load_tree() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        assert!(!tree.nodes.is_empty(), "Tree should have nodes");
        assert!(!tree.classes.is_empty(), "Tree should have classes");
    }

    #[test]
    fn test_node_connections() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        let connected_count = tree
            .nodes
            .values()
            .filter(|n| !n.out_connections.is_empty())
            .count();
        assert!(connected_count > 100, "Many nodes should have connections");
    }

    #[test]
    fn test_class_data() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        assert_eq!(tree.classes.len(), 7, "Should have 7 classes");
    }

    #[test]
    fn test_get_node() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        // Every non-root node has a numeric skill ID; get_node must find it
        let any_id = tree.nodes.values().find_map(|n| n.id).unwrap();
        assert!(tree.get_node(any_id).is_some());
    }

    #[test]
    fn test_bloodlines() {
        let tree: PassiveTree = serde_json::from_str(TREE_JSON).unwrap();
        assert!(!tree.bloodlines.is_empty(), "Tree should have bloodline data");
    }
}
```

### How to Verify Phase 1 is Complete

- [x] `cargo test` passes — all data deserialization tests green
- [x] `cargo clippy` — no warnings
- [x] App launches and renders the POE 1 passive tree (not POE 2)
- [x] Tree data is loaded from Rust via `get_tree_json`, not from `src/data.json` import
- [x] Module structure `data/{mod,tree,gems,bases,uniques,mods,skills,stat_id}.rs` exists
- [x] `GameData` is managed as `Arc<RwLock<GameData>>` in Tauri state
- [x] `StatId` enum exists with 320 variants generated from `SkillStatMap.json`
- [x] `StatId::from_name()` resolves stat names via `OnceLock<FxHashMap>`
- [x] `FxHashMap` used in tree deserialization (`rustc-hash` installed)
- [x] `DataError` enum exists and is used instead of `Box<dyn Error>` or raw `String`
- [x] `SourceId` newtype and `GameData::intern_source()` work
- [x] `DEFAULT_TREE_VERSION` constant replaces symlinks; `tools/fetch_data.ts` prints reminder to update it
- [x] STR/DEX/INT computed from `PassiveNode.granted_*` fields in `update_selected_nodes`
- [x] `StatAccumulator` in `stats.rs` accumulates other stat text into template-keyed totals
- [x] `PassiveTree` unit tests pass (load tree, node connections, class data, get_node, bloodlines)

### Suggested Reading

- [The Rust Programming Language, Ch 5: Structs](https://doc.rust-lang.org/book/ch05-00-structs.html)
- [The Rust Programming Language, Ch 6: Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [The Rust Programming Language, Ch 10.2: Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [Serde documentation](https://serde.rs/)
- [thiserror crate docs](https://docs.rs/thiserror/latest/thiserror/)
- [rustc-hash crate docs](https://docs.rs/rustc-hash/latest/rustc_hash/) — understand why `FxHashMap` is faster for small keys

---

## Phase 2: Modifier System & Passive Tree Stats

### Rust Concepts You'll Learn

- **Generics** (`ModDB` methods generic over filter functions)
- **Lifetime annotations** (references into `GameData` for zero-copy lookups)
- **Custom iterators** (`impl Iterator for ModIter<'a>`)
- **`Arc<T>`** and **`RwLock<T>`** for shared concurrent state
- **Closures** and **`Fn` traits** (for modifier filtering/mapping)
- **`From`/`Into`** trait implementations (converting between mod representations)
- **Builder pattern** (for constructing `Modifier` with many optional fields)
- **`SmallVec`** for inline-allocated small collections
- **Channel-based architecture** (`tokio::sync::watch` or `mpsc`) for decoupled state updates
- **Layered composition** (merging multiple `ModDB` layers at calc time)

### What You're Building

The modifier database — PoB's `ModDB` — is the single most important data structure. Everything in the calculation engine queries it. A `ModDB` accumulates modifiers from all sources (tree nodes, items, gems, config) and answers queries like "what's the total increased attack speed?"

### Steps

#### 2.1 — Define Core Modifier Types (`modifier/types.rs`)

```rust
use bitflags::bitflags;
use smallvec::SmallVec;
use crate::data::{StatId, SourceId};

/// How a modifier applies to a stat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModType {
    Base,      // +X flat (e.g., "+20 to Strength")
    Increase,  // X% increased (additive with other increases)
    More,      // X% more (multiplicative, applied separately)
    Flag,      // Boolean flag (e.g., "Cannot be Stunned")
    Override,  // Overrides the stat entirely
    List,      // List-type mod (appended, not summed)
}

bitflags! {
    /// Flags for what kind of damage/action this mod applies to
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ModFlag: u32 {
        const ATTACK    = 1 << 0;
        const SPELL     = 1 << 1;
        const HIT       = 1 << 2;
        const DOT       = 1 << 3;
        const MELEE     = 1 << 4;
        const RANGED    = 1 << 5;
        const AREA      = 1 << 6;
        const PROJECTILE = 1 << 7;
        const MINE      = 1 << 8;
        const TRAP      = 1 << 9;
        const TOTEM     = 1 << 10;
        // ... add more as needed
    }
}

bitflags! {
    /// Element/keyword flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeywordFlag: u32 {
        const PHYSICAL  = 1 << 0;
        const FIRE      = 1 << 1;
        const COLD      = 1 << 2;
        const LIGHTNING = 1 << 3;
        const CHAOS     = 1 << 4;
        // ... add more
    }
}

/// A single modifier from any source.
///
/// **Performance-critical struct** — there will be thousands of these in memory.
/// Uses StatId (u16 enum) instead of String for the stat name,
/// SourceId (u32 index) instead of String for the source,
/// and SmallVec for tags (most mods have 0–2 tags, avoiding heap allocation).
#[derive(Debug, Clone)]
pub struct Modifier {
    pub stat: StatId,               // e.g., StatId::Strength, StatId::AttackSpeed
    pub mod_type: ModType,
    pub value: f64,
    pub flags: ModFlag,
    pub key_flags: KeywordFlag,
    pub source: SourceId,           // index into GameData::source_names
    pub tags: SmallVec<[ModTag; 2]>,// most mods have 0–2 conditions; inline-allocated
}

/// Conditions under which a modifier applies.
/// Evaluated against a CalcContext during ModDB queries.
#[derive(Debug, Clone)]
pub enum ModTag {
    Condition(StatId),        // e.g., StatId::IsLeeching
    Multiplier(StatId),       // e.g., StatId::PowerChargeCount
    SkillType(u32),
    SlotName(u8),             // compact slot index instead of String
}

/// Context passed to every ModDB query.
/// Contains the current character state needed to evaluate conditional mods.
/// Start with a minimal struct—expand as you add conditional support.
pub struct CalcContext {
    pub flags: ModFlag,       // what kind of action we're calculating
    pub key_flags: KeywordFlag,
    // Phase 5+: add fields for conditions, multipliers, etc.
    // pub power_charges: u32,
    // pub is_leeching: bool,
}

impl CalcContext {
    /// Default context with no flags—used in Phase 2 before conditions matter.
    pub fn empty() -> Self {
        Self {
            flags: ModFlag::empty(),
            key_flags: KeywordFlag::empty(),
        }
    }
}
```

**Key learning**: `bitflags!` macro for efficient flag combination/checking. `SmallVec<[ModTag; 2]>` keeps 0–2 tags inline without heap allocation (most passive node mods are unconditional). The `CalcContext` is empty for now but accepting it in all queries means you won't need to refactor every call site later.

Add `bitflags = "2"`, `smallvec = "1"` to your `Cargo.toml` if not added in Phase 1.

#### 2.2 — Build the ModDB (`modifier/mod_db.rs`)

This is the core. Study PoB's `ModDB.lua` to understand the query model.

```rust
use rustc_hash::FxHashMap;
use super::types::*;
use crate::data::StatId;

pub struct ModDB {
    mods: FxHashMap<StatId, Vec<Modifier>>,
}

impl ModDB {
    pub fn new() -> Self {
        Self { mods: FxHashMap::default() }
    }

    pub fn add_mod(&mut self, modifier: Modifier) {
        self.mods
            .entry(modifier.stat)
            .or_default()
            .push(modifier);
    }

    /// Sum all Base-type mods for a stat.
    /// `ctx` is passed for future conditional evaluation — unused in Phase 2.
    pub fn sum_base(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods.get(&stat)
            .map(|mods| mods.iter()
                .filter(|m| m.mod_type == ModType::Base)
                .filter(|m| self.matches_context(m, ctx))
                .map(|m| m.value)
                .sum())
            .unwrap_or(0.0)
    }

    /// Sum all Increase-type mods for a stat (additive %)
    pub fn sum_increase(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods.get(&stat)
            .map(|mods| mods.iter()
                .filter(|m| m.mod_type == ModType::Increase)
                .filter(|m| self.matches_context(m, ctx))
                .map(|m| m.value)
                .sum())
            .unwrap_or(0.0)
    }

    /// Multiply all More-type mods for a stat
    pub fn product_more(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods.get(&stat)
            .map(|mods| mods.iter()
                .filter(|m| m.mod_type == ModType::More)
                .filter(|m| self.matches_context(m, ctx))
                .fold(1.0, |acc, m| acc * (1.0 + m.value / 100.0)))
            .unwrap_or(1.0)
    }

    /// Check if any Flag-type mod exists for a stat
    pub fn has_flag(&self, stat: StatId, ctx: &CalcContext) -> bool {
        self.mods.get(&stat)
            .map(|mods| mods.iter()
                .any(|m| m.mod_type == ModType::Flag && self.matches_context(m, ctx)))
            .unwrap_or(false)
    }

    /// Calculate final value: (base) × (1 + sum_increase/100) × product_more
    pub fn calculate(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        let base = self.sum_base(stat, ctx);
        let inc = self.sum_increase(stat, ctx);
        let more = self.product_more(stat, ctx);
        base * (1.0 + inc / 100.0) * more
    }

    /// Merge another ModDB into this one
    pub fn merge(&mut self, other: &ModDB) {
        for (stat, mods) in &other.mods {
            for m in mods {
                self.add_mod(m.clone());
            }
        }
    }

    /// Check if a modifier's flags/conditions match the current calc context.
    /// In Phase 2 this always returns true. Expand in Phase 5+.
    fn matches_context(&self, _modifier: &Modifier, _ctx: &CalcContext) -> bool {
        // TODO: Check modifier.flags against ctx.flags,
        //       evaluate ModTag conditions, etc.
        true
    }
}
```

**Key learning**: The `calculate()` method shows PoB's core formula: `base × (1 + sum_of_increases%) × product_of_mores`. This is how almost every stat works in POE. Note that every query takes `&CalcContext` — in Phase 2 you pass `CalcContext::empty()` everywhere, but the signature is locked in. When you implement conditional mods in Phase 5, you only change `matches_context()`, not every call site.

#### 2.3 — Build the Mod Parser (`modifier/parser.rs`)

This converts text strings like "+20 to Strength" into `Modifier` structs using `StatId`.

```rust
use crate::data::{StatId, SourceId};
use super::types::*;
use smallvec::smallvec;

/// Parse a single mod line into a Modifier (if recognized).
/// Returns None for patterns not yet implemented — log a warning.
pub fn parse_mod(text: &str, source: SourceId) -> Option<Modifier> {
    // Start with common patterns, expand over time.
    // The parser extracts (value, stat_text) then maps stat_text → StatId.
    //
    // Examples:
    //   "+10 to Strength"           → Base, StatId::Strength, value=10
    //   "15% increased Attack Speed" → Increase, StatId::AttackSpeed, value=15
    //   "10% more Spell Damage"      → More, StatId::SpellDamage, value=10
    //   "+50 to maximum Life"        → Base, StatId::Life, value=50
    //
    // Strategy: extract (value, stat_text) via regex or manual matching,
    // then call StatId::from_text(stat_text) to get the enum variant.
    // If StatId::from_text returns None, the stat is unrecognized — skip it.

    // Tip: start with manual string matching, move to regex if patterns get complex
    todo!("Implement pattern by pattern, starting with the most common")
}

/// Helper to build a simple unconditional modifier.
fn simple_mod(stat: StatId, mod_type: ModType, value: f64, source: SourceId) -> Modifier {
    Modifier {
        stat,
        mod_type,
        value,
        flags: ModFlag::empty(),
        key_flags: KeywordFlag::empty(),
        source,
        tags: smallvec![],
    }
}
```

**Strategy**: Don't try to parse all mod patterns at once. Start with these ~10 patterns that cover passive tree nodes:
1. `+X to Strength/Dexterity/Intelligence`
2. `+X to maximum Life/Mana/Energy Shield`
3. `X% increased Attack Speed/Cast Speed`
4. `X% increased maximum Life/Mana/Energy Shield`
5. `+X% to Fire/Cold/Lightning/Chaos Resistance`
6. `+X% to all Elemental Resistances`
7. `X% increased Physical Damage`
8. `X% increased Spell Damage`
9. `X% more <stat>` patterns
10. `Minions deal X% increased Damage`

Add more patterns as you need them in later phases.

#### 2.4 — Implement Passive Tree Stat Aggregation

> **Current state**: `lib.rs` already has `Mutex<BuildInfo>` and `Arc<RwLock<GameData>>`.
> The `update_selected_nodes` command computes STR/DEX/INT from `PassiveNode.granted_*` fields
> and uses `StatAccumulator` (from `stats.rs`) for all other stats as text-template totals.
> Phase 2 replaces `StatAccumulator` with `ModDB` + `ModParser` for typed stat accumulation,
> and transitions from `Mutex<BuildInfo>` to `RwLock<BuildState>`.

Replace `Mutex<BuildInfo>` with a split state architecture. `GameData` is already read-only (`Arc<RwLock<GameData>>`). `BuildState` is mutable (`RwLock` or channel-based).

**Option A: `RwLock` split state** (simpler, recommended for Phase 2)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Read-only game data — shared freely across threads.
/// Already exists as Arc<RwLock<GameData>> — can simplify to Arc<GameData>
/// since GameData is immutable after startup (only load_tree_version mutates it).
pub type SharedGameData = Arc<GameData>;

/// Mutable build state — use RwLock so calc reads don't block UI writes.
pub type SharedBuildState = Arc<RwLock<BuildState>>;

pub struct BuildState {
    pub info: BuildInfo,
    pub selected_nodes: BuildSelection,
    pub stats: BuildStats,
    // Phase 3+: pub skill_groups: Vec<SkillGroup>,
    // Phase 4+: pub equipment: Equipment,
}
```

Now update `update_selected_nodes` to use `ModDB` instead of `StatAccumulator`:

```rust
#[tauri::command]
#[specta::specta]
async fn update_selected_nodes(
    node_ids: Vec<u32>,
    game_data: tauri::State<'_, SharedGameData>,
    state: tauri::State<'_, SharedBuildState>,
) -> Result<BuildStats, String> {
    let mut build = state.write().await;
    build.selected_nodes.selected_node_ids = node_ids.iter().cloned().collect();

    // Build a ModDB from all selected nodes
    let mut mod_db = modifier::ModDB::new();
    let ctx = CalcContext::empty(); // No conditions yet

    for &node_id in &node_ids {
        if let Some(node) = game_data.tree.get_node(node_id) {
            let source = SourceId(node_id);
            for stat_text in &node.stats {
                if let Some(modifier) = modifier::parser::parse_mod(
                    stat_text, source
                ) {
                    mod_db.add_mod(modifier);
                }
            }
            // granted_* fields are separate from stat text — add them directly
            if node.granted_strength > 0 {
                mod_db.add_mod(simple_mod(
                    StatId::Strength, ModType::Base,
                    node.granted_strength as f64, source
                ));
            }
            if node.granted_dexterity > 0 {
                mod_db.add_mod(simple_mod(
                    StatId::Dexterity, ModType::Base,
                    node.granted_dexterity as f64, source
                ));
            }
            if node.granted_intelligence > 0 {
                mod_db.add_mod(simple_mod(
                    StatId::Intelligence, ModType::Base,
                    node.granted_intelligence as f64, source
                ));
            }
        }
    }

    // Calculate stats from ModDB
    let stats = BuildStats {
        total_strength: mod_db.sum_base(StatId::Strength, &ctx) as i32,
        total_dexterity: mod_db.sum_base(StatId::Dexterity, &ctx) as i32,
        total_intelligence: mod_db.sum_base(StatId::Intelligence, &ctx) as i32,
        node_count: node_ids.len() as u32,
        // Phase 2.6 adds life, mana, energy_shield, etc.
    };

    build.stats = stats.clone();
    Ok(stats)
}
```

> **Note**: `PassiveNode.granted_strength/dexterity/intelligence` are separate numeric fields,
> not stat text strings. The current code already sums them directly. When migrating to ModDB,
> add them as `ModType::Base` modifiers alongside the parsed stat text mods.

**Key learning**: `RwLock` allows multiple simultaneous readers (calc queries) while only blocking when a writer (UI state update) is active. This is a massive concurrency win over `Mutex`, which blocks all access for every operation. The `async fn` + `.write().await` pattern works naturally with Tauri's async command system.

#### 2.5 — Add Base Class Stats

Each class starts with base attribute points. These are already available in the parsed tree data
as `ClassData.base_str`, `ClassData.base_dex`, `ClassData.base_int` — no need to hardcode them.

```rust
fn add_class_base_stats(mod_db: &mut ModDB, class: &Class, tree: &PassiveTree) {
    // Map the Class enum to the class index in the tree data
    let class_index = match class {
        Class::Scion(_)   => 0,
        Class::Marauder(_) => 1,
        Class::Ranger(_)  => 2,
        Class::Witch(_)   => 3,
        Class::Duelist(_)  => 4,
        Class::Templar(_)  => 5,
        Class::Shadow(_)   => 6,
    };

    if let Some(class_data) = tree.classes.get(class_index) {
        let source = SourceId(0); // class base stats source
        mod_db.add_mod(simple_mod(
            StatId::Strength, ModType::Base,
            class_data.base_str as f64, source
        ));
        mod_db.add_mod(simple_mod(
            StatId::Dexterity, ModType::Base,
            class_data.base_dex as f64, source
        ));
        mod_db.add_mod(simple_mod(
            StatId::Intelligence, ModType::Base,
            class_data.base_int as f64, source
        ));
    }
}
```

Call `add_class_base_stats()` in `update_selected_nodes` before adding tree node mods, so class base attributes are included in the total.

> **Note**: The class-to-index mapping may vary between tree versions. Verify the order
> by checking `tree.classes[i].name` against your `Class` enum variants.

#### 2.6 — Expand `BuildStats` and Update Sidebar

> **Current state**: `BuildStats` already has `total_strength/dexterity/intelligence`, `node_count`,
> and `stat_totals: HashMap<String, f64>` (from `StatAccumulator`). Phase 2 replaces `stat_totals`
> with typed fields computed from `ModDB`.

Expand `BuildStats` to include more fields:

```rust
pub struct BuildStats {
    pub total_strength: i32,
    pub total_dexterity: i32,
    pub total_intelligence: i32,
    pub life: f64,
    pub mana: f64,
    pub energy_shield: f64,
    pub node_count: u32,
}
```

Remove the `stat_totals: HashMap<String, f64>` field — it was the `StatAccumulator` placeholder.
Delete `stats.rs` entirely once `ModDB` replaces it.

Update `Sidebar.svelte` to display all these stats.

#### 2.7 — Layered ModDB Architecture

Instead of building one monolithic `ModDB`, prepare for layered composition. Each source gets its own `ModDB` layer:

```rust
/// Layers that compose the final ModDB.
/// Changing one layer (e.g., re-selecting nodes) only rebuilds that layer.
pub struct ModDBLayers {
    pub tree: ModDB,        // from selected passive nodes
    // Phase 3+: pub gems: ModDB,
    // Phase 4+: pub items: ModDB,
    // Phase 7+: pub config: ModDB,
}

impl ModDBLayers {
    /// Merge all layers into one ModDB for calculation.
    pub fn merged(&self) -> ModDB {
        let mut combined = ModDB::new();
        combined.merge(&self.tree);
        // combined.merge(&self.gems);
        // combined.merge(&self.items);
        // combined.merge(&self.config);
        combined
    }

    /// Rebuild only the tree layer (called when nodes change).
    pub fn rebuild_tree(&mut self, node_ids: &[u32], game_data: &GameData) {
        self.tree = ModDB::new();
        for &node_id in node_ids {
            if let Some(node) = game_data.tree.get_node(node_id) {
                let source = SourceId(node_id);
                // Add granted attributes as Base mods
                if node.granted_strength > 0 {
                    self.tree.add_mod(simple_mod(
                        StatId::Strength, ModType::Base,
                        node.granted_strength as f64, source
                    ));
                }
                if node.granted_dexterity > 0 {
                    self.tree.add_mod(simple_mod(
                        StatId::Dexterity, ModType::Base,
                        node.granted_dexterity as f64, source
                    ));
                }
                if node.granted_intelligence > 0 {
                    self.tree.add_mod(simple_mod(
                        StatId::Intelligence, ModType::Base,
                        node.granted_intelligence as f64, source
                    ));
                }
                // Parse stat text lines into typed modifiers
                for stat_text in &node.stats {
                    if let Some(m) = modifier::parser::parse_mod(stat_text, source) {
                        self.tree.add_mod(m);
                    }
                }
            }
        }
    }
}
```

> **Note**: `get_node(node_id)` does the `u32` → `String` key lookup internally.
> `PassiveNode.granted_*` fields are separate from `stats` text — both must be processed.

**Key learning**: Layered composition means that when you equip a new item in Phase 4, you only rebuild the `items` layer and re-merge — you don't re-parse every passive node. This is PoB's approach and it's critical for responsive recalculation.

#### 2.8 — Tests

Write comprehensive tests:
- `parse_mod("+10 to Strength", source)` returns `Modifier { stat: StatId::Strength, mod_type: ModType::Base, value: 10.0, ... }`
- `ModDB::calculate(StatId::Life, &ctx)` with known inputs returns expected value
- Selecting known Marauder start nodes gives stats matching PoB
- `CalcContext::empty()` works with all query methods
- Layered ModDB merges correctly
- Class base stats from `tree.classes[i].base_str/base_dex/base_int` feed into ModDB

### How to Verify Phase 2 is Complete

- [ ] Selecting nodes updates real stat values in the sidebar (not zeros)
- [ ] Str/Dex/Int totals include class base stats (from `ClassData`, not hardcoded)
- [ ] ModParser handles at least 10 common passive tree patterns
- [ ] `StatAccumulator` and `stats.rs` are deleted
- [ ] `ModDB::calculate()` correctly applies Base + Increase + More formula
- [ ] All `ModDB` query methods accept `&CalcContext` parameter
- [ ] `Modifier` uses `StatId` (not String) and `SourceId` (not String)
- [ ] `ModDB` uses `FxHashMap<StatId, Vec<Modifier>>` internally
- [ ] `Mutex<BuildInfo>` is replaced with `RwLock<BuildState>` (or channel-based)
- [ ] `ModDBLayers` struct exists with at least a `tree` layer and `merged()` method
- [ ] Tests cover ModParser patterns, ModDB queries with CalcContext, and layered merge
- [ ] Selecting the same nodes as a PoB build gives matching Str/Dex/Int

### Suggested Reading

- [The Rust Programming Language, Ch 10: Generics, Traits, and Lifetimes](https://doc.rust-lang.org/book/ch10-00-generics.html)
- [The Rust Programming Language, Ch 13: Closures and Iterators](https://doc.rust-lang.org/book/ch13-00-functional-features.html)
- [bitflags crate docs](https://docs.rs/bitflags/latest/bitflags/)
- [smallvec crate docs](https://docs.rs/smallvec/latest/smallvec/) — understand when inline storage wins
- [tokio::sync::RwLock](https://docs.rs/tokio/latest/tokio/sync/struct.RwLock.html) — async-aware read-write lock

---

## Phase 3: Skill & Gem System

### Rust Concepts You'll Learn

- **Trait objects** (`Box<dyn Skill>`, `&dyn GemEffect`)
- **Dynamic dispatch** vs static dispatch and when to use each
- **Type state pattern** (model gem states: Socketed → Linked → Active)
- **Complex enum variants** with struct-like data
- **`Vec<Box<dyn T>>`** collections
- **`PartialOrd`/`Ord`** for gem sorting/priority

### What You're Building

A skill group system: players create socket groups, add active + support gems, and the system auto-resolves which supports apply to which actives.

### Steps

#### 3.1 — Bundle Gem Data

Gem data is already downloaded by `tools/fetch_data.ts` and lives at `src-tauri/data/pob/Gems.json`.
Per-skill stat data is in `src-tauri/data/pob/Skills/` (subdivided by attribute: `act_str.json`, `act_dex.json`, etc.).
No additional data fetching is needed.

#### 3.2 — Create Gem Types (`data/gems.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemData {
    pub id: String,
    pub name: String,
    pub gem_type: GemType,
    pub tags: Vec<String>,           // "Spell", "AoE", "Fire", "Duration", etc.
    pub levels: Vec<GemLevel>,       // stats per gem level
    pub required_level: u32,
    pub stat_requirements: StatRequirements,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum GemType {
    Active,
    Support,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemLevel {
    pub level: u32,
    pub mana_cost: Option<f64>,
    pub cooldown: Option<f64>,
    pub damage_effectiveness: Option<f64>,
    pub stats: Vec<GemStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GemStat {
    pub id: String,
    pub value: f64,
}
```

#### 3.3 — Create Skill Module (`skill/`)

```rust
// skill/gem.rs
pub struct ActiveGem {
    pub data: Arc<GemData>,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
}

pub struct SupportGem {
    pub data: Arc<GemData>,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
}

impl SupportGem {
    /// Check if this support can apply to an active skill based on tags
    pub fn can_support(&self, active: &ActiveGem) -> bool {
        // Check gem tags for compatibility
        // e.g., "Spell Echo" requires the active to have "Spell" tag
        todo!("Implement tag matching logic from CalcActiveSkill.lua")
    }
}
```

```rust
// skill/group.rs
pub struct SkillGroup {
    pub label: String,
    pub slot: Option<ItemSlot>,
    pub gems: Vec<GemInstance>,
    pub enabled: bool,
}

pub enum GemInstance {
    Active(ActiveGem),
    Support(SupportGem),
}

impl SkillGroup {
    /// Resolve which supports apply to which actives
    pub fn resolve(&self) -> Vec<ResolvedSkill> {
        let actives: Vec<&ActiveGem> = self.gems.iter()
            .filter_map(|g| match g {
                GemInstance::Active(a) if a.enabled => Some(a),
                _ => None,
            })
            .collect();

        let supports: Vec<&SupportGem> = self.gems.iter()
            .filter_map(|g| match g {
                GemInstance::Support(s) if s.enabled => Some(s),
                _ => None,
            })
            .collect();

        actives.iter().map(|active| {
            let applicable: Vec<_> = supports.iter()
                .filter(|s| s.can_support(active))
                .cloned()
                .collect();
            ResolvedSkill {
                active: (*active).clone(),
                supports: applicable,
            }
        }).collect()
    }
}
```

#### 3.4 — Add Tauri Commands

- `get_gem_list() -> Vec<GemSummary>` — all gems for the selector dropdown
- `add_skill_group() -> SkillGroupSummary`
- `add_gem_to_group(group_index, gem_id, level, quality)`
- `remove_gem_from_group(group_index, gem_index)`
- `set_gem_enabled(group_index, gem_index, enabled)`
- `set_main_skill(group_index, skill_index)` — which skill shows in DPS

#### 3.5 — Create `SkillsTab.svelte`

Frontend tab with:
- List of skill groups (add/remove)
- Gem selector dropdown with search
- Each group shows: active gem name, support gems auto-resolved, DPS preview
- Enable/disable toggle per gem

#### 3.6 — Add Tab Navigation

Update `src/routes/skilltree/+page.svelte` to add tab buttons: Tree | Skills | Items | Calcs | Config | Import | Notes

Start with just Tree and Skills functional; others show "Coming Soon" placeholders.

### How to Verify Phase 3 is Complete

- [ ] Can add a skill group with Fireball (Active) + Spell Echo (Support)
- [ ] Spell Echo correctly identifies it can support Fireball (Spell tag)
- [ ] Adding a non-spell active gem (e.g., Double Strike) → Spell Echo doesn't apply
- [ ] Gem level/quality can be changed
- [ ] Skills tab renders correctly with tab navigation
- [ ] Tests cover support matching logic

### Suggested Reading

- [The Rust Programming Language, Ch 17: Trait Objects](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- [Rust Design Patterns: Type State](https://rust-unofficial.github.io/patterns/patterns/behavioural/typestate.html)

---

## Phase 4: Item System

### Rust Concepts You'll Learn

- **Ownership & borrowing** patterns in complex data structures
- **`RefCell<T>`** for interior mutability
- **String parsing** (item text → structured `Item`)
- **`TryFrom`** implementations
- **Complex pattern matching** with guards
- **`Display` trait** for formatted output (item tooltips)

### Steps

#### 4.1 — Bundle Item Data

Item data is already downloaded by `tools/fetch_data.ts` and organized under `src-tauri/data/pob/`:
- `Bases/*.json` — all base item types (by category: `Amulet.json`, `Body Armour.json`, `Bow.json`, etc.)
- `Uniques/*.json` — unique items (by slot type + `Special/` for special uniques)
- `ModItem.json`, `ModJewel.json`, `ModFlask.json`, `ModJewelAbyss.json`, etc. — mod pools (prefix/suffix definitions)

No additional data fetching is needed — load these from the bundled `pob/` directory.

#### 4.2 — Create Item Types (`item/types.rs`)

Define: `Item`, `ItemSlot` (all 20+ equipment slots), `ItemMod`, `Rarity` enum (Normal/Magic/Rare/Unique), `SocketGroup`

#### 4.3 — Create Item Parser (`item/parser.rs`)

Parse item text (as copied from POE's in-game clipboard format):

```
Rarity: Unique
Tabula Rasa
Simple Robe
--------
Sockets: W-W-W-W-W-W
--------
Item Level: 100
```

→ produces `Item { rarity: Unique, name: "Tabula Rasa", base: "Simple Robe", sockets: [...], ... }`

#### 4.4 — Create Item Tooltip (`item/tooltip.rs`)

Implement `Display for Item` to generate tooltip text with colored mod lines (blue = supported, red = unsupported).

#### 4.5 — Create Crafting System (`item/crafting.rs`)

- Select a base item
- Add prefixes/suffixes from the mod pool
- Validate: max 3 prefixes + 3 suffixes for rare items
- Roll random tiers within ranges

#### 4.6 — Create Equipment Manager (`item/equip.rs`)

- `Equipment` struct: `HashMap<ItemSlot, Item>`
- `fn equip()` / `fn unequip()` with slot validation
- `fn collect_mods(&self) -> ModDB` — gathers all item mods

#### 4.7 — Create `ItemsTab.svelte`

- Equipment slot display (list or visual grid)
- Click slot → item browser (uniques search, paste from clipboard, or craft)
- Tooltip on hover
- DPS comparison when swapping items

### How to Verify Phase 4 is Complete

- [ ] Can paste a real POE item text → parsed into `Item`
- [ ] Can search uniques, select one, equip it, see mods
- [ ] Can craft a rare item with valid prefix/suffix counts
- [ ] Equipping items updates the ModDB (stat changes visible)
- [ ] Item tooltip displays correctly
- [ ] Tests cover parsing, crafting validation, equip/unequip

### Suggested Reading

- [The Rust Programming Language, Ch 4: Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [The Rust Programming Language, Ch 15: Smart Pointers](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)
- [std::fmt::Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)

---

## Phase 5: Full Calculation Engine

### Rust Concepts You'll Learn

- **`async/await`** with Tokio
- **Channels** (`tokio::sync::mpsc`) for streaming calc results
- **`rayon`** for CPU-parallel computation
- **`criterion`** benchmarking
- **Cache invalidation** patterns
- **`f64` precision** considerations

### What You're Building

This is the big one. You're porting PoB's ~5,000 lines of calculation Lua into Rust. It aggregates all modifier sources and computes every stat.

### Steps

#### 5.1 — CalcSetup (`calc/setup.rs`)

Gather all modifier sources into the layered `ModDB` and merge:

```rust
use crate::modifier::{ModDB, ModDBLayers};

pub fn setup_moddb(build: &Build, game_data: &GameData) -> ModDB {
    // Each source already maintains its own ModDB layer (from Phase 2+).
    // CalcSetup merges them and adds class base stats.
    let mut db = build.mod_layers.merged();

    // 1. Class base stats (from tree.classes[i].base_str/base_dex/base_int)
    add_class_base_stats(&mut db, &build.info.class, &game_data.tree);

    // Layers already contain:
    // 2. Tree node mods      (build.mod_layers.tree   — Phase 2)
    // 3. Equipment mods       (build.mod_layers.items  — Phase 4)
    // 4. Skill/gem mods       (build.mod_layers.gems   — Phase 3)
    // 5. Config-driven mods   (build.mod_layers.config — Phase 7)

    db
}
```

**Key learning**: Because you built layered `ModDBLayers` in Phase 2, CalcSetup is just a merge + class base stats. Each layer was already rebuilt when its source changed (e.g., `rebuild_tree()` on node selection, `rebuild_items()` on equip). This is the payoff of the upfront architecture.

#### 5.2 — CalcDefence (`calc/defence.rs`)

Calculate all defensive stats:

```rust
pub struct DefenceResult {
    pub life: f64,
    pub mana: f64,
    pub energy_shield: f64,
    pub armour: f64,
    pub evasion: f64,
    pub fire_res: f64,
    pub cold_res: f64,
    pub lightning_res: f64,
    pub chaos_res: f64,
    pub block_chance: f64,
    pub spell_block: f64,
    pub life_regen: f64,
    pub mana_regen: f64,
    // ... many more
}

pub fn calc_defence(db: &ModDB, level: u32, ctx: &CalcContext) -> DefenceResult {
    // Life = (base_life_at_level + flat_life) * (1 + increased/100) * mores
    // Base life at level = 38 + (level * 12) for POE 1
    let base_life = 38.0 + (level as f64 * 12.0);
    let flat_life = db.sum_base(StatId::Life, ctx);
    // Strength gives +1 life per 2 str
    let str_life = db.sum_base(StatId::Strength, ctx) / 2.0;
    let inc_life = db.sum_increase(StatId::Life, ctx);
    let more_life = db.product_more(StatId::Life, ctx);

    let life = (base_life + flat_life + str_life) * (1.0 + inc_life / 100.0) * more_life;

    // ... similar for all other stats

    DefenceResult { life, /* ... */ }
}
```

#### 5.3 — CalcOffence (`calc/offence.rs`)

This is the most complex part. Start with basic DPS and expand:

1. **Base damage** (from gem, weapon, added damage mods)
2. **Damage conversion** (Phys → Fire via Avatar of Fire, etc.)
3. **Increase/More modifiers** per damage type
4. **Critical strikes** (chance, multiplier, effective crit)
5. **Speed** (attacks/casts per second)
6. **Accuracy** (chance to hit for attacks)
7. **Final DPS** = average hit × speed × accuracy × crit factor
8. **DoT DPS** (separate calculation)
9. **Ailments** (bleed, poison, ignite DPS)

Start with steps 1-7 for a basic DPS number. Add 8-9 later.

#### 5.4 — CalcPerform (`calc/perform.rs`)

Orchestrate the full calculation:

```rust
pub async fn calculate(build: &Build, game_data: &GameData) -> CalcResult {
    let db = setup::setup_moddb(build, game_data);

    let defence = defence::calc_defence(&db, build.info.level);
    let offence = offence::calc_offence(&db, build.get_main_skill(), game_data);

    CalcResult { defence, offence }
}
```

#### 5.5 — Event-Driven Recalculation

Any change should trigger a recalc. Use Tauri events:

```rust
// After any state change:
let result = calc::perform::calculate(&build, &game_data).await;
app_handle.emit("calc-result", &result).unwrap();
```

Frontend listens:

```typescript
import { listen } from '@tauri-apps/api/event';
listen('calc-result', (event) => { stats = event.payload; });
```

#### 5.6 — Benchmarks

Add `criterion` benchmarks. Target: < 50ms for a full calculation pass.

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "calc_bench"
harness = false
```

### How to Verify Phase 5 is Complete

- [ ] Full DPS number displays for a main skill
- [ ] Life/ES/Mana calculated correctly (compare with PoB)
- [ ] Resistances shown and correct
- [ ] Changing gear/tree/gems triggers recalculation
- [ ] CalcsTab shows stat breakdown
- [ ] Benchmark runs and is < 50ms
- [ ] At least one full build matches PoB numbers within 1-2%

### Suggested Reading

- [Async Rust (Tokio tutorial)](https://tokio.rs/tokio/tutorial)
- [The Rust Programming Language, Ch 16: Concurrency](https://doc.rust-lang.org/book/ch16-00-concurrency.html)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- Study PoB's `CalcOffence.lua` and `CalcDefence.lua` — they're the blueprint

---

## Phase 6: Build Management & Import/Export

### Rust Concepts You'll Learn

- **`async/await`** with `reqwest` for HTTP
- **`flate2`** for zlib compression
- **`base64`** encoding
- **`quick-xml`** for XML parsing (PoB format)
- **File I/O** with error handling
- **Atomic file writes** (already have `tempfile`)
- **Multi-format serde** (JSON for storage, XML for PoB compatibility)

### Steps

#### 6.1 — Build Save/Load (`build/manager.rs`)

- Save builds as JSON files in the app data directory
- Support folders for organization
- `list_builds()` returns summaries (name, class, level, last modified)

#### 6.2 — PoB-Compatible Share Codes (`build/codec.rs`)

PoB's share code format: `Build XML → zlib compress → base64 encode`

```rust
use base64::Engine;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;

pub fn encode_build(build: &Build) -> Result<String, CodecError> {
    let xml = build_to_xml(build)?;
    let compressed = zlib_compress(xml.as_bytes())?;
    Ok(base64::engine::general_purpose::URL_SAFE.encode(&compressed))
}

pub fn decode_build(code: &str) -> Result<Build, CodecError> {
    let compressed = base64::engine::general_purpose::URL_SAFE.decode(code)?;
    let xml = zlib_decompress(&compressed)?;
    xml_to_build(&xml)
}
```

Add deps: `flate2`, `base64`, `quick-xml`

#### 6.3 — Character Import (`build/import.rs`)

```rust
pub async fn import_character(
    account: &str,
    character: &str,
    http: &reqwest::Client,
) -> Result<Build, ImportError> {
    // 1. Fetch passive tree
    // 2. Fetch items
    // 3. Convert to internal Build format
    todo!()
}
```

#### 6.4 — Undo/Redo (`build/undo.rs`)

Generic undo stack:

```rust
pub struct UndoHandler<T: Clone> {
    history: Vec<T>,
    position: usize,
}

impl<T: Clone> UndoHandler<T> {
    pub fn push(&mut self, state: T) { /* ... */ }
    pub fn undo(&mut self) -> Option<&T> { /* ... */ }
    pub fn redo(&mut self) -> Option<&T> { /* ... */ }
}
```

#### 6.5 — Wire Up Home Page

Connect save/load/delete buttons in `+page.svelte` to real Tauri commands. Show build list with metadata.

### How to Verify Phase 6 is Complete

- [ ] Save a build → close app → reopen → load → identical state
- [ ] Generate share code → paste in real PoB → build loads
- [ ] Import a real PoB share code → build displays correctly
- [ ] Undo/redo tracks tree, item, and skill changes
- [ ] Build list on home page shows all saved builds

### Suggested Reading

- [reqwest crate docs](https://docs.rs/reqwest/latest/reqwest/)
- [quick-xml crate docs](https://docs.rs/quick-xml/latest/quick_xml/)
- [flate2 crate docs](https://docs.rs/flate2/latest/flate2/)

---

## Phase 7: Configuration, Party, Jewels & Advanced Features

### Rust Concepts You'll Learn

- **Procedural macros** (optional: auto-generate config UI metadata)
- **Advanced generics** with multiple trait bounds
- **`const fn`** for compile-time computation
- **`PhantomData`** usage
- **Sealed traits** pattern
- **Newtype pattern** (`struct NodeId(u32)`, `struct ModId(String)`)
- **Advanced iterator chains**

### Steps

#### 7.1 — Configuration System (`config/`)

Mirror PoB's `ConfigOptions.lua`:
- Enemy level, boss type, enemy resistances
- Charge counts (power/frenzy/endurance)
- Buff toggles (onslaught, fortify, unholy might, etc.)
- Flask effects
- Conditions (low life, full life, leeching, etc.)

Each option feeds mods into the ModDB during `CalcSetup`.

#### 7.2 — Jewel System

- **Regular jewels**: Apply jewel mods to ModDB (like items but in tree sockets)
- **Cluster jewels**: Expand the tree with additional notable nodes
- **Timeless jewels**: Seed-based node stat replacement (lookup table from PoB's `TimelessJewelData/`)

This is complex. Timeless jewels alone are a significant sub-project.

#### 7.3 — Party Support

- Import party member builds (via share code)
- Calculate party-wide auras and curses
- Feed party effects into your character's ModDB

#### 7.4 — Frontend Tabs

- `ConfigTab.svelte` — Generated from config option metadata
- `NotesTab.svelte` — Simple textarea saved with the build
- `PartyTab.svelte` — Party member import + aura toggle

### How to Verify Phase 7 is Complete

- [ ] Power Charge: 3 → crit chance increases correctly
- [ ] Enemy resistance config changes DPS numbers
- [ ] Regular jewel equip adds stats
- [ ] Config tab shows all options from PoB
- [ ] At least one timeless jewel seed produces correct replacements

---

## Phase 8: Polish, Performance & Trade

### Rust Concepts You'll Learn

- **Profiling** with `cargo flamegraph` and `tracing`
- **Release optimization** (`lto = true`, `codegen-units = 1`, `opt-level = 3`)
- **`#[inline]`** and **`#[cold]`** hints
- **CI/CD** with GitHub Actions
- **Cross-compilation** considerations

### Steps

#### 8.1 — Trade Query Generation

- Generate trade site queries weighted by DPS impact per mod
- Open in browser via `tauri-plugin-opener`

#### 8.2 — Node Power Overlay

- For each unallocated node adjacent to the selected tree, calculate: "what DPS/Life change would selecting this node give?"
- Color-code tree nodes by impact (green = big DPS gain, red = DPS loss)
- This is PoB's most loved feature

#### 8.3 — Performance Optimization

Many critical optimizations are already baked in from Phase 1-2 (see "Performance Decisions" at the top of this document):
- ✅ `StatId` enum (320 variants, `#[repr(u16)]`) — no string hashing in modifier lookups
- ✅ `FxHashMap` — faster hashing in tree deserialization and all internal maps
- ✅ `CalcContext` — conditional mod evaluation without API changes
- ✅ Compact `Modifier` — `SourceId(u32)`, `SmallVec<[ModTag; 2]>`
- ✅ `RwLock` split state — calc reads don't block UI writes
- ✅ Layered `ModDB` — targeted cache invalidation per source
- ✅ `DEFAULT_TREE_VERSION` constant — no symlink resolution overhead at startup
- ✅ Raw JSON string for tree IPC — no double serialize/deserialize of 5 MB tree

**Remaining optimizations for Phase 8:**
- Profile with `cargo flamegraph` to find actual bottlenecks
- Compile regexes once with `OnceLock` in the mod parser (if using regex)
- Consider `rayon` for parallel per-skill DPS calculation
- Measure cache line efficiency of `Modifier` struct layout
- Target: full recalc < 20ms

#### 8.4 — Release Profile

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
opt-level = 3
strip = true
```

#### 8.5 — CI/CD

Create `.github/workflows/ci.yml`:
- `cargo test`, `cargo clippy`, `cargo fmt --check`
- `npm run check`
- Build Tauri app
- Upload artifacts

#### 8.6 — Polish

- Update `tauri.conf.json` (name, description, icons)
- Keyboard shortcuts
- Error toasts (replace console.log)
- Loading spinners during calculations
- Responsive window handling

### How to Verify Phase 8 is Complete

- [ ] Full calc benchmark < 20ms
- [ ] Node power overlay matches PoB recommendations
- [ ] Trade query opens correct URL
- [ ] CI pipeline green on push
- [ ] App builds and runs cleanly from a fresh clone
- [ ] `cargo clippy` and `cargo test` are clean

---

## Rust Learning Progression

Summary of what you'll learn in each phase:

| Phase | Primary Rust Concepts | Key Crates |
|-------|----------------------|------------|
| **1. Data** ✅ | Structs, Enums, Serde, Traits, Pattern Matching, Modules, FxHashMap, Option/Result, Newtype (StatId), Typed Errors (thiserror) | serde, serde_json, thiserror, rustc-hash |
| **2. Modifiers** | Generics, Lifetimes, Iterators, Arc, RwLock, Closures, From/Into, Builder, SmallVec, Layered Composition, Channels | bitflags, smallvec |
| **3. Skills** | Trait Objects, Dynamic Dispatch, Type State, Complex Enums | — |
| **4. Items** | Ownership, Borrowing, RefCell, String Parsing, TryFrom, Display | — |
| **5. Calc Engine** | Async/Await, Channels, Rayon Parallelism, Benchmarks, Bitflags | tokio, rayon, criterion |
| **6. Builds** | File I/O, Compression, Base64, XML, HTTP Client, Atomic Writes | flate2, base64, quick-xml, reqwest |
| **7. Advanced** | Proc Macros, Advanced Generics, const fn, PhantomData, Newtypes | (optional proc-macro crate) |
| **8. Polish** | Profiling, Release Optimization, CI/CD, Inline Hints | tracing, criterion |

---

## Reference: PoB Source Map

When implementing a feature, look at the corresponding PoB Lua source for reference. The repo is at [github.com/PathOfBuildingCommunity/PathOfBuilding](https://github.com/PathOfBuildingCommunity/PathOfBuilding) (branch: `dev`).

| Your Module | PoB Reference File(s) | Line Count (approx) | Status |
|---|---|---|---|
| `data/tree.rs` | `src/Classes/PassiveTree.lua` | ~1500 | ✅ Implemented |
| `modifier/mod_db.rs` | `src/Classes/ModDB.lua` | ~400 | ❌ Phase 2 |
| `modifier/parser.rs` | `src/Modules/ModParser.lua` | ~2500 | ❌ Phase 2 |
| `calc/offence.rs` | `src/Modules/CalcOffence.lua` | ~4000 | ❌ Phase 5 |
| `calc/defence.rs` | `src/Modules/CalcDefence.lua` | ~2500 | ❌ Phase 5 |
| `calc/active.rs` | `src/Modules/CalcActiveSkill.lua` | ~1500 | ❌ Phase 5 |
| `calc/perform.rs` | `src/Modules/CalcPerform.lua` | ~900 | ❌ Phase 5 |
| `calc/setup.rs` | `src/Modules/CalcSetup.lua` | ~1200 | ❌ Phase 5 |
| `calc/triggers.rs` | `src/Modules/CalcTriggers.lua` | ~800 | ❌ Phase 5 |
| `item/parser.rs` | `src/Classes/Item.lua` | ~2000 | ❌ Phase 4 |
| `item/crafting.rs` | `src/Classes/ItemsTab.lua` | ~3000 | ❌ Phase 4 |
| `build/codec.rs` | `src/Classes/Build.lua` (encode/decode sections) | ~500 | ❌ Phase 6 |
| `config/options.rs` | `src/Modules/ConfigOptions.lua` | ~1200 | ❌ Phase 7 |

**Total PoB calc engine**: ~15,000+ lines of Lua. Your Rust will likely be similar or slightly more compact due to stronger types.

---

## Tips for Success

1. **Work phase by phase.** Don't skip ahead. Each phase teaches concepts you need for the next.

2. **Compare with PoB constantly.** After each phase, create the same build in PoB and your app. Verify numbers match.

3. **Start the ModParser small.** You don't need to parse every mod pattern on day one. Add patterns as you encounter them in testing.

4. **Read the PoB Lua when stuck.** The calculation logic is well-structured. `CalcOffence.lua` is your primary reference for DPS formulas.

5. **Write tests early.** Every new parser pattern, every new calculation formula — test it. Use known PoB builds as integration test fixtures.

6. **Use `cargo clippy` religiously.** It teaches you idiomatic Rust better than any tutorial.

7. **The heavy optimizations are already in.** Because you baked in `StatId`, `FxHashMap`, `CalcContext`, compact `Modifier`, `RwLock`, layered `ModDB`, and raw JSON IPC from Phase 1-2, you won't need a painful optimization pass. Phase 8 performance work is for profiling and fine-tuning, not architectural rework.

8. **The modifier system is everything.** If `ModDB` and `ModParser` are solid, the calc engine is just arithmetic on top of it. Invest the most time here (Phase 2).

9. **Keep the Svelte frontend simple.** Your goal is learning Rust. The frontend just needs to call commands, display data, and handle user input. Don't over-engineer it.

10. **Commit after each milestone.** Each phase should be a series of working commits. Never have a broken main branch.
