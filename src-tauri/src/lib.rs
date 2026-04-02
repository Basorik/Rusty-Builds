mod client;
mod data;
mod models;
mod modifier;
mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use crate::data::gems::GemSummary;
use crate::data::skills::{self, GemInstance, SkillGroup, SupportCompatEntry};
use crate::data::SourceId;
use crate::modifier::ModDB;

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

#[derive(Default, Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildStats {
    pub total_strength: i32,
    pub total_dexterity: i32,
    pub total_intelligence: i32,
    pub node_count: u32,
    pub life: u32,
    pub mana: u32,
    // Accumulated stats: template key → summed numeric value.
    // e.g. "#% increased maximum Life" → 53.0
    // Boolean/qualitative stats use the full string as key with value = count of sources.
    // pub stat_totals: HashMap<String, f64>,
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
    next_group_id: u32,
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
            skill_groups: Vec::new(),
            next_group_id: 1,
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
        get_tree_json,
        get_gem_list,
        get_skill_groups,
        create_skill_group,
        delete_skill_group,
        add_gem_to_group,
        remove_gem_from_group,
        get_group_effects
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

            let resource_path = handle
                .path()
                .resource_dir()
                .expect("Failed to resolve resource directory");

            let game_data =
                data::GameData::load_from_dir(resource_path).expect("Failed to load game data");

            info!(
                "Loaded {} nodes from passive tree, {} gems",
                game_data.tree.nodes.len(),
                game_data.gems.len()
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
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<BuildStats, String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    build.selected_nodes.selected_node_ids = node_ids.iter().cloned().collect();

    let game = game_data.read().map_err(|e| e.to_string())?;

    let mut mod_db = modifier::ModDB::new();
    let ctx = modifier::CalcContext::empty();

    for &node_id in &node_ids {
        if let Some(node) = game.tree.get_node(node_id) {
            let source = SourceId(node_id);
            for stat_text in &node.stats {
                for modifier in modifier::parser::parse_display_text(stat_text, source) {
                    mod_db.add_mod(modifier);
                }
            }
        }
    }
    add_class_base_stats(&mut mod_db, &build.class, &game.tree);

    let total_str = mod_db.sum_base(data::StatId::Strength, &ctx);
    let total_dex = mod_db.sum_base(data::StatId::Dexterity, &ctx);
    let total_int = mod_db.sum_base(data::StatId::Intelligence, &ctx);

    // Life = floor((base + level*12 + tree flat) * (1 + inc%) * more) + floor(Str/2)
    // Per PoB CalcSetup.lua: base=38, life_per_level=12
    let life_base = 38.0 + (build.level as f64 * 12.0) + mod_db.sum_base(data::StatId::Life, &ctx);
    let life_inc = mod_db.sum_inc(data::StatId::Life, &ctx);
    let life_more = mod_db.product_more(data::StatId::Life, &ctx);
    let life = (life_base * (1.0 + life_inc / 100.0) * life_more).floor() as i64
        + (total_str / 2.0).floor() as i64;

    // Mana = floor((base + level*6 + tree flat) * (1 + inc%) * more) + floor(Int/2)
    // Per PoB CalcSetup.lua: base=34, mana_per_level=6
    let mana_base = 34.0 + (build.level as f64 * 6.0) + mod_db.sum_base(data::StatId::Mana, &ctx);
    let mana_inc = mod_db.sum_inc(data::StatId::Mana, &ctx);
    let mana_more = mod_db.product_more(data::StatId::Mana, &ctx);
    let mana = (mana_base * (1.0 + mana_inc / 100.0) * mana_more).floor() as i64
        + (total_int / 2.0).floor() as i64;

    let stats = BuildStats {
        total_dexterity: total_dex as i32,
        total_intelligence: total_int as i32,
        total_strength: total_str as i32,
        node_count: node_ids.len() as u32,
        life: life as u32,
        mana: mana as u32,
    };

    build.stats = stats.clone();
    Ok(stats)
}

