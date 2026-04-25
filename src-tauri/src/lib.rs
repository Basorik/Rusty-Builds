pub mod calc;
pub mod data;
pub mod item;
pub mod modifier;
pub mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::sync::{Arc, Mutex, RwLock};
use tauri::{Emitter, Manager};
use tauri_specta::{collect_commands, Builder};
use thiserror::Error;

use crate::data::gems::compute_gem_stats;
use crate::data::gems::GemSummary;
use crate::data::gems::RePoEGem;
use crate::data::item_mods::RePoEMod;
use crate::data::skills::{GemInstance, GemRef, SkillGroup};
use crate::data::uniques::UniqueItemDef;
use crate::data::{Bloodline, Class};
use crate::item::crafter::{build_crafted_item, CraftedItemSpec};
use crate::item::parser::parse_unique_item;
use crate::item::types::{Item, ItemSlot, ItemType, ModLineSource, Rarity};
use crate::storage::builds::SavedBuildData;
use crate::storage::manager::{BuildSummary, StorageManager};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashMap;

/// Type alias for the game data managed state (initially `None`, filled by the loader thread).
type GdState = Arc<RwLock<Option<data::GameData>>>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Lock Poisoned")]
    LockError,
    #[error("Game data still loading")]
    DataLoading,
    #[error("Item not found: {0}")]
    NotFound(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        AppError::LockError
    }
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}

/// Progress event payload sent to the frontend during startup data loading.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LoadProgress {
    /// Human-readable description of the current step.
    pub step: String,
    /// Completion fraction in [0.0, 1.0].
    pub fraction: f64,
    /// True when all data has been loaded and the app is ready.
    pub done: bool,
}

/// Helper: borrow `GameData` from the Option state, returning an appropriate
/// IPC error if data hasn't been loaded yet.
fn get_game(data: &Option<data::GameData>) -> Result<&data::GameData, AppError> {
    data.as_ref().ok_or(AppError::DataLoading)
}

/// A single human-readable stat line returned to the frontend for display.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GemStatLine {
    pub stat_id: String,
    pub value: f64,
}

/// The tree version loaded on startup and used as the default.
pub const DEFAULT_TREE_VERSION: &str = "3.27.0g";

/// Tracks which skill-tree nodes the user has selected for the current build.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct BuildSelection {
    selected_node_ids: std::collections::HashSet<u32>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildStats {
    pub total_strength: i32,
    pub total_dexterity: i32,
    pub total_intelligence: i32,
    pub node_count: u32,
    /// Full defensive calc results (life, mana, ES, resists, regen, etc.)
    pub defence: calc::DefenceResult,
    /// Offence calc results — `None` when no active gem is selected.
    pub offence: Option<calc::OffenceResult>,
    /// Time taken for `calc::calculate()` alone, in microseconds.
    /// Does not include IPC serialization/deserialization overhead.
    pub calc_time_us: u32,
}

/// Combined response for gem add/remove operations — returns both the updated group
/// (so the frontend can refresh the skill group list) and full recalculated stats.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GemGroupUpdate {
    pub group: data::skills::SkillGroup,
    pub stats: BuildStats,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct BuildInfo {
    pub name: String,
    pub level: u32,
    pub stats: BuildStats,
    pub class: Class,
    pub bloodline: Bloodline,
    pub selected_nodes: BuildSelection,
    pub skill_groups: Vec<SkillGroup>,
    /// The specific active gem the user has selected as the main skill for calculations.
    /// Points to a gem by group_id + gem_index. None = no main skill selected.
    pub active_gem: Option<GemRef>,
    next_group_id: u32,
    /// Persisted layered modifier database. Not serialized to the frontend —
    /// it's rebuilt from the other fields and serves as the authoritative source
    /// for all stat queries including the debug view.
    #[serde(skip)]
    #[specta(skip)]
    pub mod_db_layers: modifier::ModDBLayers,
    /// Currently equipped items, keyed by slot. Not serialized to the frontend —
    /// the frontend gets a summary via `get_equipped_items`.
    #[specta(skip)]
    pub equipped: FxHashMap<ItemSlot, Item>,
    /// All items currently in the build but not equipped.
    /// Equipped items live only in `equipped`; unequipping moves them here.
    #[specta(skip)]
    pub inventory: Vec<Item>,
    /// Monotonically increasing counter for assigning stable inventory IDs.
    #[specta(skip)]
    pub next_item_id: u32,
}

impl Default for BuildInfo {
    fn default() -> Self {
        BuildInfo {
            name: "Unsaved Build".to_string(),
            level: 1,
            stats: BuildStats::default(),
            // Must match buildState.svelte.ts default ("Marauder") so Rust state
            // is consistent before the first updateBuildInfo call from the frontend.
            class: Class::Marauder(None),
            bloodline: Bloodline::None,
            selected_nodes: BuildSelection::default(),
            skill_groups: Vec::new(),
            active_gem: None,
            next_group_id: 1,
            mod_db_layers: modifier::ModDBLayers::new(),
            equipped: FxHashMap::default(),
            inventory: Vec::new(),
            next_item_id: 1,
        }
    }
}

impl BuildInfo {
    pub fn add_item_to_inventory(&mut self, mut item: Item) {
        self.next_item_id += 1;
        item.inventory_id = self.next_item_id;
        self.inventory.push(item);
    }
}

/// A single modifier entry for the debug page.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DebugModEntry {
    pub stat: String,
    pub mod_type: String,
    pub value: f64,
    pub source: String,
    pub flags: String,
}

/// All modifiers from each ModDB layer, for the debug page.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DebugStatsResponse {
    pub tree_mods: Vec<DebugModEntry>,
    pub class_mods: Vec<DebugModEntry>,
    pub gem_mods: Vec<DebugModEntry>,
    pub items_mods: Vec<DebugModEntry>,
    pub computed: HashMap<String, DebugComputedStat>,
}

/// A fully computed stat value (base × inc × more).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DebugComputedStat {
    pub base: f64,
    pub inc: f64,
    pub more: f64,
    pub total: f64,
}

/// A unique item search result returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UniqueSearchResult {
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub variant_count: u32,
    pub variant_labels: Vec<String>,
    pub league: Option<String>,
}

/// Summary of one equipped item returned for the equipment panel.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EquippedItemSummary {
    pub slot: ItemSlot,
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub total_dps: Option<f64>,
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
    pub mod_count: u32,
}

/// A single unique item search result.
/// Re-used for both name-search and slot-browsing results.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UniqueListItem {
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub league: Option<String>,
}

/// One display line in a unique item's tooltip.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UniqueModLine {
    /// Display text with ranges like "(20-30)" preserved.
    pub text: String,
    /// Whether the stat translation system successfully resolved this line.
    /// False → show in red in the UI.
    pub is_mapped: bool,
    /// True for section headers like "Every 10 seconds:".
    pub is_header: bool,
    /// [min, max] for each `(X-Y)` range in this line, in left-to-right order.
    pub ranges: Vec<[f64; 2]>,
}

/// Full detail for one unique item variant, suitable for the roll-picker UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UniqueDetail {
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub league: Option<String>,
    pub influences: u8,
    pub base_props: Option<BaseItemProps>,
    pub implicit_lines: Vec<UniqueModLine>,
    pub explicit_lines: Vec<UniqueModLine>,
}

/// One display line in a built item's tooltip.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemModLine {
    pub text: String,
    /// "implicit" | "explicit" | "crafted" | "enchant" | "fractured"
    pub kind: String,
}

/// Full detail for one item, suitable for a PoE-style tooltip in the inventory preview.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ItemDetail {
    pub inventory_id: u32,
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub rarity: Rarity,
    pub item_level: u32,
    pub quality: u32,
    pub corrupted: bool,
    pub mirrored: bool,
    pub synthesised: bool,
    pub fractured: bool,
    pub influences: u8,
    pub req_level: u32,
    pub req_str: u32,
    pub req_dex: u32,
    pub req_int: u32,
    // Weapon stats
    pub phys_damage_min: Option<f64>,
    pub phys_damage_max: Option<f64>,
    pub attacks_per_second: Option<f64>,
    pub crit_chance: Option<f64>,
    pub total_dps: Option<f64>,
    pub phys_dps: Option<f64>,
    pub ele_dps: Option<f64>,
    // Armour stats
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
    pub block: Option<u32>,
    // Mod lines
    pub enchant_lines: Vec<ItemModLine>,
    pub implicit_lines: Vec<ItemModLine>,
    pub explicit_lines: Vec<ItemModLine>,
}

/// Summary of one base item class returned for the base-item browser.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseItemSummary {
    pub name: String,
    pub item_class: String,
    pub level_req: u32,
}

