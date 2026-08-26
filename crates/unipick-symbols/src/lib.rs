//! Unicode symbol catalog (not emoji).
//!
//! A small builtin set keeps the picker usable offline. Full symbol packs can
//! be added under `packages/` later.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// A pasteable symbol or punctuation sequence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Glyph or sequence.
    pub glyph: String,
    /// Display name.
    pub name: String,
    /// Grouping label.
    pub group: String,
    /// Search keywords.
    pub keywords: Vec<String>,
}

/// In-memory symbol catalog.
#[derive(Clone, Debug, Default)]
pub struct SymbolCatalog {
    entries: Vec<Symbol>,
}

impl SymbolCatalog {
    /// Builtin subset used by the foundation.
    pub fn builtin() -> Self {
        Self {
            entries: builtin_symbols(),
        }
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Case-insensitive search on name, glyph, and keywords.
    pub fn search(&self, query: &str) -> Vec<&Symbol> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            .filter(|symbol| {
                symbol.glyph == query
                    || symbol.name.to_ascii_lowercase().contains(&q)
                    || symbol
                        .keywords
                        .iter()
                        .any(|kw| kw.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }
}

fn builtin_symbols() -> Vec<Symbol> {
    vec![
        sym("—", "Em dash", "punctuation", &["dash", "em"]),
        sym("–", "En dash", "punctuation", &["dash", "en"]),
        sym("…", "Ellipsis", "punctuation", &["dots"]),
        sym("©", "Copyright", "legal", &["copy"]),
        sym("®", "Registered", "legal", &["reg"]),
        sym("™", "Trademark", "legal", &["tm"]),
        sym("→", "Right arrow", "arrows", &["arrow"]),
        sym("←", "Left arrow", "arrows", &["arrow"]),
        sym("≠", "Not equal", "math", &["neq", "math"]),
        sym("≤", "Less than or equal", "math", &["lte"]),
        sym("≥", "Greater than or equal", "math", &["gte"]),
        sym("€", "Euro", "currency", &["euro"]),
        sym("£", "Pound", "currency", &["gbp"]),
        sym("¥", "Yen", "currency", &["jpy"]),
    ]
}

fn sym(glyph: &str, name: &str, group: &str, keywords: &[&str]) -> Symbol {
    Symbol {
        glyph: glyph.to_string(),
        name: name.to_string(),
        group: group.to_string(),
        keywords: keywords.iter().map(|s| (*s).to_string()).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_em_dash() {
        let catalog = SymbolCatalog::builtin();
        let hits = catalog.search("em dash");
        assert_eq!(hits[0].glyph, "—");
    }
}
