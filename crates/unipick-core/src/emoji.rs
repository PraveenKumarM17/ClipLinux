//! Emoji catalog record.

use serde::{Deserialize, Serialize};

use crate::id::EmojiId;

/// A Unicode emoji (or emoji sequence) with search metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emoji {
    /// Stable catalog identity.
    pub id: EmojiId,
    /// The emoji as a Unicode string, which may be a sequence.
    pub glyph: String,
    /// Machine-friendly slug, e.g. `grinning-face`.
    pub slug: String,
    /// Human-readable CLDR short name.
    pub name: String,
    /// Unicode group, e.g. `Smileys & Emotion`.
    pub group: String,
    /// Unicode subgroup, e.g. `face-smiling`.
    pub subgroup: String,
    /// Search keywords.
    pub keywords: Vec<String>,
    /// Unicode version that introduced the emoji, e.g. `15.0`.
    pub unicode_version: String,
    /// Whether Fitzpatrick skin-tone modifiers apply.
    pub skin_tone_support: bool,
}

impl Emoji {
    /// Minimal emoji record used by tests and placeholders.
    pub fn from_glyph(glyph: impl Into<String>, name: impl Into<String>) -> Self {
        let name = name.into();
        let slug = name.to_lowercase().replace(' ', "-");
        Self {
            id: EmojiId::new(),
            glyph: glyph.into(),
            slug,
            name,
            group: "Uncategorized".to_string(),
            subgroup: "uncategorized".to_string(),
            keywords: Vec::new(),
            unicode_version: "1.0".to_string(),
            skin_tone_support: false,
        }
    }
}
