# UniPick roadmap

Status of the product relative to the [master plan](MASTER_PLAN.md).

## Current milestone: Foundation (complete when this tree lands)

- [x] Cargo workspace and crate boundaries
- [x] Domain types and traits
- [x] Placeholder backends sufficient to compile and test
- [x] CLI doctor / daemon stub / desktop stub
- [x] Architecture, privacy, and platform documentation
- [ ] Clipboard monitoring (**explicitly deferred**)
- [ ] Tauri webview runtime
- [ ] SQLite
- [ ] Remote media providers

## Milestone A — Daemon IPC and SQLite

**Ships:** persistent history API over a Unix socket, empty until a backend
records items.

**Does not ship:** watchers, hotkeys, overlay.

## Milestone B — First clipboard backend

**Ships:** one backend with tests on a named session (document which:
X11, portal, or a specific compositor protocol).

**Does not ship:** “works on all Wayland compositors” claims.

## Milestone C — Palette UI

**Ships:** Tauri v2 window, keyboard navigation, emoji/symbols/snippets panes
talking to the daemon.

**Does not ship:** GIF search, sticker store.

## Milestone D — Desktop environment adapters

**Ships:** GNOME and/or KDE integration with capability bits flipping from
`Unknown` to a real `SupportLevel`.

**Does not ship:** Hyprland/Sway-specific code unless an owner volunteers.

## Milestone E — Media

**Ships:** local stickers + one remote `MediaProvider` + on-disk cache.

**Does not ship:** scraping undocumented websites.

## Future

- Community content packs
- Plugin-ready provider loading (start with in-process traits, not `dlopen`)
- Flatpak portal-only packaging
- Optional sync is **not** planned (privacy: history stays on-device)

## Won't do

- Electron or Chromium-as-app-shell besides Tauri’s existing WebView
- Shipping compositor-specific hacks as default behavior
- Bundling a single GIF vendor into `unipick-core`
- Collecting clipboard contents for analytics
