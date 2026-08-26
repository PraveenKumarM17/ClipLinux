//! Persistence interface. SQLite implements this in `clipl-clipboard`.

use crate::error::Result;

/// Namespaced key-value persistence.
///
/// Feature crates store typed records as serialized bytes. The default
/// production implementation will be SQLite; tests use an in-memory map.
pub trait StorageBackend: Send + Sync {
    /// Write a value, replacing any previous value for the same key.
    fn put(&self, namespace: &str, key: &str, value: &[u8]) -> Result<()>;

    /// Read a value if it exists.
    fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete a value. Returns whether a value was present.
    fn delete(&self, namespace: &str, key: &str) -> Result<bool>;

    /// List keys in a namespace that start with `prefix`.
    fn list_keys(&self, namespace: &str, prefix: &str) -> Result<Vec<String>>;
}
