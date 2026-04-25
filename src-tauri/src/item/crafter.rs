//! Phase 4 — Crafted item construction.
//!
//! Builds an [`Item`] from a user-defined spec that specifies an item base,
//! quality, optional base-value overrides, and explicit mod selections with
//! user-chosen values within each stat's legal range.

use crate::data::{GameData, SourceId};
use crate::item::local_stats::compute_local_stats;
use crate::item::types::{
    BasePropertyOverride, InfluenceSet, Item, ItemRequirements, ItemType, ModLine, ModLineSource,
    Rarity,
};
use crate::modifier::parser as mod_parser;

// ─── Public spec types (also used as IPC types in lib.rs) ────────────────────

/// A single mod selection: which mod is chosen and what value each stat rolls.
/// Values are in the same order as `AvailableMod.stats`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CraftedModValue {
    pub mod_id: String,
    /// One value per stat slot in the mod.  If fewer values are provided than
    /// there are stat slots, the midpoint of the legal range is used.
    pub values: Vec<f64>,
}

/// Full spec for building a custom non-unique item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct CraftedItemSpec {
    pub base_name: String,
    /// Display name for the item.  Empty → use base name (Normal rarity).
    pub item_name: String,
    /// 0-20 — applied to local stats.
    pub quality: u32,
    /// Item level (determines which tier mods can appear; stored for reference).
    pub item_level: u32,
    // Optional exact base-value overrides (within the base item range).
    pub base_phys_min: Option<f64>,
    pub base_phys_max: Option<f64>,
    pub base_armour: Option<f64>,
    pub base_evasion: Option<f64>,
    pub base_energy_shield: Option<f64>,
    /// Implicit mods selected from the implicit pool.
    pub implicits: Vec<CraftedModValue>,
    /// Prefix explicit mods.
    pub prefixes: Vec<CraftedModValue>,
    /// Suffix explicit mods.
    pub suffixes: Vec<CraftedModValue>,
    /// Master-crafted bench mods.
    pub crafted: Vec<CraftedModValue>,
    /// Optional influence on the item: "shaper", "elder", "crusader", "hunter",
    /// "redeemer", or "warlord". `None` = no influence.
    pub influence: Option<String>,
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Construct a fully computed [`Item`] from a [`CraftedItemSpec`].
///
/// * All selected mod values are clamped to their legal stat range.
/// * Local stats (weapon DPS, armour/evasion/ES) are computed immediately.
/// * `item.inventory_id` is left at `0`; the caller assigns the ID.
pub fn build_crafted_item(
    spec: &CraftedItemSpec,
    game_data: &GameData,
    source: SourceId,
) -> Result<Item, String> {
    let base = game_data
        .bases
        .get(&spec.base_name)
        .ok_or_else(|| format!("Unknown base item: '{}'", spec.base_name))?;

    let item_class = base.item_class.clone();
    let item_type = ItemType::from_item_class(&item_class);

    let name = if spec.item_name.trim().is_empty() {
        spec.base_name.clone()
    } else {
        spec.item_name.trim().to_string()
    };

    let rarity = if spec.item_name.trim().is_empty() {
        Rarity::Normal
    } else {
        let n_explicit = spec.prefixes.len() + spec.suffixes.len() + spec.crafted.len();
        if n_explicit <= 2 {
            Rarity::Magic
        } else {
            Rarity::Rare
        }
    };

    let base_overrides = {
        let any = spec.base_phys_min.is_some()
            || spec.base_phys_max.is_some()
            || spec.base_armour.is_some()
            || spec.base_evasion.is_some()
            || spec.base_energy_shield.is_some();
        if any {
            Some(BasePropertyOverride {
                phys_damage_min: spec.base_phys_min,
                phys_damage_max: spec.base_phys_max,
                armour: spec.base_armour,
                evasion: spec.base_evasion,
                energy_shield: spec.base_energy_shield,
            })
        } else {
            None
        }
    };

    // ── Build mod lines ───────────────────────────────────────────────────────
    let implicit_lines = resolve_mod_values(
        &spec.implicits,
        ModLineSource::Implicit,
        game_data,
        source,
    );
    let mut explicit_lines = resolve_mod_values(
        &spec.prefixes,
        ModLineSource::Explicit,
        game_data,
        source,
    );
    explicit_lines.extend(resolve_mod_values(
        &spec.suffixes,
        ModLineSource::Explicit,
        game_data,
        source,
    ));
    let crafted_lines =
        resolve_mod_values(&spec.crafted, ModLineSource::Crafted, game_data, source);

    // ── Collect global modifiers ──────────────────────────────────────────────
    let mod_list = implicit_lines
        .iter()
        .chain(explicit_lines.iter())
        .chain(crafted_lines.iter())
        .filter(|ml| !ml.is_local)
        .flat_map(|ml| ml.modifiers.iter().cloned())
        .collect();

    let mut item = Item {
        name,
        base_name: spec.base_name.clone(),
        item_class,
        item_type,
        rarity,
        corrupted: false,
        mirrored: false,
        fractured: false,
        synthesised: false,
        influences: InfluenceSet::empty(),
        requirements: ItemRequirements {
            level: base.requirements.level,
            strength: base.requirements.strength,
            dexterity: base.requirements.dexterity,
            intelligence: base.requirements.intelligence,
        },
        item_level: spec.item_level,
        quality: spec.quality,
        implicit_lines,
        explicit_lines,
        crafted_lines,
        enchant_lines: Vec::new(),
        variant_list: Vec::new(),
        selected_variant: 1,
        weapon_data: None,
        armour_data: None,
        flask_data: None,
        mod_list,
        inventory_id: 0, // caller assigns
        base_overrides,
    };

    let base_ref = game_data.bases.get(&item.base_name).map(|b| b as &_);
    compute_local_stats(&mut item, base_ref);
    Ok(item)
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn resolve_mod_values(
    selections: &[CraftedModValue],
    line_source: ModLineSource,
    game_data: &GameData,
    source: SourceId,
) -> Vec<ModLine> {
    selections
        .iter()
        .filter_map(|cmv| resolve_single(cmv, line_source, game_data, source))
        .collect()
}

fn resolve_single(
    cmv: &CraftedModValue,
    line_source: ModLineSource,
    game_data: &GameData,
    source: SourceId,
) -> Option<ModLine> {
    let repoe_mod = game_data.item_mods.get(&cmv.mod_id)?;

    let mut modifiers = Vec::new();
    let mut raw_stats: Vec<(String, f64)> = Vec::new();

    for (idx, stat) in repoe_mod.stats.iter().enumerate() {
        // Use provided value, clamped to legal range.
        let raw_value = cmv
            .values
            .get(idx)
            .copied()
            .unwrap_or_else(|| (stat.min + stat.max) / 2.0);
        let value = raw_value.clamp(stat.min, stat.max);

        raw_stats.push((stat.id.clone(), value));

        let is_local = game_data
            .stat_metadata
            .get(stat.id.as_str())
            .map_or(false, |m| m.is_local);
        if !is_local {
            modifiers.extend(mod_parser::resolve(&stat.id, value, source));
        }
    }

    let is_local = raw_stats
        .iter()
        .any(|(id, _)| game_data.stat_metadata.get(id.as_str()).map_or(false, |m| m.is_local));

    // Build display text using the stat translation system so the preview
    // shows human-readable PoE-style lines instead of raw stat IDs.
    let text = {
        let lines = game_data.translations.render_mod_lines(&raw_stats);
        if lines.is_empty() {
            // Last resort: use the mod's internal name.
            if repoe_mod.name.is_empty() {
                raw_stats
                    .iter()
                    .map(|(id, v)| format!("{}: {}", id, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                repoe_mod.name.clone()
            }
        } else {
            lines.join("\n")
        }
    };

    Some(ModLine {
        text,
        modifiers,
        raw_stats,
        is_local,
        source: line_source,
    })
}
