use crate::data::item_mods::StatMeta;
use regex::Regex;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::sync::OnceLock;

// ─── Public types ─────────────────────────────────────────────────────────────

/// Inverted map from normalised display template → translation candidates.
///
/// Built once at startup from `stat_translations.json` and stored in
/// `GameData`.  Use [`InvertedTranslations::resolve_line`] to convert a
/// single PoB unique-item mod line into raw stat IDs and values.
pub struct InvertedTranslations {
    lookup: FxHashMap<String, Vec<TranslationEntry>>,
    /// Forward map: stat_id → display template with `#` placeholders.
    /// Built at load time from the first English variant of each translation entry.
    /// Used to generate human-readable mod names in the crafting UI.
    forward_map: FxHashMap<String, String>,
}

pub struct TranslationEntry {
    pub stat_ids: Vec<String>,
    pub index_handlers: Vec<Vec<String>>,
    pub format: Vec<String>,
    pub condition: Vec<StatCondition>,
}

pub struct StatCondition {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub negated: Option<bool>,
}

/// The result of resolving a single PoB display mod line.
pub struct ResolvedLine {
    /// `(raw_stat_id, raw_internal_value)` pairs, one per numeric placeholder.
    pub stats: Vec<(String, f64)>,
    /// `true` when any stat in this line has `is_local: true` in `stats.json`.
    pub is_local: bool,
}

// ─── Private deserialization types ───────────────────────────────────────────

#[derive(Deserialize)]
struct RawEntry {
    ids: Vec<String>,
    #[serde(rename = "English")]
    english: Vec<RawVariant>,
}

#[derive(Deserialize)]
struct RawVariant {
    string: String,
    format: Vec<String>,
    index_handlers: Vec<Vec<String>>,
    condition: Vec<RawCondition>,
}

#[derive(Deserialize)]
struct RawCondition {
    min: Option<f64>,
    max: Option<f64>,
    negated: Option<bool>,
}

// ─── Regex helpers (compiled once) ───────────────────────────────────────────

/// Matches `{0}`, `{1}`, ... placeholder tokens in translation template strings.
fn placeholder_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\{[0-9]+\}").unwrap())
}

/// Matches a parenthesized numeric range like `(10-15)` or `(-5-5)`.
fn paren_range_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\(-?[0-9]+(?:\.[0-9]+)?--?[0-9]+(?:\.[0-9]+)?\)").unwrap())
}

/// Matches `+N` or `-N` signed numbers that appear *inside* a line (e.g. `Chain +1 times`).
fn signed_num_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[+-][0-9]+(?:\.[0-9]+)?").unwrap())
}

/// Matches any remaining plain (unsigned) number after the signed pass.
fn plain_num_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"[0-9]+(?:\.[0-9]+)?").unwrap())
}

/// Extracts numeric tokens from a PoB line for value recovery.
///
/// Capture groups:
///   - (1, 2): parenthesized range `(X-Y)` — averaged to `(X+Y)/2`
///   - (3):    any remaining signed or unsigned number
fn number_extract_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\((-?[0-9]+(?:\.[0-9]+)?)-(-?[0-9]+(?:\.[0-9]+)?)\)|([+-]?[0-9]+(?:\.[0-9]+)?)",
        )
        .unwrap()
    })
}

// ─── InvertedTranslations impl ───────────────────────────────────────────────

