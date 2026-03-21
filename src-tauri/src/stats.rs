use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Matches signed integers and decimals like 50, +30, -5, 0.5, +12.3
static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[+-]?\d+\.?\d*").unwrap());

pub struct StatAccumulator {
    totals: HashMap<String, f64>,
}

impl StatAccumulator {
    pub fn new() -> Self {
        Self {
            totals: HashMap::new(),
        }
    }

    /// Add a single stat string. Extracts the first number, replaces all numbers
    /// with `#` to form a template key, and accumulates the value.
    /// Stats with no number get stored with value += 1 (source count).
    pub fn add(&mut self, stat: &str) {
        if let Some(m) = NUM_RE.find(stat) {
            let value: f64 = m.as_str().parse().unwrap_or(0.0);
            let template = NUM_RE.replace_all(stat, "#").to_string();
            *self.totals.entry(template).or_insert(0.0) += value;
        } else {
            // Boolean/qualitative stat — count occurrences
            *self.totals.entry(stat.to_string()).or_insert(0.0) += 1.0;
        }
    }

    /// Add all stats for a node.
    pub fn add_all(&mut self, stats: &[String]) {
        for s in stats {
            self.add(s);
        }
    }

    /// Consume and return the accumulated totals.
    pub fn into_totals(self) -> HashMap<String, f64> {
        self.totals
    }
}
