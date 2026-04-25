use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    data::{
        skills::{GemRef, SkillGroup},
        Bloodline, Class, GameData, SourceId,
    },
    item::types::{Item, ItemSlot},
    modifier::{parser as mod_parser, ModDBLayers},
    BuildInfo, BuildSelection, BuildStats,
};
#[derive(Serialize, Deserialize)]
pub struct SavedBuildData {
    pub name: String,
    pub level: u32,
    pub class: Class,
    pub bloodline: Bloodline,
    pub selected_nodes: BuildSelection,
    pub skill_groups: Vec<SkillGroup>,
    pub active_gem: Option<GemRef>,
    pub next_group_id: u32,
    pub equipped: FxHashMap<ItemSlot, Item>,
    pub inventory: Vec<Item>,
    pub next_item_id: u32,
    // Metadata
    pub tree_version: String,
    pub save_version: u32,
}

/// Re-resolve all `ModLine.modifiers` from persisted `raw_stats`, then rebuild
/// the item's `mod_list` from the non-local modifiers.
fn rehydrate_item(item: &mut Item) {
    let source = SourceId(item.inventory_id);

    // Re-resolve modifiers on every mod line from its persisted raw_stats.
    for ml in item
        .implicit_lines
        .iter_mut()
        .chain(item.explicit_lines.iter_mut())
        .chain(item.crafted_lines.iter_mut())
        .chain(item.enchant_lines.iter_mut())
    {
        ml.modifiers = ml
            .raw_stats
            .iter()
            .flat_map(|(stat_id, value)| mod_parser::resolve(stat_id, *value, source))
            .collect();
    }

    // Rebuild the global mod_list from non-local lines.
    item.mod_list = item
        .implicit_lines
        .iter()
        .chain(item.explicit_lines.iter())
        .chain(item.crafted_lines.iter())
        .chain(item.enchant_lines.iter())
        .filter(|ml| !ml.is_local)
        .flat_map(|ml| ml.modifiers.iter().cloned())
        .collect();
}

impl SavedBuildData {
    pub fn from_build(build: &BuildInfo, tree_version: String) -> Self {
        SavedBuildData {
            name: build.name.clone(),
            level: build.level,
            class: build.class.clone(),
            bloodline: build.bloodline.clone(),
            selected_nodes: build.selected_nodes.clone(),
            skill_groups: build.skill_groups.clone(),
            active_gem: build.active_gem.clone(),
            next_group_id: build.next_group_id,
            equipped: build.equipped.clone(),
            inventory: build.inventory.clone(),
            next_item_id: build.next_item_id,
            tree_version,
            save_version: 0,
        }
    }

    pub fn into_build(self, game: &GameData) -> BuildInfo {
        let mut equipped = self.equipped;
        let mut inventory = self.inventory;

        // Re-resolve all deserialized items (modifiers are #[serde(skip)]).
        for item in equipped.values_mut() {
            rehydrate_item(item);
        }
        for item in inventory.iter_mut() {
            rehydrate_item(item);
        }

        let node_ids: Vec<u32> = self
            .selected_nodes
            .selected_node_ids
            .iter()
            .copied()
            .collect();

        let mut mod_db_layers = ModDBLayers::new();
        mod_db_layers.rebuild_items(&equipped);
        mod_db_layers.rebuild_tree(&node_ids, game);
        mod_db_layers.rebuild_class(&self.class, &game.tree);
        mod_db_layers.rebuild_gems(&self.skill_groups, self.active_gem.as_ref(), game);

        BuildInfo {
            name: self.name,
            level: self.level,
            stats: BuildStats::default(),
            class: self.class,
            bloodline: self.bloodline,
            selected_nodes: self.selected_nodes,
            skill_groups: self.skill_groups,
            active_gem: self.active_gem,
            next_group_id: self.next_group_id,
            mod_db_layers,
            equipped,
            inventory,
            next_item_id: self.next_item_id,
        }
    }
}
