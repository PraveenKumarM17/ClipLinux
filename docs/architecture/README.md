# Architecture notes

Canonical document: [/ARCHITECTURE.md](../../ARCHITECTURE.md)

## Crate map

| Crate | Responsibility |
| --- | --- |
| `unipick-core` | Types + traits only |
| `unipick-protocol` | JSON envelopes for desktop ↔ daemon ↔ CLI |
| `unipick-privacy` | Rule evaluation |
| `unipick-clipboard` | History record/list (no OS watch) |
| `unipick-emoji` | Catalog search |
| `unipick-symbols` | Non-emoji symbols |
| `unipick-snippets` | Snippet CRUD |
| `unipick-media` | Provider registry |
| `unipick-platform` | XDG probe + adapter slots |

## Why core has in-memory placeholders

Tests and binaries need *some* `ClipboardBackend` and `StorageBackend`. Those
types live in `unipick_core::placeholders` so feature crates do not invent
divergent fakes. They are not production Linux backends.
