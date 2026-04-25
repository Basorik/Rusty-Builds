use super::types::*;
use crate::data::skills::{GemInstance, GemRef, SkillGroup};
use crate::modifier::stat_table;
use crate::{
    data::{GameData, SourceId, StatId},
    modifier,
};
use log::debug;
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
                    .map(|m| self.effective_value(m, ctx))
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
                    .map(|m| self.effective_value(m, ctx))
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
                    .fold(1.0, |acc, m| {
                        acc * (1.0 + self.effective_value(m, ctx) / 100.0)
                    })
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
                .map(|m| self.effective_value(m, ctx))
        })
    }

    pub fn get_max(&self, stat: StatId, ctx: &CalcContext) -> Option<f64> {
        self.mods.get(&stat).and_then(|mods| {
            mods.iter()
                .filter(|m| m.mod_type == ModType::Max && self.matches_context(m, ctx))
                .map(|m| self.effective_value(m, ctx))
                .reduce(f64::max)
        })
    }

    pub fn get_min(&self, stat: StatId, ctx: &CalcContext) -> Option<f64> {
        self.mods.get(&stat).and_then(|mods| {
            mods.iter()
                .filter(|m| m.mod_type == ModType::Min && self.matches_context(m, ctx))
                .map(|m| self.effective_value(m, ctx))
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
                    .map(|m| (self.effective_value(m, ctx), m))
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
    fn matches_context(&self, modifier: &Modifier, ctx: &CalcContext) -> bool {
        // ModFlag check: all of the modifier's flags must be in the context.
        // Mirrors PoB: `band(cfgFlags, mod.flags) ~= mod.flags → skip`.
        if !modifier.flags.is_empty() && !ctx.flags.contains(modifier.flags) {
            return false;
        }

        // KeywordFlag check: mirrors PoB's MatchKeywordFlags().
        // Default (no MATCH_ALL): ANY one of the mod's keyword bits must be
        // present in the context.  With MATCH_ALL set: ALL bits must match.
        if !modifier.keywords.is_empty() {
            let kw = modifier.keywords;
            let match_all = kw.contains(KeywordFlag::MATCH_ALL);
            let masked = kw & !KeywordFlag::MATCH_ALL;
            let passes = if match_all {
                ctx.key_flags.contains(masked)        // all-of
            } else {
                masked.is_empty() || ctx.key_flags.intersects(masked) // any-of
            };
            if !passes {
                return false;
            }
        }
        for tag in &modifier.tags {
            match tag {
                ModTag::Condition(var) => {
                    if !ctx.conditions.get(var).copied().unwrap_or(false) {
                        return false;
                    }
                }
                ModTag::ModFlagOr(flags) => {
                    let flags = ModFlag::from_bits_truncate(*flags);
                    if !ctx.flags.intersects(flags) {
                        return false;
                    }
                }
                ModTag::SkillType(_skill_type_id) => {
                    // Phase 7: check against active skill's type flags
                    // For now, skip (always passes)
                }
                ModTag::ActorCondition { var, actor } => {
                    let map = match *actor {
                        Some("enemy") => &ctx.enemy_conditions,
                        _ => &ctx.conditions, // "player" / None = self
                    };
                    if !map.get(var).copied().unwrap_or(false) {
                        return false;
                    }
                }
                ModTag::StatThreshold { stat, threshold, upper, threshold_stat } => {
                    let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                    let thresh = threshold_stat
                        .and_then(|ts| ctx.stat_values.get(&ts).copied())
                        .unwrap_or(*threshold);
                    let passes = if *upper { stat_val <= thresh } else { stat_val >= thresh };
                    if !passes {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }
    fn effective_value(&self, modifier: &Modifier, ctx: &CalcContext) -> f64 {
        let mut value = modifier.value;
        for tag in &modifier.tags {
            match tag {
                ModTag::Multiplier(var) => {
                    let mult = ctx.multipliers.get(var).copied().unwrap_or(0.0);
                    value *= mult;
                }
                ModTag::MultiplierThreshold { var, threshold } => {
                    let mult = ctx.multipliers.get(var).copied().unwrap_or(0.0);
                    if mult < *threshold {
                        return 0.0; // Below threshold — no effect
                    }
                    // gate only — value unchanged
                }
                ModTag::PerStat { stat, div } => {
                    let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                    value *= (stat_val / div).floor();
                }
                ModTag::PercentStat { stat } => {
                    let stat_val = ctx.stat_values.get(stat).copied().unwrap_or(0.0);
                    value = stat_val * value / 100.0;
                }
                _ => {}
            }
        }
        value
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
    /// Equipped item modifiers — all global mods from items in each slot.
    /// Rebuilt whenever the player equips or unequips an item.
    pub items: ModDB,
    // Phase 7+: pub config: ModDB,
}

impl ModDBLayers {
    pub fn new() -> Self {
        Self {
            tree: ModDB::new(),
            class: ModDB::new(),
            gems: ModDB::new(),
            items: ModDB::new(),
        }
    }

    /// Merge all layers into one ModDB for calculation.
    pub fn merged(&self) -> ModDB {
        let mut combined = ModDB::new();
        combined.merge(&self.tree);
        combined.merge(&self.class);
        combined.merge(&self.gems);
        combined.merge(&self.items);
        // combined.merge(&self.config);
        combined
    }

    /// Rebuild the tree layer when selected nodes change.
    /// Uses the RePoE passive tree (raw stat IDs) resolved via `parser::resolve()`.
    /// instead of display text parsing.
    pub fn rebuild_tree(&mut self, node_ids: &[u32], game_data: &GameData) {
        self.tree = ModDB::new();
        for &node_id in node_ids {
            if let Some(passive) = game_data.repoe_tree.get_passive(node_id) {
                let source = SourceId(node_id);
                for (stat_id, &value) in &passive.stats {
                    for m in crate::modifier::parser::resolve(stat_id, value as f64, source) {
                        self.tree.add_mod(m);
                    }
                }
            } else {
                debug!("Node {} not found in RePoE tree", node_id);
            }
        }
    }

    /// Rebuild the class layer when class or tree version changes.
    pub fn rebuild_class(
        &mut self,
        class: &crate::data::Class,
        tree: &crate::data::tree::PassiveTree,
    ) {
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

    /// Rebuild the items layer from the currently equipped items.
    /// Iterates every slot and adds all global modifiers from each item's `mod_list`.
    /// Also injects computed armour/evasion/ES/ward from `armour_data` as BASE mods.
    /// Call this whenever the player equips or unequips an item.
    pub fn rebuild_items(
        &mut self,
        equipped: &rustc_hash::FxHashMap<crate::item::types::ItemSlot, crate::item::types::Item>,
    ) {
        self.items = ModDB::new();
        for (_slot, item) in equipped {
            for m in &item.mod_list {
                self.items.add_mod(m.clone());
            }
            // Inject computed armour-piece stats (armour/evasion/ES/ward) as BASE mods.
            // These are computed by compute_armour() and stored in armour_data but never
            // added to mod_list, so we inject them here to include them in calculations.
            if let Some(ad) = &item.armour_data {
                let src = crate::data::SourceId(item.inventory_id);
                if ad.armour > 0.0 {
                    self.items.add_mod(crate::modifier::parser::simple_mod(
                        StatId::Armour,
                        ModType::Base,
                        ad.armour,
                        src,
                    ));
                }
                if ad.evasion > 0.0 {
                    self.items.add_mod(crate::modifier::parser::simple_mod(
                        StatId::Evasion,
                        ModType::Base,
                        ad.evasion,
                        src,
                    ));
                }
                if ad.energy_shield > 0.0 {
                    self.items.add_mod(crate::modifier::parser::simple_mod(
                        StatId::EnergyShield,
                        ModType::Base,
                        ad.energy_shield,
                        src,
                    ));
                }
                if ad.ward > 0.0 {
                    self.items.add_mod(crate::modifier::parser::simple_mod(
                        StatId::Ward,
                        ModType::Base,
                        ad.ward,
                        src,
                    ));
                }
            }
        }
    }

    /// Rebuild the gems layer from the selected active gem and all always-active gems.
    /// 1. **Main skill**: the gem pointed to by `active_gem` + its compatible supports
    ///    from the same group. This is what the user has selected for DPS calculations.
    /// 2. **Always-active gems**: any enabled active gem with `always_active = true`
    ///    across all groups (e.g. auras, heralds). Each pulls in its compatible supports.
    ///
    /// Call this whenever the active gem selection changes, a gem's level/quality changes,
    /// or `always_active` is toggled.
    pub fn rebuild_gems(
        &mut self,
        skill_groups: &[SkillGroup],
        active_gem: Option<&GemRef>,
        game_data: &GameData,
    ) {
        self.gems = ModDB::new();

        // Pass 1: main skill — the single gem the user selected for calculations.
        if let Some(gem_ref) = active_gem {
            if let Some(group) = skill_groups.iter().find(|g| g.id == gem_ref.group_id) {
                if group.enabled {
                    if let Some(active_inst) = group.gems.get(gem_ref.gem_index as usize) {
                        if active_inst.enabled && !active_inst.is_support {
                            self.add_gem_with_supports(active_inst, group, game_data);
                        }
                    }
                }
            }
        }

        // Pass 2: always-active gems (auras, heralds, warcries, etc.) from all groups.
        for group in skill_groups {
            if !group.enabled {
                continue;
            }
            for (idx, gem_inst) in group.gems.iter().enumerate() {
                if !gem_inst.enabled || gem_inst.is_support || !gem_inst.always_active {
                    continue;
                }
                // Skip if already added as the main skill.
                if let Some(gem_ref) = active_gem {
                    if gem_ref.group_id == group.id && gem_ref.gem_index as usize == idx {
                        continue;
                    }
                }
                self.add_gem_with_supports(gem_inst, group, game_data);
            }
        }
    }

    /// Add a single active gem and all its compatible supports from the group to the gems layer.
    /// Support compatibility is not yet implemented — all enabled supports are included for now.
    fn add_gem_with_supports(
        &mut self,
        active: &GemInstance,
        group: &SkillGroup,
        game_data: &GameData,
    ) {
        // Add the active gem itself.
        self.add_gem_stats(active, game_data);

        // Add all enabled supports from the same group.
        // TODO Phase 3.7: filter by can_support() once support matching is implemented.
        for gem_inst in &group.gems {
            if gem_inst.enabled && gem_inst.is_support {
                self.add_gem_stats(gem_inst, game_data);
            }
        }
    }

    /// Compute and resolve a single gem's stats at its current level/quality, then add
    /// the resulting Modifiers to the gems layer.
    /// Uses pre-resolved StatDefs from `RePoEGem.resolved_stats` (populated at load time)
    /// so that no `stat_table` hash lookup occurs at calc time.
    fn add_gem_stats(&mut self, gem_inst: &GemInstance, game_data: &GameData) {
        let Some(gem) = game_data.gems.get(&gem_inst.gem_id) else {
            return;
        };
        let Some(level_data) = gem.per_level.get(&gem_inst.level) else {
            return;
        };

        // Use a stable numeric source ID derived from the gem_id string's hash.
        let source = SourceId({
            let mut h: u32 = 2166136261;
            for b in gem_inst.gem_id.bytes() {
                h = h.wrapping_mul(16777619) ^ b as u32;
            }
            h
        });

        // Per-level stats — use pre-resolved defs (positionally aligned with static_data.stats)
        for (i, resolved) in gem.resolved_stats.iter().enumerate() {
            let Some(resolved) = resolved.as_ref() else {
                continue;
            };

            let value = level_data
                .stats
                .get(i)
                .and_then(|s| s.as_ref())
                .and_then(|s| s.value)
                .or_else(|| {
                    gem.static_data
                        .stats
                        .get(i)
                        .and_then(|s| s.as_ref())
                        .and_then(|s| s.value)
                })
                .unwrap_or(0.0);

            for def in &resolved.defs {
                self.gems.add_mod(stat_table::apply(def, value, source));
            }
        }

        // Quality stats — use pre-resolved defs
        if gem_inst.quality > 0 {
            for (stat_id, resolved) in &gem.resolved_quality_stats {
                // Find the raw value from quality_stats
                let raw_value = gem
                    .static_data
                    .quality_stats
                    .iter()
                    .find_map(|qs| qs.stats.get(stat_id))
                    .copied()
                    .unwrap_or(0.0);

                let value = (raw_value / 1000.0) * gem_inst.quality as f64;

                for def in &resolved.defs {
                    self.gems.add_mod(stat_table::apply(def, value, source));
                }
            }
        }
    }
}
