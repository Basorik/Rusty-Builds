# RePoE Data Migration Plan

> **Goal**: Replace PoB-processed data with RePoE (GGG-direct) data as the primary source everywhere possible. Eliminate display-text parsing in the calculation pipeline. Keep PoB data only where no RePoE equivalent exists.

---

## 1. Data Source Inventory

### What we keep from each source

| Source | Files Used | Why |
|---|---|---|
| **RePoE** (`data/repoe/`) | `base_items.json`, `mods.json`, `gems.json`, `stats.json`, `stat_translations.json`, `characters.json`, `item_classes.json`, `active_skill_types.json`, `cluster_jewels.json`, `essences.json`, `fossils.json`, `crafting_bench_options.json`, `passive_skill_trees/Default.json` | GGG-direct data. Internal stat IDs, full mod definitions with spawn weights, gem per-level stats, class base stats |
| **PoB** (`data/pob/`) | `SkillStatMap.json`, `Uniques/*.json` | SkillStatMap: only source of stat ID → calc variable semantic mapping. Uniques: only source of unique item mod text + base type association |
| **GGG tree** (`data/tree/`) | `3.27.0g/data.json` | Visual layout, connections, orbits, group positions — needed for SkillTree.svelte renderer |

### What we stop using

| Current Source | Replacement |
|---|---|
| `data/pob/Bases/*.json` | `data/repoe/base_items.json` + `data/repoe/item_classes.json` |
| `data/pob/Gems.json` | `data/repoe/gems.json` |
| `data/pob/Skills/*.json` | `data/repoe/gems.json` (active_skill + per_level + static + support_gem all in one file) |
| `data/pob/ModItem.json`, `ModJewel.json`, etc. | `data/repoe/mods.json` (all mod pools in one file with spawn_weights) |
| `data/pob/StatDescriptions/*.json` | `data/repoe/stat_translations.json` + `data/repoe/stat_translations/*.json` |
| `data/pob/ClusterJewels.json` | `data/repoe/cluster_jewels.json` + `cluster_jewel_notables.json` |
| `data/pob/Essence.json` | `data/repoe/essences.json` |
| Passive tree node `stats: Vec<String>` (display text) | `data/repoe/passive_skill_trees/Default.json` passives with `stats: {stat_id: value}` |

---

## 2. The Key Insight: Eliminating Text Parsing

### Current flow (display text based)
```
Passive node stats: ["+10 to Strength", "5% increased Attack Speed"]
    → regex → template key → hand-coded handler → Modifier
    ❌ Fragile, incomplete (30 templates), breaks on GGG rewording
```

### New flow (stat ID based)
```
RePoE passive: stats: {"base_strength": 10, "attack_speed_+%": 5}
    → SkillStatMap lookup: "attack_speed_+%" → {name: "Speed", type: "INC", flags: 1}
    → StatId::from_name("Speed") → StatId::Speed
    → Modifier { stat: Speed, mod_type: Inc, value: 5.0, flags: ATTACK }
    ✅ Data-driven, complete, never breaks on text changes
```

This eliminates `parse_display_text()` for the core calculation pipeline entirely. Text parsing only remains as a fallback for stats not in SkillStatMap (rare edge cases) and for tooltip rendering.

---

## 3. New Type Definitions

### 3a. SkillStatMap loader (`data/skill_stat_map.rs` — new file)

The bridge between internal stat IDs and our `StatId` enum + modifier semantics.

```rust
/// One mapping entry from SkillStatMap.json
/// Example: "attack_speed_+%" → {name: "Speed", type: "INC", flags: 1, keywordFlags: 0}
pub struct StatMapping {
    pub stat_id: StatId,          // Resolved from "name" field
    pub mod_type: ModType,        // Resolved from "type" field (BASE/INC/MORE/FLAG/...)
    pub flags: ModFlag,           // From "flags" integer (bitfield)
    pub keyword_flags: KeywordFlag, // From "keywordFlags" integer
    pub tags: SmallVec<[ModTag; 2]>, // From optional "1", "2" condition/multiplier objects
    pub div: Option<f64>,         // Some entries divide the value (e.g., "div": 10)
}

/// Global lookup: internal_stat_id (e.g. "attack_speed_+%") → Vec<StatMapping>
/// One stat ID can produce multiple modifiers (array in JSON).
pub struct SkillStatMapDB {
    map: FxHashMap<String, Vec<StatMapping>>,
}

impl SkillStatMapDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Convert an internal stat ID + value into Modifier(s).
    /// Returns empty Vec if the stat ID is not in SkillStatMap (fallback to text parser).
    pub fn resolve(&self, internal_stat_id: &str, value: f64, source: SourceId) -> Vec<Modifier>;
}
```

### 3b. RePoE Passive Tree (`data/repoe_tree.rs` — new file)

Supplements the GGG visual tree with stat ID data.

```rust
/// Stat data for a single passive node from RePoE passive_skill_trees/Default.json
pub struct PassiveStatData {
    pub stats: FxHashMap<String, f64>,  // internal_stat_id → value
    pub is_keystone: bool,
    pub is_notable: bool,
    pub is_jewel_socket: bool,
    pub is_ascendancy_starting_node: bool,
    pub ascendancy: Option<String>,
    pub skill_points: u32,
    pub name: String,
}

/// Keyed by the passive "hash" (same numeric ID as the GGG tree node skill_id)
pub struct RepoePassiveTree {
    pub passives: FxHashMap<u32, PassiveStatData>,
}

impl RepoePassiveTree {
    pub fn load(path: &Path) -> Result<Self, DataError>;
}
```

### 3c. Base Items from RePoE (`data/bases.rs` — replace stub)

