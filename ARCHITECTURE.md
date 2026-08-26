# UniPick architecture

UniPick is split so that **domain logic, platform I/O, and UI** cannot collapse
into one crate. Contributors should be able to implement a GIF provider, a
Plasma adapter, or a Svelte view without reading compositor code.

## System sketch

```
┌─────────────┐     Unix IPC      ┌──────────────────┐
│  Desktop    │ ◀──────────────▶  │  Daemon          │
│  Tauri +    │  unipick-protocol │  history, cache, │
│  Svelte 5   │                   │  privacy, probe  │
└─────────────┘                   └────────┬─────────┘
       ▲                                   │
       │ local fallback                    │ traits
┌─────────────┐                   ┌────────▼─────────┐
│  CLI        │ ────────────────▶ │  unipick-core    │
└─────────────┘                   │  types + traits  │
                                  └────────┬─────────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    ▼                      ▼                      ▼
            unipick-platform        unipick-clipboard      unipick-media
            (adapters)              unipick-privacy        unipick-emoji
                                    unipick-snippets       unipick-symbols
```

The desktop UI never calls X11 or Wayland directly. The daemon never imports
Svelte. Core never imports Tauri.

## Crate graph (allowed dependencies)

```
unipick-core                 (no UniPick crate deps)
    ↑
unipick-protocol             (core)
unipick-privacy              (core)
unipick-platform             (core)
unipick-emoji                (core)
unipick-symbols              (core)
unipick-media                (core)
unipick-snippets             (core)
unipick-clipboard            (core, privacy)
    ↑
apps/cli                     (core, platform, protocol)
apps/daemon                  (core, platform, privacy, protocol)
apps/desktop/src-tauri       (core, protocol)
```

**Forbidden:**

| From | To | Why |
| --- | --- | --- |
| `unipick-core` | Tauri, GTK, Qt, smithay, x11rb | Core must stay host-agnostic |
| `unipick-core` | `unipick-platform` | Detection is an implementation, not a domain type |
| Svelte / TypeScript | OS clipboard APIs | UI talks protocol only |
| Any crate | Electron | Product rule |

SQLite belongs in a future `StorageBackend` implementation, not in core.

## Domain types

Defined in `unipick-core` and serialized with serde so IPC, SQLite, and tests
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
| `PrivacyRule` / `SensitiveContentType` | Policy input |

## Traits

| Trait | Implementors (now / later) |
| --- | --- |
| `ClipboardBackend` | `MemoryClipboard` / X11, Wayland, portal |
| `PlatformAdapter` | `LinuxGenericAdapter` / GNOME, KDE, wlroots, Hyprland, Sway |
| `MediaProvider` | `OfflineMediaProvider` / Tenor, Giphy, others |
| `StickerPackProvider` | `EmptyStickerPackProvider` / local directory, community |
| `StorageBackend` | `MemoryStorage` / SQLite |

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
capabilities. See [PLATFORM_CAPABILITIES.md](PLATFORM_CAPABILITIES.md).

## Apps

### Desktop (`apps/desktop`)

- **UI:** Svelte 5 + TypeScript + Vite
- **Shell:** Tauri v2 (config present; the Rust crate currently compiles
  without the `tauri` crate so the workspace builds on machines without
  WebKitGTK)
- Command handlers will live in `apps/desktop/src-tauri` and call the daemon
  via `unipick-protocol`

### Daemon (`apps/daemon`)

Resident process. Foundation binary probes the session, loads default privacy
rules, and **does not** watch the clipboard.

### CLI (`apps/cli`)

`unipick doctor` prints identity and the capability matrix. `unipick ping`
builds a protocol envelope but does not open a socket yet.

## Extensions and packages

- `extensions/gnome` — Shell extension (placeholder)
- `extensions/kde` — Plasma integration (placeholder)
- `packages/emoji-data` — catalog JSON
- `packages/sticker-packs` — local packs
- `packages/themes` — design tokens (data, not code)

## Storage plan (not implemented)

SQLite will implement `StorageBackend` with namespaces (`clipboard-history`,
`snippets`, `blobs`, `privacy-rules`). Large images are blobs referenced by
`ContentRef::Blob`, never duplicated in history rows. User clipboard payloads
are local-only; they are not telemetry.

## Error handling

`unipick_core::Error` is serializable so the daemon can return it over IPC.
Platform crates map OS errors into this type. `unwrap` is restricted to tests
and true invariants.

## Testing

- Unit tests live next to the code they cover
- `tests/` is a workspace crate that composes several libraries
- Platform tests that need a display server are opt-in and must not run in
  default `cargo test`
