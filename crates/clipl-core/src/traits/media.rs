//! Media and sticker provider interfaces.

use crate::error::Result;
use crate::id::StickerPackId;
use crate::media::{MediaItem, MediaKind, StickerPack};

/// Search query for a [`MediaProvider`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaQuery {
    /// Free-text query.
    pub text: String,
    /// Optional kind filter.
    pub kind: Option<MediaKind>,
    /// Maximum results to return.
    pub limit: u32,
}

impl MediaQuery {
    /// Text-only query with a default limit of 24.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: None,
            limit: 24,
        }
    }
}

/// Replaceable GIF / media search backend.
///
/// Providers must degrade offline: [`MediaProvider::is_available`] is `false`
/// when the network (or local cache) cannot satisfy requests.
pub trait MediaProvider: Send + Sync {
    /// Provider identifier, e.g. `offline`, `giphy`.
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn display_name(&self) -> &str;

    /// Whether search can succeed right now.
    fn is_available(&self) -> bool;

    /// Search for media. Must not panic if offline; return an error instead.
    fn search(&self, query: &MediaQuery) -> Result<Vec<MediaItem>>;
}

/// Source of sticker packs (builtin, local directory, community).
pub trait StickerPackProvider: Send + Sync {
    /// Provider identifier.
    fn id(&self) -> &'static str;

    /// List installed or available packs.
    fn list_packs(&self) -> Result<Vec<StickerPack>>;

    /// Load a pack by id.
    fn load_pack(&self, id: &StickerPackId) -> Result<Option<StickerPack>>;
}