```rust
/// Release state enum — shared by base_items and gems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseState {
    Released,
    UniqueOnly,
    Unreleased,
    Legacy,
}

/// A single base item from RePoE base_items.json
pub struct BaseItem {
    pub metadata_id: String,        // "Metadata/Items/Armours/BodyArmours/BodyStr15"
    pub name: String,               // "Astral Plate"
    pub item_class: String,         // "Body Armour"
    pub domain: String,             // "item"
    pub drop_level: u32,
    pub tags: Vec<String>,          // ["str_armour", "body_armour", "armour", "default"]
    pub implicit_mod_ids: Vec<String>, // ["AllResistancesImplicitArmour1"] → links to mods.json
    pub requirements: Option<ItemRequirements>,
    pub properties: BaseItemProperties,
    pub release_state: ReleaseState,  // Enum: Released, UniqueOnly, Unreleased, Legacy
    pub inventory_width: u32,
    pub inventory_height: u32,
    pub visual_identity: VisualIdentity, // Required: {dds_file, id}
    pub inherits_from: Option<String>,   // Inheritance chain (e.g., "Metadata/Items/Armours/AbstractArmour")
    pub grants_buff: Option<GrantsBuff>, // Flasks/jewels that grant buff effects
    pub skills_granted: Option<Vec<String>>, // Skill gem metadata IDs granted by the item
}

pub struct VisualIdentity {
    pub dds_file: String,
    pub id: String,
}

pub struct GrantsBuff {
    pub id: String,
    pub stats: FxHashMap<String, i32>,  // stat_id → value
}

pub struct ItemRequirements {
    pub level: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
}

/// Flattened from the deeply-nested RePoE properties object.
/// Only the fields relevant to equipment.
pub struct BaseItemProperties {
    // Defence
    pub armour: Option<MinMax>,
    pub evasion: Option<MinMax>,
    pub energy_shield: Option<MinMax>,
    pub block: Option<u32>,
    pub movement_speed: Option<i32>,     // -5 for heavy armour

    // Weapon
    pub attack_time_ms: Option<u32>,     // Convert: 1000.0 / attack_time_ms = attacks/sec
    pub critical_strike_chance: Option<u32>, // Per-mille: divide by 100 for %
    pub physical_damage_min: Option<u32>,
    pub physical_damage_max: Option<u32>,
    pub range: Option<u32>,

    // Flask
    pub charges_max: Option<u32>,
    pub charges_per_use: Option<u32>,
    pub duration: Option<u32>,
    pub life_per_use: Option<u32>,
    pub mana_per_use: Option<u32>,

    // Misc (from schema — for non-equipment items)
    pub stack_size: Option<u32>,
    pub stack_size_currency_tab: Option<u32>,
    pub full_stack_turns_into: Option<String>,
    pub description: Option<String>,
    pub cooldown_ms: Option<u32>,
}

pub struct MinMax {
    pub min: u32,
    pub max: u32,
}

/// Derived fields not in RePoE but computable from tags/item_class
impl BaseItem {
    /// Socket limit derived from item_class (6 for body/2H, 4 for 1H/helm/boots/gloves, etc.)
    pub fn socket_limit(&self) -> u32;

    /// Defence subtype from tags: "str_armour" → Armour, "dex_armour" → Evasion, etc.
    pub fn defence_type(&self) -> Option<DefenceType>;

    /// Weapon attacks per second
    pub fn attack_rate(&self) -> Option<f64>;

    /// Crit chance as percentage
    pub fn crit_chance_pct(&self) -> Option<f64>;

    /// Is this an equippable item?
    pub fn is_equipment(&self) -> bool;
}

pub enum DefenceType {
    Armour,
    Evasion,
    EnergyShield,
    ArmourEvasion,
    ArmourEnergyShield,
    EvasionEnergyShield,
    ArmourEvasionEnergyShield,
    Ward,
}

/// Index for fast base item lookup by name
pub struct BaseItemDB {
    by_name: FxHashMap<String, BaseItem>,        // "Astral Plate" → BaseItem
    by_metadata: FxHashMap<String, String>,       // metadata_id → name (for cross-referencing)
}

impl BaseItemDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;
}
```

### 3d. Mod Definitions from RePoE (`data/mods.rs` — replace stub)

```rust
/// A single mod definition from RePoE mods.json
/// Example: "IncreasedLife2" → prefix, ilvl 11, stats: [{base_maximum_life, 25..39}]
pub struct ModDefinition {
    pub id: String,                    // "IncreasedLife2"
    pub name: String,                  // "Sanguine" (affix name)
    pub domain: String,                // "item", "flask", "jewel", "abyss_jewel", ...
    pub generation_type: String,       // "prefix", "suffix", "unique", "corrupted", ...
    pub required_level: u32,
    pub groups: Vec<String>,           // ["IncreasedLife"] — mod group for mutual exclusion
    pub implicit_tags: Vec<String>,    // ["resource", "life"] — for weighting
    pub spawn_weights: Vec<SpawnWeight>, // tag → weight (0 = blocked)
    pub generation_weights: Vec<SpawnWeight>, // extra weighting (fossils, etc.)
    pub stats: Vec<ModStat>,           // Internal stat IDs + value ranges
    pub text: Option<String>,          // Display text (for tooltips only) — optional per schema
    pub is_essence_only: bool,
    pub adds_tags: Vec<String>,        // Tags added to item when this mod is present
    pub grants_effects: Vec<GrantsEffect>, // Mods that grant skills (e.g., unique mods granting auras)
    pub gold_value: Option<f64>,       // Trade value (low priority)
    #[serde(rename = "type")]
    pub mod_type_name: String,         // Mod type string
}

pub struct GrantsEffect {
    pub granted_effect_id: String,
    pub level: u32,
}

pub struct SpawnWeight {
    pub tag: String,                   // e.g. "weapon", "body_armour", "default"
    pub weight: u32,                   // 0 = cannot spawn, 1000 = normal
}

pub struct ModStat {
    pub id: String,                    // Internal stat ID: "base_maximum_life"
    pub min: i32,                      // Minimum roll value
    pub max: i32,                      // Maximum roll value
}

/// In-memory mod database, indexed multiple ways for different lookup patterns
pub struct ModDefinitionDB {
    by_id: FxHashMap<String, ModDefinition>,

    /// Grouped by generation_type for mod pool queries
    /// e.g., all "prefix" mods, all "suffix" mods
    prefixes: Vec<String>,             // IDs of all prefix mods
    suffixes: Vec<String>,             // IDs of all suffix mods
}

impl ModDefinitionDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Get a mod definition by its ID (used for resolving base item implicits)
    pub fn get(&self, mod_id: &str) -> Option<&ModDefinition>;

    /// Find all mods that can spawn on an item with given tags at given level
    pub fn available_mods(&self, tags: &[String], item_level: u32, gen_type: &str) -> Vec<&ModDefinition>;
}
```

### 3e. Gems from RePoE (`data/gems.rs` — replace current)

