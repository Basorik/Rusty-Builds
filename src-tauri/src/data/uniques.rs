use crate::data::DataError;
use std::path::Path;

// ─── Public types ─────────────────────────────────────────────────────────────

/// Lightweight metadata extracted from a single PoB unique text block at startup.
///
/// Full parsing into an [`crate::item::types::Item`] happens on-demand in
/// [`crate::item::parser`] when the user actually equips the unique.
pub struct UniqueItemDef {
    pub name: String,
    /// Variant base type entries — `(variant_indices, base_name)`.
    /// An empty `variant_indices` means "all variants" (single base type).
    pub base_names: Vec<(Vec<usize>, String)>,
    pub league: Option<String>,
    /// Human-readable variant labels in order, e.g. `["Pre 3.0.0", "Current"]`.
    pub variant_labels: Vec<String>,
    pub has_alt_variant: bool,
    pub talisman_tier: Option<u32>,
    pub upgrade_target: Option<String>,
    /// Packed influence bits matching [`crate::item::types::InfluenceSet`] bit layout.
    pub influences: u8,
    /// Original raw PoB text block, retained for on-demand full parse.
    pub raw_text: String,
    /// Source filename, e.g. `"amulet.json"`.
    pub file_source: String,
}

impl UniqueItemDef {
    /// Number of variants this item has (at least 1).
    pub fn variant_count(&self) -> usize {
        self.variant_labels.len().max(1)
    }

    /// Base type name for a given 1-based variant index.
    /// Falls back to the first base entry if no exact match is found.
    pub fn base_for_variant(&self, variant: usize) -> &str {
        for (indices, base) in &self.base_names {
            if indices.is_empty() || indices.contains(&variant) {
                return base.as_str();
            }
        }
        self.base_names
            .first()
            .map(|(_, b)| b.as_str())
            .unwrap_or("")
    }
}

// ─── Loader ───────────────────────────────────────────────────────────────────

/// Load all PoB unique definitions from `<data_dir>/pob/Uniques/*.json`.
///
/// Each JSON file is a flat array of raw text strings with `\n` as
/// line delimiters.  Empty strings and `graft.json` (always empty) are skipped.
pub fn load_pob_uniques(data_dir: &Path) -> Result<Vec<UniqueItemDef>, DataError> {
    let dir = data_dir.join("pob/Uniques");
    let mut all: Vec<UniqueItemDef> = Vec::with_capacity(1300);

    for entry in std::fs::read_dir(&dir).map_err(DataError::Io)? {
        let entry = entry.map_err(DataError::Io)?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let file_source = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let json = std::fs::read_to_string(&path).map_err(DataError::Io)?;
        let arr: Vec<String> = serde_json::from_str(&json).map_err(DataError::Json)?;

        for raw in arr {
            if raw.is_empty() {
                continue;
            }
            if let Some(def) = parse_def(&raw, &file_source) {
                all.push(def);
            }
        }
    }

    Ok(all)
}

// ─── Public helpers ───────────────────────────────────────────────────────────

/// Strip the first leading `{...}` tag from a line, returning the remainder.
///
/// ```text
/// "{variant:1}Base Type"  →  "Base Type"
/// "No tag"               →  "No tag"
/// ```
pub fn strip_first_tag(line: &str) -> &str {
    if line.starts_with('{') {
        if let Some(close) = line.find('}') {
            return line[close + 1..].trim_start();
        }
    }
    line
}

/// Return `true` if the line's leading `{variant:N,M,...}` tag includes `variant`.
///
/// Lines without a variant tag are active for all variants.
/// `variant` is 1-based.
pub fn active_for_variant(line: &str, variant: usize) -> bool {
    if !line.starts_with("{variant:") {
        return true;
    }
    let close = match line.find('}') {
        Some(c) => c,
        None => return true,
    };
    let list = &line["{variant:".len()..close];
    list.split(',')
        .any(|s| s.trim().parse::<usize>().ok() == Some(variant))
}

/// Strip ALL leading `{...}` tags from a line and return `(tags, clean_text)`.
///
/// Tags are returned as their inner content strings, e.g. `"variant:1,2"`.
pub fn strip_all_tags(line: &str) -> (Vec<&str>, &str) {
    let mut tags: Vec<&str> = Vec::new();
    let mut rest = line;
    loop {
        if !rest.starts_with('{') {
            break;
        }
        match rest.find('}') {
            Some(close) => {
                tags.push(&rest[1..close]);
                rest = rest[close + 1..].trim_start();
            }
            None => break,
        }
    }
    (tags, rest)
}

