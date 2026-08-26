# IPC

Status: **IMPLEMENTED** (Unix-domain socket, request/response).

## Transport

- Path: `$XDG_RUNTIME_DIR/clipl/daemon.sock` (`CLIPL_RUNTIME_DIR` override)
- Framing: little-endian `u32` length + JSON `Envelope`
- Max frame: 8 MiB
- Permissions: directory `0700`, socket `0600`
- Stale socket: if connect fails, unlink and bind
- **No TCP.** Local-only.

Protocol version: `clipl_protocol::PROTOCOL_VERSION` (currently `1`).

## Requests (Phase 2)

| Request | Response |
| --- | --- |
| `Ping` | `Pong` |
| `GetStatus` | `Status(DaemonStatus)` |
| `GetCapabilities` | `Capabilities(...)` |
| `GetHistory { limit }` | `History(items)` |
| `SearchHistory { query, limit }` | `History(items)` |
| `DeleteItem { item_id }` | `Deleted { existed }` |
| `ClearHistory` | `Cleared { count }` (unpinned only) |

`DaemonStatus` contains paths, backend name, and monitoring level. It does **not** contain clipboard payloads.

One request per connection (CLI opens a new socket per command).

## PLANNED

- Event stream (`ClipboardChanged`) pushed to subscribers
- Paste request execution
