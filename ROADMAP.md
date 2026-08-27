# ClipLinux roadmap

Status of the product relative to the [master plan](MASTER_PLAN.md).

## Latest completed: Linux packages + GNOME clipboard bridge

Statically validated. GNOME Shell Super+V was **not** runtime-tested.

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
- [x] Insert into the previous app (copy + hide + Ctrl+V; GNOME extension / X11)
- [x] GNOME Shell extension clipboard bridge (text push over IPC)
- [x] Linux packages (Tauri `.deb` / `.rpm` / AppImage + GitHub Release)
- [ ] KDE / Sway / Hyprland activation backends

## Next (not started)

Snippets UI, then settings. Not GIFs, stickers, or cloud sync.

## Milestone C — Palette UI (Phase 3A + 3B)

**Shipped:** Tauri v2 picker, history, Unicode 17.0 emoji, curated symbols and
kaomoji, daemon IPC, keyboard navigation.

**Does not ship:** typing the payload as fake keys, GIF search, sticker store, snippets UI.

## Milestone D — Activation & desktop adapters (Phase 4A complete)

**Shipped in 4A (statically validated):** X11 native shortcut, GNOME Shell
**activation** extension, desktop show/hide/toggle, CLI fallback.

**Shipped next (statically validated):** restore-focus insert (Ctrl+V only) on
X11 and GNOME Wayland.

**Not runtime-tested:** live Super+V on GNOME Shell, live insert into another app.

**Does not ship:** KDE backend, Sway/Hyprland in-process grabs, generic Wayland
global hotkeys.

## Milestone E — Media

**Ships:** local stickers + one remote `MediaProvider` + on-disk cache.

**Not started.** `clipl-media` is a registry with an empty sticker library.

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
- Accounts, cloud sync, telemetry, or a ClipLinux backend server
- Fake universal Wayland global hotkeys
