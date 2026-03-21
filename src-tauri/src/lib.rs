mod client;
mod data;
mod models;
mod stats;
mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use stats::StatAccumulator;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

/// The tree version loaded on startup and used as the default.
pub const DEFAULT_TREE_VERSION: &str = "3.27.0g";

/// Tracks which skill-tree nodes the user has selected for the current build.
#[derive(Debug, Default, Serialize, Deserialize, Type)]
pub struct BuildSelection {
    selected_node_ids: HashSet<u32>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(tag = "class", content = "ascendancy")]
pub enum Class {
    Marauder(Option<MarauderAscendancy>),
    Ranger(Option<RangerAscendancy>),
    Witch(Option<WitchAscendancy>),
    Duelist(Option<DuelistAscendancy>),
    Templar(Option<TemplarAscendancy>),
    Shadow(Option<ShadowAscendancy>),
    Scion(Option<ScionAscendancy>),
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum Bloodline {
    None,
    Aul,
    Breachlord,
    Catarina,
    Delirious,
    Farrul,
    KingInTheMists,
    Lycia,
    Olroth,
    Oshabi,
    Primalist,
    Trialmaster,
    Warden,
    Warlock,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum MarauderAscendancy {
    Juggernaut,
    Berserker,
    Chieftain,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum RangerAscendancy {
    Raider,
    Deadeye,
    Pathfinder,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum WitchAscendancy {
    Necromancer,
    Occultist,
    Elementalist,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum DuelistAscendancy {
    Slayer,
    Gladiator,
    Champion,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum TemplarAscendancy {
    Inquisitor,
    Hierophant,
    Guardian,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum ShadowAscendancy {
    Assassin,
    Saboteur,
    Trickster,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum ScionAscendancy {
    Ascendant,
}

#[derive(Default, Debug, Serialize, Deserialize, Type)]
pub struct BuildStats {
    pub total_strength: i32,
    pub total_dexterity: i32,
    pub total_intelligence: i32,
    pub node_count: u32,
    /// Accumulated stats: template key → summed numeric value.
    /// e.g. "#% increased maximum Life" → 53.0
    /// Boolean/qualitative stats use the full string as key with value = count of sources.
    pub stat_totals: HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct BuildInfo {
    pub name: String,
    pub level: u32,
    pub stats: BuildStats,
    pub class: Class,
    pub bloodline: Bloodline,
    pub selected_nodes: BuildSelection,
    // Add more fields as needed, e.g., list of selected nodes, build name, etc.
}

impl Default for BuildInfo {
    fn default() -> Self {
        BuildInfo {
            name: "Unsaved Build".to_string(),
            level: 1,
            stats: BuildStats::default(),
            class: Class::Scion(None),
            bloodline: Bloodline::None,
            selected_nodes: BuildSelection::default(),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        greet,
        update_selected_nodes,
        update_build_info,
        get_available_tree_versions,
        load_tree_version,
        get_tree_json
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
            info!("Initializing Storage Manager...");
            let handle = app.handle();
            let storage_manager = storage::StorageManager::new(&handle);
            app.manage(storage_manager);
            app.manage(Mutex::new(BuildInfo::default()));

            let tree_path = handle
                .path()
                .resource_dir()
                .expect("Failed to resolve resource directory")
                .join("data/tree")
                .join(DEFAULT_TREE_VERSION)
                .join("data.json");
            let tree_json =
                std::fs::read_to_string(&tree_path).expect("Failed to read default tree data");

            let game_data =
                data::GameData::load_from_json(&tree_json).expect("Failed to load game data");

            info!(
                "Loaded {} nodes from passive tree",
                game_data.tree.nodes.len()
            );
            app.manage(Arc::new(RwLock::new(game_data)));

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
) -> Result<(), String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.level = level;
    build_info.class = character_class;
    build_info.bloodline = bloodline;

    info!(
        "Build  updated: Level {}, Class {:?}, Bloodline {:?}",
        build_info.level, build_info.class, build_info.bloodline
    );
    Ok(())
}

/// Receives the current set of selected node IDs from the frontend.
/// Stores them and returns the total count (placeholder for future stat calculations).
#[tauri::command]
#[specta::specta]
fn update_selected_nodes(
    node_ids: Vec<u32>,
    state: tauri::State<'_, Mutex<BuildInfo>>,
    game_data_state: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<BuildStats, String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.selected_nodes.selected_node_ids = node_ids.into_iter().collect();

    let game_data = game_data_state.read().map_err(|e| e.to_string())?;

    // Accumulate stats from all selected nodes
    let mut acc = StatAccumulator::new();
    let mut total_strength: i32 = 0;
    let mut total_dexterity: i32 = 0;
    let mut total_intelligence: i32 = 0;
    for &node_id in &build_info.selected_nodes.selected_node_ids {
        if let Some(node) = game_data.tree.get_node(node_id) {
            acc.add_all(&node.stats);
            total_strength += node.granted_strength as i32;
            total_dexterity += node.granted_dexterity as i32;
            total_intelligence += node.granted_intelligence as i32;
        }
    }

    let stat_totals = acc.into_totals();
    let node_count = build_info.selected_nodes.selected_node_ids.len() as u32;

    let stats = BuildStats {
        total_strength,
        total_dexterity,
        total_intelligence,
        node_count,
        stat_totals,
    };

    info!(
        "Build selection updated: {} nodes, {} unique stat lines",
        stats.node_count,
        stats.stat_totals.len()
    );
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
    game_data_state: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<(), String> {
    // Validate version to prevent path traversal
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err("Invalid version string".to_string());
    }

    let tree_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to resolve resource dir: {}", e))?
        .join("data/tree")
        .join(&version)
        .join("data.json");

    let tree_json =
        std::fs::read_to_string(&tree_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let game_data = data::GameData::load_from_json(&tree_json)
        .map_err(|e| format!("Failed to load game data: {}", e))?;

    *game_data_state.write().map_err(|e| e.to_string())? = game_data;

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
