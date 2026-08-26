//! SQLite persistence: KV [`StorageBackend`] plus typed clipboard history.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use clipl_core::{
    ClipboardContent, ClipboardItem, ClipboardItemId, Error, Result, StorageBackend, Timestamp,
};
use rusqlite::{params, Connection, OptionalExtension};

use crate::store::{HistoryQuery, HistoryStore};

const MIGRATION_V1: &str = r#"
CREATE TABLE IF NOT EXISTS kv (
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL,
    PRIMARY KEY (namespace, key)
);

CREATE TABLE IF NOT EXISTS clipboard_items (
    id TEXT PRIMARY KEY NOT NULL,
    content_type TEXT NOT NULL,
    mime TEXT,
    text_content TEXT,
    content_json TEXT NOT NULL,
    metadata_json TEXT,
    content_hash TEXT NOT NULL,
    source TEXT NOT NULL,
    source_app TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_used_at INTEGER,
    expires_at INTEGER,
    pinned INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_clipboard_created
    ON clipboard_items (pinned DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clipboard_hash
    ON clipboard_items (content_hash);
CREATE INDEX IF NOT EXISTS idx_clipboard_text
    ON clipboard_items (text_content);
"#;

/// Schema version written by the latest migration.
pub const SCHEMA_VERSION: i64 = 1;

/// SQLite-backed store. Safe for a local desktop daemon (WAL, busy timeout).
pub struct SqliteStore {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl SqliteStore {
    /// Open (or create) a database file and run migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            clipl_core::paths::ensure_user_dir(parent)?;
        }
        let conn = Connection::open(&path)
            .map_err(|err| Error::Storage(format!("open {}: {err}", path.display())))?;
        apply_pragmas(&conn)?;
        migrate(&conn)?;
        set_file_mode_0600(&path)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    /// In-memory database for tests.
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|err| Error::Storage(err.to_string()))?;
        apply_pragmas(&conn)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: PathBuf::from(":memory:"),
        })
    }

    /// Filesystem path (`:memory:` for the in-memory store).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Applied schema version.
    pub fn schema_version(&self) -> Result<i64> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|err| Error::Storage(err.to_string()))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| Error::Storage("sqlite lock poisoned".into()))
    }
}

fn apply_pragmas(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA temp_store = MEMORY;
        "#,
    )
    .map_err(|err| Error::Storage(err.to_string()))
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )
    .map_err(|err| Error::Storage(err.to_string()))?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|err| Error::Storage(err.to_string()))?;

    if current < 1 {
        conn.execute_batch(MIGRATION_V1)
            .map_err(|err| Error::Storage(err.to_string()))?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
            params![Timestamp::now().as_millis()],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
    }
    Ok(())
}

fn set_file_mode_0600(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if path == Path::new(":memory:") {
        return Ok(());
    }
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
        .map_err(|err| Error::Io(format!("{}: {err}", path.display())))
}

impl StorageBackend for SqliteStore {
    fn put(&self, namespace: &str, key: &str, value: &[u8]) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO kv (namespace, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(namespace, key) DO UPDATE SET value = excluded.value",
            params![namespace, key, value],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(())
    }

    fn get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT value FROM kv WHERE namespace = ?1 AND key = ?2",
            params![namespace, key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| Error::Storage(err.to_string()))
    }

    fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "DELETE FROM kv WHERE namespace = ?1 AND key = ?2",
                params![namespace, key],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n > 0)
    }

    fn list_keys(&self, namespace: &str, prefix: &str) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let like = format!("{prefix}%");
        let mut stmt = conn
            .prepare(
                "SELECT key FROM kv WHERE namespace = ?1 AND key LIKE ?2 ESCAPE '\\' ORDER BY key",
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        let keys = stmt
            .query_map(params![namespace, like], |row| row.get(0))
            .map_err(|err| Error::Storage(err.to_string()))?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(keys)
    }
}

impl HistoryStore for SqliteStore {
    fn insert(&self, item: &ClipboardItem) -> Result<()> {
        let conn = self.lock()?;
        let (text, mime) = text_and_mime(item);
        let content_json =
            serde_json::to_string(&item.content).map_err(|err| Error::Storage(err.to_string()))?;
        let metadata = item_metadata(item)?;
        conn.execute(
            "INSERT INTO clipboard_items (
                id, content_type, mime, text_content, content_json, metadata_json,
                content_hash, source, source_app, created_at, updated_at,
                last_used_at, expires_at, pinned
            ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                item.id.to_string(),
                item.content.type_name(),
                mime,
                text,
                content_json,
                metadata,
                item.content_hash,
                source_name(&item.source),
                item.source_app,
                item.created_at.as_millis(),
                item.updated_at.as_millis(),
                item.last_used_at.map(Timestamp::as_millis),
                item.expires_at.map(Timestamp::as_millis),
                i64::from(item.pinned),
            ],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(())
    }

