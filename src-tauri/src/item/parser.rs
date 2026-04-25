use crate::data::uniques::{active_for_variant, strip_all_tags, UniqueItemDef};
use crate::data::{GameData, SourceId};
use crate::item::local_stats::compute_local_stats;
use crate::item::types::{
    ArmourData, FlaskData, InfluenceSet, Item, ItemRequirements, ItemType, ModLine,
    ModLineSource, Rarity, WeaponData,
};
use crate::modifier::parser as mod_parser;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Build a fully populated [`Item`] from a [`UniqueItemDef`] for the given
/// 1-based variant index.
///
/// This is the on-demand parse — only called when the user actually equips
/// or inspects a unique.  The [`UniqueItemDef`] retains the raw text so this
/// can be called any number of times without going back to disk.
pub fn parse_unique_item(
    def: &UniqueItemDef,
    selected_variant: usize,
    game_data: &GameData,
    source: SourceId,
) -> Item {
    let sv = selected_variant.max(1);
    let base_name = def.base_for_variant(sv).to_string();

    // Look up item class from the base items table.
    let item_class = game_data
        .bases
        .get(&base_name)
        .map(|b| b.item_class.clone())
        .unwrap_or_default();

    let item_type = ItemType::from_item_class(&item_class);

    let influences = InfluenceSet::from_bits_truncate(def.influences);

    // ── Parse mod lines from the raw text ─────────────────────────────────────
    let lines: Vec<&str> = def
        .raw_text
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();

    // Locate the `Implicits: N` boundary.
    // Many PoB uniques omit this line entirely; in that case fall back to
    // heuristic metadata-skipping to find where the mod lines actually begin.
    let (implicit_count, mod_start) = lines
        .iter()
        .enumerate()
        .find_map(|(idx, &l)| {
            let stripped = strip_first_tag_inner(l);
            stripped.strip_prefix("Implicits: ").and_then(|n_str| {
                n_str.parse::<usize>().ok().map(|n| (n, idx + 1))
            })
        })
        .unwrap_or_else(|| (0, find_mod_start_no_implicit_marker(&lines)));

    let explicit_start = (mod_start + implicit_count).min(lines.len());

    let mut implicit_lines: Vec<ModLine> = Vec::new();
    let mut explicit_lines: Vec<ModLine> = Vec::new();
    let mut crafted_lines: Vec<ModLine> = Vec::new();

    for (i, &raw_line) in lines[mod_start..].iter().enumerate() {
        let base_source = if i < implicit_count {
            ModLineSource::Implicit
        } else {
            ModLineSource::Explicit
        };
        let line_idx = mod_start + i;
        let _ = line_idx; // suppress lint

        if let Some(mod_line) = process_mod_line(raw_line, sv, base_source, game_data, source) {
            match mod_line.source {
                ModLineSource::Implicit => implicit_lines.push(mod_line),
                ModLineSource::Crafted => crafted_lines.push(mod_line),
                _ => explicit_lines.push(mod_line),
            }
        }
    }

    // ── Collect global modifiers ──────────────────────────────────────────────
    let mod_list = implicit_lines
        .iter()
        .chain(explicit_lines.iter())
        .chain(crafted_lines.iter())
        .filter(|ml| !ml.is_local)
        .flat_map(|ml| ml.modifiers.iter().cloned())
        .collect();

    let mut item = Item {
        name: def.name.clone(),
        base_name,
        item_class,
        item_type,
        rarity: Rarity::Unique,
        corrupted: false,
        mirrored: false,
        fractured: def.influences == 0 && explicit_lines.iter().any(|l| l.source == ModLineSource::Fractured),
        synthesised: false,
        influences,
        requirements: ItemRequirements::default(),
        item_level: 0,
        quality: 20,
        implicit_lines,
        explicit_lines,
        crafted_lines,
        enchant_lines: Vec::new(),
        variant_list: def.variant_labels.clone(),
        selected_variant: sv,
        weapon_data: None,
        armour_data: None,
        flask_data: None,
        mod_list,
        inventory_id: 0,
        base_overrides: None,
    };

    let base = game_data.bases.get(&item.base_name).map(|b| b as &_);
    compute_local_stats(&mut item, base);
    item
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Process a single raw mod line: check variant, strip tags, resolve modifiers.
/// Returns `None` when the line is filtered out (wrong variant, or empty).
fn process_mod_line(
    raw_line: &str,
    selected_variant: usize,
    base_source: ModLineSource,
    game_data: &GameData,
    source: SourceId,
) -> Option<ModLine> {
    // Quick variant check before doing heavier work.
    if !active_for_variant(raw_line, selected_variant) {
        return None;
    }

    let (tags, clean_text) = strip_all_tags(raw_line);
    if clean_text.is_empty() {
        return None;
    }

    // Determine actual source from prefix tags.
    let actual_source = if tags.iter().any(|t| *t == "crafted") {
        ModLineSource::Crafted
    } else if tags.iter().any(|t| *t == "fractured") {
        ModLineSource::Fractured
    } else {
        base_source
    };

    // Resolve modifiers: try InvertedTranslations first, then the Phase-2 template map.
    let (modifiers, raw_stats, is_local) =
        if let Some(resolved) = game_data
            .translations
            .resolve_line(clean_text, &game_data.stat_metadata)
        {
            let is_local = resolved.is_local;
            let mods = resolved
                .stats
                .iter()
                .flat_map(|(stat_id, value)| mod_parser::resolve(stat_id, *value, source))
                .collect();
            let raw = resolved.stats;
            (mods, raw, is_local)
        } else {
            let mods = mod_parser::parse_display_text(clean_text, source);
            (mods, Vec::new(), false)
        };

    Some(ModLine {
        text: clean_text.to_string(),
        modifiers,
        raw_stats,
        is_local,
        source: actual_source,
    })
}

/// Strip the first leading `{...}` tag from a line (local copy to avoid
/// an extra import inside parse logic).
fn strip_first_tag_inner(line: &str) -> &str {
    if line.starts_with('{') {
        if let Some(close) = line.find('}') {
            return line[close + 1..].trim_start();
        }
    }
    line
}

/// When a PoB unique block has no `Implicits: N` line, find the line index
/// where the actual mod lines begin by skipping name + base-type + metadata.
fn find_mod_start_no_implicit_marker(lines: &[&str]) -> usize {
    // Line 0 = item name, always skip.
    let mut i = 1;

    // Skip base-type lines: either variant-tagged `{variant:N}Base Name` or
    // a single plain base-type name at index 1 (if it isn't a metadata line).
    loop {
        if i >= lines.len() {
            return i;
        }
        let raw = lines[i];
        if raw.starts_with("{variant:") || raw.starts_with("{altv") {
            // Variant-tagged base name; keep scanning for more.
            i += 1;
        } else if i == 1 && is_unique_metadata(strip_first_tag_inner(raw)) {
            // Line 1 is already metadata — item has no explicit base line.
            break;
        } else if i == 1 {
            // Plain base type name.
            i += 1;
            break;
        } else {
            // Past base-type section.
            break;
        }
    }

    // Skip every metadata line (League:, Variant:, Source:, Requires, etc.).
    while i < lines.len() && is_unique_metadata(strip_first_tag_inner(lines[i])) {
        i += 1;
    }

    i
}

/// Returns `true` if `line` (already tag-stripped) is a known PoB metadata
/// header rather than a mod line.
fn is_unique_metadata(line: &str) -> bool {
    line.starts_with("Variant: ")
        || line.starts_with("League: ")
        || line.starts_with("Source: ")
        || line.starts_with("Requires")
        || line.starts_with("LevelReq: ")
        || line.starts_with("Implicits: ")
        || line.starts_with("Has Alt Variant")
        || line.starts_with("Selected Variant")
        || line.starts_with("Talisman Tier: ")
        || line.starts_with("Limited to: ")
        || line.starts_with("Upgrade: ")
        || line.starts_with("Radius: ")
        || line.starts_with("Sockets: ")
        || line.starts_with("Notable")
        || matches!(
            line,
            "Has no Sockets"
                | "Shaper Item"
                | "Elder Item"
                | "Crusader Item"
                | "Hunter Item"
                | "Redeemer Item"
                | "Warlord Item"
                | "Searing Exarch Item"
                | "Eater of Worlds Item"
                | "Synthesised Item"
                | "Fractured Item"
                | "Corrupted Item"
                | "Mirrored Item"
                | "Duelist"
                | "Marauder"
                | "Ranger"
                | "Shadow"
                | "Witch"
                | "Templar"
                | "Scion"
        )
}

// ─── ItemType mapping ─────────────────────────────────────────────────────────
// This is implemented here rather than in types.rs to keep types.rs as a pure
// data-definition file.

impl ItemType {
    /// Convert a RePoE `item_class` string to the corresponding `ItemType` variant.
    pub fn from_item_class(class: &str) -> Self {
        match class {
            "BodyArmour" | "Body Armour" => ItemType::BodyArmour,
            "Helmet" => ItemType::Helmet,
            "Gloves" => ItemType::Gloves,
            "Boots" => ItemType::Boots,
            "Shield" => ItemType::Shield,
            "Claw" | "Claws" => ItemType::Claw,
            "Dagger" | "Daggers" => ItemType::Dagger,
            "Rune Dagger" | "RuneDagger" | "Rune Daggers" => ItemType::RuneDagger,
            "One Hand Sword" | "OneHandSword" | "One Hand Swords" => ItemType::OneHandSword,
            "Thrusting One Hand Sword" | "ThrustingOneHandSword" | "Thrusting One Hand Swords" => {
                ItemType::ThrustingOneHandSword
            }
            "One Hand Axe" | "OneHandAxe" | "One Hand Axes" => ItemType::OneHandAxe,
            "One Hand Mace" | "OneHandMace" | "One Hand Maces" => ItemType::OneHandMace,
            "Sceptre" | "Sceptres" => ItemType::Sceptre,
            "Wand" | "Wands" => ItemType::Wand,
            "Bow" | "Bows" => ItemType::Bow,
            "Two Hand Sword" | "TwoHandSword" | "Two Hand Swords" => ItemType::TwoHandSword,
            "Two Hand Axe" | "TwoHandAxe" | "Two Hand Axes" => ItemType::TwoHandAxe,
            "Two Hand Mace" | "TwoHandMace" | "Two Hand Maces" => ItemType::TwoHandMace,
            "Staff" | "Staves" => ItemType::Staff,
            "Warstaff" | "Warstaves" => ItemType::Warstaff,
            "Amulet" | "Amulets" => ItemType::Amulet,
            "Ring" | "Rings" => ItemType::Ring,
            "Belt" | "Belts" => ItemType::Belt,
            "Quiver" | "Quivers" => ItemType::Quiver,
            "LifeFlask" | "Life Flasks" => ItemType::LifeFlask,
            "ManaFlask" | "Mana Flasks" => ItemType::ManaFlask,
            "HybridFlask" | "Hybrid Flasks" => ItemType::HybridFlask,
            "UtilityFlask" | "Utility Flasks" | "Critical Utility Flasks" => {
                ItemType::UtilityFlask
            }
            "Jewel" | "Base Jewel" => ItemType::Jewel,
            "AbyssJewel" | "Abyss Jewel" => ItemType::AbyssJewel,
            "Tincture" | "Tinctures" => ItemType::Tincture,
            "FishingRod" | "Fishing Rods" => ItemType::FishingRod,
            _ => ItemType::Amulet, // safe fallback — never used for real calcs
        }
    }
}

// WeaponData / ArmourData / FlaskData Default impls are derived in types.rs.
// No additional impls needed here.
