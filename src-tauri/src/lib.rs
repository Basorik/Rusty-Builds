mod client;
mod data;
mod models;
mod modifier;
mod storage;

use log::info;
use serde::{Deserialize, Serialize};
use specta::Type;
use specta_typescript::Typescript;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use crate::data::gems::GemSummary;
use crate::data::skills::{self, GemInstance, SkillGroup, SupportCompatEntry};
use crate::data::{
    Bloodline, Class, DuelistAscendancy, MarauderAscendancy, RangerAscendancy, ScionAscendancy,
    ShadowAscendancy, TemplarAscendancy, WitchAscendancy,
};

/// The tree version loaded on startup and used as the default.
pub const DEFAULT_TREE_VERSION: &str = "3.27.0g";

/// Tracks which skill-tree nodes the user has selected for the current build.
#[derive(Debug, Default, Serialize, Deserialize, Type)]
pub struct BuildSelection {
    selected_node_ids: HashSet<u32>,
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
    /// Persisted layered modifier database. Not serialized to the frontend —
    /// it's rebuilt from the other fields and serves as the authoritative source
    /// for all stat queries including the debug view.
    #[serde(skip)]
    #[specta(skip)]
    pub mod_db_layers: modifier::ModDBLayers,
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
            next_group_id: 1,
            mod_db_layers: modifier::ModDBLayers::new(),
        }
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
        update_gem_level_quality,
        get_group_effects,
        get_debug_stats
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

            // Build the initial BuildInfo with the class layer already populated so
            // stats are non-zero before the user makes any changes.
            let mut build_info = BuildInfo::default();
            build_info.mod_db_layers.rebuild_class(&build_info.class, &game_data.tree);

            app.manage(Mutex::new(build_info));
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
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<(), String> {
    let mut build_info = state.lock().map_err(|e| e.to_string())?;
    build_info.level = level;
    build_info.class = character_class;
    build_info.bloodline = bloodline;

    // Rebuild the class layer so the ModDB reflects the new class base stats.
    let game = game_data.read().map_err(|e| e.to_string())?;
    let class = build_info.class.clone();
    build_info.mod_db_layers.rebuild_class(&class, &game.tree);

    info!(
        "Build updated: Level {}, Class {:?}, Bloodline {:?}",
        build_info.level, build_info.class, build_info.bloodline
    );
    Ok(())
}

/// Receives the current set of selected node IDs from the frontend.
/// Rebuilds the tree layer of the persisted ModDB and returns updated build stats.
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

    // Rebuild only the tree layer — class and gems layers are already up to date.
    build.mod_db_layers.rebuild_tree(&node_ids, &game);
    // Refresh gem layer in case skill group counts changed.
    let skill_groups = build.skill_groups.clone();
    build.mod_db_layers.rebuild_gems(&skill_groups);

    let mod_db = build.mod_db_layers.merged();
    let ctx = modifier::CalcContext::empty();

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

/// Updates the level and/or quality of a gem in a skill group, recomputing its stats.
#[tauri::command]
#[specta::specta]
fn update_gem_level_quality(
    group_id: u32,
    gem_index: u32,
    level: u32,
    quality: u32,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
) -> Result<SkillGroup, String> {
    if level == 0 {
        return Err("Level must be at least 1".to_string());
    }
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

    let old = &group.gems[idx];
    let effect = game
        .gems
        .get(&old.gem_id)
        .and_then(|gi| game.skills.get(&gi.granted_effect_id));

    let updated = build_gem_instance(
        &old.gem_id,
        &old.name,
        old.is_support,
        level,
        quality,
        effect,
    );
    group.gems[idx] = updated;

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

/// Returns all modifiers in the persisted ModDB layers for the debug page.
/// Reads directly from build.mod_db_layers — does NOT recompute anything.
#[tauri::command]
#[specta::specta]
fn get_debug_stats(
    game_data: tauri::State<'_, Arc<RwLock<data::GameData>>>,
    build_info: tauri::State<'_, Mutex<BuildInfo>>,
) -> Result<DebugStatsResponse, String> {
    let build = build_info.lock().map_err(|e| e.to_string())?;
    let game = game_data.read().map_err(|e| e.to_string())?;
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

    // Merge all layers and compute final values for each unique stat.
    let combined = build.mod_db_layers.merged();
    let mut computed = HashMap::new();
    let mut seen_stats = HashSet::new();
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
        computed,
    })
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
        None => (BTreeMap::new(), None),
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