    fn update(&self, item: &ClipboardItem) -> Result<()> {
        let conn = self.lock()?;
        let (text, mime) = text_and_mime(item);
        let content_json =
            serde_json::to_string(&item.content).map_err(|err| Error::Storage(err.to_string()))?;
        let metadata = item_metadata(item)?;
        let n = conn
            .execute(
                "UPDATE clipboard_items SET
                    content_type=?2, mime=?3, text_content=?4, content_json=?5,
                    metadata_json=?6, content_hash=?7, source=?8, source_app=?9,
                    created_at=?10, updated_at=?11, last_used_at=?12,
                    expires_at=?13, pinned=?14
                 WHERE id=?1",
                params![
                    item.id.to_string(),
                    item.content.type_name(),
                    mime,
                    text,
                    content_json,
                    metadata,
                    item.content_hash,
                    source_name(&item.source),
                    item.source_app,
                    item.created_at.as_millis(),
                    item.updated_at.as_millis(),
                    item.last_used_at.map(Timestamp::as_millis),
                    item.expires_at.map(Timestamp::as_millis),
                    i64::from(item.pinned),
                ],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        if n == 0 {
            return Err(Error::not_found(item.id.to_string()));
        }
        Ok(())
    }

    fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, content_json, metadata_json, content_hash, source, source_app,
                    created_at, updated_at, last_used_at, expires_at, pinned
             FROM clipboard_items WHERE id = ?1",
            params![id.to_string()],
            row_to_item,
        )
        .optional()
        .map_err(|err| Error::Storage(err.to_string()))
    }

    fn list(&self, query: &HistoryQuery) -> Result<Vec<ClipboardItem>> {
        let conn = self.lock()?;
        let now = query.now.map(Timestamp::as_millis);
        let mut stmt = conn
            .prepare(
                "SELECT id, content_json, metadata_json, content_hash, source, source_app,
                        created_at, updated_at, last_used_at, expires_at, pinned
                 FROM clipboard_items
                 WHERE (?1 IS NULL OR expires_at IS NULL OR expires_at > ?1)
                 ORDER BY pinned DESC, created_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![now, query.limit as i64], row_to_item)
            .map_err(|err| Error::Storage(err.to_string()))?;
        collect_items(rows)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardItem>> {
        let conn = self.lock()?;
        let needle = format!("%{}%", like_escape(query));
        let mut stmt = conn
            .prepare(
                "SELECT id, content_json, metadata_json, content_hash, source, source_app,
                        created_at, updated_at, last_used_at, expires_at, pinned
                 FROM clipboard_items
                 WHERE text_content IS NOT NULL
                   AND LOWER(text_content) LIKE LOWER(?1) ESCAPE '\\'
                 ORDER BY pinned DESC, created_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![needle, limit as i64], row_to_item)
            .map_err(|err| Error::Storage(err.to_string()))?;
        collect_items(rows)
    }

    fn delete(&self, id: ClipboardItemId) -> Result<bool> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "DELETE FROM clipboard_items WHERE id = ?1",
                params![id.to_string()],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n > 0)
    }

    fn clear_unpinned(&self) -> Result<u64> {
        let conn = self.lock()?;
        let n = conn
            .execute("DELETE FROM clipboard_items WHERE pinned = 0", [])
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n as u64)
    }

    fn latest(&self) -> Result<Option<ClipboardItem>> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT id, content_json, metadata_json, content_hash, source, source_app,
                    created_at, updated_at, last_used_at, expires_at, pinned
             FROM clipboard_items
             ORDER BY rowid DESC
             LIMIT 1",
            [],
            row_to_item,
        )
        .optional()
        .map_err(|err| Error::Storage(err.to_string()))
    }

    fn enforce_limit(&self, max_unpinned: usize) -> Result<u64> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "DELETE FROM clipboard_items WHERE pinned = 0 AND id IN (
                    SELECT id FROM clipboard_items WHERE pinned = 0
                    ORDER BY created_at ASC
                    LIMIT MAX(0, (SELECT COUNT(*) FROM clipboard_items WHERE pinned = 0) - ?1)
                )",
                params![max_unpinned as i64],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n as u64)
    }

    fn expire(&self, now: Timestamp, max_age: Option<Timestamp>) -> Result<u64> {
        let conn = self.lock()?;
        let n = conn
            .execute(
                "DELETE FROM clipboard_items
                 WHERE pinned = 0 AND (
                    (expires_at IS NOT NULL AND expires_at <= ?1)
                    OR (?2 IS NOT NULL AND created_at < ?2)
                 )",
                params![now.as_millis(), max_age.map(Timestamp::as_millis)],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n as u64)
    }
}

fn collect_items(
    rows: rusqlite::MappedRows<
        '_,
        impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<ClipboardItem>,
    >,
) -> Result<Vec<ClipboardItem>> {
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| Error::Storage(err.to_string()))
}