```rust
/// Color enum for gems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GemColor {
    #[serde(rename = "r")]
    Red,
    #[serde(rename = "g")]
    Green,
    #[serde(rename = "b")]
    Blue,
    #[serde(rename = "w")]
    White,
}

/// Cooldown bypass type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CooldownBypassType {
    ExpendFrenzyCharge,
    ExpendPowerCharge,
    ExpendEnduranceCharge,
}

/// Stat type enum (used in gem stat entries)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatType {
    Float,
    Constant,
    Additional,
    Implicit,
    Flag,
}

/// A gem from RePoE gems.json — replaces both GemItem and GrantedEffect
pub struct Gem {
    pub display_name: Option<String>,  // Optional per schema
    pub is_support: bool,
    pub color: GemColor,
    pub cast_time_ms: Option<u32>,     // null for support gems
    pub tags: Option<Vec<String>>,     // Optional per schema

    // Base item metadata (optional per schema)
    pub base_item: Option<GemBaseItem>,

    // Required — which translation file to use for this gem's stats
    pub stat_translation_file: String,

    // Active skill data (None for support gems)
    pub active_skill: Option<ActiveSkillData>,

    // Support gem data (None for active skills)
    pub support_gem: Option<SupportGemData>,

    // Static stats (level-independent) — required
    pub static_data: GemStaticData,    // JSON field: "static"

    // Per-level data — required, keyed by level string
    pub per_level: FxHashMap<String, GemLevelData>,

    // Optional fields
    pub secondary_granted_effect: Option<String>, // Vaal gems have a secondary effect
    pub discriminator: Option<String>,  // "alt_x" or "alt_y" — alternate quality
    pub tooltip_order: Option<Vec<String>>,
    pub quest_reward: Option<QuestReward>,
}

pub struct GemBaseItem {
    pub display_name: String,
    pub id: String,                     // Metadata path
    pub max_level: u32,
    pub release_state: ReleaseState,
    pub experience_type: String,
}

pub struct QuestReward {
    pub act: u32,
    pub classes: Vec<String>,
    pub quest: String,
}

pub struct ActiveSkillData {
    pub id: String,                     // "fireball"
    pub description: String,
    pub display_name: String,
    pub types: Option<Vec<String>>,     // Optional per schema
    pub stat_conversions: FxHashMap<String, String>,
    pub weapon_restrictions: Vec<String>,
    pub is_manually_casted: bool,
    pub is_skill_totem: bool,           // Required per schema
    pub skill_totem_life_multiplier: Option<f64>,
    pub minion_types: Option<Vec<String>>,
}

pub struct SupportGemData {
    pub allowed_types: Option<Vec<String>>,  // Optional per schema
    pub excluded_types: Option<Vec<String>>,
    pub added_types: Option<Vec<String>>,
    pub supports_gems_only: bool,
    pub letter: String,                 // Support icon letter
    pub support_name: Option<String>,
    pub support_text: Option<String>,
    pub added_minion_types: Option<Vec<String>>,
}

pub struct GemStaticData {
    pub quality_stats: Vec<QualityStatEntry>, // Required per schema
    pub crit_chance: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub stats: Option<Vec<Option<GemStatEntry>>>, // Optional; items can be null
    pub attack_speed_multiplier: Option<u32>,
    pub stored_uses: Option<u32>,       // Traps, mines
    pub damage_multiplier: Option<u32>,
    pub damage_effectiveness: Option<u32>,
    pub cooldown: Option<u32>,
    pub cooldown_bypass_type: Option<CooldownBypassType>,
    pub reservations: Option<StaticReservations>,
    pub vaal: Option<VaalData>,
    pub experience: Option<u32>,
    pub stat_text: Option<FxHashMap<String, String>>,
    pub stat_requirements: Option<GemStatRequirements>,
    pub required_level: Option<u32>,
    pub costs: Option<GemCosts>,
}

pub struct StaticReservations {
    pub mana_percent: Option<f64>,
    pub life_percent: Option<f64>,
    pub mana_flat: Option<u32>,
}

pub struct VaalData {
    pub souls: u32,
    pub stored_uses: u32,
}

/// A stat entry — all fields optional per schema. Items in the array can also be null.
pub struct GemStatEntry {
    pub id: Option<String>,            // Internal stat ID (e.g., "base_chance_to_ignite_%")
    pub value: Option<i64>,            // Integer value
    #[serde(rename = "type")]
    pub stat_type: Option<StatType>,
}

pub struct QualityStatEntry {
    pub stat: String,                   // Quality stat key
    pub stats: FxHashMap<String, u32>,  // stat_id → value per quality point
}

pub struct GemLevelData {
    pub required_level: Option<u32>,    // All per-level fields optional per schema
    pub stat_requirements: Option<GemStatRequirements>,
    pub costs: Option<GemCosts>,
    pub stats: Option<Vec<Option<GemStatEntry>>>, // Positional; items can be null
    pub damage_effectiveness: Option<u32>,
    pub damage_multiplier: Option<u32>,
    pub cost_multiplier: Option<u32>,
    pub cooldown: Option<u32>,
    pub stored_uses: Option<u32>,
    pub reservations: Option<PerLevelReservations>,
    pub experience: Option<u32>,
    pub stat_text: Option<FxHashMap<String, String>>,
}

pub struct PerLevelReservations {
    pub mana_flat: Option<u32>,
    pub mana_percent: Option<f64>,
}

pub struct GemStatRequirements {
    pub strength: Option<u32>,
    pub dexterity: Option<u32>,
    pub intelligence: Option<u32>,
}

pub struct GemCosts {
    pub mana: Option<u32>,
    pub life: Option<u32>,
    pub mana_per_minute: Option<u32>,
    pub mana_percent: Option<u32>,
    pub mana_percent_per_minute: Option<u32>,
    pub es: Option<u32>,
}

/// Gem database indexed by display name
pub struct GemDB {
    by_name: FxHashMap<String, Gem>,
    by_metadata_id: FxHashMap<String, String>, // metadata_id → display_name
}

impl GemDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;
}
```

### 3f. Stat Translations (`data/stat_translations.rs` — new file)

For tooltip/UI display only, not calculations.

```rust
/// Format enum for stat value display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatFormat {
    #[serde(rename = "ignore")]
    Ignore,
    #[serde(rename = "#")]
    Plain,
    #[serde(rename = "+#")]
    PlusMinus,
}

/// Reverse lookup: given internal stat IDs + values, produce display text.
pub struct StatTranslationDB {
    /// Keyed by sorted stat ID tuple for multi-stat translations
    entries: Vec<StatTranslationEntry>,
    /// Fast lookup: single stat_id → entry indices
    single_stat_index: FxHashMap<String, Vec<usize>>,
}

pub struct StatTranslationEntry {
    pub ids: Vec<String>,              // ["base_maximum_life"] or ["min_dmg", "max_dmg"]
    pub english: Option<Vec<TranslationVariant>>,  // Optional per schema
    pub hidden: Option<bool>,          // Hidden stats don't show in tooltips
    pub trade_stats: Option<Vec<TradeStat>>, // For trade query generation (Phase 8)
}

pub struct TranslationVariant {
    pub string: String,                // "{0} to maximum Life"
    pub format: Vec<StatFormat>,       // [PlusMinus] — how to format each value
    pub condition: Vec<ValueCondition>, // min/max/negated per stat
    pub index_handlers: Vec<Vec<String>>, // ["negate", "divide_by_one_hundred", etc.]
    pub reminder_text: Option<String>, // Grey italic text shown in-game
    pub is_markup: Option<bool>,       // Whether string contains markup tags
}

pub struct ValueCondition {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub negated: Option<bool>,
}

pub struct TradeStat {
    pub id: String,
    pub text: String,
    #[serde(rename = "type")]
    pub trade_type: String,
    pub option: Option<TradeStatOption>,
}

pub struct TradeStatOption {
    pub options: Vec<TradeOptionElement>,
}

pub struct TradeOptionElement {
    pub id: u32,
    pub text: String,
}

impl StatTranslationDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Render a stat ID + value to display text (for tooltips)
    pub fn translate(&self, stat_id: &str, value: f64) -> Option<String>;

    /// Render multiple stat IDs + values (for multi-stat translations like damage ranges)
    pub fn translate_multi(&self, stats: &[(&str, f64)]) -> Option<String>;
}
```

