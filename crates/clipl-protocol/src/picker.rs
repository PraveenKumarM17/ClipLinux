//! Compact picker DTOs. Keywords and codepoints stay in the daemon catalogs.

use serde::{Deserialize, Serialize};

/// Which picker catalog an item belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PickerKind {
    /// Unicode emoji.
    Emoji,
    /// Curated symbols.
    Symbol,
    /// Kaomoji / text faces.
    Kaomoji,
}

impl PickerKind {
    /// Stable database key.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Emoji => "emoji",
            Self::Symbol => "symbol",
            Self::Kaomoji => "kaomoji",
        }
    }
}

/// One row sent to the desktop picker. No keyword lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickerItem {
    /// Glyph to copy (skin tone already applied when relevant).
    pub glyph: String,
    /// Catalog key (default / untoned emoji glyph, or the symbol itself).
    pub base: String,
    /// Display name.
    pub name: String,
    /// Category label.
    pub category: String,
    /// Official skin-tone variants exist.
    pub has_skin_tones: bool,
    /// Official fully-qualified tone sequences (light…dark). Empty when none.
    #[serde(default)]
    pub variants: Vec<String>,
    /// User favorite.
    pub favorite: bool,
}

/// Skin-tone preference payload.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkinTonePref {
    /// `default`, `light`, `medium_light`, `medium`, `medium_dark`, `dark`.
    pub tone: String,
}
