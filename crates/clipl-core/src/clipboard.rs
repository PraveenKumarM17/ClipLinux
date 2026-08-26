//! Clipboard history records.

use serde::{Deserialize, Serialize};

use crate::emoji::Emoji;
use crate::id::{ClipboardItemId, MediaItemId, SnippetId};
use crate::privacy::SensitiveContentType;
use crate::timestamp::Timestamp;

/// A single clipboard or paste-palette entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardItem {
    /// Stable identity for history, pinning, and IPC.
    pub id: ClipboardItemId,
    /// Payload to paste or preview.
    pub content: ClipboardContent,
    /// Where the item originated.
    pub source: ClipboardSource,
    /// When the item first entered history.
    pub created_at: Timestamp,
    /// When the item was last pasted or selected, if ever.
    pub last_used_at: Option<Timestamp>,
    /// Pinned items survive eviction and appear first in the palette.
    pub pinned: bool,
    /// Optional user or system tags.
    pub tags: Vec<String>,
    /// Sensitive categories detected when the item was recorded.
    pub sensitive: Vec<SensitiveContentType>,
    /// SHA-256 hex of canonical content. Empty until the history engine fills it.
    #[serde(default)]
    pub content_hash: String,
    /// Last metadata update (dedup reuse, pin).
    #[serde(default)]
    pub updated_at: Timestamp,
    /// Optional expiry instant.
    #[serde(default)]
    pub expires_at: Option<Timestamp>,
    /// Application id when the platform provides one.
    #[serde(default)]
    pub source_app: Option<String>,
}

impl ClipboardItem {
    /// Construct a plain-text history item from the local session.
    pub fn text(text: impl Into<String>) -> Self {
        let now = Timestamp::now();
        Self {
            id: ClipboardItemId::new(),
            content: ClipboardContent::Text {
                text: text.into(),
                mime: "text/plain".to_string(),
            },
            source: ClipboardSource::LocalSession,
            created_at: now,
            last_used_at: None,
            pinned: false,
            tags: Vec::new(),
            sensitive: Vec::new(),
            content_hash: String::new(),
            updated_at: now,
            expires_at: None,
            source_app: None,
        }
    }
}

/// Payload stored in a [`ClipboardItem`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClipboardContent {
    /// Unicode text, including snippets expanded to text.
    Text {
        /// Text body.
        text: String,
        /// MIME type, usually `text/plain`.
        mime: String,
    },
    /// HTML with an optional plain-text fallback.
    Html {
        /// HTML body.
        html: String,
        /// Plain-text alternative.
        plain: Option<String>,
    },
    /// Raster image referenced inline or as a blob.
    Image {
        /// Image bytes or content-addressed blob.
        data: ContentRef,
        /// Image MIME type, e.g. `image/png`.
        mime: String,
        /// Pixel width when known.
        width: Option<u32>,
        /// Pixel height when known.
        height: Option<u32>,
    },
    /// File URIs copied from a file manager.
    Files {
        /// `file://` or other URIs.
        uris: Vec<String>,
    },
    /// A single URI or URL.
    Uri {
        /// URI string.
        uri: String,
    },
    /// A selected emoji ready to paste.
    Emoji {
        /// Emoji record.
        emoji: Emoji,
    },
    /// A GIF, sticker, or other media reference.
    Media {
        /// Media catalog identity.
        media_id: MediaItemId,
    },
    /// A snippet expanded at paste time.
    Snippet {
        /// Snippet identity.
        snippet_id: SnippetId,
    },
    /// Provider-specific payload that ClipLinux does not interpret.
    Custom {
        /// MIME type.
        mime: String,
        /// Opaque bytes.
        data: ContentRef,
    },
}

