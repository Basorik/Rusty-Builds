# Rusty Builds

**A Path of Exile 2 Skill Tree Planner built with Tauri, Rust, and Svelte**

> Desktop application for planning and visualizing Path of Exile 2 passive skill tree builds — featuring an interactive WebGL canvas, real-time stat tracking, and a type-safe Rust backend.

**Status:** Early-stage (v0.1.0) — core skill tree rendering and selection are functional; stat calculations, build persistence, and POE API integration are in progress.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Rust Backend](#rust-backend-src-tauri)
- [Svelte Frontend](#svelte-frontend-src)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Suggested Improvements & Next Steps](#suggested-improvements--next-steps)
- [License](#license)

---

## Overview

Rusty Builds is a cross-platform desktop application that lets players plan their Path of Exile 2 character builds by interacting with the full passive skill tree. The app is built on the [Tauri 2](https://v2.tauri.app/) framework, pairing a **Rust** backend for data management and API communication with a **Svelte 5 + Pixi.js** frontend for high-performance, interactive tree visualization.

### Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Framework | Tauri 2 | Desktop shell, IPC bridge, native capabilities |
| Backend | Rust (Tokio async runtime) | State management, storage, POE API client |
| Frontend | Svelte 5 (SvelteKit, static adapter) | UI framework with rune-based reactivity |
| Rendering | Pixi.js 8 (WebGL) | High-performance 2D skill tree canvas |
| Type Safety | tauri-specta + specta | Auto-generated TypeScript bindings from Rust types |
| Serialization | rkyv (zero-copy), serde/serde_json | Binary caching & JSON data handling |

---

## Features

- **Interactive Skill Tree Canvas** — Pan (click-drag), zoom (scroll wheel), and click to select nodes on a full WebGL-rendered skill tree powered by Pixi.js
- **7 Character Classes** — Marauder, Ranger, Witch, Duelist, Templar, Shadow, and Scion — each with 3 ascendancy subclasses plus alternate ascendancies
- **6 Bloodlines** — None, Crusader, Redeemer, Hunter, Assassin, Champion
- **Level Selection** — Choose character level from 1 to 100
- **Smart Node Selection** — Only adjacent (connected) nodes can be selected; deselecting a node is blocked if it would disconnect the selected cluster (BFS connectivity validation)
- **Three Node Types** — Keystone (large, deep red), Notable (medium, gold), and Regular (small, grey) — each with distinct visual sizing
- **Hover Tooltips** — Displays node name, stats, and description on mouse hover
- **Viewport Culling** — Only nodes visible in the current viewport are rendered, ensuring smooth performance even with thousands of nodes
- **Auto-Generated TypeScript Bindings** — Rust types and commands are exported as fully-typed TypeScript interfaces via tauri-specta, eliminating IPC type drift
- **Binary Node Caching** — Skill tree node data is cached to disk using rkyv zero-copy serialization with atomic file writes for crash safety

---

## Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────┐
│                    Tauri Desktop Shell                    │
│                                                          │
│  ┌─────────────────────┐    Tauri IPC    ┌────────────┐  │
│  │   Svelte Frontend   │ ◄────────────► │ Rust Backend│  │
│  │                     │   (invoke)      │            │  │
│  │  - Pixi.js Canvas   │                │ - Commands  │  │
│  │  - SvelteKit Router │                │ - State     │  │
│  │  - Auto-gen Bindings│                │ - Storage   │  │
│  │                     │                │ - POE Client│  │
│  └─────────────────────┘                └────────────┘  │
└──────────────────────────────────────────────────────────┘
```

### UI Layout

```
┌──────────────────────────────────────────────────────────┐
│  Header: [Rusty Builds]  Level ▾  Class ▾  Asc ▾  Blood ▾│
├────────────┬─────────────────────────────────────────────┤
│            │                                             │
│  Sidebar   │          Pixi.js Skill Tree Canvas          │
│  (240px)   │                                             │
│            │     - Drag to pan                           │
│  Selected: │     - Scroll to zoom (0.01x – 2x)          │
│  12 nodes  │     - Click nodes to select/deselect        │
│            │     - Hover for tooltip                     │
│  [Menu]    │                                             │
│            │                                             │
└────────────┴─────────────────────────────────────────────┘
```

---

## Rust Backend (`src-tauri/`)

The backend is responsible for application state, data persistence, external API communication, and exposing typed commands to the frontend.

### Entry Points

- **`main.rs`** — Binary entry point; delegates to `rusty_builds_lib::run()`
- **`lib.rs`** — Core application setup: registers Tauri commands, initializes plugins (logging), sets up `StorageManager`, creates shared `Mutex<BuildInfo>` state, and exports TypeScript bindings via tauri-specta (debug builds only)

### Data Models

#### Build System (`lib.rs`)

| Type | Fields | Description |
|---|---|---|
| `BuildInfo` | `name`, `level`, `stats`, `class`, `bloodline`, `selected_nodes` | Complete character build representation |
| `BuildSelection` | `selected_node_ids: HashSet<u32>` | Set of selected passive tree node IDs |
| `BuildStats` | `total_strength`, `total_dexterity`, `total_intelligence`, `node_count` | Aggregated stat totals for the build |
| `Class` | 7 variants, each with optional ascendancy | Character class with ascendancy subclass |
| `Bloodline` | 6 variants (None, Crusader, Redeemer, Hunter, Assassin, Champion) | Bloodline selection |

#### Node Data (`models.rs`)

| Type | Fields | Description |
|---|---|---|
| `LiteNode` | `id: u16`, `x: f32`, `y: f32`, `icon: String` | Lightweight node for cache storage (rkyv-serializable) |
| `NodeType` | `Notable`, `Keystone`, `Regular` | Classification of passive tree nodes |

### Tauri Commands

These are the IPC endpoints callable from the frontend:

| Command | Parameters | Returns | Description |
|---|---|---|---|
| `greet` | `name: &str` | `String` | Test/example command |
| `update_build_info` | `level`, `character_class`, `bloodline` | `Result<(), String>` | Updates the current build's class, level, and bloodline |
| `update_selected_nodes` | `node_ids: Vec<u32>` | `Result<BuildStats, String>` | Stores selected node IDs and returns calculated stats |

### POE Client (`client/poe.rs`)

A skeleton HTTP client prepared for Path of Exile API integration:

- Uses `reqwest` for async HTTP requests
- Integrates `governor` crate for rate limiting (respecting API throttle limits)
- Defines `PoeClientError` enum (wrapping `reqwest::Error`) via `thiserror`
- **Status:** Struct and error types defined; actual API methods are not yet implemented

### Storage System (`storage/`)

| Component | Purpose |
|---|---|
| `StorageManager` | Wrapper around Tauri `AppHandle`; entry point for storage operations |
| `FileCache` | Binary cache using rkyv zero-copy serialization; stores `Vec<LiteNode>` as binary files in the app data `cache/` directory; uses atomic temp-file-then-persist writes via the `tempfile` crate for crash safety |

### Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `tauri` | 2 | Desktop application framework |
| `tauri-plugin-opener` | 2 | Open files/URLs natively |
| `tauri-plugin-log` | 2.8.0 | Structured logging plugin |
| `serde` / `serde_json` | 1 | JSON serialization/deserialization |
| `tokio` | 1.49.0 | Async runtime |
| `reqwest` | 0.13.1 | HTTP client for API calls |
| `governor` | 0.10.4 | Rate limiting for API requests |
| `rkyv` | 0.8.14 | Zero-copy binary serialization for node caching |
| `tempfile` | 3.24.0 | Atomic temporary file writes |
| `specta` / `tauri-specta` / `specta-typescript` | RC | Auto-generate TypeScript bindings from Rust types |
| `thiserror` | 2.0.18 | Ergonomic error type derivation |
| `log` | 0.4.29 | Logging facade |

---

## Svelte Frontend (`src/`)

The frontend provides the interactive UI for viewing and manipulating the skill tree, built with Svelte 5's rune-based reactivity and Pixi.js for high-performance WebGL rendering.

### Framework & Build

- **Svelte 5** with rune-based reactivity (`$state`, `$derived`, `$effect`)
- **SvelteKit** with `adapter-static` — SSR disabled for Tauri compatibility (pure SPA)
- **Vite 6** as the build tool
- **TypeScript ~5.6** for type checking

### Routing

| Route | File | Description |
|---|---|---|
| `/` | `src/routes/+page.svelte` | Home page — "Rusty Builds" hero section, "New Build" button, placeholder for saved builds list (save/load/delete UI scaffolded) |
| `/skilltree` | `src/routes/skilltree/+page.svelte` | Main builder — composes Header, Sidebar, and SkillTree components; manages selected node state and class selection |

### Components

#### `SkillTree.svelte` — Interactive Skill Tree Canvas

The heart of the application. A Pixi.js-powered WebGL canvas that renders the complete passive skill tree.

- **Rendering** — Parses `data.json` to calculate absolute node positions using orbital math (`x = r·sin(θ)`, `y = -r·cos(θ)`); draws node sprites and connection lines
- **Node Visuals** — Three sizes/colors: Keystone (60px, deep red `#992222`), Notable (40px, gold `#997700`), Regular (20px, grey `#2a2a2a`)
- **Selection System** — Class start node auto-selected and protected; only adjacent nodes are selectable; BFS traversal validates that deselecting a node won't disconnect the selected cluster
- **Spatial Grid** — O(1) hit detection using 300px grid cells for efficient click-to-node mapping
- **Controls** — Click-drag to pan (5px dead zone), scroll to zoom (0.01x–2x range, zooms toward cursor), click to toggle node selection
- **Visual Feedback** — Light blue glow ring on selected nodes, highlighted connections between selected nodes, hover tooltip with name/stats/description
- **Performance** — Viewport culling (only visible nodes are rendered); device pixel ratio support for retina displays; window resize handler

#### `Header.svelte` — Build Configuration Ribbon

Top-fixed bar with build configuration dropdowns:

- **Level** — Dropdown selector (1–100)
- **Class** — 7 character classes (Scion, Marauder, Ranger, Witch, Duelist, Templar, Shadow)
- **Ascendancy** — Dynamically populated based on selected class (e.g., Marauder → Juggernaut / Berserker / Chieftain)
- **Bloodline** — 6 options (None, Crusader, Redeemer, Hunter, Assassin, Champion)
- Calls `updateBuildInfo()` on every dropdown change to sync state with the Rust backend
- "Rusty Builds" brand link navigates back to home

#### `Sidebar.svelte` — Stats & Navigation Panel

Right-side dark panel (240px):

- Displays real-time selected node count
- Menu button to return to the home page
- Placeholder section for future detailed stat display ("More stats coming soon")

### Data & Bindings

- **`data.json`** — Complete Path of Exile 2 skill tree data (~10,000+ lines): 7 character classes with base stats and ascendancies, group coordinates with orbital positioning data, and node definitions
- **`bindings.ts`** — Auto-generated by tauri-specta from Rust types. Provides fully-typed IPC functions (`greet`, `updateSelectedNodes`, `updateBuildInfo`) and TypeScript types (`Class`, `Bloodline`, `BuildStats`, `Result<T, E>`)

### Styling

Custom dark theme inspired by Path of Exile's aesthetic — no CSS framework:

| Element | Color | Hex |
|---|---|---|
| Background | Near-black | `#0a0a0a` |
| Primary accent | Gold/brass | `#c8a95e` |
| Keystone nodes | Deep red | `#992222` |
| Notable nodes | Brown/gold | `#997700` |
| Regular nodes | Dark grey | `#2a2a2a` |
| Selection glow | Blue | `#4488ff` |
| Text | Light tan | `#dfcf99` |

### Frontend Dependencies

| Package | Version | Purpose |
|---|---|---|
| `@tauri-apps/api` | ^2 | Tauri IPC bridge for calling Rust commands |
| `@tauri-apps/plugin-opener` | ^2 | Open files/URLs from the frontend |
| `pixi.js` | ^8.16.0 | WebGL 2D rendering engine for the skill tree canvas |
| `svelte` | ^5.0.0 | UI framework with rune-based reactivity |
| `@sveltejs/kit` | ^2.9.0 | Application framework (routing, static adapter) |
| `vite` | ^6.0.3 | Build tool and dev server |
| `typescript` | ~5.6.2 | Type checking |

---

## Project Structure

```
Rusty-Builds/
├── package.json                # Frontend dependencies & scripts
├── svelte.config.js            # SvelteKit config (static adapter)
├── vite.config.js              # Vite build config
├── tsconfig.json               # TypeScript configuration
│
├── src/                        # ── Svelte Frontend ──
│   ├── app.html                # HTML shell
│   ├── App.svelte              # Root component (loads data, sets global styles)
│   ├── bindings.ts             # Auto-generated Rust↔TS type bindings (tauri-specta)
│   ├── data.json               # Complete POE2 skill tree data
│   ├── components/
│   │   ├── Header.svelte       # Build config ribbon (level, class, ascendancy, bloodline)
│   │   ├── Sidebar.svelte      # Node counter, navigation, stats placeholder
│   │   └── SkillTree.svelte    # Pixi.js WebGL skill tree canvas
│   └── routes/
│       ├── +layout.ts          # Disables SSR for Tauri SPA mode
│       ├── +page.svelte        # Home page (new build, saved builds)
│       └── skilltree/
│           └── +page.svelte    # Skill tree builder page
│
├── src-tauri/                  # ── Rust Backend ──
│   ├── Cargo.toml              # Rust dependencies
│   ├── tauri.conf.json         # Tauri app configuration (window, identifier, plugins)
│   ├── build.rs                # Tauri build script
│   ├── capabilities/
│   │   └── default.json        # Tauri security capabilities
│   ├── icons/                  # Application icons
│   └── src/
│       ├── main.rs             # Binary entry point
│       ├── lib.rs              # App setup, commands, build types, specta export
│       ├── models.rs           # LiteNode, NodeType (rkyv-serializable)
│       ├── client/
│       │   ├── mod.rs          # Client module declarations
│       │   └── poe.rs          # POE API client (skeleton)
│       └── storage/
│           ├── mod.rs          # Storage module declarations
│           ├── manager.rs      # StorageManager (app handle wrapper)
│           └── file_system.rs  # FileCache (rkyv binary caching)
│
└── static/                     # Static assets
```

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) (LTS recommended)
- [Tauri CLI](https://v2.tauri.app/start/create-project/) — installed via npm or cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/Basorik/Rusty-Builds.git
cd Rusty-Builds

# Install frontend dependencies
npm install

# Run in development mode (starts both Vite dev server and Tauri)
npm run tauri dev

# Build for production
npm run tauri build
```

### Available Scripts

| Command | Description |
|---|---|
| `npm run dev` | Start Vite dev server only (frontend) |
| `npm run build` | Build frontend for production |
| `npm run preview` | Preview production build |
| `npm run check` | Run svelte-check type checking |
| `npm run tauri dev` | Launch full Tauri app in development mode |
| `npm run tauri build` | Build distributable desktop application |

---

## Suggested Improvements & Next Steps

### High Priority

1. **Implement POE API Client** — The `PoeClient` struct in `client/poe.rs` is a skeleton with only error types defined. Implement actual endpoints to fetch live skill tree data, passive node details, and character information from the Path of Exile 2 API. The `governor` rate limiter is already integrated and ready to use.

2. **Complete Stat Calculations** — `update_selected_nodes` currently returns placeholder stats (zeroed strength/dexterity/intelligence). Implement actual POE2 stat aggregation by reading node stat values from the tree data and summing them based on the selected node set.

3. **Wire Up Build Save/Load** — The home page has UI scaffolding for saved builds (save, load, and delete buttons), but there is no backend integration. Connect the `StorageManager` and `FileCache` to persist and retrieve full `BuildInfo` objects to/from disk.

4. **Display Stats in Sidebar** — The sidebar currently shows "More stats coming soon." Wire the `BuildStats` returned from `update_selected_nodes` into the sidebar to display strength, dexterity, intelligence, and derived statistics in real time.

### Medium Priority

5. **Render Ascendancy Sub-Trees** — Ascendancy data (subclasses, flavour text, nodes) exists in `data.json` but is not yet rendered on the canvas. Add a dedicated ascendancy tree view or overlay that appears when an ascendancy is selected.

6. **Node Search & Filter** — Add the ability to search for specific nodes by name, stat type, or keyword, and highlight matching nodes on the canvas for easy discovery.

7. **Build Sharing & Export** — Allow users to export builds as shareable JSON files or encoded URL strings for community sharing and import.

8. **Undo/Redo** — There is no undo/redo for node selection changes. Implement a history stack to allow users to step backward and forward through their selection changes.

### Lower Priority

9. **Automated Testing** — No tests exist currently. Add Rust unit tests for stat calculation logic, storage operations, and POE client behavior; add frontend component tests for SkillTree interaction, Header state management, and IPC integration.

10. **CI/CD Pipeline** — Set up GitHub Actions workflows for automated building, testing, linting, and release artifact generation across platforms.

11. **Sprite Atlas Optimization** — The SkillTree currently creates individual sprites per node. Using a sprite atlas/spritesheet would reduce draw calls and improve rendering performance for large trees.

12. **Update App Metadata** — `tauri.conf.json` still describes the app as "A Tauri App." Update the description, window title, and application icons to reflect the actual product.

13. **Accessibility & Responsiveness** — The app is currently desktop-only with no keyboard navigation or screen reader support. Consider adding keyboard controls for tree navigation and responsive breakpoints for varying window sizes.

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.



1. Build your character in PoB → File → Save Build → save as .xml
   (or: File → Copy Build Code → paste the XML into tests/fixtures/sources/mybuild.xml)

2. bun run tool:gen-fixtures
   # writes tests/fixtures/mybuild.json with real PoB values

3. Add to src-tauri/tests/pob_parity.rs:
   #[test]
   fn pob_parity_mybuild() { run_fixture("mybuild"); }

4. cargo test --test pob_parity