fn row_to_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipboardItem> {
    let id_str: String = row.get(0)?;
    let content_json: String = row.get(1)?;
    let metadata_json: String = row.get(2)?;
    let content_hash: String = row.get(3)?;
    let source_str: String = row.get(4)?;
    let source_app: Option<String> = row.get(5)?;
    let created_at: i64 = row.get(6)?;
    let updated_at: i64 = row.get(7)?;
    let last_used_at: Option<i64> = row.get(8)?;
    let expires_at: Option<i64> = row.get(9)?;
    let pinned: i64 = row.get(10)?;

    let content: ClipboardContent = serde_json::from_str(&content_json)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
    let meta: ItemMeta = serde_json::from_str(&metadata_json)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
    let id = id_str
        .parse()
        .map_err(|err: clipl_core::Error| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;

    Ok(ClipboardItem {
        id,
        content,
        source: parse_source(&source_str),
        created_at: Timestamp::from_millis(created_at),
        last_used_at: last_used_at.map(Timestamp::from_millis),
        pinned: pinned != 0,
        tags: meta.tags,
        sensitive: meta.sensitive,
        content_hash,
        updated_at: Timestamp::from_millis(updated_at),
        expires_at: expires_at.map(Timestamp::from_millis),
        source_app,
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ItemMeta {
    tags: Vec<String>,
    sensitive: Vec<clipl_core::SensitiveContentType>,
}

fn item_metadata(item: &ClipboardItem) -> Result<String> {
    serde_json::to_string(&ItemMeta {
        tags: item.tags.clone(),
        sensitive: item.sensitive.clone(),
    })
    .map_err(|err| Error::Storage(err.to_string()))
}

fn text_and_mime(item: &ClipboardItem) -> (Option<String>, Option<String>) {
    (
        item.content.text_for_scan().map(str::to_string),
        item.content.mime().map(str::to_string),
    )
}

fn source_name(source: &clipl_core::ClipboardSource) -> &'static str {
    match source {
        clipl_core::ClipboardSource::LocalSession => "local",
        clipl_core::ClipboardSource::ClipLinux => "clipl",
        clipl_core::ClipboardSource::Import => "import",
        clipl_core::ClipboardSource::Unknown => "unknown",
        _ => "unknown",
    }
}

fn parse_source(name: &str) -> clipl_core::ClipboardSource {
    match name {
        "local" => clipl_core::ClipboardSource::LocalSession,
        "clipl" => clipl_core::ClipboardSource::ClipLinux,
        "import" => clipl_core::ClipboardSource::Import,
        _ => clipl_core::ClipboardSource::Unknown,
    }
}

fn like_escape(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::HistoryQuery;
    use clipl_core::{ClipboardItem, Timestamp};

    #[test]
    fn migrates_to_v1() {
        let store = SqliteStore::memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn kv_round_trip() {
        use clipl_core::StorageBackend;
        let store = SqliteStore::memory().unwrap();
        StorageBackend::put(&store, "n", "k", b"v").unwrap();
        assert_eq!(
            StorageBackend::get(&store, "n", "k").unwrap(),
            Some(b"v".to_vec())
        );
        assert!(StorageBackend::delete(&store, "n", "k").unwrap());
        assert_eq!(StorageBackend::get(&store, "n", "k").unwrap(), None);
    }

    #[test]
    fn insert_list_search_delete() {
        let store = SqliteStore::memory().unwrap();
        let item = ClipboardItem::text("searchable unique phrase");
        HistoryStore::insert(&store, &item).unwrap();
        assert_eq!(
            HistoryStore::list(&store, &HistoryQuery::latest(10))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(HistoryStore::search(&store, "UNIQUE", 10).unwrap().len(), 1);
        assert!(HistoryStore::delete(&store, item.id).unwrap());
        assert!(HistoryStore::list(&store, &HistoryQuery::latest(10))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn pin_survives_limit_and_clear() {
        let store = SqliteStore::memory().unwrap();
        let mut keep = ClipboardItem::text("keep");
        keep.pinned = true;
        keep.content_hash = "a".into();
        let drop_me = ClipboardItem::text("drop");
        HistoryStore::insert(&store, &keep).unwrap();
        HistoryStore::insert(&store, &drop_me).unwrap();
        HistoryStore::enforce_limit(&store, 0).unwrap();
        assert!(HistoryStore::get(&store, keep.id).unwrap().is_some());
        assert!(HistoryStore::get(&store, drop_me.id).unwrap().is_none());
        HistoryStore::insert(&store, &ClipboardItem::text("later")).unwrap();
        assert_eq!(HistoryStore::clear_unpinned(&store).unwrap(), 1);
        assert!(HistoryStore::get(&store, keep.id).unwrap().is_some());
    }

    #[test]
    fn expire_by_timestamp() {
        let store = SqliteStore::memory().unwrap();
        let mut item = ClipboardItem::text("old");
        item.created_at = Timestamp::from_millis(1);
        item.updated_at = Timestamp::from_millis(1);
        HistoryStore::insert(&store, &item).unwrap();
        let cutoff = Timestamp::from_millis(1000);
        HistoryStore::expire(&store, Timestamp::now(), Some(cutoff)).unwrap();
        assert!(HistoryStore::get(&store, item.id).unwrap().is_none());
    }
}
