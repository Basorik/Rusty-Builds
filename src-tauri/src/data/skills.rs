use serde::{Deserialize, Serialize};

use specta::Type;

use crate::data::gems::RePoEGem;

/// A gem placed in a socket group — tracks identity, level, quality, and computed stats.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct GemInstance {
    pub gem_id: String,
    pub name: String,
    pub is_support: bool,
    pub level: u32,
    pub quality: u32,
    pub enabled: bool,
    /// If true, this gem's stats are always fed into the ModDB regardless of which active
    /// gem the user has selected as their main skill. Intended for auras, heralds, warcries,
    /// and other persistent-effect skills.
    pub always_active: bool,
}

/// A group of linked gems — one active skill plus its supports.
#[derive(Debug, Default, Clone, Serialize, Deserialize, Type)]
pub struct SkillGroup {
    pub id: u32,
    pub label: String,
    pub gems: Vec<GemInstance>,
    pub enabled: bool,
}

/// Points to a specific gem within a skill group.
/// Used by BuildInfo to identify the user's selected main skill for calculations.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct GemRef {
    /// The skill group this gem belongs to.
    pub group_id: u32,
    /// Index of the gem within `SkillGroup.gems`.
    pub gem_index: u32,
}
/// Resolve which supports apply to an active gem within a skill group.
/// Returns the list of applicable support GrantedEffect IDs.
///
/// Mirrors PoB's `calcs.createActiveSkill()` two-pass logic:
/// Pass 1: Check compatibility and accumulate addSkillTypes from compatible supports.
///         Repeat until no new supports are added (handles cross-dependencies).
/// Pass 2: Build final list of compatible supports.
pub fn resolve_supports(active: &RePoEGem, supports: &[&RePoEGem]) -> Vec<usize> {
    let active_skill = match &active.active_skill {
        Some(skill) => skill,
        None => return Vec::new(),
    };

    //Start with active gems list of tags
    let mut effective_types: Vec<String> = active_skill.types.clone();
    let mut compatible = vec![false; supports.len()];
    let mut added_new = true;

    //pass 1
    while added_new {
        added_new = false;
        for (i, support) in supports.iter().enumerate() {
            if compatible[i] {
                continue;
            }
            let Some(info) = &support.support_gem else {
                continue;
            };

            //check excluded types on support
            if let Some(excluded) = &info.excluded_types {
                if effective_types.iter().any(|t| excluded.contains(t)) {
                    continue;
                }
            }

            //check required types on support
            let matches = match &info.allowed_types {
                Some(allowed) if !allowed.is_empty() => {
                    effective_types.iter().any(|t| allowed.contains(t))
                }
                _ => true,
            };
            if matches {
                compatible[i] = true;
                if let Some(added) = &info.added_types {
                    for t in added {
                        if !effective_types.contains(t) {
                            effective_types.push(t.clone());
                            added_new = true;
                        }
                    }
                }
            }
        }
    }
    compatible
        .iter()
        .enumerate()
        .filter(|(_, &c)| c)
        .map(|(i, _)| i)
        .collect()
}
/// With a given support gem and the current active skill types return if the support gem will work
pub fn can_support(support: &RePoEGem, active: &RePoEGem) -> bool {
    let support_info = match &support.support_gem {
        Some(info) => info,
        None => return false, // not a support gem
    };
    let active_skill = match &active.active_skill {
        Some(skill) => skill,
        None => return false, // not an active gem
    };

    // Exclude check: if active has any excluded type, reject
    if let Some(excluded) = &support_info.excluded_types {
        if active_skill.types.iter().any(|t| excluded.contains(t)) {
            return false;
        }
    }

    match &support_info.allowed_types {
        Some(allowed) if !allowed.is_empty() => {
            active_skill.types.iter().any(|t| allowed.contains(t))
        }
        _ => true,
    }
}
