# ClipLinux roadmap

Status of the product relative to the [master plan](MASTER_PLAN.md).

## Current milestone: Clipboard history (Phase 2)

- [x] Cargo workspace and crate boundaries
- [x] Domain types and traits
- [x] SQLite persistence + migrations
- [x] Privacy detectors (PEM, JWT, high-confidence tokens, Luhn cards, OTP)
- [x] Consecutive dedup + retention
- [x] Unix-domain IPC + CLI history commands
- [x] X11 CLIPBOARD watch via XFixes (text)
- [x] Honest Wayland / GNOME Unsupported watch
- [ ] Global hotkeys / overlay popup
- [ ] Tauri webview runtime
- [ ] Remote media providers
- [ ] GNOME Shell extension clipboard bridge

## Milestone C — Palette UI

**Ships:** Tauri v2 window, keyboard navigation, emoji/symbols/snippets panes
talking to the daemon.

**Does not ship:** GIF search, sticker store.

## Milestone D — Desktop environment adapters

**Ships:** GNOME extension clipboard push and/or KDE integration with
capability bits flipping from `Unsupported`/`Unknown` to a real `SupportLevel`.

**Does not ship:** “generic Wayland clipboard watch works everywhere”.

## Milestone E — Media

**Ships:** local stickers + one remote `MediaProvider` + on-disk cache.

## Future

- Community content packs
- Plugin-ready provider loading
- Flatpak portal-only packaging
- Optional sync is **not** planned (privacy: history stays on-device)

## Won't do

- Electron or Chromium-as-app-shell besides Tauri’s existing WebView
- Shipping compositor-specific hacks as default behavior
- Polling `xclip` / `wl-paste` as the default watch path
- Bundling a single GIF vendor into `clipl-core`
- Collecting clipboard contents for analytics