impl InvertedTranslations {
    /// Build the inverted map from the raw `stat_translations.json` content.
    ///
    /// # Normalisation applied to each template string
    ///
    /// 1. Replace `{0}`, `{1}`, … with `#`
    /// 2. Lowercase the result
    ///
    /// The POB line must be normalised the same way before lookup —
    /// see [`normalize_to_key`].
    pub fn build(json: &str) -> Result<Self, serde_json::Error> {
        let raw: Vec<RawEntry> = serde_json::from_str(json)?;
        let mut lookup: FxHashMap<String, Vec<TranslationEntry>> =
            FxHashMap::with_capacity_and_hasher(raw.len() * 2, Default::default());
        let mut forward_map: FxHashMap<String, String> =
            FxHashMap::with_capacity_and_hasher(raw.len() * 3, Default::default());

        for entry in &raw {
            // Populate forward map from the first English variant.
            if let Some(variant) = entry.english.first() {
                let template = placeholder_re()
                    .replace_all(&variant.string, "#")
                    .into_owned();
                for stat_id in &entry.ids {
                    forward_map
                        .entry(stat_id.clone())
                        .or_insert_with(|| template.clone());
                }
            }

            for variant in &entry.english {
                // Normalise the template string to a lookup key
                let key = placeholder_re()
                    .replace_all(&variant.string, "#")
                    .to_lowercase();

                let te = TranslationEntry {
                    stat_ids: entry.ids.clone(),
                    index_handlers: variant.index_handlers.clone(),
                    format: variant.format.clone(),
                    condition: variant
                        .condition
                        .iter()
                        .map(|c| StatCondition {
                            min: c.min,
                            max: c.max,
                            negated: c.negated,
                        })
                        .collect(),
                };

                lookup.entry(key).or_default().push(te);
            }
        }

        Ok(InvertedTranslations { lookup, forward_map })
    }

    /// Render a list of `(stat_id, value)` pairs into human-readable PoE-style
    /// display lines.
    ///
    /// Stats that share a translation entry (e.g. "adds # to # fire damage")
    /// are grouped into one line by consuming as many consecutive values as
    /// there are `#` placeholders in the template.
    ///
    /// Returns an empty `Vec` if no stats could be translated.
    pub fn render_mod_lines(&self, stat_values: &[(String, f64)]) -> Vec<String> {
        let mut lines = Vec::new();
        let mut i = 0;
        while i < stat_values.len() {
            let (stat_id, first_val) = &stat_values[i];
            let Some(template) = self.forward_map.get(stat_id.as_str()) else {
                i += 1;
                continue;
            };
            let slots = template.matches('#').count();
            // Build a result string by substituting each slot.
            let mut result = template.clone();
            for j in 0..slots {
                let v = stat_values.get(i + j).map(|(_, v)| *v).unwrap_or(0.0);
                result = substitute_one_hash(&result, v);
            }
            lines.push(result);
            i += slots.max(1);
        }
        // If nothing was translated, fall back to a generic stub rather than silence.
        if lines.is_empty() && !stat_values.is_empty() {
            let vals: Vec<String> = stat_values
                .iter()
                .map(|(id, v)| format!("{}: {}", id, format_poe_value(*v)))
                .collect();
            lines.push(vals.join(", "));
        }
        lines
    }

    /// Return the display template for a stat ID, e.g. `"#% increased Armour"`.
    /// Returns `None` if no translation is available for that stat.
    pub fn stat_display_template(&self, stat_id: &str) -> Option<&str> {
        self.forward_map.get(stat_id).map(|s| s.as_str())
    }

