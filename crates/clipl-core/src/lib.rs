//! ClipLinux domain model and extension traits.
//!
//! This crate is the stable center of the ecosystem. It must not depend on
//! Tauri, Svelte, GTK, Qt, SQLite drivers, or compositor-specific crates.
//! Platform behavior lives behind the traits defined here and is implemented
//! in sibling crates.

#![forbid(unsafe_code)]

pub mod activation;
pub mod capabilities;
pub mod clipboard;
pub mod config;
pub mod emoji;
pub mod error;
pub mod id;
pub mod media;
pub mod paths;
pub mod placeholders;
pub mod platform;
pub mod privacy;
pub mod snippet;
pub mod timestamp;
pub mod traits;

pub use activation::{
    ActivationBackendKind, ActivationBehavior, ActivationCapability, ActivationRequest,
    ActivationSlotState, ActivationSnapshot, ActivationStatus, Shortcut, DEFAULT_SHORTCUT,
    GNOME_EXTENSION_UUID,
};
pub use capabilities::{Capability, PlatformCapabilities, SupportLevel};
pub use clipboard::{ClipboardContent, ClipboardItem, ClipboardSource, ContentRef};
pub use config::{
    ActivationConfig, ActivationGnomeConfig, ActivationX11Config, ClipLinuxConfig, ClipboardConfig,
    HistoryConfig, InsertConfig, PrivacyConfig,
};
pub use emoji::{Emoji, SkinTone};
pub use error::{Error, Result};
pub use id::{ClipboardItemId, EmojiId, MediaItemId, PrivacyRuleId, SnippetId, StickerPackId};
pub use media::{MediaItem, MediaKind, MediaSource, PackSource, StickerPack};
pub use paths::{ClipLinuxPaths, APP_DIR_NAME};
pub use platform::{DesktopEnvironment, Platform, PlatformIdentity, SessionType};
pub use privacy::{PrivacyAction, PrivacyMatcher, PrivacyRule, SensitiveContentType};
pub use snippet::Snippet;
pub use timestamp::Timestamp;
pub use traits::{
    ActivationBackend, ClipboardBackend, ClipboardWatcher, MediaProvider, MediaQuery,
    PlatformAdapter, StickerPackProvider, StorageBackend,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_types_are_serde_round_trippable() {
        let item = ClipboardItem::text("hello");
        let json = serde_json::to_string(&item).expect("serialize");
        let back: ClipboardItem = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(item.content, back.content);
    }
}