/// Full properties of a single base item, returned when the user selects one
/// in the crafting panel so they can configure exact value rolls.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseItemProps {
    pub name: String,
    pub item_class: String,
    pub level_req: u32,
    pub tags: Vec<String>,
    // Weapon
    pub phys_damage_min: Option<f64>,
    pub phys_damage_max: Option<f64>,
    pub attack_time_ms: Option<u32>,
    pub crit_chance_permyriad: Option<u32>,
    // Armour / shield
    pub armour_min: Option<f64>,
    pub armour_max: Option<f64>,
    pub evasion_min: Option<f64>,
    pub evasion_max: Option<f64>,
    pub energy_shield_min: Option<f64>,
    pub energy_shield_max: Option<f64>,
    pub block: Option<u32>,
    // Flask
    pub charges_max: Option<u32>,
    pub life_per_use: Option<f64>,
    pub mana_per_use: Option<f64>,
}

/// A single stat slot within an available mod.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AvailableModStat {
    pub stat_id: String,
    pub min: f64,
    pub max: f64,
}

/// A mod that can be applied to a given base item.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AvailableMod {
    pub mod_id: String,
    /// Human-readable tier/class name, e.g. "of the Ox", "Sturdy".
    pub name: String,
    /// "prefix", "suffix", "unique", "corrupted", etc.
    pub generation_type: String,
    /// Mutual-exclusion groups (e.g. cannot have two mods from the same group).
    pub groups: Vec<String>,
    pub required_level: u32,
    pub stats: Vec<AvailableModStat>,
}

/// All mods available for a given base item, grouped by category.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseMods {
    /// Auto-populated from the base item's `implicits` field.
    pub implicits: Vec<AvailableMod>,
    pub prefixes: Vec<AvailableMod>,
    pub suffixes: Vec<AvailableMod>,
    /// Master-crafted bench mods.
    pub crafted: Vec<AvailableMod>,
}

// ─── Hierarchical base browser types ─────────────────────────────────────────

/// One subcategory within the base-item browser, e.g. "Evasion Boots".
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseSubcategory {
    pub label: String,
    pub bases: Vec<BaseItemSummary>,
}

/// Top-level category in the base browser, e.g. "Armour".
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseCategory {
    pub name: String,
    pub subcategories: Vec<BaseSubcategory>,
}

// ─── Tiered mod types ─────────────────────────────────────────────────────────

/// One tier of a single logical mod (e.g. T1 life roll).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ModTierInfo {
    pub mod_id: String,
    /// 1 = best tier (highest required level / strongest roll), 2 = second best, …
    pub tier: u32,
    pub required_level: u32,
    pub stats: Vec<AvailableModStat>,
}

/// All tiers of one logical modifier grouped together.
/// e.g. all tiers of "# increased maximum Life" as a suffix.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StatModGroup {
    /// Human-readable display name from stat translations, e.g. "#% increased maximum Life".
    pub display_name: String,
    pub generation_type: String,
    /// First mutual-exclusion group name (items can hold only one mod per group).
    pub group: String,
    /// Tiers sorted best-first (T1 at index 0).
    pub tiers: Vec<ModTierInfo>,
}

/// All mods for a base item, tiered and grouped by stat — for the crafting UI.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BaseModGroups {
    pub implicits: Vec<StatModGroup>,
    pub prefixes: Vec<StatModGroup>,
    pub suffixes: Vec<StatModGroup>,
    pub crafted: Vec<StatModGroup>,
}
// They are re-exported via the import above for tauri-specta visibility.

/// Summary of a single item in the build inventory (equipped or not).
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InventoryItemSummary {
    pub inventory_id: u32,
    pub name: String,
    pub base_name: String,
    pub item_class: String,
    pub rarity: Rarity,
    pub total_dps: Option<f64>,
    pub armour: Option<f64>,
    pub evasion: Option<f64>,
    pub energy_shield: Option<f64>,
    pub mod_count: u32,
    /// `Some(slot)` when the item is currently equipped in that slot.
    pub equipped_slot: Option<ItemSlot>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        greet,
        update_selected_nodes,
        update_build_info,
        get_available_tree_versions,
        load_tree_version,
        get_tree_json,
        get_gem_list,
        get_gem_stats_at,
        get_skill_groups,
        create_skill_group,
        delete_skill_group,
        add_gem_to_group,
        remove_gem_from_group,
        update_gem_level_quality,
        get_group_effects,
        set_active_gem,
        set_gem_always_active,
        get_debug_stats,
        equip_item,
        unequip_item,
        get_equipped_items,
        search_uniques,
        get_base_items,
        get_item_classes,
        get_base_item_props,
        get_mods_for_base,
        get_base_categories,
        get_mods_for_base_grouped,
        add_crafted_item,
        get_inventory_items,
        get_inventory_for_slot,
        equip_from_inventory,
        remove_inventory_item,
        get_uniques_for_class,
        get_unique_detail,
        add_unique_to_inventory,
        get_item_detail_by_slot,
        get_item_detail_by_id,
        list_builds,
        save_build,
        load_build,
        delete_build,
        rename_build
    ]);

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    let invoke_handler = builder.invoke_handler();
    tauri::Builder::default()
        // .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .setup(move |app| {
            // Game data starts as None; the loader thread fills it in.
            let gd_state: GdState = Arc::new(RwLock::new(None));
            app.manage(Mutex::new(BuildInfo::default()));
            app.manage(gd_state.clone());
            app.manage(
                StorageManager::new(app.handle()).expect("Failed to initialize build storage"),
            );

            // Spawn background thread so the window opens immediately while
            // data loads in the background.
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let resource_path = handle
                    .path()
                    .resource_dir()
                    .expect("Failed to resolve resource directory");

                let gd =
                    match data::GameData::load_with_progress(resource_path, |step, fraction| {
                        let _ = handle.emit(
                            "loading_progress",
                            LoadProgress {
                                step: step.to_string(),
                                fraction,
                                done: false,
                            },
                        );
                    }) {
                        Ok(gd) => gd,
                        Err(e) => {
                            log::error!("Failed to load game data: {}", e);
                            return;
                        }
                    };

                info!(
                    "Loaded {} GGG tree nodes, {} RePoE passives, {} gems",
                    gd.tree.nodes.len(),
                    gd.repoe_tree.passives.len(),
                    gd.gems.len()
                );

                // Populate class modifier layer now that tree data is available.
                {
                    let build_state = handle.state::<Mutex<BuildInfo>>();
                    let mut bi = build_state.lock().expect("BuildInfo mutex poisoned");
                    let class = bi.class.clone();
                    bi.mod_db_layers.rebuild_class(&class, &gd.tree);
                }

                *gd_state.write().unwrap() = Some(gd);
                let _ = handle.emit(
                    "loading_progress",
                    LoadProgress {
                        step: "Ready.".to_string(),
                        fraction: 1.0,
                        done: true,
                    },
                );
            });

            builder.mount_events(app);
            Ok(())
        })
        .invoke_handler(invoke_handler)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    info!("Running Greet: {name}");
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Receives the build info from the frontend.
#[tauri::command]
#[specta::specta]
fn update_build_info(
    level: u32,
    character_class: Class,
    bloodline: Bloodline,
    state: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.level = level;
    build_info.class = character_class;
    build_info.bloodline = bloodline;

    // Rebuild the class layer so the ModDB reflects the new class base stats.
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let class = build_info.class.clone();
    build_info.mod_db_layers.rebuild_class(&class, &game.tree);

    info!(
        "Build updated: Level {}, Class {:?}, Bloodline {:?}",
        build_info.level, build_info.class, build_info.bloodline
    );

    let stats = compute_stats(
        &build_info.mod_db_layers,
        build_info.level,
        &build_info.class,
        build_info.active_gem.as_ref(),
        &build_info.skill_groups,
        &build_info.equipped,
        game,
        &build_info.selected_nodes,
    );
    build_info.stats = stats.clone();
    Ok(stats)
}

/// Receives the current set of selected node IDs from the frontend.
/// Rebuilds the tree layer of the persisted ModDB and returns updated build stats.
#[tauri::command]
#[specta::specta]
fn update_selected_nodes(
    node_ids: Vec<u32>,
    game_data: tauri::State<'_, GdState>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.selected_nodes.selected_node_ids = node_ids.iter().cloned().collect();

    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;

    // Rebuild only the tree layer — class and gems layers are already up to date.
    build.mod_db_layers.rebuild_tree(&node_ids, &game);
    // Refresh gem layer in case skill group counts changed.
    // let skill_groups = build.skill_groups.clone();
    // build.mod_db_layers.rebuild_gems(&skill_groups);

    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );

    build.stats = stats.clone();
    Ok(stats)
}

#[tauri::command]
#[specta::specta]
fn get_available_tree_versions(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let tree_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource dir: {}", e))?
        .join("data/tree");

    let entries = std::fs::read_dir(&tree_dir)
        .map_err(|e| format!("Failed to read tree directory: {}", e))?;

    let mut versions: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    versions.sort_by(|a, b| b.cmp(a)); // Sort descending so newest is first
    Ok(versions)
}

