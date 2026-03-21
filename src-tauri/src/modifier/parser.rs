use crate::data::{ StatId, SourceId };
use super::types::*;
use smallvec::Smallvec;


/// Parse a single display-text mod line into a Modifier (if recognized).
/// Returns None for patterns not yet implemented — log a warning.
///
/// This handles human-readable stat text from passive nodes and items.
/// Gem/skill stats use the SkillStatMap direct mapping path instead (Phase 3).
pub fn parse_display_text(text: &str, source: SourceId) -> Option<Modifier> {
    // Start with common patterns, expand over time.
    // The parser extracts (value, stat_text) then maps stat_text → StatId.
    //
    // Examples:
    //   "+10 to Strength"           → Base, StatId::Strength, value=10
    //   "15% increased Attack Speed" → Inc, StatId::AttackSpeed, value=15
    //   "10% more Spell Damage"      → More, StatId::SpellDamage, value=10
    //   "+50 to maximum Life"        → Base, StatId::Life, value=50
    //
    // Strategy: extract (value, stat_text) via regex or manual matching,
    // then call StatId::from_text(stat_text) to get the enum variant.
    // If StatId::from_text returns None, the stat is unrecognized — skip it.

    // Tip: start with manual string matching, move to regex if patterns get complex
    todo!("Implement pattern by pattern, starting with the most common")
}