fn add_class_base_stats(mod_db: &mut ModDB, class: &Class, tree: &data::PassiveTree) {
    // Map the Class enum to the class index in the tree data
    let class_index = match class {
        Class::Scion(_) => 0,
        Class::Marauder(_) => 1,
        Class::Ranger(_) => 2,
        Class::Witch(_) => 3,
        Class::Duelist(_) => 4,
        Class::Templar(_) => 5,
        Class::Shadow(_) => 6,
    };

    if let Some(class_data) = tree.classes.get(class_index) {
        let source = SourceId(0); // class base stats source
        mod_db.add_mod(modifier::parser::simple_mod(
            data::StatId::Strength,
            modifier::ModType::Base,
            class_data.base_str as f64,
            source,
        ));
        mod_db.add_mod(modifier::parser::simple_mod(
            data::StatId::Dexterity,
            modifier::ModType::Base,
            class_data.base_dex as f64,
            source,
        ));
        mod_db.add_mod(modifier::parser::simple_mod(
            data::StatId::Intelligence,
            modifier::ModType::Base,
            class_data.base_int as f64,
            source,
        ));
    }
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

    let resource_path = app
        .path()
        .resource_dir()
        .expect("Failed to resolve resource directory");

    let game_data = data::GameData::load_from_dir(resource_path).expect("Failed to load game data");

    info!(
        "Loaded {} nodes from passive tree, {} gems",
        game_data.tree.nodes.len(),
        game_data.gems.len()
    );

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

/// Returns a lightweight summary of every gem for the frontend selector.
#[tauri::command]
#[specta::specta]
fn get_gem_list(
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<Vec<GemSummary>, String> {
    let game = game_data.read().map_err(|e| e.to_string())?;
    let mut list: Vec<GemSummary> = game
        .gems
        .iter()
        .map(|(id, gem)| {
            let description = game
                .skills
                .get(&gem.granted_effect_id)
                .and_then(|effect| effect.description.clone());
            GemSummary {
                id: id.clone(),
                name: gem.name.clone(),
                tag_string: gem.tag_string.clone(),
                is_support: gem.tags.get("support").copied().unwrap_or(false),
                color: gem.gem_color(),
                description,
            }
        })
        .collect();
    list.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(list)
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
        compatibility: Vec::new(),
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
) -> Result<(), String> {
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let len_before = build.skill_groups.len();
    build.skill_groups.retain(|g| g.id != group_id);
    if build.skill_groups.len() == len_before {
        return Err(format!("Skill group {} not found", group_id));
    }
    Ok(())
}

/// Adds a gem to a skill group. Validates the gem ID exists, builds a GemInstance with computed stats.
#[tauri::command]
#[specta::specta]
fn add_gem_to_group(
    group_id: u32,
    gem_id: String,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<SkillGroup, String> {
    let game = game_data.read().map_err(|e| e.to_string())?;
    let gem_item = game
        .gems
        .get(&gem_id)
        .ok_or_else(|| format!("Unknown gem ID: {}", gem_id))?;

    let is_support = gem_item.tags.get("support").copied().unwrap_or(false);
    let level = gem_item.natural_max_level;
    let effect = game.skills.get(&gem_item.granted_effect_id);

    let instance = build_gem_instance(&gem_id, &gem_item.name, is_support, level, 0, effect);

    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let group = build
        .skill_groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Skill group {} not found", group_id))?;
    group.gems.push(instance);

    // Recompute compatibility
    recompute_compatibility(group, &game);

    Ok(group.clone())
}

/// Removes a gem from a skill group by its index in the gem list.
#[tauri::command]
#[specta::specta]
fn remove_gem_from_group(
    group_id: u32,
    gem_index: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<SkillGroup, String> {
    let game = game_data.read().map_err(|e| e.to_string())?;
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let group = build
        .skill_groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Skill group {} not found", group_id))?;
    let idx = gem_index as usize;
    if idx >= group.gems.len() {
        return Err(format!(
            "Gem index {} out of range (group has {} gems)",
            gem_index,
            group.gems.len()
        ));
    }
    group.gems.remove(idx);

    // Recompute compatibility
    recompute_compatibility(group, &game);

    Ok(group.clone())
}

/// Returns the skill group with up-to-date effects and compatibility.
#[tauri::command]
#[specta::specta]
fn get_group_effects(
    group_id: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<SkillGroup, String> {
    let game = game_data.read().map_err(|e| e.to_string())?;
    let mut build = build_info.lock().map_err(|e| e.to_string())?;
    let group = build
        .skill_groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Skill group {} not found", group_id))?;
    recompute_compatibility(group, &game);
    Ok(group.clone())
}

/// Build a GemInstance with computed stats from game data.
fn build_gem_instance(
    gem_id: &str,
    name: &str,
    is_support: bool,
    level: u32,
    quality: u32,
    effect: Option<&data::skills::GrantedEffect>,
) -> GemInstance {
    let (stats, level_data) = match effect {
        Some(eff) => {
            let s = skills::build_skill_instance_stats(eff, level, quality, "Default");
            let li = (level as usize).saturating_sub(1);
            (s.into_iter().collect(), eff.levels.get(li))
        }
        None => (HashMap::new(), None),
    };

    GemInstance {
        gem_id: gem_id.to_string(),
        name: name.to_string(),
        is_support,
        level,
        quality,
        enabled: true,
        stats,
        mana_cost: level_data.and_then(|ld| ld.cost.as_ref().and_then(|c| c.get("Mana").copied())),
        crit_chance: level_data.and_then(|ld| ld.crit_chance),
        damage_effectiveness: level_data.and_then(|ld| ld.damage_effectiveness),
        mana_multiplier: level_data.and_then(|ld| ld.mana_multiplier),
        cooldown: level_data.and_then(|ld| ld.cooldown),
        attack_speed_multiplier: level_data.and_then(|ld| ld.attack_speed_multiplier),
    }
}

/// Recompute support compatibility for all active×support pairs in a group.
fn recompute_compatibility(group: &mut SkillGroup, game: &data::GameData) {
    group.compatibility.clear();

    let support_effects: Vec<(String, Option<&data::skills::GrantedEffect>)> = group
        .gems
        .iter()
        .filter(|g| g.is_support)
        .map(|g| {
            let eff = game
                .gems
                .get(&g.gem_id)
                .and_then(|gi| game.skills.get(&gi.granted_effect_id));
            (g.gem_id.clone(), eff)
        })
        .collect();

    for gem in &group.gems {
        if gem.is_support {
            continue;
        }
        let active_effect = game
            .gems
            .get(&gem.gem_id)
            .and_then(|gi| game.skills.get(&gi.granted_effect_id));

        let active_effect = match active_effect {
            Some(e) => e,
            None => continue,
        };

        let supports_for_resolve: Vec<(String, &data::skills::GrantedEffect)> = support_effects
            .iter()
            .filter_map(|(id, eff)| eff.map(|e| (id.clone(), e)))
            .collect();

        let compatible_ids = skills::resolve_supports(active_effect, &supports_for_resolve, true);

        for (support_id, _) in &support_effects {
            group.compatibility.push(SupportCompatEntry {
                support_gem_id: support_id.clone(),
                active_gem_id: gem.gem_id.clone(),
                compatible: compatible_ids.contains(support_id),
            });
        }
    }
}
