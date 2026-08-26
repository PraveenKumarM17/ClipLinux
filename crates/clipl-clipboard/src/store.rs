//! Typed clipboard history persistence.

use clipl_core::{ClipboardItem, ClipboardItemId, Result, Timestamp};

/// Query for listing history.
#[derive(Clone, Debug)]
pub struct HistoryQuery {
    /// Maximum rows.
    pub limit: usize,
    /// Skip expired items when `now` is set.
    pub now: Option<Timestamp>,
}

impl HistoryQuery {
    /// Latest `limit` items.
    pub fn latest(limit: usize) -> Self {
        Self {
            limit,
            now: Some(Timestamp::now()),
        }
    }
}

/// Persistence surface used by [`crate::HistoryEngine`].
pub trait HistoryStore: Send + Sync {
    /// Insert a new row.
    fn insert(&self, item: &ClipboardItem) -> Result<()>;

    /// Replace a row in place (dedup reuse, pin).
    fn update(&self, item: &ClipboardItem) -> Result<()>;

    /// Fetch by id.
    fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>>;

    /// Newest first. Pinned items still sort by `created_at` among themselves
    /// but are listed before unpinned items.
    fn list(&self, query: &HistoryQuery) -> Result<Vec<ClipboardItem>>;

    /// Case-insensitive substring search over `text_content`.
    fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardItem>>;

    /// Delete one row.
    fn delete(&self, id: ClipboardItemId) -> Result<bool>;

    /// Delete unpinned rows. Returns the number removed.
    fn clear_unpinned(&self) -> Result<u64>;

    /// Most recently created item, if any.
    fn latest(&self) -> Result<Option<ClipboardItem>>;

    /// Delete unpinned items beyond `max_unpinned`, oldest first.
    fn enforce_limit(&self, max_unpinned: usize) -> Result<u64>;

    /// Delete unpinned items with `expires_at <= now` or older than `max_age`.
    fn expire(&self, now: Timestamp, max_age: Option<Timestamp>) -> Result<u64>;
}
