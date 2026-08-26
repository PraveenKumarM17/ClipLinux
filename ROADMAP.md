# ClipLinux roadmap

Status of the product relative to the [master plan](MASTER_PLAN.md).

## Current milestone: Activation (Phase 4A)

- [x] Cargo workspace and crate boundaries
- [x] Domain types and traits
- [x] SQLite persistence + migrations
- [x] Privacy detectors (PEM, JWT, high-confidence tokens, Luhn cards, OTP)
- [x] Consecutive dedup + retention
- [x] Unix-domain IPC + CLI history commands
- [x] X11 CLIPBOARD watch via XFixes (text)
- [x] Honest Wayland / GNOME Unsupported watch
- [x] Tauri v2 + Svelte history picker (daemon IPC, search, pin/delete/copy)
- [x] Unicode 17.0 emoji catalog, search, categories, skin tones, favorites
- [x] Curated symbols + kaomoji picker
- [x] Capability-based activation (X11 `XGrabKey`, GNOME Shell extension)
- [x] `clipl open` / `toggle` / `hide`
- [ ] KDE / Sway / Hyprland activation backends
- [ ] Overlay popup positioning
- [ ] Snippets catalogs in the UI
- [ ] Remote media providers
- [ ] GNOME Shell extension clipboard bridge

## Milestone C — Palette UI (Phase 3A + 3B)

**Ships:** Tauri v2 picker, history, Unicode 17.0 emoji, curated symbols and
kaomoji, daemon IPC, keyboard navigation.

**Does not ship:** auto-paste, GIF search, sticker store, packaging, snippets UI.

## Milestone D — Activation & desktop adapters (Phase 4A started)

**Ships in 4A:** X11 native shortcut, GNOME Shell **activation** extension,
desktop show/hide/toggle, CLI fallback.

**Does not ship:** KDE backend, Sway/Hyprland in-process grabs, generic Wayland
global hotkeys, GNOME clipboard push.

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
