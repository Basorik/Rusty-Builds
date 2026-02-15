mod client;
mod models;
mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

/// Tracks which skill-tree nodes the user has selected for the current build.
#[derive(Debug, Default, Serialize, Deserialize, Type)]
pub struct BuildSelection {
    selected_node_ids: HashSet<u32>,
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum NodeType {
    Notable,
    Keystone,
    Regular,
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
    Crusader,
    Redeemer,
    Hunter,
    Assassin,
    Champion,
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
        update_build_info
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

    // In a real implementation, you would iterate over build_info.selected_nodes.selected_node_ids,
    // look up the node stats from your loaded Graph/HashMap, and sum them up.
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
) -> Result<BuildStats, String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.selected_nodes.selected_node_ids = node_ids.into_iter().collect();

    // In a real implementation, you would iterate over build_info.selected_nodes.selected_node_ids,
    // look up the node stats from your loaded Graph/HashMap, and sum them up.
    let stats = BuildStats {
        total_strength: 0, // Placeholder: Calculate actual stats here
        total_dexterity: 0,
        total_intelligence: 0,
        node_count: build_info.selected_nodes.selected_node_ids.len() as u32,
    };

    info!(
        "Build selection updated: {} nodes selected",
        stats.node_count
    );
    Ok(stats)
}
