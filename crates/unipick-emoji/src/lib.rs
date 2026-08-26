//! Emoji catalog and search.
//!
//! Full Unicode data lives in `packages/emoji-data`. This crate ships a tiny
//! builtin subset so offline search and tests work without extra files.

#![forbid(unsafe_code)]

use unipick_core::{Emoji, EmojiId};

/// In-memory emoji catalog.
#[derive(Clone, Debug, Default)]
pub struct EmojiCatalog {
    entries: Vec<Emoji>,
}

impl EmojiCatalog {
    /// Load the builtin subset (not the full Unicode set).
    pub fn builtin() -> Self {
        Self {
            entries: builtin_emoji(),
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

    /// Search by glyph, name, slug, or keyword. Case-insensitive.
    pub fn search(&self, query: &str) -> Vec<&Emoji> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return self.entries.iter().collect();
        }
        self.entries
            .iter()
            .filter(|emoji| {
                emoji.glyph == query
                    || emoji.name.to_ascii_lowercase().contains(&q)
                    || emoji.slug.to_ascii_lowercase().contains(&q)
                    || emoji
                        .keywords
                        .iter()
                        .any(|kw| kw.to_ascii_lowercase().contains(&q))
            })
            .collect()
    }

    /// Lookup by slug.
    pub fn by_slug(&self, slug: &str) -> Option<&Emoji> {
        self.entries.iter().find(|emoji| emoji.slug == slug)
    }
}

fn builtin_emoji() -> Vec<Emoji> {
    vec![
        make(
            "😀",
            "grinning-face",
            "Grinning Face",
            "Smileys & Emotion",
            &["smile", "happy"],
        ),
        make(
            "😂",
            "face-with-tears-of-joy",
            "Face with Tears of Joy",
            "Smileys & Emotion",
            &["lol", "joy"],
        ),
        make(
            "❤️",
            "red-heart",
            "Red Heart",
            "Smileys & Emotion",
            &["love", "heart"],
        ),
        make(
            "👍",
            "thumbs-up",
            "Thumbs Up",
            "People & Body",
            &["ok", "yes"],
        ),
        make(
            "🚀",
            "rocket",
            "Rocket",
            "Travel & Places",
            &["ship", "launch"],
        ),
        make(
            "🐧",
            "penguin",
            "Penguin",
            "Animals & Nature",
            &["linux", "tux"],
        ),
        make(
            "✅",
            "check-mark-button",
            "Check Mark Button",
            "Symbols",
            &["done", "yes"],
        ),
        make("🔥", "fire", "Fire", "Travel & Places", &["hot", "lit"]),
    ]
}

fn make(glyph: &str, slug: &str, name: &str, group: &str, keywords: &[&str]) -> Emoji {
    Emoji {
        id: EmojiId::new(),
        glyph: glyph.to_string(),
        slug: slug.to_string(),
        name: name.to_string(),
        group: group.to_string(),
        subgroup: "foundation".to_string(),
        keywords: keywords.iter().map(|s| (*s).to_string()).collect(),
        unicode_version: "1.0".to_string(),
        skin_tone_support: matches!(slug, "thumbs-up"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_finds_linux_penguin() {
        let catalog = EmojiCatalog::builtin();
        let hits = catalog.search("linux");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].glyph, "🐧");
    }

    #[test]
    fn empty_query_returns_all() {
        let catalog = EmojiCatalog::builtin();
        assert_eq!(catalog.search("").len(), catalog.len());
    }
}
