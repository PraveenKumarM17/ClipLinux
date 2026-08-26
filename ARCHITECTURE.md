# ClipLinux architecture

ClipLinux is split so that **domain logic, platform I/O, and UI** cannot collapse
into one crate. Contributors should be able to implement a GIF provider, a
Plasma adapter, or a Svelte view without reading compositor code.

## System sketch

```
┌─────────────┐     Unix IPC      ┌──────────────────┐
│  Desktop    │ ◀──────────────▶  │  Daemon          │
│  Tauri +    │  clipl-protocol │  history, cache, │
│  Svelte 5   │                   │  privacy, probe  │
└─────────────┘                   └────────┬─────────┘
       ▲                                   │
       │ local fallback                    │ traits
┌─────────────┐                   ┌────────▼─────────┐
│  CLI        │ ────────────────▶ │  clipl-core    │
└─────────────┘                   │  types + traits  │
                                  └────────┬─────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
            clipl-platform        clipl-clipboard      clipl-media
            (adapters)              clipl-privacy        clipl-emoji
                                    clipl-snippets       clipl-symbols
```

The desktop UI never calls X11 or Wayland directly. The daemon never imports
Svelte. Core never imports Tauri.

## Crate graph (allowed dependencies)

```
clipl-core                 (no ClipLinux crate deps)
    ↑
clipl-protocol             (core)
clipl-privacy              (core)
clipl-platform             (core)
clipl-emoji                (core)
clipl-symbols              (core)
clipl-media                (core)
clipl-snippets             (core)
clipl-clipboard            (core, privacy)
    ↑
apps/cli                     (core, platform, protocol)
apps/daemon                  (core, platform, privacy, protocol, clipboard)
apps/desktop/src-tauri       (core, protocol)
```

**Forbidden:**

| From | To | Why |
| --- | --- | --- |
| `clipl-core` | Tauri, GTK, Qt, smithay, x11rb | Core must stay host-agnostic |
| `clipl-core` | `clipl-platform` | Detection is an implementation, not a domain type |
| Svelte / TypeScript | OS clipboard APIs | UI talks protocol only |
| Any crate | Electron | Product rule |

SQLite implements `StorageBackend` (`kv` table) and the typed clipboard
repository (`clipboard_items`) in `clipl-clipboard::SqliteStore`. Core still
has no SQLite dependency.

## Domain types

Defined in `clipl-core` and serialized with serde so IPC, SQLite, and tests
share one shape:

| Type | Role |
| --- | --- |
| `ClipboardItem` | History record (id, content, source, timestamps, pin, tags, sensitive labels) |
| `ClipboardContent` | Text, HTML, image, files, URI, emoji, media, snippet, custom |
| `ContentRef` | Inline bytes vs content-addressed blob |
| `MediaItem` / `StickerPack` | Pasteable media and packs |
| `Snippet` | Named, optionally triggered text |
| `Emoji` | Glyph plus catalog metadata |
| `Platform` / `SessionType` / `DesktopEnvironment` | Session identity |
| `Capability` / `SupportLevel` / `PlatformCapabilities` | Honest feature matrix |
| `ActivationCapability` / `Shortcut` / `ActivationRequest` | Per-session picker activation |
| `PrivacyRule` / `SensitiveContentType` | Policy input |

## Traits

| Trait | Implementors (now / later) |
| --- | --- |
| `ClipboardBackend` | `MemoryClipboard`, X11 (text watch); Wayland/GNOME stubs |
| `PlatformAdapter` | `LinuxGenericAdapter` / GNOME, KDE, wlroots, Hyprland, Sway |
| `ActivationBackend` | X11 grab, GNOME Shell slot, KDE/Sway/Hyprland placeholders |
| `MediaProvider` | `OfflineMediaProvider` / Tenor, Giphy, others |
| `StickerPackProvider` | `EmptyStickerPackProvider` / local directory, community |
| `StorageBackend` | `MemoryStorage`, `SqliteStore` |

Traits are synchronous in the foundation. Async runtimes (tokio) can wrap them
in the daemon without leaking into core.

## Capability detection

`PlatformCapabilities` maps each `Capability` to a `SupportLevel`:

- **Native** — first-class protocol or toolkit support
- **Portal** — xdg-desktop-portal or DE-owned API
- **Fallback** — documented, tested degraded path (never an implicit hack)
- **Unsupported** — probed and unavailable
- **Unknown** — not probed; **preferred over a lie**

Adapters must not set `Native` because “it worked on my machine”. Wayland
clipboard watch, global hotkeys, and overlay popups are independent
capabilities. Global hotkeys are **not** one API: X11 may grab a chord;
GNOME Wayland uses a Shell extension. See
[PLATFORM_CAPABILITIES.md](PLATFORM_CAPABILITIES.md) and
[docs/architecture/activation.md](docs/architecture/activation.md).

## Apps

### Desktop (`apps/desktop`)

- **UI:** Svelte 5 + TypeScript + Vite (history picker; other tabs are placeholders)
- **Shell:** Tauri v2, application id `io.clipl.ClipLinux`
- **Host:** `apps/desktop/src-tauri` talks to `clipl-daemon` over `clipl-protocol`.
  It does not open SQLite. Default Cargo features omit Tauri so workspace tests
  do not need WebKitGTK; `npm run tauri dev` passes `--features tauri-app`.

See [docs/architecture/desktop.md](docs/architecture/desktop.md) and
[docs/architecture/desktop-daemon-boundary.md](docs/architecture/desktop-daemon-boundary.md).
Emoji/symbols search stays in the daemon (`clipl-emoji`, `clipl-symbols`); the
webview only receives compact `PickerItem` rows.

### Daemon (`apps/daemon`)

Resident process: capability probe, SQLite, privacy engine, optional clipboard
watch, Unix-socket IPC. See [docs/architecture/daemon.md](docs/architecture/daemon.md).

### CLI (`apps/cli`)

`clipl doctor` probes locally. `status`, `ping`, `open` / `hide` / `toggle`,
and `history` talk to the daemon over the Unix socket.

## Extensions and packages

- `extensions/gnome` — Shell extension (activation shortcut → Unix IPC)
- `extensions/kde` — Plasma integration (placeholder)
- `packages/emoji-data` — catalog JSON
- `packages/sticker-packs` — local packs
- `packages/themes` — design tokens (data, not code)

## Storage

See [docs/architecture/storage.md](docs/architecture/storage.md). SQLite is the
production `StorageBackend`; tests keep `MemoryStorage`.

## Error handling

`clipl_core::Error` is serializable so the daemon can return it over IPC.
Platform crates map OS errors into this type. `unwrap` is restricted to tests
and true invariants.

## Testing

- Unit tests live next to the code they cover
- `tests/` is a workspace crate that composes several libraries
- Platform tests that need a display server are opt-in and must not run in
  default `cargo test`