### 3g. Characters (`data/characters.rs` — new file)

```rust
/// Unarmed attack stats per character class
pub struct UnarmedStats {
    pub attack_time: u32,
    pub min_physical_damage: u32,
    pub max_physical_damage: u32,
    pub range: u32,
}

/// Class base stats from RePoE characters.json
/// Root type is Vec (array), not object — build FxHashMap during loading
pub struct CharacterClass {
    pub name: String,           // "Marauder"
    pub integer_id: u32,
    pub metadata_id: String,    // "Metadata/Characters/Marauder"
    pub description: Option<String>,
    pub base_strength: u32,
    pub base_dexterity: u32,
    pub base_intelligence: u32,
    pub base_life: u32,         // Always 38 in PoE1 but good to have from data
    pub base_mana: u32,         // Always 34
    pub unarmed: UnarmedStats,  // Required per schema — unarmed attack properties
}

/// Loaded once at startup
pub struct CharacterDB {
    by_name: FxHashMap<String, CharacterClass>,
}

impl CharacterDB {
    /// Loads from Vec<CharacterClass> JSON array, indexes by name
    pub fn load(path: &Path) -> Result<Self, DataError>;
}
```

### 3h. Stat Metadata (`data/stats_info.rs` — new file)

Discovered from `stats.json` schema. Critical for Phase 4 — determines local vs global stat application.

```rust
/// Stat metadata from RePoE stats.json
pub struct StatInfo {
    pub is_local: bool,           // Local stats apply to the item, not the character
    pub is_aliased: bool,
    pub alias: StatAlias,
}

pub struct StatAlias {
    pub when_in_main_hand: Option<String>,  // Stat ID alias when in main hand
    pub when_in_off_hand: Option<String>,   // Stat ID alias when in off hand
}

/// Loaded once; queried during item stat resolution
pub struct StatsDB {
    stats: FxHashMap<String, StatInfo>,
}

impl StatsDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Check if a stat is local to the item (e.g., +% Phys Damage on a weapon)
    pub fn is_local(&self, stat_id: &str) -> bool;

    /// Get the aliased stat ID for a given weapon slot (dual-wielding)
    pub fn alias_for_slot(&self, stat_id: &str, slot: WeaponSlot) -> Option<&str>;
}
```

### 3i. Item Classes (`data/item_classes.rs` — new file)

Discovered from `item_classes.json` schema. Maps item class IDs to influence tags and categories.

```rust
/// Item class metadata from RePoE item_classes.json
pub struct ItemClassInfo {
    pub name: String,
    pub category: Option<String>,
    pub category_id: Option<String>,
    pub influence_tags: Option<Vec<String>>, // Shaper/Elder/etc. crafting tags
}

pub struct ItemClassDB {
    classes: FxHashMap<String, ItemClassInfo>,
}

impl ItemClassDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Get influence tags for an item class (needed for shaper/elder crafting)
    pub fn influence_tags(&self, item_class: &str) -> Option<&[String]>;
}
```

---

## 4. Updated GameData Structure

```rust
pub struct GameData {
    // Visual tree (still from GGG tree export — needed for frontend rendering)
    pub tree: PassiveTree,

    // RePoE stat data for passives (internal stat IDs, no display text)
    pub repoe_passives: RepoePassiveTree,

    // Stat ID → modifier semantics (from PoB — only PoB dependency)
    pub skill_stat_map: SkillStatMapDB,

    // Stat metadata — is_local, aliases (from RePoE stats.json)
    pub stats_info: StatsDB,

    // Base items (from RePoE)
    pub base_items: BaseItemDB,

    // Item class metadata — influence tags, categories (from RePoE)
    pub item_classes: ItemClassDB,

    // Mod definitions (from RePoE)
    pub mod_defs: ModDefinitionDB,

    // Gems — both active skills and supports (from RePoE)
    pub gems: GemDB,

    // Class base stats + unarmed (from RePoE)
    pub characters: CharacterDB,

    // Stat translations — for tooltips only (from RePoE)
    pub stat_translations: StatTranslationDB,

    // Internal bookkeeping
    pub source_names: Vec<String>,
}
```

---

## 5. Changes to Existing Phases

### Phase 1 Changes: Foundation & Data Layer

#### 5.1a — GameData loading (`data/mod.rs`)

**Current**: Loads `data/tree/{version}/data.json`, `data/pob/Gems.json`, `data/pob/Skills/*.json`

**New**: Load from multiple sources in `GameData::load_from_dir()`:

```
data/tree/{version}/data.json          → tree (PassiveTree — unchanged)
data/repoe/passive_skill_trees/Default.json → repoe_passives (RepoePassiveTree)
data/pob/SkillStatMap.json             → skill_stat_map (SkillStatMapDB)
data/repoe/stats.json                  → stats_info (StatsDB)
data/repoe/base_items.json             → base_items (BaseItemDB)
data/repoe/item_classes.json           → item_classes (ItemClassDB)
data/repoe/mods.json                   → mod_defs (ModDefinitionDB)
data/repoe/gems.json                   → gems (GemDB)
data/repoe/characters.json             → characters (CharacterDB)
data/repoe/stat_translations.json      → stat_translations (StatTranslationDB)
```

**Migration steps**:
1. Create `data/skill_stat_map.rs` — `SkillStatMapDB` loader + `resolve()` method
2. Create `data/repoe_tree.rs` — `RepoePassiveTree` loader
3. Create `data/characters.rs` — `CharacterDB` loader
4. Create `data/stats_info.rs` — `StatsDB` loader (is_local, aliases)
5. Create `data/item_classes.rs` — `ItemClassDB` loader (influence tags)
6. Update `data/mod.rs` — add new fields to `GameData`, update `load_from_dir()`
7. Update `load_tree_version` command — also reload `repoe_passives` when tree version changes

#### 5.1b — Passive tree stat handling

**Current**: `PassiveNode.stats: Vec<String>` contains display text, parsed by `parse_display_text()` on every node selection change.

**New**: Look up the node's hash in `repoe_passives` → get `stats: {stat_id: value}` → resolve through `SkillStatMapDB` → `Modifier`. No text parsing.

**Migration**: The GGG tree `PassiveNode.stats` field stays (needed for tooltip display), but calculations never read it. All calculation reads go through `repoe_passives`.

