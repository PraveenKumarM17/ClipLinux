//! Compile-time placeholders. Not production Linux backends.

use std::collections::BTreeMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::capabilities::PlatformCapabilities;
use crate::clipboard::ClipboardContent;
use crate::error::{Error, Result};
use crate::id::StickerPackId;
use crate::media::{MediaItem, StickerPack};
use crate::platform::PlatformIdentity;
use crate::traits::{
    ClipboardBackend, ClipboardWatcher, MediaProvider, MediaQuery, PlatformAdapter,
    StickerPackProvider, StorageBackend,
};

struct MemoryClipboardInner {
    content: Mutex<Option<ClipboardContent>>,
    generation: Mutex<u64>,
    changed: Condvar,
}

/// Clipboard backend that stores a single in-process value and can wait on writes.
#[derive(Clone)]
pub struct MemoryClipboard {
    inner: Arc<MemoryClipboardInner>,
}

impl Default for MemoryClipboard {
    fn default() -> Self {
        Self {
            inner: Arc::new(MemoryClipboardInner {
                content: Mutex::new(None),
                generation: Mutex::new(0),
                changed: Condvar::new(),
            }),
        }
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn read(&self) -> Result<Option<ClipboardContent>> {
        let guard = self
            .inner
            .content
            .lock()
            .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
        Ok(guard.clone())
    }

    fn write(&self, content: &ClipboardContent) -> Result<()> {
        {
            let mut guard = self
                .inner
                .content
                .lock()
                .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
            *guard = Some(content.clone());
        }
        {
            let mut gen = self
                .inner
                .generation
                .lock()
                .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
            *gen = gen.saturating_add(1);
        }
        self.inner.changed.notify_all();
        Ok(())
    }

    fn supports_watch(&self) -> bool {
        true
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn watch(&self) -> Result<Box<dyn ClipboardWatcher>> {
        let gen = *self
            .inner
            .generation
            .lock()
            .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
        Ok(Box::new(MemoryWatcher {
            inner: Arc::clone(&self.inner),
            last_gen: gen,
        }))
    }
}

struct MemoryWatcher {
    inner: Arc<MemoryClipboardInner>,
    last_gen: u64,
}

impl ClipboardWatcher for MemoryWatcher {
    fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ClipboardContent>> {
        let deadline = Instant::now() + timeout;
        let mut gen = self
            .inner
            .generation
            .lock()
            .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
        loop {
            if *gen != self.last_gen {
                self.last_gen = *gen;
                drop(gen);
                return self
                    .inner
                    .content
                    .lock()
                    .map(|guard| guard.clone())
                    .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()));
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(None);
            }
            let (g, wait) = self
                .inner
                .changed
                .wait_timeout(gen, deadline.saturating_duration_since(now))
                .map_err(|_| Error::Clipboard("memory clipboard lock poisoned".into()))?;
            gen = g;
            if wait.timed_out() && *gen == self.last_gen {
                return Ok(None);
            }
        }
    }
}

/// Adapter that reports unknown capabilities besides local storage.
#[derive(Debug, Default, Clone)]
pub struct UnsupportedPlatformAdapter {
    identity: PlatformIdentity,
}

impl UnsupportedPlatformAdapter {
    /// Create an adapter for an explicit identity.
    pub fn new(identity: PlatformIdentity) -> Self {
        Self { identity }
    }
}

impl PlatformAdapter for UnsupportedPlatformAdapter {
    fn name(&self) -> &'static str {
        "unsupported"
    }

    fn identity(&self) -> PlatformIdentity {
        self.identity.clone()
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::conservative_linux()
    }
}

/// Media provider that never hits the network and returns no results.
#[derive(Debug, Default, Clone)]
pub struct OfflineMediaProvider;

impl MediaProvider for OfflineMediaProvider {
    fn id(&self) -> &'static str {
        "offline"
    }

    fn display_name(&self) -> &str {
        "Offline (no remote media)"
    }

    fn is_available(&self) -> bool {
        true
    }

    fn search(&self, _query: &MediaQuery) -> Result<Vec<MediaItem>> {
        Ok(Vec::new())
    }
}

/// Sticker provider with no installed packs.
#[derive(Debug, Default, Clone)]
pub struct EmptyStickerPackProvider;

impl StickerPackProvider for EmptyStickerPackProvider {
    fn id(&self) -> &'static str {
        "empty"
    }

    fn list_packs(&self) -> Result<Vec<StickerPack>> {
        Ok(Vec::new())
    }

    fn load_pack(&self, _id: &StickerPackId) -> Result<Option<StickerPack>> {
        Ok(None)
    }
}

/// Process-local key-value store used by tests and foundation binaries.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    inner: Mutex<BTreeMap<(String, String), Vec<u8>>>,
}

impl StorageBackend for MemoryStorage {
    fn put(&self, namespace: &str, key: &str, value: &[u8]) -> Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::Storage("memory storage lock poisoned".into()))?;
        guard.insert((namespace.to_string(), key.to_string()), value.to_vec());
        Ok(())
    }

    fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| Error::Storage("memory storage lock poisoned".into()))?;
        Ok(guard
            .get(&(namespace.to_string(), key.to_string()))
            .cloned())
    }

    fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| Error::Storage("memory storage lock poisoned".into()))?;
        Ok(guard
            .remove(&(namespace.to_string(), key.to_string()))
            .is_some())
    }

    fn list_keys(&self, namespace: &str, prefix: &str) -> Result<Vec<String>> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| Error::Storage("memory storage lock poisoned".into()))?;
        let mut keys: Vec<String> = guard
            .keys()
            .filter(|(ns, key)| ns == namespace && key.starts_with(prefix))
            .map(|(_, key)| key.clone())
            .collect();
        keys.sort();
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clipboard::ClipboardItem;

    #[test]
    fn memory_clipboard_round_trip() {
        let backend = MemoryClipboard::default();
        let item = ClipboardItem::text("copied");
        backend.write(&item.content).expect("write");
        let read = backend.read().expect("read").expect("present");
        assert_eq!(read, item.content);
    }

    #[test]
    fn memory_clipboard_watch_sees_write() {
        let backend = MemoryClipboard::default();
        let mut watcher = backend.watch().unwrap();
        let content = ClipboardItem::text("watched").content;
        backend.write(&content).unwrap();
        let got = watcher
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .expect("event");
        assert_eq!(got, content);
    }

    #[test]
    fn memory_storage_lists_prefix() {
        let storage = MemoryStorage::default();
        storage.put("clip", "a1", b"1").unwrap();
        storage.put("clip", "a2", b"2").unwrap();
        storage.put("clip", "b1", b"3").unwrap();
        storage.put("other", "a9", b"9").unwrap();
        let keys = storage.list_keys("clip", "a").unwrap();
        assert_eq!(keys, vec!["a1".to_string(), "a2".to_string()]);
    }
}