#[tauri::command]
#[specta::specta]
fn load_tree_version(
    version: String,
    app: tauri::AppHandle,
    game_data_state: tauri::State<'_, GdState>,
) -> Result<(), String> {
    // Validate version to prevent path traversal
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err("Invalid version string".to_string());
    }

    let resource_path = app
        .path()
        .resource_dir()
        .expect("Failed to resolve resource directory");

    let game_data = data::GameData::load_from_dir(resource_path).expect("Failed to load game data");

    info!(
        "Reloaded: {} GGG tree nodes, {} RePoE passives, {} gems",
        game_data.tree.nodes.len(),
        game_data.repoe_tree.passives.len(),
        game_data.gems.len()
    );

    *game_data_state.write().map_err(|e| e.to_string())? = Some(game_data);

    info!("Successfully loaded tree version: {}", version);
    Ok(())
}

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

/// Returns a lightweight summary of every gem for the frontend selector.
#[tauri::command]
#[specta::specta]
fn get_gem_list(game_data: tauri::State<'_, GdState>) -> Result<Vec<GemSummary>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let mut list: Vec<GemSummary> = game
        .gems
        .iter()
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

/// Returns the computed stats for a specific gem at the given level and quality.
/// Used by the frontend info panel to show per-stat values.
#[tauri::command]
#[specta::specta]
fn get_gem_stats_at(
    gem_id: String,
    level: u32,
    quality: u32,
    game_data: tauri::State<'_, GdState>,
) -> Result<Vec<GemStatLine>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let gem = game
        .gems
        .get(&gem_id)
        .ok_or_else(|| format!("Gem '{}' not found", gem_id))?;
    let stats = compute_gem_stats(gem, level, quality);
    Ok(stats
        .into_iter()
        .map(|(stat_id, value)| GemStatLine { stat_id, value })
        .collect())
}

/// Returns all skill groups for the current build.
#[tauri::command]
#[specta::specta]
fn get_skill_groups(
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<Vec<SkillGroup>, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    Ok(build.skill_groups.clone())
}

/// Creates a new empty skill group and returns it.
#[tauri::command]
#[specta::specta]
fn create_skill_group(
    label: String,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<SkillGroup, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let group = SkillGroup {
        id: build.next_group_id,
        label,
        gems: Vec::new(),
        enabled: true,
    };
    build.next_group_id += 1;
    build.skill_groups.push(group.clone());
    Ok(group)
}

/// Deletes a skill group by ID.
#[tauri::command]
#[specta::specta]
fn delete_skill_group(
    group_id: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<(), String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let len_before = build.skill_groups.len();
    build.skill_groups.retain(|g| g.id != group_id);
    if build.skill_groups.len() == len_before {
        return Err(format!("Skill group {} not found", group_id));
    }
    // If the deleted group contained the active gem, clear the active selection.
    if build.active_gem.as_ref().map(|r| r.group_id) == Some(group_id) {
        build.active_gem = None;
        let gd_lock = game_data.read().map_err(|e| e.to_string())?;
        let game = get_game(&*gd_lock)?;
        let groups = build.skill_groups.clone();
        build.mod_db_layers.rebuild_gems(&groups, None, &game);
    }
    Ok(())
}

/// Adds a gem to a skill group.
#[tauri::command]
#[specta::specta]
fn add_gem_to_group(
    group_id: u32,
    gem_id: String,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<GemGroupUpdate, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let gem: &RePoEGem = game
        .gems
        .get(&gem_id)
        .ok_or_else(|| format!("Gem '{}' not found", gem_id))?;
    let gem_instance = GemInstance {
        gem_id,
        name: gem.display_name.clone().unwrap_or_default(),
        is_support: gem.is_support,
        level: gem.base_item.clone().unwrap().max_level,
        quality: 0,
        enabled: true,
        always_active: false,
    };
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let updated_group = {
        let skill_group = build
            .skill_groups
            .iter_mut()
            .find(|x| x.id == group_id)
            .ok_or_else(|| format!("Skill group {} not found", group_id))?;
        skill_group.gems.push(gem_instance);
        skill_group.clone()
    };
    let groups = build.skill_groups.clone();
    let active = build.active_gem.clone();
    build
        .mod_db_layers
        .rebuild_gems(&groups, active.as_ref(), &game);
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(GemGroupUpdate {
        group: updated_group,
        stats,
    })
}

/// Removes a gem from a skill group by its index in the gem list.
#[tauri::command]
#[specta::specta]
fn remove_gem_from_group(
    group_id: u32,
    gem_index: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<GemGroupUpdate, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;

    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let updated_group = {
        let skill_group = build
            .skill_groups
            .iter_mut()
            .find(|x| x.id == group_id)
            .ok_or_else(|| format!("Skill group {} not found", group_id))?;
        skill_group.gems.remove(gem_index as usize);
        skill_group.clone()
    };
    let groups = build.skill_groups.clone();
    let active = build.active_gem.clone();
    build
        .mod_db_layers
        .rebuild_gems(&groups, active.as_ref(), &game);
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(GemGroupUpdate {
        group: updated_group,
        stats,
    })
}

/// Updates the level and/or quality of a gem in a skill group, recomputing its stats.
#[tauri::command]
#[specta::specta]
fn update_gem_level_quality(
    _group_id: u32,
    _gem_index: u32,
    _level: u32,
    _quality: u32,
    _build_info: tauri::State<'_, Mutex<BuildInfo>>,
    _game_data: tauri::State<'_, GdState>,
) -> Result<SkillGroup, String> {
    Err("Not yet implemented: gem module under construction".to_string())
}

/// Returns the skill group with up-to-date effects and compatibility.
#[tauri::command]
#[specta::specta]
fn get_group_effects(
    _group_id: u32,
    _build_info: tauri::State<'_, Mutex<BuildInfo>>,
    _game_data: tauri::State<'_, GdState>,
) -> Result<SkillGroup, String> {
    Err("Not yet implemented: gem module under construction".to_string())
}

