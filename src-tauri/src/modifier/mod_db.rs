use super::types::*;
use crate::{
    data::{GameData, SourceId, StatId},
    modifier,
};
use rustc_hash::FxHashMap;

#[derive(Debug, Default)]
pub struct ModDB {
    mods: FxHashMap<StatId, Vec<Modifier>>,
}

impl ModDB {
    pub fn new() -> Self {
        Self {
            mods: FxHashMap::default(),
        }
    }

    pub fn add_mod(&mut self, modifier: Modifier) {
        self.mods.entry(modifier.stat).or_default().push(modifier);
    }

    pub fn sum_base(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods
            .get(&stat)
            .map(|mods| {
                mods.iter()
                    .filter(|m| m.mod_type == ModType::Base)
                    .filter(|m| self.matches_context(m, ctx))
                    .map(|m| m.value)
                    .sum()
            })
            .unwrap_or(0.0)
    }

    pub fn sum_inc(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods
            .get(&stat)
            .map(|mods| {
                mods.iter()
                    .filter(|m| m.mod_type == ModType::Inc)
                    .filter(|m| self.matches_context(m, ctx))
                    .map(|m| m.value)
                    .sum()
            })
            .unwrap_or(0.0)
    }

    pub fn product_more(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        self.mods
            .get(&stat)
            .map(|mods| {
                mods.iter()
                    .filter(|m| m.mod_type == ModType::More)
                    .filter(|m| self.matches_context(m, ctx))
                    .fold(1.0, |acc, m| acc * (1.0 + m.value / 100.0))
            })
            .unwrap_or(1.0)
    }

    pub fn has_flag(&self, stat: StatId, ctx: &CalcContext) -> bool {
        self.mods
            .get(&stat)
            .map(|mods| {
                mods.iter()
                    .any(|m| m.mod_type == ModType::Flag && self.matches_context(m, ctx))
            })
            .unwrap_or(false)
    }

    pub fn get_override(&self, stat: StatId, ctx: &CalcContext) -> Option<f64> {
        self.mods.get(&stat).and_then(|mods| {
            mods.iter()
                .find(|m| m.mod_type == ModType::Override && self.matches_context(m, ctx))
                .map(|m| m.value)
        })
    }

    pub fn get_max(&self, stat: StatId, ctx: &CalcContext) -> Option<f64> {
        self.mods.get(&stat).and_then(|mods| {
            mods.iter()
                .filter(|m| m.mod_type == ModType::Max && self.matches_context(m, ctx))
                .map(|m| m.value)
                .reduce(f64::max)
        })
    }

    pub fn get_min(&self, stat: StatId, ctx: &CalcContext) -> Option<f64> {
        self.mods.get(&stat).and_then(|mods| {
            mods.iter()
                .filter(|m| m.mod_type == ModType::Min && self.matches_context(m, ctx))
                .map(|m| m.value)
                .reduce(f64::min)
        })
    }

