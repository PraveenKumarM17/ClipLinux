//! Emoji catalog record.

use serde::{Deserialize, Serialize};

use crate::id::EmojiId;

/// A Unicode emoji (or emoji sequence) with search metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emoji {
    /// Stable catalog identity derived from the glyph.
    pub id: EmojiId,
    /// The emoji as a Unicode string, which may be a sequence.
    pub glyph: String,
    /// Machine-friendly slug, e.g. `grinning-face`.
    pub slug: String,
    /// Human-readable CLDR / Unicode short name.
    pub name: String,
    /// Unicode group, e.g. `Smileys & Emotion`.
    pub group: String,
    /// Unicode subgroup, e.g. `face-smiling`.
    pub subgroup: String,
    /// Search keywords (names, CLDR annotations, aliases).
    pub keywords: Vec<String>,
    /// Extra aliases (often empty; keywords already include them).
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Unicode scalar values for the fully-qualified sequence.
    #[serde(default)]
    pub codepoints: Vec<u32>,
    /// Unicode version that introduced the emoji, e.g. `17.0`.
    pub unicode_version: String,
    /// Whether official Fitzpatrick variants exist in the catalog.
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
            aliases: Vec::new(),
            codepoints: Vec::new(),
            unicode_version: "1.0".to_string(),
            skin_tone_support: false,
        }
    }
}

/// Default / Fitzpatrick skin-tone preference.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinTone {
    /// Standard emoji presentation (typically yellow).
    #[default]
    Default,
    /// Light skin tone (U+1F3FB).
    Light,
    /// Medium-light skin tone (U+1F3FC).
    MediumLight,
    /// Medium skin tone (U+1F3FD).
    Medium,
    /// Medium-dark skin tone (U+1F3FE).
    MediumDark,
    /// Dark skin tone (U+1F3FF).
    Dark,
}

impl SkinTone {
    /// Index into a 5-length official variant list (`None` for default).
    pub fn variant_index(self) -> Option<usize> {
        match self {
            Self::Default => None,
            Self::Light => Some(0),
            Self::MediumLight => Some(1),
            Self::Medium => Some(2),
            Self::MediumDark => Some(3),
            Self::Dark => Some(4),
        }
    }
}
