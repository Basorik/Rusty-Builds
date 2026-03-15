# Data Files

This directory contains game data used by Rusty Builds for Path of Exile 1 build planning. All files are fetched by the `tools/fetch_data.ts` script.

```
bun run tool:fetch-data   # download everything
bun run tool:fetch-pob    # PoB data only
bun run tool:fetch-tree   # official skill tree only
```

> **These files are `.gitignore`d.** Run the fetch script after cloning.

---

## `tree/` — Official Skill Tree

Source: [grindinggear/skilltree-export](https://github.com/grindinggear/skilltree-export) (latest release)

Each release is saved into its own subfolder so multiple versions can coexist. The currently active version is indicated by `active.json`, a symlink that Rust embeds at compile time.

```
tree/
├── active.json          → symlink to {tag}/data.json  (used by Rust)
└── 3.27.0g/
    └── data.json
```

To **switch versions**, update the symlink:

```sh
cd src-tauri/data/tree
ln -sf 3.26.0/data.json active.json
```

Then rebuild the app. Running `bun run tool:fetch-tree` always downloads the latest release and points `active.json` at it automatically.

| File | Description |
|------|-------------|
| `active.json` | Symlink to the currently active `{tag}/data.json`. |
| `{tag}/data.json` | Full skill tree for that patch version (~5.3 MB). |

### `data.json` structure

| Key | Type | Description |
|-----|------|-------------|
| `tree` | `string` | Tree version identifier (e.g. `"Default"`). |
| `classes` | `array[7]` | One entry per class with `name`, `base_str`, `base_dex`, `base_int`, and nested `ascendancies` array. |
| `alternate_ascendancies` | `array[13]` | Bloodline ascendancies — id, name, flavour text, and colour info. |
| `groups` | `dict[748]` | Spatial groups of nodes. Each has `x`, `y`, `orbits`, `nodes`, and optionally `isProxy` / `ascendancyName`. |
| `nodes` | `dict[3281]` | Every passive node keyed by hash. Contains `skill`, `name`, `stats[]`, `group`, `orbit`, `orbitIndex`, connections (`out[]`, `in[]`), and flags like `isKeystone`, `isNotable`, `isAscendancyStart`, `isBloodline`, `ascendancyName`, etc. |
| `extraImages` | `dict[6]` | Background art offsets per class starting area. |
| `jewelSlots` | `array[60]` | Node hashes that accept jewels. |
| `min_x`, `min_y`, `max_x`, `max_y` | `int` | Bounding box of the tree in world coordinates. |
| `constants` | `dict` | Orbit radii, skills-per-orbit, class/attribute mappings. |
| `sprites` | `dict[31]` | Sprite sheet definitions (`background`, `normalActive`, `notableActive`, `keystoneActive`, etc.) with filenames and per-icon coords. |
| `imageZoomLevels` | `array[4]` | Available zoom levels for sprite sheets. |
| `points` | `dict` | `totalPoints` and `ascendancyPoints` available to allocate. |

---

## `pob/` — Path of Building Data

Source: [PathOfBuildingCommunity/PathOfBuilding](https://github.com/pob-community/repoe-fork) (`master` branch, `RePoE/data/`)

These files power all the mod, gem, minion, and crafting calculations.

### Core Game Data

| File | Size | Description |
|------|------|-------------|
| `Misc.json` | 24 KB | Game-wide constants and scaling tables. Top-level keys include `characterConstants` (per-level base stats), `gameConstants` (resist caps, charge values, etc.), `monsterAccuracyTable`, `monsterAilmentThresholdTable`, `mapLevelLifeMult`, `mapLevelBossLifeMult`, and more. |
| `Costs.json` | 1.8 KB | Skill cost definitions — array of 14 entries mapping `Stat` → `Resource` (life, mana, ES, rage, etc.) with a `Divisor`. |
| `Pantheons.json` | 7.4 KB | All 12 Pantheon powers (major + minor gods). Each entry has `isMajorGod` and `souls` listing the upgrades and their stat bonuses. |

### Skill Gems & Stats

| File | Size | Description |
|------|------|-------------|
| `Gems.json` | 425 KB | **810 gems** keyed by metadata path. Each gem has `name`, `baseTypeName`, `tags`, `naturalMaxLevel`, attribute requirements (`reqStr/Dex/Int`), `grantedEffectId`, and full per-level stat progressions. |
| `SkillStatMap.json` | 141 KB | **707 stat IDs** → display-text mappings. Maps internal stat keys like `accuracy_rating_+%` to the text shown in-game. Used to resolve node/mod stats to human-readable form. |
| `FlavourText.json` | 235 KB | 1,337 unique item flavour texts — `id`, `name`, `text`. |

### Monsters & Bosses

| File | Size | Description |
|------|------|-------------|
| `Bosses.json` | 1.7 KB | 22 boss definitions with `armourMult`, `evasionMult`, and `isUber` flag. Keys: Atziri, Shaper, Maven, Sirus, Uber Elder, etc. |
| `BossSkills.json` | 4.8 KB | 10 specific boss attacks (Shaper Ball, Maven Fireball, Eater Beam, etc.) with `DamageType`, `DamageMultipliers`, `DamagePenetrations`, `critChance`, `speed`, and `UberDamageMultiplier`. |
| `Minions.json` | 50 KB | **62 minion types** with full stat blocks: `life`, `damage`, `damageSpread`, `attackTime`, `attackRange`, `accuracy`, `armour`, `energyShield`, resists, and per-level scaling. |
| `Spectres.json` | 212 KB | **260 spectrable monsters** keyed by metadata path. Same stat block format as Minions plus spectre-specific data. |

### Modifier Data

All `Mod*.json` files share a common **modifier schema**: each mod entry has `affix`, `group`, `level`, `modTags`, `statOrder`, `type`, `weightKey`/`weightVal` (spawn weights), and numbered stat lines (`"1"`, `"2"`, …) containing `min`/`max` ranges and the stat ID.

| File | Size | Entries | Description |
|------|------|---------|-------------|
| `ModItem.json` | 4.2 MB | 11,511 | **All item modifiers** — prefixes, suffixes, implicits, eldritch implicits, enchantments, corruptions, etc. The largest mod file. |
| `ModCache.json` | 3.3 MB | 13,003 | Mod display-text → mod-ID lookup cache. Keys are the human-readable stat lines; values are arrays of matching mod IDs. |
| `ModJewelAbyss.json` | 291 KB | 721 | Abyss jewel modifiers. |
| `ModJewelCluster.json` | 284 KB | 557 | Cluster jewel modifiers. |
| `ModMaster.json` | — | 728 | Crafting bench (master) mods. **Note:** this is an array, not a dict. Each entry also has a `types` field for applicable item classes. |
| `ModGraft.json` | 164 KB | 485 | Necropolis graft mods. |
| `ModJewel.json` | 146 KB | 424 | Standard jewel modifiers. |
| `ModVeiled.json` | 106 KB | 257 | Veiled modifiers (Jun/unveil). |
| `ModFoulborn.json` | 102 KB | 341 | Foulborn (mutated unique) mods. |
| `ModFlask.json` | 86 KB | 246 | Flask modifiers (prefixes & suffixes). |
| `ModJewelCharm.json` | 86 KB | 242 | Charm jewel modifiers. |
| `ModTincture.json` | 22 KB | 66 | Tincture modifiers. |
| `ModNecropolis.json` | 18 KB | 51 | Necropolis crafting modifiers. |
| `ModMap.json` | 12 KB | 3 | Map modifiers split into `AffixData`, `Prefix`, and `Suffix` sub-dicts. |
| `ModFoulbornMap.json` | 23 KB | 258 | Foulborn mods indexed by unique item name — values are arrays of applicable mod IDs. |
| `QueryMods.json` | 1.4 MB | 8 | Mod search indices grouped by source (`Explicit`, `Implicit`, `Corrupted`, `Eater`, `Exarch`, `Synthesis`, `Scourge`, `PassiveNode`). Used for filtering/querying mods by category. |

### Crafting & Enchantments

| File | Size | Description |
|------|------|-------------|
| `Crucible.json` | 1.1 MB | **2,491 crucible tree mods** with `nodeLocation`, `nodeType`, and `tier` in addition to standard mod fields. |
| `Essence.json` | 124 KB | 105 essences keyed by metadata path. Each has `name`, `tier`, `type`, and `mods` mapping item slots to granted mod IDs. |
| `BeastCraft.json` | 2.2 KB | 8 beastcrafting recipe mods (Aspect skills: Avian, Cat, Crab, Spider — regular and level-30 variants). |
| `EnchantmentHelmet.json` | — | 274 skill-specific helmet enchantments. Keyed by skill name, values contain `MERCILESS` and `ENDGAME` tiers. |
| `EnchantmentBoots.json` | — | Boot enchantments across `CRUEL`, `MERCILESS`, and `ENDGAME` tiers. |
| `EnchantmentGloves.json` | — | Glove enchantments across `NORMAL`, `CRUEL`, `MERCILESS`, and `ENDGAME` tiers. |
| `EnchantmentBelt.json` | — | Belt enchantments (`DEDICATION` source). |
| `EnchantmentBody.json` | — | Body armour enchantments (`HARVEST`, `HEIST` sources). |
| `EnchantmentWeapon.json` | — | Weapon enchantments (`HARVEST`, `HEIST` sources). |
| `EnchantmentFlask.json` | — | Flask enchantments (`ENKINDLING`, `INSTILLING` orb types). |

### Cluster Jewels & Tattoos

| File | Size | Description |
|------|------|-------------|
| `ClusterJewels.json` | 35 KB | Cluster jewel definitions with `jewels` (Large/Medium/Small sizing), `keystones`, `notableSortOrder`, and `orbitOffsets`. |
| `TattooPassives.json` | 133 KB | Tattoo passive data with `groups` and `nodes` sub-dicts — mirrors the skill tree format for tattoo-replaced passives. |

### Rares

| File | Size | Description |
|------|------|-------------|
| `Rares.json` | 38 KB | 205 rare item templates (array). Used for rare item generation/simulation. |

---

## Usage in Rust

The active tree version is embedded at compile time via:

```rust
const TREE_JSON: &str = include_str!("../data/tree/active.json");
```

PoB data files are intended to be loaded on demand for stat resolution, mod lookups, and DPS calculations. The common pattern is:

```rust
let data: HashMap<String, serde_json::Value> = serde_json::from_str(raw)?;
```

## Usage in TypeScript

Import the JSON directly or fetch at runtime:

```ts
const tree = await fetch('/data/tree/data.json').then(r => r.json());
const gems = await fetch('/data/pob/Gems.json').then(r => r.json());
```