### Phase 2 Changes: Modifier System

#### 5.2a — `ModDBLayers::rebuild_tree()` (`modifier/mod_db.rs`)

**Current**:
```rust
for stat_text in &node.stats {
    let mods = parser::parse_display_text(stat_text, source);
    // ...
}
```

**New**:
```rust
// Look up node's internal stat data from RePoE passive tree
if let Some(passive_data) = game.repoe_passives.passives.get(&node_id) {
    for (stat_id, value) in &passive_data.stats {
        let mods = game.skill_stat_map.resolve(stat_id, *value, source);
        for m in mods {
            tree_db.add(m);
        }
    }
}
```

**Impact**: Eliminates the `parse_display_text()` call in the hot path. The template_map and regex are no longer needed for tree calculations.

#### 5.2b — `ModDBLayers::rebuild_class()` (`modifier/mod_db.rs`)

**Current**: Reads `ClassData.base_str/base_dex/base_int` from the GGG tree's `classes` map.

**New**: Reads from `CharacterDB` (loaded from RePoE `characters.json`). The data is identical but now comes from a cleaner source.

```rust
if let Some(char_class) = game.characters.by_name.get(class_name) {
    class_db.add(simple_mod(StatId::Strength, ModType::Base, char_class.base_strength as f64, source));
    class_db.add(simple_mod(StatId::Dexterity, ModType::Base, char_class.base_dexterity as f64, source));
    class_db.add(simple_mod(StatId::Intelligence, ModType::Base, char_class.base_intelligence as f64, source));
}
```

#### 5.2c — `parse_display_text()` role change (`modifier/parser.rs`)

**Current**: Primary calculation path for all stat text → Modifier conversion.

**New**: Demoted to **fallback only**. Used for:
1. Stats not found in SkillStatMap (rare — mostly display-only stats)
2. Unique item mod text (PoB Uniques provide display text, not stat IDs)
3. Tooltip generation (stat_translations handles this better, but text parser can be a backup)

The template_map stays but is no longer on the critical path. It can be expanded lazily as needed for unique items in Phase 4.

#### 5.2d — `StatId` enum (`data/stat_id.rs`)

**Current**: 320 variants generated from SkillStatMap.json calc variable names. Each variant maps to a "calc variable" (e.g., `Life`, `Speed`, `Accuracy`).

**New**: No change needed to the enum itself. The `SkillStatMapDB` bridges internal stat IDs (e.g., `"attack_speed_+%"`) to these `StatId` variants (e.g., `StatId::Speed`). The enum remains the authoritative key for ModDB queries.

### Phase 3 Changes: Skill & Gem System

#### 5.3a — Gem loading (`data/gems.rs`)

**Current**: Loads from two separate PoB files:
- `data/pob/Gems.json` → `GemItem` (name, color, requirements, tags)
- `data/pob/Skills/*.json` → `GrantedEffect` (per-level stats, support compatibility, etc.)
- Links via `GemItem.granted_effect_id → GrantedEffect.name`

**New**: Single load from `data/repoe/gems.json` → `Gem` struct. Everything is in one file:
- Gem metadata (name, color, tags, requirements)
- Active skill data (types, stat_conversions, weapon_restrictions)
- Support data (allowed_types, excluded_types)
- Static stats with **internal stat IDs** (not display text)
- Per-level stats with values that align positionally with static stat IDs
- Quality stats

**Key advantage**: RePoE's `static.stats` array gives you the internal stat IDs directly:
```json
{"id": "base_chance_to_ignite_%", "value": 25, "type": "constant"}
```
These resolve through `SkillStatMapDB` without any text parsing.

#### 5.3b — Gem stat compilation (`data/skills.rs`)

**Current**: `build_skill_instance_stats()` uses positional indices from PoB's `GrantedEffect.levels[].stat_values` map, cross-referenced with `GrantedEffect.stats` (a list of stat name strings that map to calc variables).

**New**: RePoE uses the same positional system — `per_level[level].stats[]` aligns with `static.stats[]` which contains the internal stat IDs. The compilation function changes to:

```rust
/// Build stat values at a given level/quality
pub fn compile_gem_stats(gem: &Gem, level: u32, quality: u32) -> Vec<(String, f64)> {
    // For each stat in static.stats where the ID is known:
    //   1. Get the per-level value from per_level[level].stats[index]
    //   2. Apply quality bonus if applicable
    //   3. Return (internal_stat_id, computed_value)
    //
    // Then resolve each (stat_id, value) through SkillStatMapDB → Modifier
}
```

#### 5.3c — Support compatibility (`data/skills.rs`)

**Current**: Uses PoB's postfix boolean expressions in `require_skill_types`/`exclude_skill_types` with numeric skill type constants.

**New**: RePoE uses string type names directly:
```json
"allowed_types": ["Attack", "ThresholdJewelRangedAttack"],
"excluded_types": ["Channel"]
```

Map these to the existing skill_type constants using `active_skill_types.json` (which is an ordered array where index+1 = skill_type_id, or use the string names directly for a cleaner API).

---

## 6. Phase 4 Impact: Item System

Phase 4 benefits the most from this migration. With RePoE:

### Base items
- Load from `base_items.json` instead of PoB `Bases/*.json`
- No need for 22 separate category files — one file has everything
- Implicit mod IDs link directly to `mods.json` → internal stat IDs → `SkillStatMapDB`

### Mod pools
- `mods.json` has all mod tiers with `spawn_weights` per tag
- `generation_type` distinguishes prefix/suffix/corrupted/veiled/etc.
- No need to merge PoB's `ModItem.json` + `ModJewel.json` + `ModFlask.json` etc.

### Item implicits (zero text parsing)
```rust
fn resolve_implicits(base: &BaseItem, mod_db: &ModDefinitionDB, ssm: &SkillStatMapDB) -> Vec<Modifier> {
    let mut result = Vec::new();
    for mod_id in &base.implicit_mod_ids {
        if let Some(mod_def) = mod_db.get(mod_id) {
            for stat in &mod_def.stats {
                // Use midpoint of range for default value, or a specific roll
                let value = (stat.min + stat.max) as f64 / 2.0;
                let mods = ssm.resolve(&stat.id, value, source);
                result.extend(mods);
            }
        }
    }
    result
}
```

### Unique items
**Still requires PoB data** — RePoE's `uniques.json` only has name/art/item_class metadata, without mod text or stat values. Continue using `data/pob/Uniques/*.json` for unique item definitions.

---

## 7. Implementation Order

### Step 1: SkillStatMapDB (foundation for everything else)
- **New file**: `src-tauri/src/data/skill_stat_map.rs`
- Load `SkillStatMap.json`, parse each entry's `name`/`type`/`flags`/`keywordFlags`/conditions
- Implement `resolve(stat_id, value, source) -> Vec<Modifier>`
- **Test**: Verify `resolve("attack_speed_+%", 5.0, src)` → `Modifier { stat: Speed, mod_type: Inc, value: 5.0, flags: ATTACK }`
- **Test**: Verify conditional entries like `"accuracy_rating_+%_when_on_low_life"` produce `ModTag::Condition`

