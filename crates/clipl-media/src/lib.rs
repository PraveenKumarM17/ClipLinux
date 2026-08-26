//! Media provider registry.
//!
//! Remote GIF providers are not bundled. The registry always includes an
//! offline provider so the picker works without a network.

#![forbid(unsafe_code)]

use clipl_core::placeholders::{EmptyStickerPackProvider, OfflineMediaProvider};
use clipl_core::{
    MediaItem, MediaProvider, MediaQuery, Result, StickerPack, StickerPackId, StickerPackProvider,
};

/// Ordered list of media providers. First available provider that returns
/// results wins for a given query; later ones are fallbacks.
pub struct MediaRegistry {
    providers: Vec<Box<dyn MediaProvider>>,
}

impl Default for MediaRegistry {
    fn default() -> Self {
        Self {
            providers: vec![Box::new(OfflineMediaProvider)],
        }
    }
}

impl MediaRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider. Duplicate ids replace the previous entry.
    pub fn register(&mut self, provider: Box<dyn MediaProvider>) {
        self.providers.retain(|p| p.id() != provider.id());
        self.providers.push(provider);
    }

    /// Providers currently able to serve requests.
    pub fn available(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.id())
            .collect()
    }

    /// Search every available provider and concatenate results up to `limit`.
    pub fn search(&self, query: &MediaQuery) -> Result<Vec<MediaItem>> {
        let mut out = Vec::new();
        for provider in self.providers.iter().filter(|p| p.is_available()) {
            let mut items = provider.search(query)?;
            out.append(&mut items);
            if out.len() >= query.limit as usize {
                out.truncate(query.limit as usize);
                break;
            }
        }
        Ok(out)
    }
}

/// Local sticker packs. The filesystem scanner is not implemented yet.
#[derive(Debug, Default)]
pub struct LocalStickerLibrary {
    inner: EmptyStickerPackProvider,
}

impl StickerPackProvider for LocalStickerLibrary {
    fn id(&self) -> &'static str {
        "local-stickers"
    }

    fn list_packs(&self) -> Result<Vec<StickerPack>> {
        self.inner.list_packs()
    }

    fn load_pack(&self, id: &StickerPackId) -> Result<Option<StickerPack>> {
        self.inner.load_pack(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_offline_only() {
        let registry = MediaRegistry::default();
        assert_eq!(registry.available(), vec!["offline"]);
        let results = registry.search(&MediaQuery::new("cats")).unwrap();
        assert!(results.is_empty());
    }
}