    pub fn tabulate(
        &self,
        mod_type: Option<ModType>,
        stat: StatId,
        ctx: &CalcContext,
    ) -> Vec<(f64, &Modifier)> {
        self.mods
            .get(&stat)
            .map(|mods| {
                mods.iter()
                    .filter(|m| mod_type.map_or(true, |t| m.mod_type == t))
                    .filter(|m| self.matches_context(m, ctx))
                    .map(|m| (m.value, m))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn sum_base_multi(&self, stats: &[StatId], ctx: &CalcContext) -> f64 {
        stats.iter().map(|s| self.sum_base(*s, ctx)).sum()
    }

    pub fn calculate(&self, stat: StatId, ctx: &CalcContext) -> f64 {
        if let Some(val) = self.get_override(stat, ctx) {
            return val;
        }
        let base = self.sum_base(stat, ctx);
        let inc = self.sum_inc(stat, ctx);
        let more = self.product_more(stat, ctx);
        base * (1.0 + inc / 100.0) * more
    }

    pub fn merge(&mut self, other: &ModDB) {
        for (_stat, mods) in &other.mods {
            for m in mods {
                self.add_mod(m.clone());
            }
        }
    }

    /// Iterate all (StatId, modifier list) entries in the ModDB.
    pub fn iter_all(&self) -> impl Iterator<Item = (&StatId, &Vec<Modifier>)> {
        self.mods.iter()
    }

    /// Check if a modifier's flags/conditions match the current calc context.
    /// In Phase 2 this always returns true. Expand in Phase 5+.
    fn matches_context(&self, _modifier: &Modifier, _ctx: &CalcContext) -> bool {
        // TODO: Check modifier.flags against ctx.flags,
        //       evaluate ModTag conditions, etc.
        true
    }
}

/// Layered composition of modifier sources.
/// Each layer is independently rebuilt when its source changes,
/// avoiding re-parsing unrelated layers on every update.
#[derive(Debug, Default)]
pub struct ModDBLayers {
    /// Passive skill tree nodes selected by the player.
    pub tree: ModDB,
    /// Class base stats (Str/Dex/Int from tree.classes[i]).
    /// Rebuilt whenever the class or tree version changes.
    pub class: ModDB,
    /// Skill gem stats — active and support gems in all socket groups.
    /// Rebuilt whenever the player changes a gem or its level/quality.
    pub gems: ModDB,
    // Phase 4+: pub items: ModDB,
    // Phase 7+: pub config: ModDB,
}

impl ModDBLayers {
    pub fn new() -> Self {
        Self {
            tree: ModDB::new(),
            class: ModDB::new(),
            gems: ModDB::new(),
        }
    }

    /// Merge all layers into one ModDB for calculation.
    pub fn merged(&self) -> ModDB {
        let mut combined = ModDB::new();
        combined.merge(&self.tree);
        combined.merge(&self.class);
        combined.merge(&self.gems);
        // combined.merge(&self.items);
        // combined.merge(&self.config);
        combined
    }

    /// Rebuild the tree layer when selected nodes change.
    pub fn rebuild_tree(&mut self, node_ids: &[u32], game_data: &GameData) {
        self.tree = ModDB::new();
        for &node_id in node_ids {
            if let Some(node) = game_data.tree.get_node(node_id) {
                let source = SourceId(node_id);
                for stat_text in &node.stats {
                    for m in modifier::parser::parse_display_text(stat_text, source) {
                        self.tree.add_mod(m);
                    }
                }
            }
        }
    }

    /// Rebuild the class layer when class or tree version changes.
    pub fn rebuild_class(&mut self, class: &crate::data::Class, tree: &crate::data::tree::PassiveTree) {
        self.class = ModDB::new();
        if let Some(class_data) = tree.get_class_data(class) {
            let source = SourceId(0); // source 0 = class base stats
            self.class.add_mod(modifier::parser::simple_mod(
                crate::data::StatId::Strength,
                super::types::ModType::Base,
                class_data.base_str as f64,
                source,
            ));
            self.class.add_mod(modifier::parser::simple_mod(
                crate::data::StatId::Dexterity,
                super::types::ModType::Base,
                class_data.base_dex as f64,
                source,
            ));
            self.class.add_mod(modifier::parser::simple_mod(
                crate::data::StatId::Intelligence,
                super::types::ModType::Base,
                class_data.base_int as f64,
                source,
            ));
        }
    }

    /// Rebuild the gems layer from all active skill groups.
    /// Call this whenever a gem is added, removed, or its level/quality changes.
    pub fn rebuild_gems(&mut self, skill_groups: &[crate::data::skills::SkillGroup]) {
        self.gems = ModDB::new();
        // TODO Phase 5: parse gem stat strings into Modifiers and add them here.
        // For now the layer is empty — gem stats are not yet fed into ModDB calculations.
        let _ = skill_groups; // suppress unused warning
    }
}