### Step 2: RepoePassiveTree + CharacterDB
- **New files**: `data/repoe_tree.rs`, `data/characters.rs`
- Load `passive_skill_trees/Default.json` → `RepoePassiveTree`
- Load `characters.json` → `CharacterDB`
- **Test**: Verify known node hashes map to correct stat IDs and values

### Step 3: Update GameData + rebuild_tree
- Add `repoe_passives`, `skill_stat_map`, `characters` to `GameData`
- Update `GameData::load_from_dir()`
- Rewrite `ModDBLayers::rebuild_tree()` to use RePoE stats + SkillStatMapDB
- Rewrite `ModDBLayers::rebuild_class()` to use CharacterDB
- **Test**: Verify BuildStats (life, mana, str, dex, int) produce identical values as before

### Step 4: Gems migration
- **Rewrite**: `data/gems.rs` to load from RePoE `gems.json`
- **Update**: `data/skills.rs` gem stat compilation to use new Gem struct + RePoE stat IDs
- **Update**: Support compatibility to use string types instead of numeric postfix expressions
- **Update**: lib.rs commands that reference GemItem/GrantedEffect
- **Test**: Verify gem stats at various levels match PoB values

### Step 5: Base items + Mod definitions (Phase 4 foundation)
- **Implement**: `data/bases.rs` — `BaseItemDB` from RePoE `base_items.json`
- **Implement**: `data/mods.rs` — `ModDefinitionDB` from RePoE `mods.json`
- **Test**: Verify implicit resolution chain: base item → mod_id → mods.json → stat_ids → SkillStatMapDB → Modifiers

### Step 6: Stat translations (tooltip support)
- **New file**: `data/stat_translations.rs`
- Load `stat_translations.json` for display text rendering
- Used by frontend tooltips and debug views, not calculations

