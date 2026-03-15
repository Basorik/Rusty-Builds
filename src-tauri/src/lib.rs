mod client;
mod models;
mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::collections::{HashMap, HashSet};
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

// ---------------------------------------------------------------------------
// Tree data: node ID → list of stat strings, loaded once at startup
// ---------------------------------------------------------------------------

/// Holds the parsed stat strings for every node, keyed by node ID.
pub struct TreeData {
    /// node_id → vec of raw stat strings from data.json
    pub node_stats: HashMap<u32, Vec<String>>,
}

impl TreeData {
    /// Parse `data.json` content and extract each node's `stats` array.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let root: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;

        let nodes_obj = root
            .get("nodes")
            .and_then(|v| v.as_object())
            .ok_or_else(|| "Missing 'nodes' object in tree data".to_string())?;

        let mut node_stats: HashMap<u32, Vec<String>> = HashMap::with_capacity(nodes_obj.len());

        for (id_str, node_val) in nodes_obj {
            let id: u32 = match id_str.parse() {
                Ok(n) => n,
                Err(_) => continue, // skip "root" or other non-numeric keys
            };

            if let Some(stats_arr) = node_val.get("stats").and_then(|v| v.as_array()) {
                let stats: Vec<String> = stats_arr
                    .iter()
                    .filter_map(|s| s.as_str().map(String::from))
                    .collect();
                if !stats.is_empty() {
                    node_stats.insert(id, stats);
                }
            }
        }

        info!("Loaded stats for {} nodes", node_stats.len());
        Ok(TreeData { node_stats })
    }
}

// ---------------------------------------------------------------------------
// Stat accumulator: parses numbers from stat strings and sums by template
// ---------------------------------------------------------------------------

use regex::Regex;
use std::sync::LazyLock;

/// Matches signed integers and decimals like 50, +30, -5, 0.5, +12.3
static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[+-]?\d+\.?\d*").unwrap());

pub struct StatAccumulator {
    totals: HashMap<String, f64>,
}

impl StatAccumulator {
    pub fn new() -> Self {
        Self {
            totals: HashMap::new(),
        }
    }

    /// Add a single stat string. Extracts the first number, replaces all numbers
    /// with `#` to form a template key, and accumulates the value.
    /// Stats with no number get stored with value += 1 (source count).
    pub fn add(&mut self, stat: &str) {
        if let Some(m) = NUM_RE.find(stat) {
            let value: f64 = m.as_str().parse().unwrap_or(0.0);
            let template = NUM_RE.replace_all(stat, "#").to_string();
            *self.totals.entry(template).or_insert(0.0) += value;
        } else {
            // Boolean/qualitative stat — count occurrences
            *self.totals.entry(stat.to_string()).or_insert(0.0) += 1.0;
        }
    }

    /// Add all stats for a node.
    pub fn add_all(&mut self, stats: &[String]) {
        for s in stats {
            self.add(s);
        }
    }

    /// Consume and return the accumulated totals.
    pub fn into_totals(self) -> HashMap<String, f64> {
        self.totals
    }
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

            // Load tree data (node stats) from the active versioned tree file.
            // Switch versions by updating the src-tauri/data/tree/active.json symlink
            // (done automatically by `bun run tool:fetch-tree`).
            const TREE_JSON: &str = include_str!("../data/tree/active.json");
            let tree_data = TreeData::from_json(TREE_JSON)
                .expect("Failed to parse tree data");
            app.manage(tree_data);

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
    tree_data: tauri::State<'_, TreeData>,
) -> Result<BuildStats, String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.selected_nodes.selected_node_ids = node_ids.into_iter().collect();

    // Accumulate stats from all selected nodes
    let mut acc = StatAccumulator::new();
    for &node_id in &build_info.selected_nodes.selected_node_ids {
        if let Some(stats) = tree_data.node_stats.get(&node_id) {
            acc.add_all(stats);
        }
    }

    let stat_totals = acc.into_totals();
    let node_count = build_info.selected_nodes.selected_node_ids.len() as u32;

    let stats = BuildStats {
        total_strength: 0,
        total_dexterity: 0,
        total_intelligence: 0,
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
