pub mod bases;
pub mod gems;
pub mod item_mods;
pub mod repoe_tree;
pub mod skills;
pub mod stat_id;
pub mod stat_translations;
pub mod tree;
pub mod uniques;

use rustc_hash::FxHashMap;
pub use stat_id::StatId;
use std::path::{Path, PathBuf};
use thiserror::Error;
pub use tree::{Bloodline, Class, PassiveTree};

use crate::data::bases::RePoEBaseItem;
use crate::data::gems::RePoEGem;
use crate::data::gems::ResolvedGemStat;
use crate::data::item_mods::RePoEMod;
use crate::data::item_mods::StatMeta;
use crate::data::repoe_tree::RePoETree;
use crate::data::stat_translations::InvertedTranslations;
use crate::data::uniques::UniqueItemDef;
use crate::DEFAULT_TREE_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(pub u32);

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub trait DataLoader: Sized {
    fn load_from_file(path: &Path) -> Result<Self, DataError>;
    fn load_from_json(json: &str) -> Result<Self, DataError>;
}

pub struct GameData {
    pub tree: PassiveTree,
    pub repoe_tree: RePoETree,
    pub gems: FxHashMap<String, RePoEGem>,
    pub bases: FxHashMap<String, RePoEBaseItem>,
    pub item_mods: FxHashMap<String, RePoEMod>,
    pub stat_metadata: FxHashMap<String, StatMeta>,
    pub translations: InvertedTranslations,
    pub uniques: Vec<UniqueItemDef>,
    pub source_names: Vec<String>,
}

impl GameData {
    pub fn load_from_dir(resource_dir: PathBuf) -> Result<Self, DataError> {
        Self::load_with_progress(resource_dir, |_, _| {})
    }

    /// Load all game data, calling `progress` after each major step.
    ///
    /// `progress(step_name, fraction)` — `fraction` is in `[0.0, 1.0]`.
    pub fn load_with_progress(
        resource_dir: PathBuf,
        mut progress: impl FnMut(&str, f64),
    ) -> Result<Self, DataError> {
        progress("Reading passive skill tree…", 0.0 / 8.0);
        let tree_path = resource_dir
            .join("data/tree")
            .join(DEFAULT_TREE_VERSION)
            .join("data.json");
        let tree_json =
            std::fs::read_to_string(&tree_path).expect("Failed to read default tree data");

        progress("Loading RePoE passive tree…", 1.0 / 8.0);
        let repoe_tree_path = resource_dir.join("data/repoe/passive_skill_trees/Default.json");
        let repoe_tree =
            RePoETree::load_from_file(&repoe_tree_path).expect("Failed to load RePoE passive tree");

        progress("Loading gems…", 2.0 / 8.0);
        let gems_path = resource_dir.join("data/repoe/gems.json");
        let gems_json = std::fs::read_to_string(&gems_path)?;

        progress("Loading base items…", 3.0 / 8.0);
        let bases_path = resource_dir.join("data/repoe/base_items.json");
        let bases_json = std::fs::read_to_string(&bases_path)?;

        progress("Loading item modifiers…", 4.0 / 8.0);
        let item_mods_path = resource_dir.join("data/repoe/mods.json");
        let item_mods_json = std::fs::read_to_string(&item_mods_path)?;

        progress("Loading stat metadata…", 5.0 / 8.0);
        let stat_meta_path = resource_dir.join("data/repoe/stats.json");
        let stat_meta_json = std::fs::read_to_string(&stat_meta_path)?;

        progress("Loading stat translations…", 6.0 / 8.0);
        let stat_translations_path = resource_dir.join("data/repoe/stat_translations.json");
        let stat_translations_json = std::fs::read_to_string(&stat_translations_path)?;

        progress("Parsing passive tree…", 7.0 / 8.0);
        let tree = PassiveTree::load_from_json(&tree_json)?;
        let gems = serde_json::from_str::<FxHashMap<String, RePoEGem>>(&gems_json)?;
        let raw_bases = serde_json::from_str::<FxHashMap<String, RePoEBaseItem>>(&bases_json)?;
        let bases: FxHashMap<String, RePoEBaseItem> = raw_bases
            .into_values()
            .filter(|b| bases::EQUIPPABLE_CLASSES.contains(&b.item_class.as_str()))
            .map(|b| (b.name.clone(), b))
            .collect();

        let raw_mods = serde_json::from_str::<FxHashMap<String, RePoEMod>>(&item_mods_json)?;
        let item_mods: FxHashMap<String, RePoEMod> = raw_mods
            .into_iter()
            .filter(|(_, m)| item_mods::MOD_DOMAINS.contains(&m.domain.as_str()))
            .collect();

        let stat_metadata = serde_json::from_str::<FxHashMap<String, StatMeta>>(&stat_meta_json)?;
        progress("Building translation tables…", 7.5 / 8.0);
        let translations =
            InvertedTranslations::build(&stat_translations_json).map_err(DataError::Json)?;

        progress("Loading unique items…", 7.8 / 8.0);
        let uniques = uniques::load_pob_uniques(&resource_dir.join("data")).unwrap_or_else(|e| {
            log::warn!("Failed to load PoB uniques: {}", e);
            Vec::new()
        });

        let mut gd = GameData {
            tree,
            repoe_tree,
            gems,
            bases,
            item_mods,
            stat_metadata,
            translations,
            uniques,
            source_names: Vec::new(),
        };
        progress("Pre-resolving gem stats…", 8.5 / 10.0);
        gd.pre_resolve_gems();
        progress("Ready.", 1.0);
        Ok(gd)
    }

    pub fn intern_source(&mut self, name: &str) -> SourceId {
        if let Some(pos) = self.source_names.iter().position(|s| s == name) {
            return SourceId(pos as u32);
        }
        let id = SourceId(self.source_names.len() as u32);
        self.source_names.push(name.to_owned());
        id
    }

    /// Pre-resolve all gem stat IDs against `stat_table()` at load time.
    /// This converts each gem's stat ID strings into `StatDef` slices so that
    /// calculation-time lookups are a direct index instead of a hash lookup.
    fn pre_resolve_gems(&mut self) {
        use crate::modifier::stat_table::stat_table;

        let table = stat_table();

        for gem in self.gems.values_mut() {
            // Resolve static stats (positionally aligned with per_level.stats)
            gem.resolved_stats = gem
                .static_data
                .stats
                .iter()
                .map(|slot| {
                    let stat_def = slot.as_ref()?;
                    let stat_id = stat_def.id.as_ref()?;
                    if stat_def.stat_type.as_deref() == Some("implicit") {
                        return None;
                    }
                    let defs = table.get(stat_id.as_str())?;
                    Some(ResolvedGemStat { defs: defs.clone() })
                })
                .collect();

            // Resolve quality stats
            gem.resolved_quality_stats = gem
                .static_data
                .quality_stats
                .iter()
                .flat_map(|qs| qs.stats.keys())
                .filter_map(|stat_id| {
                    let defs = table.get(stat_id.as_str())?;
                    Some((stat_id.clone(), ResolvedGemStat { defs: defs.clone() }))
                })
                .collect();
        }
    }
}
