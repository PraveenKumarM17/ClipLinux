# Desktop application

Status: **IMPLEMENTED** for Phase 3A/3B picker plus Phase 4A show/hide and
restore-focus insert (Ctrl+V).

ClipLinux desktop (`clipl-desktop`, application id `io.clipl.ClipLinux`) is a
compact Tauri v2 window with a Svelte 5 UI. It is a clipboard **picker**, not a
dashboard. The process stays running when the window is hidden.

## Layout

```
apps/desktop/
  src/                 Svelte UI (history, emoji, symbols/kaomoji; snippets placeholder)
  src-tauri/           Rust host (IPC client + Tauri commands)
  src-tauri/icons/     Window icons (`icon.png` is required by `generate_context!`)
  scripts/tauri.sh     Enables the `tauri-app` Cargo feature for WebView builds
```

The Rust crate is a workspace member. **Default features are empty** so
`cargo test --workspace` does not link WebKitGTK. Production builds pass
`--features tauri-app` (the npm `tauri` script does this). Workspace tests
do not launch the WebView; `npm run tauri dev` is a separate runtime that
needs WebKitGTK on the host.

## Window

- About 440×560, resizable, minimum 360×420
- Centered, always on top, **not** in the taskbar / app switcher
- Starts **hidden**; Super+Alt+V (or `clipl toggle`) shows it
- Escape, the close button, and clicking another window **hide** it (the process stays running)
- Overlay positioning relative to the focused window is still later

On startup the host connects to `clipl-daemon`, fetches status and recent
history, focuses search, and **subscribes** for activation events.

The search box is **universal**. A non-empty query searches clipboard
history, emoji, symbols, and kaomoji together and shows grouped results.
Empty query browses the selected tab (history list or emoji/symbol
categories). Snippets are not searched yet.

| Tab | Status |
| --- | --- |
| History | Functional: list, pin, delete, clear, copy |
| Emoji / Symbols | Functional offline pickers |
| Snips | Placeholder |

Keyboard (universal search and history browse):

| Key | Action |
| --- | --- |
| ↑ / Ctrl+K | Previous item |
| ↓ / Ctrl+J | Next item |
| Enter | Insert selected text into the app you were typing in (Ctrl+V). Copied either way. |
| Escape | Clear search if non-empty; otherwise **hide** |
| Ctrl+F | Focus search |
| Delete | Confirm-delete selected **unpinned** item |
| Ctrl+Shift+Delete | Confirm-clear unpinned history |

## Status chip

The chip is labeled in text (not color-only):

| State | Meaning |
| --- | --- |
| Starting | First connect in progress |
| Connected | Daemon up and clipboard watch is running |
| Monitoring unavailable | Daemon up; this session cannot watch the clipboard (typical GNOME + Wayland) |
| Disconnected | Socket missing / daemon not running |
| Error | Protocol mismatch or daemon error payload |

Disconnected is **not** the same as empty history. The UI shows a retry action
and the startup command (`cargo run -p clipl-daemon`) instead of an empty list.

Reconnect uses exponential backoff (1s … 16s) and does not hammer the socket.

## Copy, hide, then insert

The webview never writes the OS clipboard and never opens SQLite.

1. UI sends the item id (or glyph) to a Tauri command
2. Host writes CLIPBOARD (GTK `Clipboard::store()` when available, else `arboard`)
3. Host **hides** the picker so the next Ctrl+V cannot land in search
4. Host sends `InsertIntoApp` to the daemon
5. Daemon restores the window that had focus **when the shortcut fired** and sends **only** Ctrl+V
6. If the watch thread sees the copy hash within ~3s, it does not insert a row

ClipLinux never types the payload as fake keys. If insert cannot be delivered
(generic Wayland, `clipl open` with no shortcut snapshot, insert disabled),
the text is still on the clipboard; press Ctrl+V yourself.

On GNOME Wayland this needs the Shell extension's `SubscribeInsert` helper.
GTK clipboard write is used because `arboard` often fails without
`wlr-data-control`.

## Tests

Workspace Rust tests mock the Unix socket and a `RecordingClipboard` sink. They
do not need a display server or a running daemon.

Frontend unit tests (Vitest) cover debounce, keyboard mapping, Escape, and the
empty vs disconnected list surfaces.
