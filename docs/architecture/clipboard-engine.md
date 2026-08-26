# Clipboard engine

Status: **IMPLEMENTED** for text history (privacy → dedup → SQLite).
OS watch is **PARTIALLY IMPLEMENTED** (X11 Native; Wayland Unsupported).

## Pipeline

```
ClipboardContent
    → persistable? (text / html / uri only in this phase)
    → SHA-256 content_hash
    → privacy evaluate()  (detectors + rules)
    → Exclude / Confirm  → do not write
    → consecutive dedup  → UPDATE latest row
    → INSERT
    → retention (max_items, max_age, expires_at)
```

Privacy always runs **before** SQLite. Excluded items leave no row.

## Duplicate policy (Phase 2)

Configurable as `clipboard.deduplication_policy`:

| Value | Behaviour |
| --- | --- |
| `consecutive` (default) | If the **most recently inserted** row has the same hash, reuse it (`updated_at` / `last_used_at`). Do not insert. |
| `none` | Always insert (privacy still applies). |

**Non-consecutive** copies of the same text (A, B, A) insert a third row. This is intentional so the latest capture is A again without rewriting older history.

Hash: SHA-256 of `ClipboardContent::canonical_bytes` (type tag + UTF-8). Hashes are stored on the row. Logs never include payload bytes or hashes of excluded secrets.

## What is persisted

**IMPLEMENTED:** `Text`, `Html`, `Uri`.

**SKIPPED:** images, files, custom blobs (architecture is ready; blob store is later).

## Tests

Use `MemoryClipboard` / `MemoryHistoryStore` / `SqliteStore::memory()`. Default `cargo test` never opens the host clipboard.
