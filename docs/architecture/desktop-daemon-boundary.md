# Desktop ↔ daemon boundary

Status: **IMPLEMENTED** for Phase 3A/3B picker plus Phase 4A activation
subscribe.

The desktop process is a **client**. `clipl-daemon` is the **source of truth**
for clipboard history, privacy redaction, pin/delete/clear, picker catalogs,
and copy loop-prevention. Packaged builds may **start** the daemon if the
socket is missing; they still never open SQLite in the UI process.

```
┌──────────────────────────┐     Unix socket      ┌─────────────────────┐
│  Svelte webview          │                      │  clipl-daemon       │
│  (no socket, no SQLite)  │                      │  SQLite + privacy   │
└────────────▲─────────────┘                      │  clipboard watch    │
             │ invoke                             └──────────▲──────────┘
┌────────────┴─────────────┐   clipl-protocol                │
│  clipl-desktop (Rust)    │  length-prefixed JSON  ─────────┘
│  DaemonClient            │
│  arboard (copy only)     │
└──────────────────────────┘
```

## Why the UI does not open SQLite

- Privacy filtering (`for_client`) must run in one place
- History limits, pin eviction, and skip-hash live in the daemon
- The webview must not see secret payloads just because it asked for history
- Multiple clients (CLI + desktop) stay consistent

`apps/desktop/src-tauri` may depend on `clipl-core` and `clipl-protocol` only.
It must not depend on `clipl-clipboard` or rusqlite.

## Typed commands

The webview calls Tauri commands, never raw sockets:

| Command | IPC |
| --- | --- |
| `cmd_get_daemon_status` | `GetStatus` |
| `cmd_get_history` | `GetHistory` |
| `cmd_search_history` | `SearchHistory` (empty query → recent history) |
| `cmd_delete_history_item` | `DeleteItem` |
| `cmd_clear_history` | `ClearHistory` (unpinned only) |
| `cmd_pin_history_item` / `cmd_unpin_history_item` | `PinItem` / `UnpinItem` |
| `cmd_copy_history_item` | `CopyItem` then OS clipboard write |
| `cmd_hide_picker` / `cmd_show_picker` / `cmd_toggle_picker` | local window visibility |
| `cmd_close_window` | hide (does not quit) |

On startup the host also opens a long-lived `SubscribeDesktop` connection so
the daemon can push `ActivatePicker` events. The webview never opens that
socket.

History rows sent to the webview are DTOs (`preview`, `hidden`, timestamps).
Full payloads are not forwarded except the one-shot `Copied.text` used by the
Rust host after `CopyItem` — that string is not stored in the UI model.

## Vite without Tauri

`npm run dev` (Vite alone) cannot reach the daemon. The UI detects a missing
Tauri runtime and shows a disconnected state with `npm run tauri dev` as the
hint. It must not crash.
