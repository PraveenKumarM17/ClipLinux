//! Snippet storage and lookup.

#![forbid(unsafe_code)]

use clipl_core::placeholders::MemoryStorage;
use clipl_core::{Error, Result, Snippet, SnippetId, StorageBackend};

const NAMESPACE: &str = "snippets";

/// Snippet library backed by a [`StorageBackend`].
pub struct SnippetLibrary<S: StorageBackend = MemoryStorage> {
    storage: S,
}

impl Default for SnippetLibrary<MemoryStorage> {
    fn default() -> Self {
        Self::new(MemoryStorage::default())
    }
}

impl<S: StorageBackend> SnippetLibrary<S> {
    /// Create a library.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Insert or replace a snippet.
    pub fn upsert(&self, snippet: &Snippet) -> Result<()> {
        let bytes = serde_json::to_vec(snippet).map_err(|err| Error::Storage(err.to_string()))?;
        self.storage.put(NAMESPACE, &snippet.id.to_string(), &bytes)
    }

    /// Fetch one snippet.
    pub fn get(&self, id: SnippetId) -> Result<Option<Snippet>> {
        match self.storage.get(NAMESPACE, &id.to_string())? {
            Some(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).map_err(|err| Error::Invalid(err.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    /// List all snippets, sorted by name.
    pub fn list(&self) -> Result<Vec<Snippet>> {
        let mut items: Vec<Snippet> = Vec::new();
        for key in self.storage.list_keys(NAMESPACE, "")? {
            if let Some(bytes) = self.storage.get(NAMESPACE, &key)? {
                items.push(
                    serde_json::from_slice(&bytes)
                        .map_err(|err| Error::Invalid(err.to_string()))?,
                );
            }
        }
        items.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });
        Ok(items)
    }

    /// Delete a snippet.
    pub fn delete(&self, id: SnippetId) -> Result<bool> {
        self.storage.delete(NAMESPACE, &id.to_string())
    }

    /// Find by trigger token, if any.
    pub fn by_trigger(&self, trigger: &str) -> Result<Option<Snippet>> {
        Ok(self
            .list()?
            .into_iter()
            .find(|snippet| snippet.trigger.as_deref() == Some(trigger)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_list_delete() {
        let lib = SnippetLibrary::default();
        let mut snippet = Snippet::new("Signature", "Best,\nPraveen");
        snippet.trigger = Some(";sig".into());
        lib.upsert(&snippet).unwrap();
        assert_eq!(lib.list().unwrap().len(), 1);
        assert_eq!(lib.by_trigger(";sig").unwrap().unwrap().id, snippet.id);
        assert!(lib.delete(snippet.id).unwrap());
        assert!(lib.list().unwrap().is_empty());
    }
}
