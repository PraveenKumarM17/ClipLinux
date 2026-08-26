//! Clipboard history management.
//!
//! OS clipboard watching lives in `clipl-platform` and the daemon. This crate
//! records, deduplicates, and queries history. Privacy runs **before** SQLite.

#![forbid(unsafe_code)]

mod engine;
mod hash;
mod memory;
mod sqlite;
mod store;

use clipl_core::{
    ClipLinuxConfig, ClipboardBackend, ClipboardItem, ClipboardItemId, PrivacyRule, Result,
};

pub use engine::{for_client, HistoryEngine, RecordOutcome, Recorded};
pub use hash::content_hash;
pub use memory::MemoryHistoryStore;
pub use sqlite::{SqliteStore, SCHEMA_VERSION};
pub use store::{HistoryQuery, HistoryStore};

/// Convenience history handle used by tests (in-memory store).
pub struct ClipboardHistory {
    engine: HistoryEngine<MemoryHistoryStore>,
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new(clipl_privacy::default_rules())
    }
}

impl ClipboardHistory {
    /// In-memory history with the given privacy rules and default config.
    pub fn new(rules: Vec<PrivacyRule>) -> Self {
        Self::with_config(rules, ClipLinuxConfig::default())
    }

    /// In-memory history with explicit config.
    pub fn with_config(rules: Vec<PrivacyRule>, config: ClipLinuxConfig) -> Self {
        Self {
            engine: HistoryEngine::new(MemoryHistoryStore::default(), rules, config),
        }
    }

    /// Record an item unless privacy policy excludes it.
    pub fn record(&self, item: &ClipboardItem) -> Result<RecordOutcome> {
        Ok(self.engine.record(item)?.outcome)
    }

    /// Fetch one item.
    pub fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>> {
        self.engine.get(id)
    }

    /// List items, newest first.
    pub fn list(&self, limit: usize) -> Result<Vec<ClipboardItem>> {
        self.engine.list(limit)
    }

    /// Remove an item.
    pub fn delete(&self, id: ClipboardItemId) -> Result<bool> {
        self.engine.delete(id)
    }

    /// Copy an item onto a clipboard backend (paste pipeline comes later).
    pub fn write_to_backend<B: ClipboardBackend>(
        &self,
        id: ClipboardItemId,
        backend: &B,
    ) -> Result<()> {
        let item = self
            .get(id)?
            .ok_or_else(|| clipl_core::Error::not_found(id.to_string()))?;
        backend.write(&item.content)
    }

    /// Engine for advanced tests.
    pub fn engine(&self) -> &HistoryEngine<MemoryHistoryStore> {
        &self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::{ClipboardItem, SensitiveContentType, Timestamp};

    #[test]
    fn records_and_lists_text() {
        let history = ClipboardHistory::default();
        let item = ClipboardItem::text("alpha");
        assert_eq!(history.record(&item).unwrap(), RecordOutcome::Stored);
        let listed = history.list(10).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, item.id);
    }

    #[test]
    fn excludes_labelled_secrets() {
        let history = ClipboardHistory::default();
        let mut item = ClipboardItem::text("secret");
        item.sensitive.push(SensitiveContentType::Password);
        assert_eq!(history.record(&item).unwrap(), RecordOutcome::Excluded);
        assert!(history.list(10).unwrap().is_empty());
    }

    #[test]
    fn privacy_runs_before_persist_for_pem() {
        let history = ClipboardHistory::default();
        let item = ClipboardItem::text(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nfake\n-----END OPENSSH PRIVATE KEY-----",
        );
        assert_eq!(history.record(&item).unwrap(), RecordOutcome::Excluded);
        assert!(history.list(10).unwrap().is_empty());
    }

    #[test]
    fn consecutive_duplicate_reuses_row() {
        let history = ClipboardHistory::default();
        let first = ClipboardItem::text("same");
        let second = ClipboardItem::text("same");
        assert_eq!(history.record(&first).unwrap(), RecordOutcome::Stored);
        assert_eq!(history.record(&second).unwrap(), RecordOutcome::Reused);
        assert_eq!(history.list(10).unwrap().len(), 1);
    }

    #[test]
    fn non_consecutive_duplicate_inserts_again() {
        let history = ClipboardHistory::default();
        history.record(&ClipboardItem::text("a")).unwrap();
        history.record(&ClipboardItem::text("b")).unwrap();
        history.record(&ClipboardItem::text("a")).unwrap();
        assert_eq!(history.list(10).unwrap().len(), 3);
    }

    #[test]
    fn pinned_survives_limit() {
        let config = ClipLinuxConfig {
            history: clipl_core::HistoryConfig {
                max_items: 2,
                max_age_days: 0,
                ..clipl_core::HistoryConfig::default()
            },
            ..ClipLinuxConfig::default()
        };
        let history = ClipboardHistory::with_config(clipl_privacy::default_rules(), config);
        let pin = ClipboardItem::text("keep");
        history.record(&pin).unwrap();
        history.engine().set_pinned(pin.id, true).unwrap();
        history.record(&ClipboardItem::text("two")).unwrap();
        history.record(&ClipboardItem::text("three")).unwrap();
        let listed = history.list(10).unwrap();
        assert!(listed.iter().any(|item| item.id == pin.id && item.pinned));
        assert_eq!(listed.iter().filter(|item| !item.pinned).count(), 2);
    }

    #[test]
    fn max_age_expires_unpinned() {
        let config = ClipLinuxConfig {
            history: clipl_core::HistoryConfig {
                max_age_days: 1,
                ..clipl_core::HistoryConfig::default()
            },
            ..ClipLinuxConfig::default()
        };
        let store = MemoryHistoryStore::default();
        let engine = HistoryEngine::new(store, clipl_privacy::default_rules(), config);
        let mut old = ClipboardItem::text("old");
        old.created_at = Timestamp::from_millis(1);
        old.updated_at = old.created_at;
        old.content_hash = content_hash(&old.content);
        engine.store().insert(&old).unwrap();
        engine.apply_retention().unwrap();
        assert!(engine.get(old.id).unwrap().is_none());
    }

    #[test]
    fn refuses_to_delete_pinned() {
        let history = ClipboardHistory::default();
        let item = ClipboardItem::text("keep");
        history.record(&item).unwrap();
        history.engine().set_pinned(item.id, true).unwrap();
        assert!(history.engine().delete(item.id).is_err());
        assert!(history.engine().get(item.id).unwrap().is_some());
    }

    #[test]
    fn for_client_hides_sensitive_payload() {
        let mut item = ClipboardItem::text("super-secret");
        item.sensitive
            .push(clipl_core::SensitiveContentType::Password);
        let out = for_client(item);
        assert!(out.content.text_for_scan().unwrap().is_empty());
        assert!(!out.sensitive.is_empty());
    }
}
