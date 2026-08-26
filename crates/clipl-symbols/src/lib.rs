//! Curated symbol and kaomoji catalogs (offline).

#![forbid(unsafe_code)]

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const SYMBOLS_JSON: &str = include_str!("../../../packages/symbols-data/symbols.json");
const KAOMOJI_JSON: &str = include_str!("../../../packages/symbols-data/kaomoji.json");

static SYMBOLS: OnceLock<SymbolCatalog> = OnceLock::new();
static KAOMOJI: OnceLock<SymbolCatalog> = OnceLock::new();

/// A pasteable symbol, punctuation sequence, or kaomoji.
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

#[derive(Deserialize)]
struct PackedFile {
    groups: Vec<String>,
    #[serde(default)]
    symbols: Vec<PackedItem>,
    #[serde(default)]
    kaomoji: Vec<PackedItem>,
}

#[derive(Deserialize)]
struct PackedItem {
    g: String,
    n: String,
    c: String,
    #[serde(default)]
    k: Vec<String>,
}

/// In-memory catalog.
#[derive(Clone, Debug)]
pub struct SymbolCatalog {
    entries: Vec<Symbol>,
    groups: Vec<String>,
}

impl SymbolCatalog {
    /// Curated Unicode symbols.
    pub fn symbols() -> &'static Self {
        SYMBOLS.get_or_init(|| Self::from_packed(SYMBOLS_JSON))
    }

    /// Curated kaomoji / text faces.
    pub fn kaomoji() -> &'static Self {
        KAOMOJI.get_or_init(|| Self::from_packed(KAOMOJI_JSON))
    }

    /// Builtin subset used by older tests (`symbols()`).
    pub fn builtin() -> &'static Self {
        Self::symbols()
    }

    fn from_packed(json: &str) -> Self {
        let file: PackedFile =
            serde_json::from_str(json).expect("packed symbol catalog must parse");
        let entries: Vec<Symbol> = file
            .symbols
            .into_iter()
            .chain(file.kaomoji)
            .map(|item| Symbol {
                glyph: item.g,
                name: item.n,
                group: item.c,
                keywords: item.k,
            })
            .collect();
        Self {
            entries,
            groups: file.groups,
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

    /// Category names.
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    /// Ranked case-insensitive search. Empty query returns no rows.
    pub fn search(&self, query: &str, limit: usize) -> Vec<&Symbol> {
        let limit = limit.clamp(1, 400);
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return Vec::new();
        }
        let tokens: Vec<&str> = q.split_whitespace().collect();
        let mut scored: Vec<(u32, &Symbol)> = self
            .entries
            .iter()
            .filter_map(|symbol| {
                let score = score_symbol(symbol, query.trim(), &q, &tokens);
                (score > 0).then_some((score, symbol))
            })
            .collect();
        scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.name.cmp(&b.1.name))
                .then_with(|| a.1.glyph.cmp(&b.1.glyph))
        });
        scored.truncate(limit);
        scored.into_iter().map(|(_, s)| s).collect()
    }

    /// All items in a group.
    pub fn list_group(&self, group: &str) -> Vec<&Symbol> {
        self.entries
            .iter()
            .filter(|symbol| symbol.group.eq_ignore_ascii_case(group))
            .collect()
    }

    /// Lookup by glyph.
    pub fn by_glyph(&self, glyph: &str) -> Option<&Symbol> {
        self.entries.iter().find(|symbol| symbol.glyph == glyph)
    }
}

fn score_symbol(symbol: &Symbol, raw: &str, lowered: &str, tokens: &[&str]) -> u32 {
    if symbol.glyph == raw {
        return 10_000;
    }
    let name = symbol.name.to_ascii_lowercase();
    if name == lowered {
        return 9_000;
    }
    if symbol
        .keywords
        .iter()
        .any(|kw| kw.eq_ignore_ascii_case(lowered))
    {
        return 8_500;
    }
    if tokens.iter().all(|tok| name.contains(tok)) {
        return 5_500;
    }
    let mut matched = 0u32;
    for tok in tokens {
        if name.contains(tok)
            || symbol
                .keywords
                .iter()
                .any(|kw| kw == tok || kw.contains(tok))
        {
            matched += 1;
        }
    }
    if matched == tokens.len() as u32 && matched > 0 {
        return 4_000;
    }
    if matched > 0 {
        return 1_000 + matched * 50;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_em_dash() {
        let catalog = SymbolCatalog::builtin();
        let hits = catalog.search("em dash", 5);
        assert_eq!(hits[0].glyph, "—");
    }

    #[test]
    fn math_category_and_search() {
        let catalog = SymbolCatalog::symbols();
        assert!(catalog.list_group("Math").iter().any(|s| s.glyph == "≠"));
        assert_eq!(catalog.search("euro", 3)[0].glyph, "€");
        assert!(catalog.search("", 10).is_empty());
    }

    #[test]
    fn kaomoji_preserves_unicode_and_categories() {
        let catalog = SymbolCatalog::kaomoji();
        assert!(catalog.groups().contains(&"Table Flip".to_string()));
        let flip = catalog.search("table flip", 5);
        assert!(flip[0].glyph.contains("┻"));
        let shrug = catalog.search("shrug", 5);
        assert!(shrug.iter().any(|s| s.glyph.contains("ツ")));
        let lenny = catalog.search("lenny", 3);
        assert_eq!(lenny[0].glyph, "( ͡° ͜ʖ ͡°)");
        assert!(catalog
            .list_group("Cute")
            .iter()
            .any(|s| s.glyph.contains("ʕ")));
    }
}