#[tauri::command]
#[specta::specta]
fn set_active_gem(
    gem_ref: Option<GemRef>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.active_gem = gem_ref.clone();
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let groups = build.skill_groups.clone();
    build
        .mod_db_layers
        .rebuild_gems(&groups, gem_ref.as_ref(), &game);

    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Toggles the always_active flag on a gem. Always-active gems (auras, heralds, etc.)
/// contribute to stat calculations regardless of which main skill is selected.
#[tauri::command]
#[specta::specta]
fn set_gem_always_active(
    group_id: u32,
    gem_index: u32,
    always_active: bool,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<SkillGroup, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let group = build
        .skill_groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Skill group {} not found", group_id))?;
    let idx = gem_index as usize;
    if idx >= group.gems.len() {
        return Err(format!("Gem index {} out of range", gem_index));
    }
    group.gems[idx].always_active = always_active;
    let result = group.clone();
    // Rebuild gems layer so always_active change takes effect immediately.
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let groups = build.skill_groups.clone();
    let active_gem = build.active_gem.clone();
    build
        .mod_db_layers
        .rebuild_gems(&groups, active_gem.as_ref(), &game);
    Ok(result)
}
/// Returns all modifiers in the persisted ModDB layers for the debug page.
/// Reads directly from build.mod_db_layers — does NOT recompute anything.
#[tauri::command]
#[specta::specta]
fn get_debug_stats(
    game_data: tauri::State<'_, GdState>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<DebugStatsResponse, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let ctx = modifier::CalcContext::empty();

    // Helper: extract mods from a ModDB layer into DebugModEntry list.
    fn extract_mods(db: &modifier::ModDB, tree: &data::PassiveTree) -> Vec<DebugModEntry> {
        let mut entries = Vec::new();
        for (stat_id, mods) in db.iter_all() {
            let stat_name = format!("{:?}", stat_id);
            for m in mods {
                let source_id = m.source.0;
                let source_label = if source_id == 0 {
                    "Class Base".to_string()
                } else if let Some(node) = tree.get_node(source_id) {
                    let name = node.name.as_deref().unwrap_or("Unknown");
                    format!("{} ({})", name, source_id)
                } else {
                    format!("Source:{}", source_id)
                };
                entries.push(DebugModEntry {
                    stat: stat_name.clone(),
                    mod_type: format!("{:?}", m.mod_type),
                    value: m.value,
                    source: source_label,
                    flags: if m.flags.is_empty() {
                        String::new()
                    } else {
                        format!("{:?}", m.flags)
                    },
                });
            }
        }
        entries.sort_by(|a, b| a.stat.cmp(&b.stat).then(a.source.cmp(&b.source)));
        entries
    }

    // Read directly from the persisted layers — no rebuild.
    let tree_mods = extract_mods(&build.mod_db_layers.tree, &game.tree);
    let class_mods = extract_mods(&build.mod_db_layers.class, &game.tree);
    let gem_mods = extract_mods(&build.mod_db_layers.gems, &game.tree);
    let items_mods = extract_mods(&build.mod_db_layers.items, &game.tree);

    // Merge all layers and compute final values for each unique stat.
    let combined = build.mod_db_layers.merged();
    let mut computed = HashMap::new();
    let mut seen_stats = FxHashSet::default();
    for (stat_id, _) in combined.iter_all() {
        seen_stats.insert(*stat_id);
    }
    for stat_id in seen_stats {
        let base = combined.sum_base(stat_id, &ctx);
        let inc = combined.sum_inc(stat_id, &ctx);
        let more = combined.product_more(stat_id, &ctx);
        let total = combined.calculate(stat_id, &ctx);
        computed.insert(
            format!("{:?}", stat_id),
            DebugComputedStat {
                base,
                inc,
                more,
                total,
            },
        );
    }

    Ok(DebugStatsResponse {
        tree_mods,
        class_mods,
        gem_mods,
        items_mods,
        computed,
    })
}

/// Equips a unique item into the given slot, rebuilding the items ModDB layer and
/// returning updated build stats. Validates slot/type compatibility before equipping.
#[tauri::command]
#[specta::specta]
fn equip_item(
    slot: ItemSlot,
    unique_name: String,
    variant: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let def: &UniqueItemDef = game
        .uniques
        .iter()
        .find(|u| u.name == unique_name)
        .ok_or_else(|| format!("Unique '{}' not found", unique_name))?;

    // Parse the item to learn its item_type before slot-check.
    let source = data::SourceId(slot as u32 + 1000);
    let mut item = parse_unique_item(def, variant as usize, &game, source);

    // Slot compatibility check.
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let weapon1_type = build.equipped.get(&ItemSlot::Weapon1).map(|i| i.item_type);
    if !ItemSlot::is_compatible(slot, item.item_type, weapon1_type) {
        return Err(format!(
            "{:?} cannot be equipped in slot {:?}",
            item.item_type, slot
        ));
    }

    // Assign a new inventory ID so this item can be tracked in the inventory.
    build.next_item_id += 1;
    item.inventory_id = build.next_item_id;

    // If something is already in the slot, move it to the unequipped inventory.
    if let Some(displaced) = build.equipped.remove(&slot) {
        build.inventory.push(displaced);
    }

    build.equipped.insert(slot, item);
    let equipped_map = build.equipped.clone();
    build.mod_db_layers.rebuild_items(&equipped_map);

    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Removes the item in the given slot and returns updated build stats.
#[tauri::command]
#[specta::specta]
fn unequip_item(
    slot: ItemSlot,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    // Move item to unequipped inventory rather than discarding it.
    if let Some(item) = build.equipped.remove(&slot) {
        build.inventory.push(item);
    }
    let equipped_map = build.equipped.clone();
    build.mod_db_layers.rebuild_items(&equipped_map);

    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Returns a summary of all currently equipped items.
#[tauri::command]
#[specta::specta]
fn get_equipped_items(
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<Vec<EquippedItemSummary>, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let mut result: Vec<EquippedItemSummary> = build
        .equipped
        .iter()
        .map(|(&slot, item)| EquippedItemSummary {
            slot,
            name: item.name.clone(),
            base_name: item.base_name.clone(),
            item_class: item.item_class.clone(),
            total_dps: item.weapon_data.as_ref().map(|w| w.total_dps),
            armour: item
                .armour_data
                .as_ref()
                .map(|a| a.armour)
                .filter(|&v| v > 0.0),
            evasion: item
                .armour_data
                .as_ref()
                .map(|a| a.evasion)
                .filter(|&v| v > 0.0),
            energy_shield: item
                .armour_data
                .as_ref()
                .map(|a| a.energy_shield)
                .filter(|&v| v > 0.0),
            mod_count: (item.implicit_lines.len()
                + item.explicit_lines.len()
                + item.crafted_lines.len()
                + item.enchant_lines.len()) as u32,
        })
        .collect();
    result.sort_by_key(|e| e.slot as u8);
    Ok(result)
}

/// Returns the full PoE-style detail for the item equipped in the given slot.
#[tauri::command]
#[specta::specta]
fn get_item_detail_by_slot(
    slot: ItemSlot,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<ItemDetail, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let item = build
        .equipped
        .get(&slot)
        .ok_or_else(|| format!("No item equipped in slot {slot:?}"))?;
    Ok(item_to_detail(item))
}

/// Returns the full PoE-style detail for any inventory item by its inventory ID.
/// Searches both unequipped inventory and equipped slots.
#[tauri::command]
#[specta::specta]
fn get_item_detail_by_id(
    inventory_id: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<ItemDetail, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    if let Some(item) = build
        .inventory
        .iter()
        .find(|i| i.inventory_id == inventory_id)
    {
        return Ok(item_to_detail(item));
    }
    for item in build.equipped.values() {
        if item.inventory_id == inventory_id {
            return Ok(item_to_detail(item));
        }
    }
    Err(format!("Item with id {inventory_id} not found"))
}

// ─── Unique browser helpers (private) ────────────────────────────────────────

/// Returns the 1-based variant index of the "current" (latest non-historical)
/// variant, given a slice of variant labels like `["Pre 3.0.0", "Current"]`.
///
/// * Labels that start with "Pre " followed by a digit are classified as
///   historical and skipped.
/// * If no labels exist, variant 1 is returned.
/// * If every label is historical, the last variant is returned as fallback.
fn unique_current_variant(labels: &[String]) -> usize {
    if labels.is_empty() {
        return 1;
    }
    for (i, label) in labels.iter().enumerate().rev() {
        // "Pre 3.0.0", "Pre 3.19.0", etc. — any label starting "Pre " + digit.
        let after_pre = label.strip_prefix("Pre ").unwrap_or("");
        if !after_pre.starts_with(|c: char| c.is_ascii_digit()) {
            return i + 1; // 1-based
        }
    }
    labels.len() // fallback: last variant
}

/// Heuristic to find the first mod line in a raw unique text block that has no
/// explicit `Implicits:` marker.  Skips the name, base name(s), and any leading
/// tag/metadata lines.
fn unique_find_mod_start_heuristic(raw_lines: &[&str]) -> usize {
    // Index 0 = name, index 1 = first base type or metadata.
    // We skip lines that look like metadata: start with '{', pure keywords, etc.
    for (i, &line) in raw_lines.iter().enumerate().skip(1) {
        let s = if line.starts_with('{') {
            line.find('}')
                .map(|c| line[c + 1..].trim_start())
                .unwrap_or(line)
        } else {
            line
        };
        // Once we hit lines that look like actual mods (contain letters and
        // possibly numbers/% but are not pure title tokens), treat i as the
        // first mod line.  A very rough heuristic: if the clean line contains a
        // space or special characters it's a mod.
        if s.contains(' ') || s.contains('%') || s.contains('+') {
            return i;
        }
    }
    // If nothing matched, fall back to after name + base = line 2.
    raw_lines.len().min(2)
}

/// Searches unique items by name (case-insensitive substring match).
/// Returns up to 50 results.
#[tauri::command]
#[specta::specta]
fn search_uniques(
    query: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<Vec<UniqueSearchResult>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let q = query.to_lowercase();
    let results: Vec<UniqueSearchResult> = game
        .uniques
        .iter()
        .filter(|u| u.name.to_lowercase().contains(&q))
        .take(50)
        .map(|u| {
            let base_name = u.base_for_variant(1).to_string();
            let item_class = game
                .bases
                .get(&base_name)
                .map(|b| b.item_class.clone())
                .unwrap_or_default();
            UniqueSearchResult {
                name: u.name.clone(),
                base_name,
                item_class,
                variant_count: u.variant_count() as u32,
                variant_labels: u.variant_labels.clone(),
                league: u.league.clone(),
            }
        })
        .collect();
    Ok(results)
}

/// Returns all uniques matching the given name query and/or item class filter.
/// Pass an empty `query` and/or `None` item_class to list uniques broadly.
#[tauri::command]
#[specta::specta]
fn get_uniques_for_class(
    item_class: Option<String>,
    query: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<Vec<UniqueListItem>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let q = query.to_lowercase();
    let ic_filter = item_class.as_deref().unwrap_or("").to_lowercase();

    let mut results: Vec<UniqueListItem> = game
        .uniques
        .iter()
        .filter_map(|u| {
            if !q.is_empty() && !u.name.to_lowercase().contains(&q) {
                return None;
            }
            // Determine base/class for the "current" variant.
            let cv = unique_current_variant(&u.variant_labels);
            let base_name = u.base_for_variant(cv).to_string();
            let item_class_str = game
                .bases
                .get(&base_name)
                .map(|b| b.item_class.clone())
                .unwrap_or_default();
            if !ic_filter.is_empty() && item_class_str.to_lowercase() != ic_filter {
                return None;
            }
            Some(UniqueListItem {
                name: u.name.clone(),
                base_name,
                item_class: item_class_str,
                league: u.league.clone(),
            })
        })
        .take(200)
        .collect();

    results.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(results)
}

/// Returns full detail for a unique item at its "current" (latest non-historical) variant.
/// Each display line carries its ranges, mapped status, and source (implicit/explicit).
#[tauri::command]
#[specta::specta]
fn get_unique_detail(
    name: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<UniqueDetail, String> {
    use data::uniques::{active_for_variant, strip_all_tags};
    use regex::Regex;
    use std::sync::OnceLock;

    static RANGE_RE: OnceLock<Regex> = OnceLock::new();
    let range_re = RANGE_RE.get_or_init(|| {
        Regex::new(r"\((-?[0-9]+(?:\.[0-9]+)?)-(-?[0-9]+(?:\.[0-9]+)?)\)").unwrap()
    });

    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let def = game
        .uniques
        .iter()
        .find(|u| u.name == name)
        .ok_or_else(|| format!("Unique '{}' not found", name))?;

    let cv = unique_current_variant(&def.variant_labels);
    let base_name = def.base_for_variant(cv).to_string();

    let item_class = game
        .bases
        .get(&base_name)
        .map(|b| b.item_class.clone())
        .unwrap_or_default();

    let base_props = game.bases.get(&base_name).map(|b| {
        let p = &b.properties;
        BaseItemProps {
            name: b.name.clone(),
            item_class: b.item_class.clone(),
            level_req: b.drop_level,
            tags: b.tags.clone(),
            phys_damage_min: p.physical_damage_min,
            phys_damage_max: p.physical_damage_max,
            attack_time_ms: p.attack_time,
            crit_chance_permyriad: p.critical_strike_chance,
            armour_min: p.armour.as_ref().map(|m| m.min),
            armour_max: p.armour.as_ref().map(|m| m.max),
            evasion_min: p.evasion.as_ref().map(|m| m.min),
            evasion_max: p.evasion.as_ref().map(|m| m.max),
            energy_shield_min: p.energy_shield.as_ref().map(|m| m.min),
            energy_shield_max: p.energy_shield.as_ref().map(|m| m.max),
            block: p.block,
            charges_max: p.charges_max,
            life_per_use: p.life_per_use,
            mana_per_use: p.mana_per_use,
        }
    });

    let influences = def.influences;

    // Parse mod lines from the raw text block.
    let raw_lines: Vec<&str> = def
        .raw_text
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    // Locate Implicits: N boundary (same heuristic as parser.rs).
    let (implicit_count, mod_start) = raw_lines
        .iter()
        .enumerate()
        .find_map(|(idx, &l)| {
            let s = if l.starts_with('{') {
                l.find('}').map(|c| l[c + 1..].trim_start()).unwrap_or(l)
            } else {
                l
            };
            s.strip_prefix("Implicits: ")
                .and_then(|n| n.parse::<usize>().ok())
                .map(|n| (n, idx + 1))
        })
        .unwrap_or_else(|| {
            // No Implicits marker: skip name + base + metadata lines.
            let start = unique_find_mod_start_heuristic(&raw_lines);
            (0, start)
        });

    let mut implicit_lines: Vec<UniqueModLine> = Vec::new();
    let mut explicit_lines: Vec<UniqueModLine> = Vec::new();

    for (i, &raw_line) in raw_lines[mod_start..].iter().enumerate() {
        if !active_for_variant(raw_line, cv) {
            continue;
        }
        let (_, clean) = strip_all_tags(raw_line);
        if clean.is_empty() {
            continue;
        }

        let is_implicit = i < implicit_count;
        let is_header = clean.trim_end().ends_with(':')
            || clean.starts_with("(The ")
            || clean.starts_with("(This ");

        let ranges: Vec<[f64; 2]> = range_re
            .captures_iter(clean)
            .map(|cap| {
                let lo: f64 = cap[1].parse().unwrap_or(0.0);
                let hi: f64 = cap[2].parse().unwrap_or(0.0);
                [lo, hi]
            })
            .collect();

        let is_mapped = !is_header
            && game
                .translations
                .resolve_line(clean, &game.stat_metadata)
                .is_some();

        let line = UniqueModLine {
            text: clean.to_string(),
            is_mapped,
            is_header,
            ranges,
        };
        if is_implicit {
            implicit_lines.push(line);
        } else {
            explicit_lines.push(line);
        }
    }

    Ok(UniqueDetail {
        name: def.name.clone(),
        base_name,
        item_class,
        league: def.league.clone(),
        influences,
        base_props,
        implicit_lines,
        explicit_lines,
    })
}

/// Adds a unique item to the build inventory with user-specified roll values.
///
/// `rolls` is a flat list of numeric values corresponding to every `(X-Y)` range
/// that appears across `(implicit_lines ++ explicit_lines)` of the current variant,
/// in left-to-right / top-to-bottom order matching `get_unique_detail` output.
#[tauri::command]
#[specta::specta]
fn add_unique_to_inventory(
    name: String,
    rolls: Vec<f64>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    use data::uniques::{active_for_variant, strip_all_tags};
    use regex::Regex;
    use std::sync::OnceLock;

    static RANGE_RE2: OnceLock<Regex> = OnceLock::new();
    let range_re = RANGE_RE2.get_or_init(|| {
        Regex::new(r"\((-?[0-9]+(?:\.[0-9]+)?)-(-?[0-9]+(?:\.[0-9]+)?)\)").unwrap()
    });

    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let def = game
        .uniques
        .iter()
        .find(|u| u.name == name)
        .ok_or_else(|| format!("Unique '{}' not found", name))?;

    let cv = unique_current_variant(&def.variant_labels);

    // Build the simplified raw text with ranges substituted by chosen values.
    // Structure: Name \n BaseName \n Implicits: N \n <mod lines>
    let raw_lines: Vec<&str> = def
        .raw_text
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    let (implicit_count, mod_start) = raw_lines
        .iter()
        .enumerate()
        .find_map(|(idx, &l)| {
            let s = if l.starts_with('{') {
                l.find('}').map(|c| l[c + 1..].trim_start()).unwrap_or(l)
            } else {
                l
            };
            s.strip_prefix("Implicits: ")
                .and_then(|n| n.parse::<usize>().ok())
                .map(|n| (n, idx + 1))
        })
        .unwrap_or_else(|| (0, unique_find_mod_start_heuristic(&raw_lines)));

    let base_name = def.base_for_variant(cv).to_string();

    let mut roll_idx = 0usize;
    let mut implicit_mod_lines: Vec<String> = Vec::new();
    let mut explicit_mod_lines: Vec<String> = Vec::new();

    for (i, &raw_line) in raw_lines[mod_start..].iter().enumerate() {
        if !active_for_variant(raw_line, cv) {
            continue;
        }
        let (_, clean) = strip_all_tags(raw_line);
        if clean.is_empty() {
            continue;
        }

        // Substitute (X-Y) ranges with the chosen roll values using find_iter.
        let mut substituted = String::new();
        let mut last_end = 0usize;
        for m in range_re.find_iter(clean) {
            substituted.push_str(&clean[last_end..m.start()]);
            let v = rolls.get(roll_idx).copied().unwrap_or(0.0);
            roll_idx += 1;
            if v == v.trunc() {
                substituted.push_str(&format!("{:.0}", v));
            } else {
                substituted.push_str(&format!("{}", v));
            }
            last_end = m.end();
        }
        substituted.push_str(&clean[last_end..]);

        let is_implicit = i < implicit_count;
        if is_implicit {
            implicit_mod_lines.push(substituted);
        } else {
            explicit_mod_lines.push(substituted);
        }
    }

    // Reconstruct a simplified raw text block (no variant tags — already filtered).
    let mut new_raw = format!(
        "{}\n{}\nImplicits: {}\n",
        def.name,
        base_name,
        implicit_mod_lines.len()
    );
    for l in &implicit_mod_lines {
        new_raw.push_str(l);
        new_raw.push('\n');
    }
    for l in &explicit_mod_lines {
        new_raw.push_str(l);
        new_raw.push('\n');
    }

    // Create a temporary UniqueItemDef with the substituted text and no variants.
    let temp_def = data::uniques::UniqueItemDef {
        name: def.name.clone(),
        base_names: vec![(vec![], base_name)],
        league: def.league.clone(),
        variant_labels: vec![],
        has_alt_variant: false,
        talisman_tier: def.talisman_tier,
        upgrade_target: def.upgrade_target.clone(),
        influences: def.influences,
        raw_text: new_raw,
        file_source: def.file_source.clone(),
    };

    let source = data::SourceId(5000); // non-slot inventory source
    let mut item = item::parser::parse_unique_item(&temp_def, 1, &game, source);

    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.add_item_to_inventory(item);

    let equipped_map = build.equipped.clone();
    build.mod_db_layers.rebuild_items(&equipped_map);
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Returns all base items of the given item class, sorted by level requirement.
#[tauri::command]
#[specta::specta]
fn get_base_items(
    item_class: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<Vec<BaseItemSummary>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let mut results: Vec<BaseItemSummary> = game
        .bases
        .values()
        .filter(|b| b.item_class == item_class)
        .map(|b| BaseItemSummary {
            name: b.name.clone(),
            item_class: b.item_class.clone(),
            level_req: b.drop_level,
        })
        .collect();
    results.sort_by_key(|b| b.level_req);
    Ok(results)
}

/// Returns all distinct item classes available for base-item browsing.
#[tauri::command]
#[specta::specta]
fn get_item_classes(game_data: tauri::State<'_, GdState>) -> Result<Vec<String>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let mut classes: FxHashSet<String> = FxHashSet::default();
    for base in game.bases.values() {
        classes.insert(base.item_class.clone());
    }
    let mut result: Vec<String> = classes.into_iter().collect();
    result.sort();
    Ok(result)
}

/// Returns full properties for a single base item (for the crafting panel).
#[tauri::command]
#[specta::specta]
fn get_base_item_props(
    base_name: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<BaseItemProps, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let b = game
        .bases
        .get(&base_name)
        .ok_or_else(|| format!("Base '{}' not found", base_name))?;
    let p = &b.properties;
    Ok(BaseItemProps {
        name: b.name.clone(),
        item_class: b.item_class.clone(),
        level_req: b.drop_level,
        tags: b.tags.clone(),
        phys_damage_min: p.physical_damage_min,
        phys_damage_max: p.physical_damage_max,
        attack_time_ms: p.attack_time,
        crit_chance_permyriad: p.critical_strike_chance,
        armour_min: p.armour.as_ref().map(|m| m.min),
        armour_max: p.armour.as_ref().map(|m| m.max),
        evasion_min: p.evasion.as_ref().map(|m| m.min),
        evasion_max: p.evasion.as_ref().map(|m| m.max),
        energy_shield_min: p.energy_shield.as_ref().map(|m| m.min),
        energy_shield_max: p.energy_shield.as_ref().map(|m| m.max),
        block: p.block,
        charges_max: p.charges_max,
        life_per_use: p.life_per_use,
        mana_per_use: p.mana_per_use,
    })
}

/// Returns grouped available mods for a given base item name.
/// Implicits are pre-populated from the base's implicit list.
/// Prefixes/suffixes are filtered by spawn weight against the base's tags.
#[tauri::command]
#[specta::specta]
fn get_mods_for_base(
    base_name: String,
    game_data: tauri::State<'_, GdState>,
) -> Result<BaseMods, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let base = game
        .bases
        .get(&base_name)
        .ok_or_else(|| format!("Base '{}' not found", base_name))?;

    let base_tags: FxHashSet<&str> = base.tags.iter().map(|t| t.as_str()).collect();

    // Implicits come from the base item's built-in implicit mod IDs.
    let implicits: Vec<AvailableMod> = base
        .implicits
        .iter()
        .filter_map(|mod_id| {
            game.item_mods
                .get(mod_id)
                .map(|m| repoe_mod_to_available(mod_id, m))
        })
        .collect();

    let mut prefixes: Vec<AvailableMod> = Vec::new();
    let mut suffixes: Vec<AvailableMod> = Vec::new();
    let mut crafted: Vec<AvailableMod> = Vec::new();

    for (mod_id, repoe_mod) in &game.item_mods {
        // Check that at least one of the base's tags has a non-zero spawn weight.
        let can_spawn = repoe_mod
            .spawn_weights
            .iter()
            .any(|sw| sw.weight > 0 && base_tags.contains(sw.tag.as_str()));
        if !can_spawn || repoe_mod.is_essence_only {
            continue;
        }

        let available = repoe_mod_to_available(mod_id, repoe_mod);
        match repoe_mod.domain.as_str() {
            "crafted" => crafted.push(available),
            _ => match repoe_mod.generation_type.as_str() {
                "prefix" => prefixes.push(available),
                "suffix" => suffixes.push(available),
                _ => {}
            },
        }
    }

    prefixes.sort_by_key(|m| m.required_level);
    suffixes.sort_by_key(|m| m.required_level);
    crafted.sort_by_key(|m| m.required_level);

    Ok(BaseMods {
        implicits,
        prefixes,
        suffixes,
        crafted,
    })
}

/// Builds a crafted item from the given spec and adds it to the build inventory.
/// The item does NOT become equipped — call `equip_from_inventory` to equip it.
#[tauri::command]
#[specta::specta]
fn add_crafted_item(
    spec: CraftedItemSpec,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let source = data::SourceId(0);
    let mut item = build_crafted_item(&spec, &game, source)?;

    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.add_item_to_inventory(item);

    // Stats unchanged (item not equipped yet — inventory only).
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Returns all items in the build (equipped + unequipped inventory).
#[tauri::command]
#[specta::specta]
fn get_inventory_items(
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<Vec<InventoryItemSummary>, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let mut items: Vec<InventoryItemSummary> = build
        .inventory
        .iter()
        .map(|item| item_to_inventory_summary(item, None))
        .collect();
    for (&slot, item) in &build.equipped {
        items.push(item_to_inventory_summary(item, Some(slot)));
    }
    items.sort_by_key(|i| i.inventory_id);
    Ok(items)
}

/// Returns unequipped inventory items that are compatible with the given slot.
#[tauri::command]
#[specta::specta]
fn get_inventory_for_slot(
    slot: ItemSlot,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<Vec<InventoryItemSummary>, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let weapon1_type: Option<ItemType> =
        build.equipped.get(&ItemSlot::Weapon1).map(|i| i.item_type);
    let items = build
        .inventory
        .iter()
        .filter(|item| ItemSlot::is_compatible(slot, item.item_type, weapon1_type))
        .map(|item| item_to_inventory_summary(item, None))
        .collect();
    Ok(items)
}

/// Equips an item from the unequipped inventory into the given slot.
/// If another item is already in that slot it is moved to inventory.
#[tauri::command]
#[specta::specta]
fn equip_from_inventory(
    inventory_id: u32,
    slot: ItemSlot,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;

    let pos = build
        .inventory
        .iter()
        .position(|i| i.inventory_id == inventory_id)
        .ok_or_else(|| format!("Item {} not found in inventory", inventory_id))?;

    let weapon1_type: Option<ItemType> = if slot == ItemSlot::Weapon2 {
        build.equipped.get(&ItemSlot::Weapon1).map(|i| i.item_type)
    } else {
        None
    };
    let item_type = build.inventory[pos].item_type;
    if !ItemSlot::is_compatible(slot, item_type, weapon1_type) {
        return Err(format!("{:?} cannot go in slot {:?}", item_type, slot));
    }

    let item = build.inventory.remove(pos);
    if let Some(displaced) = build.equipped.remove(&slot) {
        build.inventory.push(displaced);
    }
    build.equipped.insert(slot, item);

    let equipped_map = build.equipped.clone();
    build.mod_db_layers.rebuild_items(&equipped_map);
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Permanently removes an item from the build inventory (and unequips it first if equipped).
#[tauri::command]
#[specta::specta]
fn remove_inventory_item(
    inventory_id: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    // Remove from unequipped inventory.
    build.inventory.retain(|i| i.inventory_id != inventory_id);
    // Also unequip if currently equipped.
    let was_equipped = build
        .equipped
        .iter()
        .find(|(_, i)| i.inventory_id == inventory_id)
        .map(|(&slot, _)| slot);
    if let Some(slot) = was_equipped {
        build.equipped.remove(&slot);
    }
    let equipped_map = build.equipped.clone();
    build.mod_db_layers.rebuild_items(&equipped_map);
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let stats = compute_stats(
        &build.mod_db_layers,
        build.level,
        &build.class,
        build.active_gem.as_ref(),
        &build.skill_groups,
        &build.equipped,
        game,
        &build.selected_nodes,
    );
    build.stats = stats.clone();
    Ok(stats)
}

/// Convert an `Item` + optional slot into an `InventoryItemSummary`.
fn item_to_detail(item: &Item) -> ItemDetail {
    let influences = item.influences.bits();

    let (
        phys_damage_min,
        phys_damage_max,
        attacks_per_second,
        crit_chance,
        total_dps,
        phys_dps,
        ele_dps,
    ) = if let Some(wd) = &item.weapon_data {
        (
            Some(wd.phys_min),
            Some(wd.phys_max),
            Some(wd.attacks_per_second),
            Some(wd.crit_chance),
            Some(wd.total_dps),
            Some(wd.phys_dps),
            Some(wd.ele_dps),
        )
    } else {
        (None, None, None, None, None, None, None)
    };

    let (armour, evasion, energy_shield, block) = if let Some(ad) = &item.armour_data {
        (
            if ad.armour > 0.0 {
                Some(ad.armour)
            } else {
                None
            },
            if ad.evasion > 0.0 {
                Some(ad.evasion)
            } else {
                None
            },
            if ad.energy_shield > 0.0 {
                Some(ad.energy_shield)
            } else {
                None
            },
            if ad.block > 0 { Some(ad.block) } else { None },
        )
    } else {
        (None, None, None, None)
    };

    fn line_kind(src: ModLineSource) -> &'static str {
        match src {
            ModLineSource::Implicit => "implicit",
            ModLineSource::Explicit => "explicit",
            ModLineSource::Crafted => "crafted",
            ModLineSource::Enchant => "enchant",
            ModLineSource::Fractured => "fractured",
        }
    }

    let enchant_lines = item
        .enchant_lines
        .iter()
        .map(|l| ItemModLine {
            text: l.text.clone(),
            kind: line_kind(l.source).to_string(),
        })
        .collect();
    let implicit_lines = item
        .implicit_lines
        .iter()
        .map(|l| ItemModLine {
            text: l.text.clone(),
            kind: line_kind(l.source).to_string(),
        })
        .collect();
    let mut explicit_lines: Vec<ItemModLine> = item
        .explicit_lines
        .iter()
        .map(|l| ItemModLine {
            text: l.text.clone(),
            kind: line_kind(l.source).to_string(),
        })
        .collect();
    explicit_lines.extend(item.crafted_lines.iter().map(|l| ItemModLine {
        text: l.text.clone(),
        kind: "crafted".to_string(),
    }));

    ItemDetail {
        inventory_id: item.inventory_id,
        name: item.name.clone(),
        base_name: item.base_name.clone(),
        item_class: item.item_class.clone(),
        rarity: item.rarity,
        item_level: item.item_level,
        quality: item.quality,
        corrupted: item.corrupted,
        mirrored: item.mirrored,
        synthesised: item.synthesised,
        fractured: item.fractured,
        influences,
        req_level: item.requirements.level,
        req_str: item.requirements.strength,
        req_dex: item.requirements.dexterity,
        req_int: item.requirements.intelligence,
        phys_damage_min,
        phys_damage_max,
        attacks_per_second,
        crit_chance,
        total_dps,
        phys_dps,
        ele_dps,
        armour,
        evasion,
        energy_shield,
        block,
        enchant_lines,
        implicit_lines,
        explicit_lines,
    }
}

fn item_to_inventory_summary(item: &Item, equipped_slot: Option<ItemSlot>) -> InventoryItemSummary {
    InventoryItemSummary {
        inventory_id: item.inventory_id,
        name: item.name.clone(),
        base_name: item.base_name.clone(),
        item_class: item.item_class.clone(),
        rarity: item.rarity,
        total_dps: item.weapon_data.as_ref().map(|w| w.total_dps),
        armour: item
            .armour_data
            .as_ref()
            .map(|a| a.armour)
            .filter(|&v| v > 0.0),
        evasion: item
            .armour_data
            .as_ref()
            .map(|a| a.evasion)
            .filter(|&v| v > 0.0),
        energy_shield: item
            .armour_data
            .as_ref()
            .map(|a| a.energy_shield)
            .filter(|&v| v > 0.0),
        mod_count: (item.implicit_lines.len()
            + item.explicit_lines.len()
            + item.crafted_lines.len()
            + item.enchant_lines.len()) as u32,
        equipped_slot,
    }
}

/// Convert a `RePoEMod` entry into the `AvailableMod` IPC type.
fn repoe_mod_to_available(mod_id: &str, repoe_mod: &RePoEMod) -> AvailableMod {
    AvailableMod {
        mod_id: mod_id.to_string(),
        name: repoe_mod.name.clone(),
        generation_type: repoe_mod.generation_type.clone(),
        groups: repoe_mod.groups.clone(),
        required_level: repoe_mod.required_level,
        stats: repoe_mod
            .stats
            .iter()
            .map(|s| AvailableModStat {
                stat_id: s.id.clone(),
                min: s.min,
                max: s.max,
            })
            .collect(),
    }
}

// ─── Base category helpers ────────────────────────────────────────────────────

fn armour_type_label(tags: &[String]) -> &'static str {
    let str = tags.iter().any(|t| t == "str_armour");
    let dex = tags.iter().any(|t| t == "dex_armour");
    let int = tags.iter().any(|t| t == "int_armour");
    match (str, dex, int) {
        (true, false, false) => "Armour",
        (false, true, false) => "Evasion",
        (false, false, true) => "Energy Shield",
        (true, true, false) => "Armour/Evasion",
        (true, false, true) => "Armour/Energy Shield",
        (false, true, true) => "Evasion/Energy Shield",
        _ => "Mixed",
    }
}

fn classify_base(item_class: &str, tags: &[String]) -> Option<(&'static str, String)> {
    use crate::data::bases::EQUIPPABLE_CLASSES;
    if !EQUIPPABLE_CLASSES.contains(&item_class) {
        return None;
    }
    let sub = match item_class {
        // ── Weapons ──────────────────────────────────────────
        "One Hand Sword" | "Thrusting One Hand Sword" => "Swords (1H)".to_string(),
        "One Hand Axe" => "Axes (1H)".to_string(),
        "One Hand Mace" => "Maces (1H)".to_string(),
        "Claw" => "Claws".to_string(),
        "Dagger" | "Rune Dagger" => "Daggers".to_string(),
        "Wand" => "Wands".to_string(),
        "Sceptre" => "Sceptres".to_string(),
        "Two Hand Sword" => "Swords (2H)".to_string(),
        "Two Hand Axe" => "Axes (2H)".to_string(),
        "Two Hand Mace" => "Maces (2H)".to_string(),
        "Staff" | "Warstaff" => "Staves".to_string(),
        "Bow" => "Bows".to_string(),
        "FishingRod" => "Other".to_string(),
        // ── Armour ───────────────────────────────────────────
        "Helmet" => format!("{} Helmets", armour_type_label(tags)),
        "Body Armour" => format!("{} Body Armours", armour_type_label(tags)),
        "Gloves" => format!("{} Gloves", armour_type_label(tags)),
        "Boots" => format!("{} Boots", armour_type_label(tags)),
        "Shield" => format!("{} Shields", armour_type_label(tags)),
        // ── Accessories ──────────────────────────────────────
        "Amulet" => "Amulets".to_string(),
        "Ring" => "Rings".to_string(),
        "Belt" => "Belts".to_string(),
        "Quiver" => "Quivers".to_string(),
        // ── Flasks ───────────────────────────────────────────
        "LifeFlask" => "Life Flasks".to_string(),
        "ManaFlask" => "Mana Flasks".to_string(),
        "HybridFlask" => "Hybrid Flasks".to_string(),
        "UtilityFlask" => "Utility Flasks".to_string(),
        // ── Jewels ───────────────────────────────────────────
        "Jewel" => "Jewels".to_string(),
        "AbyssJewel" => "Abyss Jewels".to_string(),
        "Tincture" => "Tinctures".to_string(),
        _ => return None,
    };

    let cat: &'static str = match item_class {
        "One Hand Sword"
        | "Thrusting One Hand Sword"
        | "One Hand Axe"
        | "One Hand Mace"
        | "Claw"
        | "Dagger"
        | "Rune Dagger"
        | "Wand"
        | "Sceptre"
        | "Two Hand Sword"
        | "Two Hand Axe"
        | "Two Hand Mace"
        | "Staff"
        | "Warstaff"
        | "Bow"
        | "FishingRod" => "Weapons",
        "Helmet" | "Body Armour" | "Gloves" | "Boots" | "Shield" => "Armour",
        "Amulet" | "Ring" | "Belt" | "Quiver" => "Accessories",
        "LifeFlask" | "ManaFlask" | "HybridFlask" | "UtilityFlask" => "Flasks",
        "Jewel" | "AbyssJewel" | "Tincture" => "Jewels",
        _ => return None,
    };
    Some((cat, sub))
}

/// Returns the full hierarchical category tree for the crafting base browser.
#[tauri::command]
#[specta::specta]
fn get_base_categories(game_data: tauri::State<'_, GdState>) -> Result<Vec<BaseCategory>, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;

    use std::collections::BTreeMap;
    let mut tree: BTreeMap<&'static str, BTreeMap<String, Vec<BaseItemSummary>>> = BTreeMap::new();

    for base in game.bases.values() {
        if let Some((cat, sub)) = classify_base(&base.item_class, &base.tags) {
            let summary = BaseItemSummary {
                name: base.name.clone(),
                item_class: base.item_class.clone(),
                level_req: base.requirements.level,
            };
            tree.entry(cat)
                .or_default()
                .entry(sub)
                .or_default()
                .push(summary);
        }
    }

    // Sort bases within each subcategory by level requirement.
    for cat_map in tree.values_mut() {
        for bases in cat_map.values_mut() {
            bases.sort_by_key(|b| b.level_req);
        }
    }

    // Emit in a fixed category order.
    let category_order: &[&str] = &["Weapons", "Armour", "Accessories", "Flasks", "Jewels"];
    let mut result: Vec<BaseCategory> = category_order
        .iter()
        .filter_map(|&cat_name| {
            tree.remove(cat_name).map(|subcat_map| BaseCategory {
                name: cat_name.to_string(),
                subcategories: subcat_map
                    .into_iter()
                    .map(|(label, bases)| BaseSubcategory { label, bases })
                    .collect(),
            })
        })
        .collect();

    // Append the virtual "Unique" category — no subcategories; the frontend
    // renders its own slot-filter + search UI when this category is selected.
    result.push(BaseCategory {
        name: "Unique".to_string(),
        subcategories: vec![],
    });

    Ok(result)
}

// ─── Tiered mod helpers ───────────────────────────────────────────────────────

fn make_mod_display_name(
    m: &data::item_mods::RePoEMod,
    translations: &data::stat_translations::InvertedTranslations,
) -> String {
    // Try the first stat's translation template.
    if let Some(stat) = m.stats.first() {
        if let Some(template) = translations.stat_display_template(&stat.id) {
            return template.to_string();
        }
    }
    // Fallback: none of the stats had a translation — use the mod name.
    if !m.name.is_empty() {
        return m.name.clone();
    }
    // Last resort: join stat IDs with basic formatting.
    m.stats
        .iter()
        .map(|s| {
            s.id.replace("local_", "")
                .replace("_+%", " %")
                .replace('_', " ")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn group_into_stat_mod_groups(
    mods: Vec<(&str, &data::item_mods::RePoEMod)>,
    translations: &data::stat_translations::InvertedTranslations,
) -> Vec<StatModGroup> {
    use std::collections::BTreeMap;

    // Key: (first_group_name, sorted_stat_ids) — uniquely identifies a "stat family".
    let mut families: BTreeMap<(String, String), Vec<(String, &data::item_mods::RePoEMod)>> =
        BTreeMap::new();

    for (mod_id, m) in mods {
        let group_key = m
            .groups
            .first()
            .cloned()
            .unwrap_or_else(|| mod_id.to_string());
        let mut stat_ids: Vec<&str> = m.stats.iter().map(|s| s.id.as_str()).collect();
        stat_ids.sort_unstable();
        let stat_sig = stat_ids.join(";");
        families
            .entry((group_key, stat_sig))
            .or_default()
            .push((mod_id.to_string(), m));
    }

    let mut result: Vec<StatModGroup> = families
        .into_values()
        .map(|mut tier_mods| {
            // Highest required_level first → T1.
            tier_mods.sort_by(|a, b| b.1.required_level.cmp(&a.1.required_level));

            let first = tier_mods[0].1;
            let display_name = make_mod_display_name(first, translations);
            let gen_type = first.generation_type.clone();
            let group = first.groups.first().cloned().unwrap_or_default();

            let tiers: Vec<ModTierInfo> = tier_mods
                .iter()
                .enumerate()
                .map(|(i, (mid, m))| ModTierInfo {
                    mod_id: mid.clone(),
                    tier: (i + 1) as u32,
                    required_level: m.required_level,
                    stats: m
                        .stats
                        .iter()
                        .map(|s| AvailableModStat {
                            stat_id: s.id.clone(),
                            min: s.min,
                            max: s.max,
                        })
                        .collect(),
                })
                .collect();

            StatModGroup {
                display_name,
                generation_type: gen_type,
                group,
                tiers,
            }
        })
        .collect();

    result.sort_by(|a, b| a.display_name.cmp(&b.display_name));
    result
}

/// Returns mods for a base item organised into tiered stat groups.
/// Pass `influence` = "shaper" | "elder" | "crusader" | "hunter" | "redeemer" | "warlord"
/// (or omit) to include influence-exclusive modifiers.
#[tauri::command]
#[specta::specta]
fn get_mods_for_base_grouped(
    base_name: String,
    influence: Option<String>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BaseModGroups, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let base = game
        .bases
        .get(&base_name)
        .ok_or_else(|| format!("Base '{}' not found", base_name))?;

    let mut base_tags: FxHashSet<&str> = base.tags.iter().map(|t| t.as_str()).collect();

    // Inject influence tag so influenced mods become visible.
    if let Some(ref inf) = influence {
        let tag = match inf.as_str() {
            "shaper" => "shaper_item",
            "elder" => "elder_item",
            "crusader" => "crusader_item",
            "hunter" => "hunter_item",
            "redeemer" => "redeemer_item",
            "warlord" => "warlord_item",
            _ => "",
        };
        if !tag.is_empty() {
            base_tags.insert(tag);
        }
    }

    let mut prefix_raw: Vec<(&str, &RePoEMod)> = Vec::new();
    let mut suffix_raw: Vec<(&str, &RePoEMod)> = Vec::new();
    let mut crafted_raw: Vec<(&str, &RePoEMod)> = Vec::new();

    let base_domain = base.domain.as_str();

    for (mod_id, m) in &game.item_mods {
        // Domain filter: mod domain must match the base item's domain or be a bench craft.
        // This prevents flask mods (domain "flask") from appearing on armour (domain "item").
        if m.domain != "crafted" && m.domain.as_str() != base_domain {
            continue;
        }
        // Ordered first-match spawn weight check (PoE semantics):
        // iterate entries in order; the FIRST entry whose tag is in the base's tags wins.
        // If that weight is 0 (or no tag matches), the mod cannot spawn here.
        let effective_weight = m
            .spawn_weights
            .iter()
            .find(|sw| base_tags.contains(sw.tag.as_str()))
            .map(|sw| sw.weight)
            .unwrap_or(0);
        if effective_weight == 0 || m.is_essence_only {
            continue;
        }
        match m.domain.as_str() {
            "crafted" => crafted_raw.push((mod_id.as_str(), m)),
            _ => match m.generation_type.as_str() {
                "prefix" => prefix_raw.push((mod_id.as_str(), m)),
                "suffix" => suffix_raw.push((mod_id.as_str(), m)),
                _ => {}
            },
        }
    }

    // Implicits come from the base item's explicit implicit list.
    let implicit_raw: Vec<(&str, &RePoEMod)> = base
        .implicits
        .iter()
        .filter_map(|mod_id| {
            game.item_mods
                .get(mod_id.as_str())
                .map(|m| (mod_id.as_str(), m))
        })
        .collect();

    Ok(BaseModGroups {
        implicits: group_into_stat_mod_groups(implicit_raw, &game.translations),
        prefixes: group_into_stat_mod_groups(prefix_raw, &game.translations),
        suffixes: group_into_stat_mod_groups(suffix_raw, &game.translations),
        crafted: group_into_stat_mod_groups(crafted_raw, &game.translations),
    })
}

/// Private helper: compute BuildStats from a merged ModDB.
/// Calls `calc_defense` for all defensive stats.
/// Pass `offence_ctx` to include an `OffenceResult` for the active gem.
fn compute_stats(
    layers: &modifier::ModDBLayers,
    level: u32,
    class: &Class,
    active_gem: Option<&GemRef>,
    skill_groups: &[SkillGroup],
    equipped: &FxHashMap<ItemSlot, Item>,
    game_data: &data::GameData,
    selected_nodes: &BuildSelection,
) -> BuildStats {
    let t0 = std::time::Instant::now();
    let result = calc::calculate(
        layers,
        level,
        class,
        active_gem,
        skill_groups,
        equipped,
        game_data,
    );
    let calc_time_us = t0.elapsed().as_micros() as u32;
    BuildStats {
        total_strength: result.attributes.strength,
        total_dexterity: result.attributes.dexterity,
        total_intelligence: result.attributes.intelligence,
        node_count: selected_nodes.selected_node_ids.len() as u32,
        defence: result.defense,
        offence: if active_gem.is_some() {
            Some(result.offence)
        } else {
            None
        },
        calc_time_us,
    }
}

#[tauri::command]
#[specta::specta]
fn list_builds(storage: tauri::State<'_, StorageManager>) -> Result<Vec<BuildSummary>, String> {
    storage.list_builds().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn save_build(
    name: String,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    storage: tauri::State<'_, StorageManager>,
) -> Result<String, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.name = name;
    let save_data = SavedBuildData::from_build(&build, DEFAULT_TREE_VERSION.to_string());
    storage.save_build(&save_data).map_err(|e| e.to_string())
}
#[tauri::command]
#[specta::specta]
fn load_build(
    id: String,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    storage: tauri::State<'_, StorageManager>,
    game_data: tauri::State<'_, GdState>,
) -> Result<BuildInfo, String> {
    let gd_lock = game_data.read().map_err(|e| e.to_string())?;
    let game = get_game(&*gd_lock)?;
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let saved_data = storage.load_build(&id).map_err(|e| e.to_string())?;
    let loaded = saved_data.into_build(game);
    let stats = compute_stats(
        &loaded.mod_db_layers,
        loaded.level,
        &loaded.class,
        loaded.active_gem.as_ref(),
        &loaded.skill_groups,
        &loaded.equipped,
        game,
        &loaded.selected_nodes,
    );
    *build = loaded;
    build.stats = stats;
    // Serialize the non-skipped fields for the frontend via serde.
    let response: BuildInfo =
        serde_json::from_value(serde_json::to_value(&*build).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(response)
}

#[tauri::command]
#[specta::specta]
fn delete_build(id: String, storage: tauri::State<'_, StorageManager>) -> Result<(), String> {
    storage.delete_build(&id).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
fn rename_build(
    id: String,
    new_name: String,
    storage: tauri::State<'_, StorageManager>,
) -> Result<(), String> {
    storage
        .rename_build(&id, &new_name)
        .map_err(|e| e.to_string())
}
