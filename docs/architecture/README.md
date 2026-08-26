# Architecture notes

Canonical document: [/ARCHITECTURE.md](../../ARCHITECTURE.md)

## Crate map

| Crate | Responsibility |
| --- | --- |
| `clipl-core` | Types + traits only |
| `clipl-protocol` | JSON envelopes for desktop ↔ daemon ↔ CLI |
| `clipl-privacy` | Rule evaluation |
| `clipl-clipboard` | History engine, SQLite, dedup |
| `clipl-emoji` | Catalog search |
| `clipl-symbols` | Non-emoji symbols |
| `clipl-snippets` | Snippet CRUD |
| `clipl-media` | Provider registry |
| `clipl-platform` | XDG probe + clipboard backends |

See also:

- [clipboard-engine.md](clipboard-engine.md)
- [daemon.md](daemon.md)
- [ipc.md](ipc.md)
- [storage.md](storage.md)

## Why core has in-memory placeholders

Tests and binaries need *some* `ClipboardBackend` and `StorageBackend`. Those
types live in `clipl_core::placeholders` so feature crates do not invent
divergent fakes. They are not production Linux backends.