// ─── Private parser ───────────────────────────────────────────────────────────

/// Parse a single raw PoB unique text block into a [`UniqueItemDef`].
fn parse_def(raw: &str, file_source: &str) -> Option<UniqueItemDef> {
    let lines: Vec<&str> = raw
        .split('\n')
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }

    let name = lines[0].to_string();
    let mut i = 1usize;

    // ── Base type lines ────────────────────────────────────────────────────────
    // Lines 1..M may be base type lines:
    //   {variant:1,2,3,4}Sundering Axe    ← one entry per variant group
    //   {variant:5}Ezomyte Axe
    // OR a single plain base type name with no prefix:
    //   Amber Amulet
    let mut base_names: Vec<(Vec<usize>, String)> = Vec::new();
    loop {
        if i >= lines.len() {
            break;
        }
        let line = lines[i];
        if line.starts_with("{variant:") {
            let close = match line.find('}') {
                Some(c) => c,
                None => {
                    i += 1;
                    continue;
                }
            };
            let variant_str = &line["{variant:".len()..close];
            let base = line[close + 1..].trim().to_string();
            let indices: Vec<usize> = variant_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            base_names.push((indices, base));
            i += 1;
        } else if i == 1 {
            // First non-name line that isn't a variant-tagged base type:
            // accept it as the plain (all-variants) base type only when it
            // doesn't look like a metadata skip line.
            if !is_metadata_line(line) {
                base_names.push((vec![], line.to_string()));
                i += 1;
            }
            break; // one plain base; stop looking for more
        } else {
            break;
        }
    }

    // ── Metadata lines ─────────────────────────────────────────────────────────
    let mut league: Option<String> = None;
    let mut variant_labels: Vec<String> = Vec::new();
    let mut has_alt_variant = false;
    let mut talisman_tier: Option<u32> = None;
    let mut upgrade_target: Option<String> = None;
    let mut influences: u8 = 0;

    while i < lines.len() {
        let raw_line = lines[i];
        // Metadata lines themselves can carry {variant:N} prefixes — strip it.
        let line = strip_first_tag(raw_line);
        i += 1;

        if line.starts_with("Implicits: ") {
            break;
        }
        if let Some(v) = line.strip_prefix("Variant: ") {
            variant_labels.push(v.to_string());
        } else if let Some(l) = line.strip_prefix("League: ") {
            league = Some(l.to_string());
        } else if line.starts_with("Has Alt Variant") {
            has_alt_variant = true;
        } else if let Some(rest) = line.strip_prefix("Talisman Tier: ") {
            talisman_tier = rest.parse().ok();
        } else if let Some(rest) = line.strip_prefix("Upgrade: ") {
            upgrade_target = Some(rest.to_string());
        } else {
            influences |= influence_bits(line);
        }
    }

    Some(UniqueItemDef {
        name,
        base_names,
        league,
        variant_labels,
        has_alt_variant,
        talisman_tier,
        upgrade_target,
        influences,
        raw_text: raw.to_string(),
        file_source: file_source.to_string(),
    })
}

/// Returns `true` if `line` (already tag-stripped) is a known metadata skip line
/// rather than a plain base type name.
fn is_metadata_line(line: &str) -> bool {
    let l = strip_first_tag(line);
    l.starts_with("Variant: ")
        || l.starts_with("League: ")
        || l.starts_with("Source: ")
        || l.starts_with("Requires")
        || l.starts_with("LevelReq: ")
        || l.starts_with("Implicits: ")
        || l.starts_with("Has Alt Variant")
        || l.starts_with("Selected Variant")
        || l.starts_with("Talisman Tier: ")
        || l.starts_with("Limited to: ")
        || l.starts_with("Upgrade: ")
        || l.starts_with("Radius: ")
        || l.starts_with("Sockets: ")
        || l.starts_with("Notable")
        || matches!(
            l,
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

/// Map an influence label string to the corresponding influence-set bit.
fn influence_bits(line: &str) -> u8 {
    match line {
        "Shaper Item" => 0b0000_0001,
        "Elder Item" => 0b0000_0010,
        "Crusader Item" => 0b0000_0100,
        "Hunter Item" => 0b0000_1000,
        "Redeemer Item" => 0b0001_0000,
        "Warlord Item" => 0b0010_0000,
        "Searing Exarch Item" => 0b0100_0000,
        "Eater of Worlds Item" => 0b1000_0000,
        _ => 0,
    }
}
