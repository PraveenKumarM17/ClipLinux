# Desktop application

Status: **IMPLEMENTED** for Phase 3A/3B picker plus Phase 4A show/hide.

ClipLinux desktop (`clipl-desktop`, application id `io.clipl.ClipLinux`) is a
compact Tauri v2 window with a Svelte 5 UI. It is a clipboard **picker**, not a
dashboard. The process stays running when the window is hidden.

## Layout

```
apps/desktop/
  src/                 Svelte UI (history, search, placeholders)
  src-tauri/           Rust host (IPC client + Tauri commands)
  scripts/tauri.sh     Enables the `tauri-app` Cargo feature for WebView builds
```

The Rust crate is a workspace member. **Default features are empty** so
`cargo test --workspace` does not link WebKitGTK. Production builds pass
`--features tauri-app` (the npm `tauri` script does this).

## Window

- About 440×560, resizable, minimum 360×420
- Centered, not maximized, native titlebar
- Escape and the titlebar close button **hide** the window (they do not quit)
- Overlay positioning relative to the focused window is still later

On startup the host connects to `clipl-daemon`, fetches status and recent
history, focuses search, and **subscribes** for activation events.

| Tab | Status |
| --- | --- |
| History | Functional: list, search, pin, delete, clear, copy |
| Emoji / Symbols | Functional offline pickers |
| Snips | Placeholder |

Keyboard (history tab):

| Key | Action |
| --- | --- |
| ↑ / Ctrl+K | Previous item |
| ↓ / Ctrl+J | Next item |
| Enter | Copy selected text to the OS clipboard |
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

## Copy and loop prevention

The webview never writes the OS clipboard and never opens SQLite.

1. UI sends the item id to a Tauri command
2. Host sends `CopyItem` to the daemon
3. Daemon returns text (never for hidden/sensitive rows) and records a skip-hash
4. Host writes the text with `arboard`
5. If the watch thread sees that same hash within ~3s, it does not insert a row

On GNOME Wayland, `arboard` may fail. The UI surfaces the error. This phase
does not inject keys into foreign windows.

## Tests

Workspace Rust tests mock the Unix socket and a `RecordingClipboard` sink. They
do not need a display server or a running daemon.

Frontend unit tests (Vitest) cover debounce, keyboard mapping, Escape, and the
empty vs disconnected list surfaces.
