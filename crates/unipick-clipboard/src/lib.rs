//! Clipboard history management.
//!
//! This crate records and queries history. It does **not** monitor the OS
//! clipboard; that belongs to the daemon once a real [`ClipboardBackend`]
//! is selected by capability detection.

#![forbid(unsafe_code)]

use unipick_core::placeholders::MemoryStorage;
use unipick_core::{
    ClipboardBackend, ClipboardItem, ClipboardItemId, Error, PrivacyRule, Result, StorageBackend,
};
use unipick_privacy::{decide, PrivacyDecision};

const NAMESPACE: &str = "clipboard-history";

/// In-memory (or injected) clipboard history.
pub struct ClipboardHistory<S: StorageBackend = MemoryStorage> {
    storage: S,
    rules: Vec<PrivacyRule>,
}

impl Default for ClipboardHistory<MemoryStorage> {
    fn default() -> Self {
        Self::new(MemoryStorage::default(), unipick_privacy::default_rules())
    }
}

impl<S: StorageBackend> ClipboardHistory<S> {
    /// Create a history store with the given backend and privacy rules.
    pub fn new(storage: S, rules: Vec<PrivacyRule>) -> Self {
        Self { storage, rules }
    }

    /// Record an item unless privacy policy excludes it.
    pub fn record(&self, item: &ClipboardItem) -> Result<RecordOutcome> {
        match decide(item, &self.rules) {
            PrivacyDecision::Exclude => return Ok(RecordOutcome::Excluded),
            PrivacyDecision::Confirm => return Ok(RecordOutcome::NeedsConfirmation),
            PrivacyDecision::Redact | PrivacyDecision::Expire { .. } | PrivacyDecision::Allow => {}
        }
        let bytes = serde_json::to_vec(item).map_err(|err| Error::Storage(err.to_string()))?;
        self.storage.put(NAMESPACE, &item.id.to_string(), &bytes)?;
        Ok(RecordOutcome::Stored)
    }

    /// Fetch one item.
    pub fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>> {
        match self.storage.get(NAMESPACE, &id.to_string())? {
            Some(bytes) => {
                let item = serde_json::from_slice(&bytes)
                    .map_err(|err| Error::Invalid(err.to_string()))?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    /// List items, newest first (by `created_at`).
    pub fn list(&self, limit: usize) -> Result<Vec<ClipboardItem>> {
        let keys = self.storage.list_keys(NAMESPACE, "")?;
        let mut items = Vec::new();
        for key in keys {
            if let Some(bytes) = self.storage.get(NAMESPACE, &key)? {
                let item: ClipboardItem = serde_json::from_slice(&bytes)
                    .map_err(|err| Error::Invalid(err.to_string()))?;
                items.push(item);
            }
        }
        items.sort_by_key(|item| std::cmp::Reverse(item.created_at));
        items.truncate(limit);
        Ok(items)
    }

    /// Remove an item.
    pub fn delete(&self, id: ClipboardItemId) -> Result<bool> {
        self.storage.delete(NAMESPACE, &id.to_string())
    }

    /// Copy an item onto a clipboard backend (paste pipeline comes later).
    pub fn write_to_backend<B: ClipboardBackend>(
        &self,
        id: ClipboardItemId,
        backend: &B,
    ) -> Result<()> {
        let item = self
            .get(id)?
            .ok_or_else(|| Error::not_found(id.to_string()))?;
        backend.write(&item.content)
    }
}

/// Result of attempting to persist a clipboard item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordOutcome {
    /// Item was stored.
    Stored,
    /// Privacy policy excluded the item.
    Excluded,
    /// Caller must confirm before storing.
    NeedsConfirmation,
}

#[cfg(test)]
mod tests {
    use super::*;
    use unipick_core::{ClipboardItem, SensitiveContentType};

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
}
