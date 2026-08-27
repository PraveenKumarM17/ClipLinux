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

## Requests

| Request | Response |
| --- | --- |
| `Ping` | `Pong` |
| `GetStatus` | `Status(DaemonStatus)` |
| `GetCapabilities` | `Capabilities(...)` |
| `GetHistory { limit }` | `History(items)` (payloads redacted when sensitive) |
| `SearchHistory { query, limit }` | `History(items)` |
| `DeleteItem { item_id }` | `Deleted { existed }` (pinned items are refused) |
| `ClearHistory` | `Cleared { count }` (unpinned only) |
| `PinItem { item_id }` / `UnpinItem { item_id }` | `Pinned { item_id, pinned }` |
| `CopyItem { item_id }` | `Copied { item_id, text }` then the client writes the OS clipboard |
| `SearchEmoji` / `ListEmojiCategory` / `GetFrequentlyUsedEmoji` | `PickerList` |
| `RecordEmojiUsage` | `PickerUsage` |
| `FavoriteEmoji` / `UnfavoriteEmoji` / `GetFavoriteEmoji` | `PickerFavorite` / `PickerList` |
| `GetSkinTonePref` / `SetSkinTonePref` | `SkinTone` |
| `SearchSymbols` / `ListSymbolCategory` | `PickerList` |
| `SearchKaomoji` / `ListKaomojiCategory` | `PickerList` |
| `FavoritePicker` / `UnfavoritePicker` / `GetFavoritePicker` | `PickerFavorite` / `PickerList` |
| `ShowDesktop` / `HideDesktop` / `ToggleDesktop` | `DesktopRouted { delivered }` |
| `SubscribeDesktop` | `DesktopSubscribed { replaced }` then `Event::ActivatePicker` on that connection |
| `SubscribeInsert` | `InsertSubscribed { replaced }` then `Event::InsertIntoApp` on that connection |
| `InsertIntoApp` | `Inserted { delivered, reason }` |
| `GetActivationStatus` | `Activation(ActivationReport)` |

History replies are sanitized with `for_client`: hidden items keep their
metadata but not the secret payload.

`CopyItem` records a skip-hash on the daemon so the watch thread does not
insert a duplicate row for the echo (about 3 seconds). Hidden items cannot be
copied.

`DaemonStatus` contains paths, backend name, monitoring level, and an
activation report. It does **not** contain clipboard payloads.

Ordinary commands still open one request per connection. The desktop keeps a
**second** `SubscribeDesktop` connection open so the daemon can push
`ActivatePicker` events. The GNOME extension keeps a **third**
`SubscribeInsert` connection open for restore-focus + Ctrl+V. On Show/Toggle the
daemon first sends `Event::PrepareInsert` so the extension can snapshot the
focused window **before** the picker takes focus (needed when GNOME's custom
shortcut runs `clipl toggle`). Only one subscriber is stored per hub; a new
connection replaces the previous one.

`InsertIntoApp` does not include the clipboard payload. The desktop must
already have written CLIPBOARD and hidden the picker. If no insert backend
is available, `delivered` is false and `reason` tells the user to press
Ctrl+V.

## PLANNED

- Event stream (`ClipboardChanged`) pushed to subscribers
- Paste request execution
