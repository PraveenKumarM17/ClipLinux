//! Compact catalog loaded once from packaged Unicode data.

use std::collections::HashMap;
use std::sync::OnceLock;

use clipl_core::{Emoji, EmojiId, SkinTone};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::search::{rank_records, SearchHit};
use crate::skin::apply_skin;

/// Unicode Emoji version packed into the compact catalog.
pub const UNICODE_VERSION: &str = "17.0";

const COMPACT_JSON: &str = include_str!("../../../packages/emoji-data/emoji.compact.json");

static CATALOG: OnceLock<EmojiCatalog> = OnceLock::new();

#[derive(Deserialize)]
struct CompactFile {
    unicode: String,
    groups: Vec<String>,
    emoji: Vec<CompactEmoji>,
}

#[derive(Deserialize)]
struct CompactEmoji {
    g: String,
    n: String,
    s: String,
    c: usize,
    u: String,
    v: String,
    p: Vec<u32>,
    k: Vec<String>,
    t: Option<Vec<String>>,
}

/// One catalog row plus official skin-tone sequences.
#[derive(Clone, Debug)]
pub struct EmojiRecord {
    /// Public emoji metadata.
    pub emoji: Emoji,
    /// Official fully-qualified sequences for light…dark, if present.
    pub skin_tones: Option<[String; 5]>,
}

impl EmojiRecord {
    /// Glyph after applying a stored preference. Never invents sequences.
    pub fn glyph_for(&self, tone: SkinTone) -> &str {
        apply_skin(self, tone)
    }
}

/// In-memory emoji catalog.
#[derive(Clone, Debug)]
pub struct EmojiCatalog {
    records: Vec<EmojiRecord>,
    groups: Vec<String>,
    by_glyph: HashMap<String, usize>,
    by_slug: HashMap<String, usize>,
}

impl EmojiCatalog {
    /// Shared process catalog (parsed once).
    pub fn shared() -> &'static Self {
        CATALOG.get_or_init(Self::load_packed)
    }

    /// Parse the packed dataset. Panics only if the generated file is corrupt
    /// (a build/packaging bug).
    pub fn load_packed() -> Self {
        let file: CompactFile =
            serde_json::from_str(COMPACT_JSON).expect("packed emoji catalog must parse");
        debug_assert_eq!(file.unicode, UNICODE_VERSION);
        let mut records = Vec::with_capacity(file.emoji.len());
        let mut by_glyph = HashMap::with_capacity(file.emoji.len());
        let mut by_slug = HashMap::with_capacity(file.emoji.len());
        for (idx, row) in file.emoji.into_iter().enumerate() {
            let group = file
                .groups
                .get(row.c)
                .cloned()
                .unwrap_or_else(|| "Symbols".into());
            let skin_tones = row.t.and_then(|tones| {
                if tones.len() == 5 {
                    Some([
                        tones[0].clone(),
                        tones[1].clone(),
                        tones[2].clone(),
                        tones[3].clone(),
                        tones[4].clone(),
                    ])
                } else {
                    None
                }
            });
            let emoji = Emoji {
                id: glyph_id(&row.g),
                glyph: row.g.clone(),
                slug: row.s.clone(),
                name: row.n,
                group,
                subgroup: row.u,
                keywords: row.k,
                aliases: Vec::new(),
                codepoints: row.p,
                unicode_version: row.v,
                skin_tone_support: skin_tones.is_some(),
            };
            by_glyph.insert(row.g, idx);
            by_slug.insert(row.s, idx);
            records.push(EmojiRecord { emoji, skin_tones });
        }
        Self {
            records,
            groups: file.groups,
            by_glyph,
            by_slug,
        }
    }

    /// Number of base (non-tone) emoji.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Unicode group names in CLDR / emoji-test order.
    pub fn groups(&self) -> &[String] {
        &self.groups
    }

    /// Lookup by fully-qualified default glyph.
    pub fn by_glyph(&self, glyph: &str) -> Option<&EmojiRecord> {
        self.by_glyph.get(glyph).map(|idx| &self.records[*idx])
    }

    /// Lookup by slug.
    pub fn by_slug(&self, slug: &str) -> Option<&EmojiRecord> {
        self.by_slug.get(slug).map(|idx| &self.records[*idx])
    }

    /// All records in catalog order.
    pub fn records(&self) -> &[EmojiRecord] {
        &self.records
    }

    /// Ranked search. Empty query yields no hits (callers list a category).
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchHit<'_>> {
        rank_records(&self.records, query, limit)
    }

    /// List a Unicode group (not Frequently Used).
    pub fn list_group(&self, group: &str) -> Vec<&EmojiRecord> {
        self.records
            .iter()
            .filter(|record| record.emoji.group.eq_ignore_ascii_case(group))
            .collect()
    }
}

fn glyph_id(glyph: &str) -> EmojiId {
    let hash = Sha256::digest(glyph.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    EmojiId::from_uuid(Uuid::from_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_catalog_is_unicode_17() {
        let catalog = EmojiCatalog::load_packed();
        assert_eq!(catalog.groups().len(), 9);
        assert!(catalog.len() > 2000);
        assert_eq!(catalog.by_slug("penguin").unwrap().emoji.glyph, "🐧");
    }
}
