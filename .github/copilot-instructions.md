# Rusty Builds — Copilot Instructions

> **Purpose**: Authoritative reference for AI agents working on this codebase. Read this file first to avoid stale assumptions about frameworks, versions, patterns, and project state.

---

## 1. Project Overview

**Rusty Builds** is a **Path of Exile 1** build planner — a Rust-powered rebuild of [Path of Building](https://github.com/PathOfBuildingCommunity/PathOfBuilding). It is a desktop app targeting full calculation parity with PoB.

- **Repository**: `Basorik/Rusty-Builds` (branch: `main`)
- **License**: MIT
- **Target game**: Path of Exile **1** only (not PoE 2)

---

## 2. Technology Stack & Versions

### Runtime & Toolchains

| Tool | Version | Notes |
|---|---|---|
| Rust | `1.91.1` (edition 2021) | No `rust-toolchain.toml`; uses system default |
| Bun | `1.3.8` | Package manager **and** script runner (not npm/yarn) |
| Node.js | `v24.11.1` | Available but Bun is preferred |

### Backend (Rust — `src-tauri/`)

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | `2` | Desktop shell, IPC commands, window management |
| `tauri-build` | `2` | Build-time codegen |
| `tauri-plugin-opener` | `2` | OS file/URL opener |
| `tauri-plugin-log` | `2.8.0` | Structured logging (uses `log` crate macros) |
| `serde` | `1` (with `derive`) | Serialization/deserialization |
| `serde_json` | `1` | JSON parsing |
| `specta` | `=2.0.0-rc.22` | Type-safe TypeScript bindings (pinned RC) |
| `tauri-specta` | `=2.0.0-rc.21` | Tauri ↔ specta bridge (pinned RC) |
| `specta-typescript` | `0.0.9` | TS code generator |
| `tokio` | `1.49.0` | Async runtime |
| `reqwest` | `0.13.1` | HTTP client |
| `governor` | `0.10.4` | Rate limiting (for future PoE API) |
| `rkyv` | `0.8.14` | Zero-copy binary serialization |
| `tempfile` | `3.24.0` | Atomic file writes |
| `regex` | `1` | Stat string parsing |
| `thiserror` | `2.0.18` | Typed error enums |
| `rustc-hash` | `2.1.1` | `FxHashMap`/`FxHashSet` — fast non-crypto hashing |
| `bitflags` | `2.11.0` | Modifier flags (Phase 2+) |
| `log` | `0.4.29` | Logging facade |

> **Pinned versions**: `specta` and `tauri-specta` are pinned to exact RC versions. Do not bump without testing TypeScript binding generation.

### Frontend (SvelteKit — `src/`)

| Package | Version | Notes |
|---|---|---|
| `svelte` | `^5.0.0` | **Svelte 5** — uses runes (`$state`, `$derived`, `$effect`, `$props`, `$bindable`) |
| `@sveltejs/kit` | `^2.9.0` | SvelteKit 2 — file-based routing |
| `@sveltejs/adapter-static` | `^3.0.6` | SPA mode (no SSR) — required for Tauri |
| `@sveltejs/vite-plugin-svelte` | `^5.0.0` | Vite integration |
| `vite` | `^6.0.3` | Build tool |
| `typescript` | `~5.6.2` | Strict mode enabled |
| `pixi.js` | `^8.16.0` | **PixiJS 8** — WebGL skill tree renderer |
| `@tauri-apps/api` | `^2` | Tauri IPC from frontend |
| `@tauri-apps/plugin-opener` | `^2` | Frontend opener plugin |
| `bun-types` | `^1.3.10` | Bun type definitions (dev tooling) |
| `svelte-check` | `^4.0.0` | Svelte type checker |

---

## 3. Critical Framework Conventions

### Svelte 5 Runes (NOT Svelte 4 Stores)

This project uses **Svelte 5 runes exclusively**. Never use `writable()`, `$:`, or Svelte 4 store patterns.

```svelte
<!-- ✅ Correct: Svelte 5 runes -->
let count = $state(0);
let doubled = $derived(count * 2);
$effect(() => { console.log(count); });
let { value = $bindable(0) } = $props();

<!-- ❌ Wrong: Svelte 4 patterns — do NOT use -->
import { writable } from 'svelte/store';
$: doubled = count * 2;
export let value;
```

### PixiJS 8 (NOT PixiJS 7)

Uses PixiJS 8 API with `Application.init()` (async), `Graphics.setStrokeStyle()`, and `Container`-based scene graph. PixiJS 7's `new Application({ ... })` constructor pattern is deprecated.

### Tauri 2 (NOT Tauri 1)

Uses Tauri v2 APIs. Commands are registered via `tauri_specta::collect_commands![]`. The capability system (not Tauri 1's allowlist) controls permissions in `src-tauri/capabilities/default.json`.

### tauri-specta Bindings

TypeScript bindings are **auto-generated** from Rust types at dev startup. The generated file is `src/bindings.ts`. Never edit it manually. Types derive both `specta::Type` and `serde::{Serialize, Deserialize}`.

To regenerate: bindings export automatically in debug builds (`#[cfg(debug_assertions)]`) during `tauri::Builder::setup`.

---

## 4. Architecture

### IPC Boundary

```
Svelte Frontend  ──IPC (tauri commands)──►  Rust Backend
                 ◄──── Result<T, String> ──
```

- Frontend calls `commands.updateSelectedNodes(ids)` (from `bindings.ts`)
- Rust handles via `#[tauri::command]` + `#[specta::specta]` functions in `lib.rs`
- Errors cross IPC as `String` — internal Rust uses typed `thiserror` enums

### Tauri Commands (Current)

| Command | Parameters | Returns | Status |
|---|---|---|---|
| `greet` | `name: &str` | `String` | ✅ Test/example |
| `update_build_info` | `level, character_class, bloodline` | `Result<(), String>` | ✅ Working |
| `update_selected_nodes` | `node_ids: Vec<u32>` | `Result<BuildStats, String>` | ⚠️ Partial — accumulates stats but STR/DEX/INT are zeroed |

### State Management

| State | Type | Scope |
|---|---|---|
| `BuildInfo` | `Mutex<BuildInfo>` | Managed Tauri state — holds class, level, bloodline, selections, stats |
| `TreeData` | `TreeData` (immutable) | Managed Tauri state — loaded once from `include_str!` at startup |
| `StorageManager` | `StorageManager` | Managed Tauri state — stub, not yet functional |

### Frontend Routing

| Route | Component | Purpose |
|---|---|---|
| `/` | `+page.svelte` | Home — new build button, saved builds list (placeholder) |
| `/skilltree` | `skilltree/+page.svelte` | Main build editor — composes Header + Sidebar + SkillTree |

SSR is disabled (`export const ssr = false` in `+layout.ts`). Adapter-static with `fallback: "index.html"` for SPA mode.

---

## 5. File Structure

### Rust Backend (`src-tauri/src/`)

```
src-tauri/src/
├── main.rs                     # Binary entry → lib::run()
├── lib.rs                      # App setup, commands, types (BuildInfo, BuildStats, Class, etc.)
├── models.rs                   # LiteNode (rkyv-serialized, legacy)
├── data/
│   ├── mod.rs                  # Re-exports: PassiveTree, StatId
│   ├── stat_id.rs              # StatId enum (320 variants from SkillStatMap.json)
│   ├── tree.rs                 # PassiveNode struct (stub)
│   ├── bases.rs                # Empty stub
│   ├── gems.rs                 # Empty stub
│   ├── mods.rs                 # Empty stub
│   ├── skills.rs               # Empty stub
│   └── uniques.rs              # Empty stub
├── client/
│   ├── mod.rs                  # Re-exports poe module
│   └── poe.rs                  # PoeClient skeleton (empty)
└── storage/
    ├── mod.rs                  # Re-exports StorageManager
    ├── manager.rs              # StorageManager stub (no-op)
    └── file_system.rs          # FileCache — rkyv binary caching for LiteNode
```

### Svelte Frontend (`src/`)

```
src/
├── app.html                    # HTML shell
├── App.svelte                  # Legacy root (unused — routing uses +page.svelte)
├── bindings.ts                 # AUTO-GENERATED by tauri-specta — never edit
├── data.json                   # PoE 1 passive tree data (~160k lines)
├── routes/
│   ├── +layout.ts              # ssr = false
│   ├── +page.svelte            # Home page (build management UI placeholder)
│   └── skilltree/
│       └── +page.svelte        # Main editor page
└── components/
    ├── Header.svelte           # Class/ascendancy/bloodline/level selectors
    ├── Sidebar.svelte          # Node counts, stats placeholder
    └── SkillTree.svelte        # PixiJS canvas — tree rendering, selection, pan/zoom
```

### Data Files (`src-tauri/data/`)

```
src-tauri/data/
├── tree/
│   ├── 3.27.0g/data.json       # Versioned PoE 1 passive tree (~5.3 MB)
│   └── active.json             # Symlink → 3.27.0g/data.json
└── pob/                        # PoB game data from repoe-fork (123 files)
    ├── SkillStatMap.json        # Internal stat ID → calc variable mapping (707 entries)
    ├── Gems.json                # All skill/support gems
    ├── Minions.json, Spectres.json
    ├── ModItem.json, ModJewel.json, ModFlask.json, ...
    ├── ClusterJewels.json
    ├── Bases/                   # Base item types by category
    ├── Skills/                  # Skill data by attribute
    ├── StatDescriptions/        # 22 JSON files — stat display text templates
    ├── TimelessJewelData/       # Seed/conversion tables
    └── Uniques/                 # Unique item data (including Special/)
```

---

## 6. Key Types (Rust)

### `StatId` (`data/stat_id.rs`)

320-variant `#[repr(u16)]` enum sourced from `SkillStatMap.json` calc variable names. These are the stats the calc engine queries (e.g., `Life`, `CritChance`, `FireResist`), not display text keys.

- Lookup: `StatId::from_name("CritChance")` → `Some(StatId::CritChance)`
- Implementation: `OnceLock<FxHashMap<&'static str, StatId>>` — initialized once, O(1) thereafter
- 5 `Condition:*` variants mapped as `ConditionCanGainRage` etc. (Rust identifiers can't contain `:`)
- `from_name()` still accepts the colon form: `"Condition:CanGainRage"` → `ConditionCanGainRage`

### `BuildInfo` (`lib.rs`)

```rust
pub struct BuildInfo {
    pub name: String,
    pub level: u32,
    pub stats: BuildStats,
    pub class: Class,
    pub bloodline: Bloodline,
    pub selected_nodes: BuildSelection,
}
```

### `BuildStats` (`lib.rs`)

```rust
pub struct BuildStats {
    pub total_strength: i32,     // Currently always 0 (placeholder)
    pub total_dexterity: i32,    // Currently always 0 (placeholder)
    pub total_intelligence: i32, // Currently always 0 (placeholder)
    pub node_count: u32,
    pub stat_totals: HashMap<String, f64>,  // Template key → summed value
}
```

### `Class` (`lib.rs`)

Tagged enum with optional ascendancy:
```rust
#[serde(tag = "class", content = "ascendancy")]
pub enum Class {
    Marauder(Option<MarauderAscendancy>),
    Ranger(Option<RangerAscendancy>),
    // ... 7 classes total
}
```

### `StatAccumulator` (`lib.rs`)

Current placeholder stat system. Parses numbers from display strings using regex, replaces numbers with `#` to form template keys, and sums values. Will be replaced by `ModDB` + `StatId` in Phase 2.

### `TreeData` (`lib.rs`)

Loaded at startup via `include_str!("../data/tree/active.json")`. Stores `node_id → Vec<String>` (raw stat display strings) for every passive tree node.

---

## 7. Data Pipeline

### `tools/fetch_data.ts`

Bun script that downloads all game data files:

```bash
bun run tool:fetch-data       # Download everything (PoB + tree)
bun run tool:fetch-pob        # Only repoe-fork files (123 files)
bun run tool:fetch-tree       # Only passive skill tree
```

- **PoB data**: Recursively crawls `repoe-fork.github.io/pob-data/poe1/` index pages, downloads all `.json` files (excluding `.min.json`)
- **Tree data**: Fetches latest release tag from `grindinggear/skilltree-export` GitHub API, downloads `data.json` to `src-tauri/data/tree/{tag}/data.json`, creates `active.json` symlink
- **Current tree version**: `3.27.0g`

### Data File Roles

| File | Role | Used By |
|---|---|---|
| `SkillStatMap.json` | Maps internal stat IDs → calc variable names + modifier types (BASE/INC/MORE/FLAG) | `stat_id.rs` generation, future `ModParser` |
| `StatDescriptions/*.json` | Maps internal stat IDs → human-readable display text templates | Future tooltip rendering |
| `Gems.json` | All skill/support gem data with per-level stats | Future `data/gems.rs` |
| `Bases/*.json` | Base item types by category | Future `data/bases.rs` |
| `Uniques/*.json` | Unique item database | Future `data/uniques.rs` |
| `Mod*.json` | Item/jewel/flask/map modifier pools | Future `data/mods.rs` |
| `tree/active.json` | PoE 1 passive skill tree (nodes, groups, connections, constants) | `TreeData`, `SkillTree.svelte` |

---

## 8. SkillTree Component (`SkillTree.svelte`)

The most complex frontend component. Key implementation details:

- **Renderer**: PixiJS 8 WebGL canvas with sprite-based nodes (shared circle texture)
- **Scene graph**: `mainContainer` → `connectionGraphics` → `nodeContainer` → `selectionGraphics` → `highlightGraphics`
- **Pan/zoom**: Manual transform tracking (`transform.x`, `transform.y`, `transform.k`), mouse-relative zoom
- **Hit detection**: Spatial grid (`GRID_CELL_SIZE = 300`) for O(1) node lookups
- **Viewport culling**: Only renders sprites within the visible area + 100px padding
- **Node selection**: BFS-validated connectivity from class start node; deselection checks graph remains connected
- **Ascendancy**: Separate `selectedAscNodeIds` set, max 8 points, separate adjacency graph per ascendancy/bloodline group
- **Backend sync**: Debounced (50ms) `commands.updateSelectedNodes()` call after selection changes
- **Props**: `treeData`, `selectedCount` ($bindable), `ascSelectedCount` ($bindable), `selectedClass`, `selectedAscendancy`, `selectedBloodline`

---

## 9. Build & Development

### Commands

```bash
bun run tauri dev          # Start dev mode (Vite + Tauri, hot reload)
bun run dev                # Vite dev server only (no Tauri shell)
bun run build              # Production build (Vite)
bun run tauri build        # Full desktop app build
bun run check              # svelte-check type checking
bun run tool:fetch-data    # Download all game data
```

### Tauri Configuration

- **Dev URL**: `http://localhost:1420` (Vite dev server, strict port)
- **Frontend dist**: `../build` (adapter-static output)
- **Before dev**: `bun run dev`
- **Before build**: `bun run build`
- **Window**: 800×600 default, title "rusty-builds"
- **CSP**: null (disabled)
- **Capabilities**: `core:default`, `opener:default`
- **Bundle targets**: all platforms

---

## 10. Development Roadmap (from PLAN.md)

The project follows an 8-phase plan. Each phase builds on the last.

| Phase | Name | Status | Core Deliverable |
|---|---|---|---|
| **1** | Foundation & Data Layer | 🔄 In Progress | Load all game data into typed Rust structs |
| **2** | Modifier System & Passive Tree Stats | ❌ Not Started | `ModDB`, `ModParser`, typed stat accumulation |
| **3** | Skill & Gem System | ❌ Not Started | Active skills, support gems, socket groups |
| **4** | Item System | ❌ Not Started | Items, affixes, crafting, equipment |
| **5** | Full Calculation Engine | ❌ Not Started | DPS, defence, offence — full PoB parity |
| **6** | Build Management & Import/Export | ❌ Not Started | Save/load, PoB share codes, character import |
| **7** | Configuration, Party, Jewels, Advanced | ❌ Not Started | Config options, party buffs, timeless jewels |
| **8** | Polish, Performance & Trade | ❌ Not Started | Profiling, optimization, trade query generation |

### Phase 1 Progress

- ✅ PoE 1 tree data fetched and versioned (`3.27.0g`)
- ✅ `tools/fetch_data.ts` — recursive data crawler (123 PoB files)
- ✅ `data/stat_id.rs` — 320-variant `StatId` enum from `SkillStatMap.json`
- ✅ `data/mod.rs` — module structure with stubs for bases, gems, mods, skills, tree, uniques
- ✅ `TreeData` loads node stats at startup via `include_str!`
- ✅ `StatAccumulator` sums stat values from display strings (placeholder)
- ⬜ Typed `PassiveTree`/`PassiveNode` structs (tree.rs is a stub)
- ⬜ Load gems, bases, uniques, mods into typed structs
- ⬜ `GameData` aggregate struct served via Tauri state
- ⬜ Replace `HashMap<String, f64>` stat accumulation with `StatId`-keyed `ModDB`

### Key Design Decisions (Locked)

1. **`StatId` enum** (not String) for all modifier lookups — avoids string hashing in hot paths
2. **`FxHashMap`** via `rustc-hash` — faster non-crypto hashing; stat IDs are small integers
3. **`CalcContext` parameter** on all ModDB queries — required for conditional mods
4. **Compact `Modifier` struct** — `SourceId(u32)`, `SmallVec<[ModTag; 2]>` for inline tags
5. **Layered `ModDB`** — separate tree/item/gem/config layers; targeted cache invalidation
6. **Typed errors internally** — `String` only at IPC boundary
7. **Node selection stays in PixiJS** — 60fps responsiveness; all calculations in Rust

---

## 11. Conventions & Gotchas

### Rust Conventions
- Use `FxHashMap` / `FxHashSet` everywhere (not `std::collections::HashMap`)
- Use `StatId` enum for stat keys (not `String`) when building the modifier system
- Internal errors use `thiserror` enums; only convert to `String` at the IPC boundary
- Lib crate name is `rusty_builds_lib` (not `rusty_builds`) due to Windows cargo bug
- Logging via `log` macros (`info!`, `warn!`, `error!`) — backend is `tauri-plugin-log`

### Frontend Conventions
- All Tauri IPC calls go through `commands` object from `src/bindings.ts`
- TypeScript strict mode enabled
- Components use Svelte 5 `$props()` with destructuring and `$bindable()`
- `data.json` in `src/` is the frontend's copy of the tree data (same content as `active.json`)

### File Generation
- `src/bindings.ts` — auto-generated in debug builds, never edit manually
- `src-tauri/src/data/stat_id.rs` — generated by inline Python from `SkillStatMap.json`, marked "DO NOT EDIT MANUALLY"
- `src-tauri/data/tree/active.json` — symlink managed by `tools/fetch_data.ts`

### Data Source Terminology
- **SkillStatMap**: Maps internal game stat IDs → calc engine variable names + types (BASE/INC/MORE/FLAG). This drives `StatId` and the future `ModParser`.
- **StatDescriptions**: Maps internal game stat IDs → display text templates (`"{0}% increased maximum Life"`). Used for tooltips/UI, not calculations.
- **repoe-fork**: Community-maintained structured export of PoE game data (`repoe-fork.github.io/pob-data/poe1/`). Source of all `src-tauri/data/pob/` files.
- **skilltree-export**: GGG's official passive tree JSON (`grindinggear/skilltree-export`). Source of `src-tauri/data/tree/`.

---

## 12. PoB Reference

The primary reference implementation is [Path of Building Community](https://github.com/PathOfBuildingCommunity/PathOfBuilding) (branch: `dev`).

Key PoB source files and their Rust equivalents:

| Rust Module (planned) | PoB Lua File | ~Lines |
|---|---|---|
| `modifier/mod_db.rs` | `src/Classes/ModDB.lua` | 400 |
| `modifier/parser.rs` | `src/Modules/ModParser.lua` | 2500 |
| `calc/offence.rs` | `src/Modules/CalcOffence.lua` | 4000 |
| `calc/defence.rs` | `src/Modules/CalcDefence.lua` | 2500 |
| `calc/setup.rs` | `src/Modules/CalcSetup.lua` | 1200 |
| `calc/perform.rs` | `src/Modules/CalcPerform.lua` | 900 |
| `item/parser.rs` | `src/Classes/Item.lua` | 2000 |

The core calc formula: `base × (1 + sum_of_increases%) × product_of_mores`