### Step 7: Cleanup
- Remove PoB Gems.json / Skills/*.json loading code
- Remove PoB Bases/*.json loading code
- Demote `parse_display_text()` to fallback/utility status
- Update copilot-instructions.md with new data source strategy

---

## 8. Validation Strategy

At each step, verify that calculated values haven't changed:

1. **Snapshot test**: Before migration, capture BuildStats for a known set of selected nodes
2. **After each step**: Run the same node set, compare BuildStats
3. **Cross-reference**: For gem stats, compare RePoE-derived values against PoB Skills/*.json values
4. **Edge cases to test**:
   - Nodes with stats not in SkillStatMap (should fall back gracefully)
   - Multi-stat passive nodes (e.g., "+10 to Str and Dex" → two internal stat IDs)
   - Ascendancy nodes
   - Keystone nodes (often FLAG-type stats)
   - Class base stats for all 7 classes
   - Support gem compatibility (ensure string-based type matching gives same results)

---

## 9. Files Modified Summary

| File | Action | Description |
|---|---|---|
| `data/skill_stat_map.rs` | **New** | SkillStatMap loader + resolve() |
| `data/repoe_tree.rs` | **New** | RePoE passive tree stat data loader |
| `data/characters.rs` | **New** | RePoE character class base stats + unarmed |
| `data/stats_info.rs` | **New** | RePoE stats.json loader (is_local, aliases) |
| `data/item_classes.rs` | **New** | RePoE item_classes.json loader (influence tags) |
| `data/stat_translations.rs` | **New** | Stat translation DB for tooltips |
| `data/bases.rs` | **Rewrite** | RePoE base_items.json loader |
| `data/mods.rs` | **Rewrite** | RePoE mods.json loader |
| `data/gems.rs` | **Rewrite** | RePoE gems.json loader (replaces GemItem) |
| `data/skills.rs` | **Modify** | Update gem stat compilation for new Gem type |
| `data/mod.rs` | **Modify** | Update GameData struct + load_from_dir() |
| `data/stat_id.rs` | **No change** | StatId enum stays as-is |
| `modifier/mod_db.rs` | **Modify** | rebuild_tree/rebuild_class use new data sources |
| `modifier/parser.rs` | **Modify** | Demote to fallback; keep for unique items |
| `modifier/types.rs` | **No change** | Modifier/ModType/ModFlag stay as-is |
| `lib.rs` | **Modify** | Update commands for new Gem type, update GameData construction |

---

## 10. What Stays from PoB

| PoB File | Reason |
|---|---|
| `SkillStatMap.json` | Only source mapping internal stat IDs → calc variable semantics |
| `Uniques/*.json` | Only source of unique item mod text + base type. RePoE uniques.json is metadata only |

Everything else migrates to RePoE.

---

## 11. RePoE Schema Reference & Type Corrections

The RePoE repo provides official JSON Schema files at `https://github.com/repoe-fork/repoe/tree/master/RePoE/schema` generated via `datamodel-codegen`. These schemas were cross-referenced against the type definitions in Section 3 to identify discrepancies. This section documents all corrections needed when implementing the Rust structs.

> **Note**: No schema exists for `passive_skill_trees/` — that data format must be inferred from the actual downloaded files.

### 11a. `base_items.json` Schema Corrections

**Root type**: `Dict<String, BaseItemsSchemaValue>` (metadata path keys → item values)

**Required fields** (schema `"required"` array):
`domain`, `drop_level`, `implicits`, `inventory_height`, `inventory_width`, `item_class`, `name`, `properties`, `release_state`, `tags`, `visual_identity`

**Optional fields**: `requirements`, `grants_buff`, `skills_granted`, `inherits_from`

**Corrections to Section 3c `BaseItem` struct**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `release_state` type | `String` | Enum: `"released"`, `"unique_only"`, `"unreleased"`, `"legacy"` | Use `ReleaseState` enum |
| Missing `inherits_from` | Not in struct | `Option<String>` — inheritance chain (e.g., `"Metadata/Items/Armours/AbstractArmour"`) | Add field |
| Missing `visual_identity` | Not in struct | Required: `{dds_file: String, id: String}` | Add field (useful for item art lookup) |
| Missing `grants_buff` | Not in struct | Optional: `{id: String, stats: HashMap<String, i32>}` — flasks/jewels that grant buff effects | Add `Option<GrantsBuff>` field |
| Missing `skills_granted` | Not in struct | Optional: `Vec<String>` — skill gem metadata IDs granted by the item | Add `Option<Vec<String>>` field |
| Properties missing fields | Only equipment props | Also has: `stack_size`, `stack_size_currency_tab`, `full_stack_turns_into`, `mana_burn_ms`, `cooldown_ms`, `description`, `directions`, `monster_id`, `monster_ability_text`, `monster_category` | Add as `Option` fields for completeness, or use `#[serde(flatten)] other: HashMap<String, Value>` |
| `requirements` inner fields | All `u32` | Schema requires all 4 (dex/int/level/str) as integers when Requirements is present, but Requirements itself is optional | Correct — plan has `Option<ItemRequirements>`, inner fields should be `u32` (not optional) |
| Armour/evasion/ES min/max | `MinMax { min: u32, max: u32 }` | `{min: int, max: int}` — both required when present | Correct |

**Updated `ReleaseState` enum** (used by both base_items and gems):
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseState {
    Released,
    UniqueOnly,
    Unreleased,
    Legacy,
}
```

### 11b. `mods.json` Schema Corrections

**Root type**: `Dict<String, ModsSchemaValue>` (mod id → mod definition)

**Required fields**: `adds_tags`, `domain`, `generation_type`, `generation_weights`, `grants_effects`, `groups`, `implicit_tags`, `is_essence_only`, `name`, `required_level`, `spawn_weights`, `stats`, `type`

**Optional fields**: `text`, `gold_value`

**Corrections to Section 3d `ModDefinition` struct**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `text` optionality | `text: String` (required) | **Optional** — not all mods have display text | Change to `text: Option<String>` |
| Missing `grants_effects` | Not in struct | **Required**: `Vec<{granted_effect_id: String, level: u32}>` — mods that grant skills (e.g., unique mods granting auras) | Add `grants_effects: Vec<GrantsEffect>` |
| Missing `gold_value` | Not in struct | Optional: `f64` | Add `gold_value: Option<f64>` (low priority — trade feature) |

**New type to add**:
```rust
pub struct GrantsEffect {
    pub granted_effect_id: String,
    pub level: u32,
}
```

### 11c. `gems.json` Schema Corrections

**Root type**: `Dict<String, GemsSchemaValue>` (metadata path → gem)

**Required fields**: `color`, `is_support`, `per_level`, `stat_translation_file`, `static`

**Optional fields**: `active_skill`, `base_item`, `cast_time`, `display_name`, `quest_reward`, `tags`, `tooltip_order`, `discriminator`, `secondary_granted_effect`, `support_gem`

**Corrections to Section 3e `Gem` struct**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `display_name` optionality | Required `String` | **Optional** — some gems may not have a display name | Change to `Option<String>` |
| `tags` optionality | `Vec<String>` | **Optional** | Change to `Option<Vec<String>>` |
| Missing `stat_translation_file` | Not in struct | **Required**: `String` — which translation file to use for this gem's stats | Add field |
| Missing `secondary_granted_effect` | Not in struct | Optional `String` — vaal gems have a secondary effect | Add `Option<String>` |
| Missing `discriminator` | Not in struct | Optional enum: `"alt_x"` or `"alt_y"` — for gems with alternate quality | Add `Option<String>` |
| Missing `tooltip_order` | Not in struct | Optional `Vec<String>` | Add if needed for UI |
| Missing `quest_reward` | Not in struct | Optional: `{act: u32, classes: Vec<String>, quest: String}` | Add if needed for gem acquisition UI |

**ActiveSkill corrections**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `types` optionality | Required `Vec<String>` | **Optional** | Change to `Option<Vec<String>>` |
| Missing `is_skill_totem` | Not in struct | **Required** `bool` | Add field |
| Missing `skill_totem_life_multiplier` | Not in struct | Optional `f64` | Add `Option<f64>` |
| Missing `minion_types` | Not in struct | Optional `Vec<String>` | Add `Option<Vec<String>>` |

**SupportGem corrections**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `allowed_types` | Required `Vec<String>` | **Optional** — not all supports restrict types | Change to `Option<Vec<String>>` |
| `excluded_types` | `Option<Vec<String>>` | Optional — correct | ✅ |
| Missing `support_name` | Not in struct | Optional `String` | Add `Option<String>` |
| Missing `support_text` | Not in struct | Optional `String` — support description text | Add `Option<String>` |
| Missing `added_minion_types` | Not in struct | Optional `Vec<String>` | Add `Option<Vec<String>>` |

**Static (GemStaticData) corrections**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `stats` optionality | Implied required | **Optional**, and items can be `null` | Change to `Option<Vec<Option<GemStatEntry>>>` |
| Missing `attack_speed_multiplier` | Not in struct | Optional `u32` | Add `Option<u32>` |
| Missing `stored_uses` | Not in struct | Optional `u32` — traps, mines | Add `Option<u32>` |
| Missing `damage_multiplier` | Not in struct | Optional `u32` | Add `Option<u32>` |
| Missing `cooldown` | Not in struct | Optional `u32` | Add `Option<u32>` |
| Missing `cooldown_bypass_type` | Not in struct | Optional enum: `"expend_frenzy_charge"`, `"expend_power_charge"`, `"expend_endurance_charge"` | Add `Option<CooldownBypassType>` |
| Missing `reservations` | Not in struct | Optional: `{mana_percent: Option<f64>, life_percent: Option<f64>, mana_flat: Option<u32>}` | Add `Option<StaticReservations>` |
| Missing `vaal` | Not in struct | Optional: `{souls: u32, stored_uses: u32}` | Add `Option<VaalData>` |
| Missing `experience` | Not in struct | Optional `u32` | Add `Option<u32>` |
| Missing `stat_text` | Not in struct | Optional `HashMap<String, String>` | Add `Option<HashMap<String, String>>` |

**PerLevel (GemLevelData) corrections**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `required_level` | Required `u32` | **Optional** — ALL per-level fields are optional per schema | Change to `Option<u32>` |
| Stat entries can be null | `Vec<Option<GemLevelStat>>` | `Vec<Option<Stat>>` where Stat fields (id, value, type) are all optional | Use `Option` for all Stat fields |
| Missing `damage_multiplier` | Not in struct | Optional `u32` — distinct from damage_effectiveness | Add `Option<u32>` |
| Missing `stored_uses` | Not in struct | Optional `u32` | Add `Option<u32>` |

**Stat entry** (used in both static.stats and per_level.stats):
```rust
/// A stat entry — all fields optional per schema. Items in the array can also be null.
pub struct GemStatEntry {
    pub id: Option<String>,        // Internal stat ID (e.g., "base_chance_to_ignite_%")
    pub value: Option<i64>,        // Integer value
    #[serde(rename = "type")]
    pub stat_type: Option<StatType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatType {
    Float,
    Constant,
    Additional,
    Implicit,
    Flag,
}
```

**Color enum** (used by gems):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GemColor {
    #[serde(rename = "r")]
    Red,
    #[serde(rename = "g")]
    Green,
    #[serde(rename = "b")]
    Blue,
    #[serde(rename = "w")]
    White,
}
```

### 11d. `characters.json` Schema Corrections

**Root type**: `Vec<CharactersSchemaElement>` — **Array, not object!**

**Required fields**: `base_stats`, `integer_id`, `metadata_id`, `name`

**Optional fields**: `description`

**Corrections to Section 3g `CharacterClass` struct**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| Collection type | `CharacterDB { by_name: FxHashMap }` | Root is an **array**, not a map. Build the FxHashMap during loading. | Load as `Vec`, then index by name |
| Missing `metadata_id` | Not in struct | **Required** `String` (e.g., `"Metadata/Characters/Marauder"`) | Add field |
| Missing `description` | Not in struct | Optional `String` | Add `Option<String>` |
| Missing `unarmed` | Not in struct | **Required** nested object: `{attack_time: u32, max_physical_damage: u32, min_physical_damage: u32, range: u32}` | Add `unarmed: UnarmedStats` |

**New type to add**:
```rust
/// Unarmed attack stats per character class
pub struct UnarmedStats {
    pub attack_time: u32,
    pub min_physical_damage: u32,
    pub max_physical_damage: u32,
    pub range: u32,
}
```

### 11e. `stat_translations.json` Schema Corrections

**Root type**: `Vec<StatTranslationsSchemaElement>` — Array

**Required fields**: `ids` only (everything else optional)

**Optional fields**: `English`, `French`, `German`, `Japanese`, `Korean`, `Portuguese`, `Russian`, `Spanish`, `Thai`, `Traditional Chinese`, `trade_stats`, `hidden`

**Corrections to Section 3f `StatTranslationDB`**:

| Issue | Current Plan | Schema Says | Fix |
|---|---|---|---|
| `english` optionality | Implied required | **Optional** — some entries may only have non-English translations | Use `Option<Vec<TranslationVariant>>` during deserialization, skip entries with no English |
| Missing `hidden` | Not in struct | Optional `bool` — hidden stats don't show in tooltips | Add `hidden: Option<bool>` to entry, filter these during tooltip rendering |
| Missing `trade_stats` | Not in struct | Optional: `Vec<{id: String, text: String, type: String, option: Option<TradeStatOption>}>` | Add if needed for trade query generation (Phase 8) |
| Missing `reminder_text` | Not in `TranslationVariant` | Optional `String` on each variant | Add `Option<String>` — shown as grey italic text in-game |
| Missing `is_markup` | Not in `TranslationVariant` | Optional `bool` — whether string contains markup tags | Add `Option<bool>` |
| `format` type | `Vec<String>` | Enum: `"ignore"`, `"#"`, `"+#"` | Use enum or keep as String (3 values) |

**Format enum**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatFormat {
    #[serde(rename = "ignore")]
    Ignore,
    #[serde(rename = "#")]
    Plain,
    #[serde(rename = "+#")]
    PlusMinus,
}
```

### 11f. `stats.json` Schema (→ Section 3h `StatsDB`)

**Root type**: `Dict<String, StatsSchemaValue>` (internal stat id → stat info)

**Required fields**: `alias`, `is_aliased`, `is_local`

**Alias fields** (all optional within the alias object): `when_in_off_hand`, `when_in_main_hand`

**Purpose**: The `is_local` field is critical for the item system — it determines whether a stat applies locally to the item (e.g., `+% increased Physical Damage` on a weapon) or globally to the character. The `alias` fields handle dual-wielding stat mapping.

```rust
/// Stat metadata from RePoE stats.json
pub struct StatInfo {
    pub is_local: bool,
    pub is_aliased: bool,
    pub alias: StatAlias,
}

pub struct StatAlias {
    pub when_in_main_hand: Option<String>,
    pub when_in_off_hand: Option<String>,
}

/// Loaded once; queried during item stat resolution
pub struct StatsDB {
    stats: FxHashMap<String, StatInfo>,
}

impl StatsDB {
    pub fn load(path: &Path) -> Result<Self, DataError>;

    /// Check if a stat is local to the item
    pub fn is_local(&self, stat_id: &str) -> bool;

    /// Get the aliased stat ID for a given weapon slot
    pub fn alias_for_slot(&self, stat_id: &str, slot: WeaponSlot) -> Option<&str>;
}
```

**Impact**: Add `stats: StatsDB` to `GameData`. Used during item stat resolution (Phase 4) to determine local vs global application.

### 11g. `item_classes.json` Schema (→ Section 3i `ItemClassDB`)

**Root type**: `Dict<String, ItemClassValue>` (item class id → class info)

**Required fields**: `name` only

**Optional fields**: `category`, `category_id`, `influence_tags`

```rust
/// Item class metadata from RePoE item_classes.json
pub struct ItemClassInfo {
    pub name: String,
    pub category: Option<String>,
    pub category_id: Option<String>,
    pub influence_tags: Option<Vec<String>>,
}

pub struct ItemClassDB {
    classes: FxHashMap<String, ItemClassInfo>,
}
```

**Purpose**: Maps item class IDs (from base_items.json `item_class` field) to influence tags (needed for shaper/elder/etc. crafting) and category grouping.

### 11h. `cluster_jewels.json` Schema Summary

Three cluster sizes (Large/Medium/Small), each with:
- `max_skills`, `min_skills`, `total_indices` (u32)
- `notable_indices`, `small_indices`, `socket_indices` (Vec<u32>)
- `passive_skills`: Vec of `{id, name, stats: HashMap<String, i32>, stat_text: Vec<String>, tag}`

Passive skills within clusters use the same `stats: {stat_id: value}` pattern as the passive tree — these also resolve through `SkillStatMapDB` without text parsing.

### 11i. Schema-Informed Implementation Notes

1. **Use `#[serde(default)]`** liberally — many "required" fields in the schema may have edge cases in practice. Serde's `default` attribute provides resilient deserialization.

2. **Consider `#[serde(deny_unknown_fields)]` during development** only — catch unexpected fields early, then remove for production to handle future game patches gracefully.

3. **Nullable array items**: Gems `static.stats` and `per_level.stats` arrays can contain `null` entries. Use `Vec<Option<GemStatEntry>>` with serde.

4. **Enum string mappings**: Several fields use string enums (`ReleaseState`, `GemColor`, `StatType`, `CooldownBypassType`, `StatFormat`). Use serde's `rename` or `rename_all` attributes for clean Rust enums.

5. **`inherits_from` chains**: Base items form an inheritance tree via `inherits_from`. For Phase 4 item creation, this can be used to determine shared properties (e.g., all 2H axes inherit from `AbstractTwoHandAxe`).

6. **Potential for codegen**: These JSON Schemas are compatible with tools like `typify` or `schematools` for auto-generating Rust types. However, hand-crafted types with strategic `Option` usage and derived helper methods are preferred for this project — we only need a subset of fields and want clean APIs.

> **All corrections from this section have been applied back to Sections 3c–3i and Section 4.** This section is retained as a reference for the schema analysis and rationale behind each change.
