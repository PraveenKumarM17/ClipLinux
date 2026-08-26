//! Media and sticker domain types.

use serde::{Deserialize, Serialize};

use crate::clipboard::ContentRef;
use crate::id::{MediaItemId, StickerPackId};
use crate::timestamp::Timestamp;

/// A GIF, sticker, or other pasteable media record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaItem {
    /// Stable identity.
    pub id: MediaItemId,
    /// Kind of media.
    pub kind: MediaKind,
    /// Optional title from a provider or pack.
    pub title: Option<String>,
    /// Optional preview image.
    pub preview: Option<ContentRef>,
    /// Where the bytes can be resolved.
    pub source: MediaSource,
    /// Attribution required by a remote provider, if any.
    pub attribution: Option<String>,
    /// MIME type of the primary payload.
    pub mime: String,
    /// True when the item can be used without a network request.
    pub offline_available: bool,
}

/// Classification of a [`MediaItem`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaKind {
    /// Animated or still GIF.
    Gif,
    /// Sticker image, typically with transparency.
    Sticker,
    /// Short video clip.
    VideoClip,
    /// Static raster or vector image.
    Image,
}

/// How to resolve media bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MediaSource {
    /// Local file path.
    File { path: String },
    /// Remote URL owned by a media provider.
    Remote {
        /// Provider identifier, e.g. `giphy` or `tenor`.
        provider_id: String,
        /// Provider-specific resource locator.
        locator: String,
    },
    /// Bytes already stored via [`crate::StorageBackend`].
    Cached { hash: String },
}

/// A named collection of stickers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StickerPack {
    /// Stable identity.
    pub id: StickerPackId,
    /// Display name.
    pub name: String,
    /// Optional author or pack publisher.
    pub author: Option<String>,
    /// Stickers in the pack.
    pub items: Vec<MediaItem>,
    /// Origin of the pack.
    pub source: PackSource,
    /// When the pack was installed locally.
    pub installed_at: Timestamp,
}

/// Origin of a sticker pack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PackSource {
    /// Shipped with ClipLinux.
    Builtin,
    /// User-installed local directory.
    Local { path: String },
    /// Community content pack.
    Community { pack_id: String },
}
