# ClipLinux master plan

ClipLinux is a **universal paste, clipboard, and expression platform for Linux**.
The product is an ecosystem, not a single window:

1. **ClipLinux Desktop** — compact, keyboard-first palette (clipboard history,
   emoji, GIF search, stickers, symbols, snippets).
2. **ClipLinux Daemon** — background clipboard monitoring, history, privacy
   filtering, capability detection, media cache.
3. **ClipLinux CLI** — terminal access, diagnostics, capability checks.
4. **Platform integrations** — X11, generic Wayland, GNOME, KDE Plasma,
   wlroots compositors, with future Hyprland and Sway adapters.
5. **Extensible content** — replaceable GIF providers, local sticker packs,
   community packs, themes, plugin-ready interfaces.

This file is the **delivery sequence and product constraints**. The live
checklist is [ROADMAP.md](ROADMAP.md). Implementation details live in
[ARCHITECTURE.md](ARCHITECTURE.md). Dates are deliberately omitted; each
milestone ends when its exit criteria are met, not when a calendar says so.

Completed phases below are historical. They describe what shipped, not what to
rebuild.

## Principles that constrain every milestone

- Rust is the core language. The desktop shell is **Tauri v2**. The UI is
  **Svelte 5 + TypeScript**. Persistence is **SQLite**.
- `clipl-core` must not depend on Tauri, Svelte, GTK, Qt, or compositor
  crates.
- Platform-specific code is isolated behind traits (`ClipboardBackend`,
  `PlatformAdapter`, …).
- Wayland and X11 are different platforms. Capability detection is required;
  undocumented compositor hacks are not.
- Privacy is a product feature, not a settings afterthought.
- Offline emoji, symbols, snippets, history, and local stickers must work
  without a network.
- Media providers are replaceable. No provider is hardcoded into core.
- Electron is out of scope forever.
- Offline-first, local-only, no account, no cloud sync, no telemetry.
- The tree must stay approachable for first-time open-source contributors.

## Phase 0 — Foundation

**Status:** complete.

Workspace, domain types, privacy rule types, capability model, CLI `doctor`,
and documentation. Exit: `cargo test --workspace` passed.

## Phase 1 — Persistence and daemon process

**Status:** folded into Phase 2 (SQLite + IPC shipped together with watch).

## Phase 2 — Clipboard history on one honest path

**Status:** complete for text history.

- SQLite `StorageBackend` + typed `clipboard_items`
- Unix-socket IPC using `clipl-protocol`
- Daemon stays resident, loads privacy rules, exposes ping/history/status
- X11 `CLIPBOARD` via XFixes is the Native watch path
- Generic Wayland and GNOME Wayland report Unsupported (no silent polling)
- Privacy engine runs **before** bytes hit SQLite
- If watch is `Unsupported`, the daemon still serves IPC; it does not poll
  `xclip` / `wl-paste`

**Did not ship in this phase:** global hotkeys, overlay, auto-paste, Tauri UI,
GIF APIs, GNOME/KDE extension implementations. Those came later or remain
planned.

## Phase 3 — Desktop palette (ROADMAP 3A + 3B)

**Status:** complete for history, emoji, and symbols. Snippets UI is not
implemented.

- Tauri v2 + Svelte 5 picker talks to `clipl-daemon` over Unix IPC
- History search, pin, delete, clear, copy (desktop never opens SQLite)
- Unicode 17.0 emoji catalog, search, categories, skin tones, favorites
- Curated symbols and kaomoji picker
- Snippets tab is an honest placeholder
- Theme JSON in `packages/themes` exists as a future token source; the UI
  currently uses hardcoded CSS in `apps/desktop/src/app.css`

## Phase 4 — Activation (ROADMAP 4A)

**Status:** complete and **statically validated**. The GNOME Shell shortcut
path was **not** runtime-tested (no live Super+V session test).

- X11 `XGrabKey` of `activation.shortcut` (never on Wayland)
- GNOME Shell extension `clipl@io.clipl` sends `ToggleDesktop` over the
  local socket
- `clipl open` / `hide` / `toggle`
- KDE / Sway / Hyprland remain named slots, not implementations

There is no fake universal Wayland global hotkey.

## Phase 5 — Remaining desktop-environment adapters

**Status:** planned.

- KDE Plasma global shortcut / documented Plasma APIs
- GNOME clipboard **bridge** (the current extension is activation only)
- Hyprland and Sway remain compositor-config bindings, not in-process grabs

## Phase 6 — Media and stickers

**Status:** planned. `clipl-media` is a registry + empty sticker library.

- Local sticker scanner
- First remote GIF provider behind `MediaProvider`
- Disk cache with retention and privacy rules (no GIF query logging by default)

## Phase 7 — Ecosystem

**Status:** planned.

- Community pack format
- Theme documentation and loading `packages/themes` into the UI
- Plugin-ready provider traits stabilized (no dynamic loading required yet)

## Ownership of later work

Do not start snippets UI, GIFs, stickers, cloud sync, or new
platform backends from a cleanup pass. The next product work is listed in
[ROADMAP.md](ROADMAP.md). Guessing Wayland clipboard behavior from X11 code
remains a defect, not a shortcut.
