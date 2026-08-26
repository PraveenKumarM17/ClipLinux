//! In-memory [`HistoryStore`] used by unit tests.

use std::sync::Mutex;

use clipl_core::placeholders::MemoryStorage;
use clipl_core::{ClipboardItem, ClipboardItemId, Error, Result, StorageBackend, Timestamp};

use crate::store::{HistoryQuery, HistoryStore};

const NAMESPACE: &str = "clipboard-history";

/// JSON documents in [`MemoryStorage`].
pub struct MemoryHistoryStore {
    storage: MemoryStorage,
    /// Insertion order so `latest()` is the last capture, not an arbitrary UUID.
    order: Mutex<Vec<ClipboardItemId>>,
}

impl Default for MemoryHistoryStore {
    fn default() -> Self {
        Self {
            storage: MemoryStorage::default(),
            order: Mutex::new(Vec::new()),
        }
    }
}

impl MemoryHistoryStore {
    fn load_all(&self) -> Result<Vec<ClipboardItem>> {
        let mut items = Vec::new();
        for key in self.storage.list_keys(NAMESPACE, "")? {
            if let Some(bytes) = self.storage.get(NAMESPACE, &key)? {
                items.push(
                    serde_json::from_slice(&bytes)
                        .map_err(|err| Error::Invalid(err.to_string()))?,
                );
            }
        }
        Ok(items)
    }

    fn write(&self, item: &ClipboardItem) -> Result<()> {
        let bytes = serde_json::to_vec(item).map_err(|err| Error::Storage(err.to_string()))?;
        self.storage.put(NAMESPACE, &item.id.to_string(), &bytes)
    }
}

impl HistoryStore for MemoryHistoryStore {
    fn insert(&self, item: &ClipboardItem) -> Result<()> {
        self.write(item)?;
        self.order
            .lock()
            .map_err(|_| Error::Storage("memory history lock poisoned".into()))?
            .push(item.id);
        Ok(())
    }

    fn update(&self, item: &ClipboardItem) -> Result<()> {
        self.write(item)
    }

    fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>> {
        match self.storage.get(NAMESPACE, &id.to_string())? {
            Some(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).map_err(|err| Error::Invalid(err.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    fn list(&self, query: &HistoryQuery) -> Result<Vec<ClipboardItem>> {
        let mut items = self.load_all()?;
        if let Some(now) = query.now {
            items.retain(|item| item.expires_at.map_or(true, |exp| exp > now));
        }
        items.sort_by(|a, b| match b.pinned.cmp(&a.pinned) {
            std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at),
            other => other,
        });
        items.truncate(query.limit);
        Ok(items)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardItem>> {
        let needle = query.to_ascii_lowercase();
        let mut items: Vec<_> = self
            .load_all()?
            .into_iter()
            .filter(|item| {
                item.content
                    .text_for_scan()
                    .is_some_and(|text| text.to_ascii_lowercase().contains(&needle))
            })
            .collect();
        items.sort_by(|a, b| match b.pinned.cmp(&a.pinned) {
            std::cmp::Ordering::Equal => b.created_at.cmp(&a.created_at),
            other => other,
        });
        items.truncate(limit);
        Ok(items)
    }

    fn delete(&self, id: ClipboardItemId) -> Result<bool> {
        let deleted = self.storage.delete(NAMESPACE, &id.to_string())?;
        if deleted {
            self.order
                .lock()
                .map_err(|_| Error::Storage("memory history lock poisoned".into()))?
                .retain(|existing| *existing != id);
        }
        Ok(deleted)
    }

    fn clear_unpinned(&self) -> Result<u64> {
        let mut n = 0u64;
        for item in self.load_all()? {
            if !item.pinned && self.delete(item.id)? {
                n += 1;
            }
        }
        Ok(n)
    }

    fn latest(&self) -> Result<Option<ClipboardItem>> {
        let order = self
            .order
            .lock()
            .map_err(|_| Error::Storage("memory history lock poisoned".into()))?;
        let Some(id) = order.last().copied() else {
            return Ok(None);
        };
        drop(order);
        self.get(id)
    }

    fn enforce_limit(&self, max_unpinned: usize) -> Result<u64> {
        let mut unpinned: Vec<_> = self
            .load_all()?
            .into_iter()
            .filter(|item| !item.pinned)
            .collect();
        unpinned.sort_by_key(|item| item.created_at);
        let mut removed = 0u64;
        while unpinned.len() > max_unpinned {
            if let Some(item) = unpinned.first() {
                let id = item.id;
                unpinned.remove(0);
                if self.delete(id)? {
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }

    fn expire(&self, now: Timestamp, max_age: Option<Timestamp>) -> Result<u64> {
        let mut removed = 0u64;
        for item in self.load_all()? {
            if item.pinned {
                continue;
            }
            let expired = item.expires_at.is_some_and(|exp| exp <= now);
            let aged = max_age.is_some_and(|cutoff| item.created_at < cutoff);
            if (expired || aged) && self.delete(item.id)? {
                removed += 1;
            }
        }
        Ok(removed)
    }
}