    /// Attempt to resolve a single PoB display mod line.
    ///
    /// Returns `None` if the line is not found in the map or cannot be
    /// matched (wrong number of values, condition mismatch, etc.).
    ///
    /// # PoB line pre-processing applied internally
    ///
    /// 1. Strip `{tag}` / `{variant}` prefixes
    /// 2. Trim whitespace
    /// 3. Skip header lines (`Every N seconds:`) and reminder text (`(The …`)
    /// 4. Strip leading `+`
    /// 5. Normalise em-dashes to `-`
    /// 6. Lowercase
    /// 7. Replace `(X-Y)` ranges → `#`
    /// 8. Replace `+N`/`-N` signed numbers → `#`
    /// 9. Replace remaining plain numbers → `#`
    pub fn resolve_line(
        &self,
        raw_line: &str,
        stat_meta: &FxHashMap<String, StatMeta>,
    ) -> Option<ResolvedLine> {
        let stripped = strip_tags(raw_line);
        if stripped.is_empty() {
            return None;
        }

        // Skip non-mod lines
        let lower = stripped.to_lowercase();
        if lower.ends_with(':') || lower.starts_with("(the ") || lower.starts_with("(this ") {
            return None;
        }

        let numbers = extract_numbers(&stripped);
        let key = normalize_to_key(&stripped);

        let candidates = self.lookup.get(&key)?;

        'candidate: for candidate in candidates {
            // Positions of numeric (non-"ignore") placeholders
            let numeric_slots: Vec<usize> = candidate
                .format
                .iter()
                .enumerate()
                .filter(|(_, f)| f.as_str() != "ignore")
                .map(|(i, _)| i)
                .collect();

            if numbers.len() < numeric_slots.len() {
                continue;
            }

            // Compute raw values and check conditions
            let mut raw_values: Vec<f64> = Vec::with_capacity(numeric_slots.len());
            for (numeric_idx, &slot_idx) in numeric_slots.iter().enumerate() {
                let displayed = numbers[numeric_idx];
                let handlers = candidate
                    .index_handlers
                    .get(slot_idx)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let raw = invert_handlers(handlers, displayed);

                if let Some(cond) = candidate.condition.get(slot_idx) {
                    if let Some(min) = cond.min {
                        if raw < min {
                            continue 'candidate;
                        }
                    }
                    if let Some(max) = cond.max {
                        if raw > max {
                            continue 'candidate;
                        }
                    }
                }

                raw_values.push(raw);
            }

            // Build output stats, skipping "ignore"-format stat positions
            let mut stats: Vec<(String, f64)> = Vec::with_capacity(candidate.stat_ids.len());
            let mut raw_idx = 0;
            for (slot_idx, stat_id) in candidate.stat_ids.iter().enumerate() {
                let fmt = candidate
                    .format
                    .get(slot_idx)
                    .map(String::as_str)
                    .unwrap_or("#");
                if fmt != "ignore" {
                    if let Some(&v) = raw_values.get(raw_idx) {
                        stats.push((stat_id.clone(), v));
                        raw_idx += 1;
                    }
                }
                // "ignore"-format positions have no numeric value; skip them
            }

            if stats.is_empty() {
                continue;
            }

            let is_local = stats
                .iter()
                .any(|(id, _)| stat_meta.get(id.as_str()).map_or(false, |m| m.is_local));

            return Some(ResolvedLine { stats, is_local });
        }

        None
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Remove `{...}` tag/variant annotation prefixes from a PoB line.
fn strip_tags(line: &str) -> String {
    let mut s = String::with_capacity(line.len());
    let mut depth: usize = 0;
    for ch in line.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            c if depth == 0 => s.push(c),
            _ => {}
        }
    }
    s.trim().to_owned()
}

/// Normalise a stripped PoB line into a lookup key matching the translation map.
fn normalize_to_key(stripped: &str) -> String {
    // Strip leading +, normalise em-dashes, lowercase
    let s = stripped.trim_start_matches('+');
    let s = s.replace('\u{2013}', "-").replace('\u{2014}', "-");
    let s = s.to_lowercase();

    // Replace parenthesised ranges first (before the signed-number pass
    // which would otherwise split them into two matches)
    let s = paren_range_re().replace_all(&s, "#").into_owned();

    // Replace signed numbers (+N / -N inside the line, e.g. "Chain +1 times")
    let s = signed_num_re().replace_all(&s, "#").into_owned();

    // Replace any remaining plain (unsigned) numbers
    plain_num_re().replace_all(&s, "#").into_owned()
}

/// Extract displayed numeric values from a stripped PoB line, left-to-right.
///
/// - Parenthesised ranges `(X-Y)` → `(X + Y) / 2.0`
/// - All other numbers → parsed directly (sign preserved)
fn extract_numbers(stripped: &str) -> Vec<f64> {
    number_extract_re()
        .captures_iter(stripped)
        .map(|cap| {
            if let (Some(lo), Some(hi)) = (cap.get(1), cap.get(2)) {
                let lo: f64 = lo.as_str().parse().unwrap_or(0.0);
                let hi: f64 = hi.as_str().parse().unwrap_or(0.0);
                (lo + hi) / 2.0
            } else if let Some(n) = cap.get(3) {
                n.as_str().parse().unwrap_or(0.0)
            } else {
                0.0
            }
        })
        .collect()
}

