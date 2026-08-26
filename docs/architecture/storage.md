# Storage

Status: **IMPLEMENTED** (SQLite + in-memory).

## Locations

| File | Path |
| --- | --- |
| Database | `$XDG_DATA_HOME/clipl/history.sqlite3` |
| Config | `$XDG_CONFIG_HOME/clipl/config.toml` |
| Socket | `$XDG_RUNTIME_DIR/clipl/daemon.sock` |

Overrides: `CLIPL_DATA_DIR`, `CLIPL_CONFIG_DIR`, `CLIPL_CONFIG_PATH`, `CLIPL_RUNTIME_DIR`.

Directories are created with mode `0700`. The database file is `0600`.

## SQLite settings

- `foreign_keys=ON`
- `journal_mode=WAL`
- `synchronous=NORMAL`
- `busy_timeout=5000`

Access is serialized with a mutex. The watch thread does not poll the database.

## Schema version

`schema_migrations.version` — current: **1**.

```sql
CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

CREATE TABLE kv (
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL,
    PRIMARY KEY (namespace, key)
);

CREATE TABLE clipboard_items (
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
```

`kv` implements `StorageBackend` (snippets and other namespaces later).
`clipboard_items` is the typed history store. `text_content` is searchable; images are not stored in this phase.

## Implementations

| Type | Use |
| --- | --- |
| `MemoryStorage` / `MemoryHistoryStore` | Tests |
| `SqliteStore` | Daemon production path |

Pinned rows survive `max_items`, age expiry, and `ClearHistory`. Explicit delete still removes them.