impl ClipboardContent {
    /// Return a short, privacy-conscious preview suitable for lists.
    pub fn preview(&self, max_chars: usize) -> String {
        match self {
            Self::Text { text, .. } => truncate(text, max_chars),
            Self::Html { plain, html, .. } => truncate(plain.as_deref().unwrap_or(html), max_chars),
            Self::Image {
                mime,
                width,
                height,
                ..
            } => {
                format!("image ({mime}, {width:?}x{height:?})")
            }
            Self::Files { uris } => format!("{} file(s)", uris.len()),
            Self::Uri { uri } => truncate(uri, max_chars),
            Self::Emoji { emoji } => emoji.glyph.clone(),
            Self::Media { media_id } => format!("media {media_id}"),
            Self::Snippet { snippet_id } => format!("snippet {snippet_id}"),
            Self::Custom { mime, .. } => format!("custom ({mime})"),
        }
    }

    /// Whether this payload is textual.
    pub fn is_text(&self) -> bool {
        matches!(
            self,
            Self::Text { .. } | Self::Html { .. } | Self::Uri { .. }
        )
    }

    /// Text examined by privacy detectors and search. Images return `None`.
    pub fn text_for_scan(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            Self::Html { plain, html, .. } => Some(plain.as_deref().unwrap_or(html)),
            Self::Uri { uri } => Some(uri),
            _ => None,
        }
    }

    /// MIME type when the variant carries one.
    pub fn mime(&self) -> Option<&str> {
        match self {
            Self::Text { mime, .. } | Self::Image { mime, .. } | Self::Custom { mime, .. } => {
                Some(mime)
            }
            Self::Html { .. } => Some("text/html"),
            _ => None,
        }
    }

    /// Stable type tag stored in SQLite `content_type`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Text { .. } => "text",
            Self::Html { .. } => "html",
            Self::Image { .. } => "image",
            Self::Files { .. } => "files",
            Self::Uri { .. } => "uri",
            Self::Emoji { .. } => "emoji",
            Self::Media { .. } => "media",
            Self::Snippet { .. } => "snippet",
            Self::Custom { .. } => "custom",
        }
    }

    /// Canonical bytes hashed for deduplication (not a secret log field).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        match self {
            Self::Text { text, mime } => {
                let mut out = Vec::from(&b"text\0"[..]);
                out.extend_from_slice(mime.as_bytes());
                out.push(0);
                out.extend_from_slice(text.as_bytes());
                out
            }
            Self::Html { html, plain } => {
                let mut out = Vec::from(&b"html\0"[..]);
                out.extend_from_slice(html.as_bytes());
                out.push(0);
                if let Some(plain) = plain {
                    out.extend_from_slice(plain.as_bytes());
                }
                out
            }
            Self::Uri { uri } => {
                let mut out = Vec::from(&b"uri\0"[..]);
                out.extend_from_slice(uri.as_bytes());
                out
            }
            other => format!("opaque:{}", other.type_name()).into_bytes(),
        }
    }
}

/// Location of payload bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentRef {
    /// Small payloads stored next to metadata.
    Inline(Vec<u8>),
    /// Content-addressed blob in a [`crate::StorageBackend`].
    Blob {
        /// Hex-encoded content hash.
        hash: String,
        /// Size in bytes.
        size: u64,
    },
}

impl ContentRef {
    /// Inline UTF-8 text.
    pub fn inline_text(text: &str) -> Self {
        Self::Inline(text.as_bytes().to_vec())
    }

    /// Byte length of an inline payload, or the recorded blob size.
    pub fn len(&self) -> usize {
        match self {
            Self::Inline(bytes) => bytes.len(),
            Self::Blob { size, .. } => *size as usize,
        }
    }

    /// Whether the reference holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Origin of a clipboard item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClipboardSource {
    /// Copied from an application in the current session.
    LocalSession,
    /// Created inside ClipLinux (emoji, snippet, media picker).
    ClipLinux,
    /// Imported from a file or backup.
    Import,
    /// Unknown or unspecified origin.
    Unknown,
}

fn truncate(input: &str, max_chars: usize) -> String {
    let mut chars = input.chars();
    let taken: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{taken}…")
    } else {
        taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_preview_truncates() {
        let content = ClipboardContent::Text {
            text: "abcdefghijklmnopqrstuvwxyz".to_string(),
            mime: "text/plain".to_string(),
        };
        assert_eq!(content.preview(5), "abcde…");
    }
}