/// Apply the inverse of a handler chain to recover the raw stat value from the
/// displayed value.
///
/// Handlers are applied in reverse order (the forward chain is `h[0](h[1](...(raw)))`
/// so the inverse is `h[0]⁻¹(h[1]⁻¹(displayed))`).
fn invert_handlers(handlers: &[String], displayed: f64) -> f64 {
    let mut value = displayed;
    for handler in handlers.iter().rev() {
        value = match handler.as_str() {
            "negate" => -value,

            "divide_by_one_hundred"
            | "divide_by_one_hundred_2dp"
            | "divide_by_one_hundred_2dp_if_required" => value * 100.0,

            "divide_by_one_hundred_and_negate" => value * -100.0,

            "divide_by_ten_0dp" | "divide_by_ten_1dp" | "divide_by_ten_1dp_if_required" => {
                value * 10.0
            }

            "divide_by_five" => value * 5.0,
            "divide_by_four" => value * 4.0,
            "divide_by_three" => value * 3.0,
            "divide_by_two_0dp" => value * 2.0,
            "divide_by_six" => value * 6.0,
            "divide_by_twelve" => value * 12.0,
            "divide_by_fifteen_0dp" => value * 15.0,
            "divide_by_twenty" => value * 20.0,
            "divide_by_one_thousand" => value * 1000.0,

            "double" => value / 2.0,
            "negate_and_double" => -(value / 2.0),

            "milliseconds_to_seconds"
            | "milliseconds_to_seconds_0dp"
            | "milliseconds_to_seconds_1dp"
            | "milliseconds_to_seconds_2dp"
            | "milliseconds_to_seconds_2dp_if_required" => value * 1000.0,

            "deciseconds_to_seconds" => value * 10.0,

            "per_minute_to_per_second"
            | "per_minute_to_per_second_0dp"
            | "per_minute_to_per_second_1dp"
            | "per_minute_to_per_second_2dp"
            | "per_minute_to_per_second_2dp_if_required" => value * 60.0,

            "permyriad_per_minute_to_%_per_second" => value * 6000.0,

            "times_twenty" => value / 20.0,
            "times_one_point_five" => value / 1.5,
            "30%_of_value" => value / 0.3,
            "60%_of_value" => value / 0.6,
            "plus_two_hundred" => value - 200.0,
            "multiplicative_damage_modifier" => value + 100.0,
            "old_leech_percent" => value * 5.0,
            "old_leech_permyriad" => value * 50.0,
            "locations_to_metres" => value * 10.0,
            "divide_by_twenty_then_double_0dp" => value * 10.0,

            // Display-only / lookup handlers — no numeric transformation
            _ => value,
        };
    }
    value
}

// ─── Rendering helpers ────────────────────────────────────────────────────────

/// Format a raw stat value as a PoE-style number string (integer or 1 decimal).
fn format_poe_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{:.1}", v)
    }
}

/// Substitute the first `#` in `template` with `value`.
///
/// If the template contains `+#` and the value is negative, the entire `+#`
/// token is replaced with the signed value (e.g. `+# to Fire Res` with -10
/// → `-10 to Fire Res`).
fn substitute_one_hash(template: &str, value: f64) -> String {
    let formatted = format_poe_value(value.abs());
    if value < 0.0 {
        // Try "+#" first so we replace the sign prefix correctly.
        if template.contains("+#") {
            return template.replacen("+#", &format!("-{}", formatted), 1);
        }
        template.replacen('#', &format!("-{}", formatted), 1)
    } else {
        template.replacen('#', &formatted, 1)
    }
}
