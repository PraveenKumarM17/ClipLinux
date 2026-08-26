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

This file is the delivery sequence. Implementation details live in
[ARCHITECTURE.md](ARCHITECTURE.md). Dates are deliberately omitted; each
milestone ends when its exit criteria are met, not when a calendar says so.

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

**Out of this phase:** global hotkeys, overlay, auto-paste, Tauri UI, GIF APIs,
GNOME/KDE extension implementations.

## Phase 3 — Desktop palette shell

- Wire Tauri v2 to the Svelte 5 UI
- Keyboard-first popup, themed from `packages/themes`
- Talk to the daemon; never embed compositor logic in Svelte

## Phase 4 — Emoji, symbols, snippets (offline)

- Load `packages/emoji-data`
- Symbol catalog expansion
- Snippet CRUD in the UI
- Paste via the backend that Phase 2 proved

## Phase 5 — Desktop-environment adapters

- GNOME extension and KDE integration as first-class adapters
- Generic Wayland and wlroots adapters with explicit `SupportLevel`
- Hyprland and Sway remain named slots until a maintainer owns them

## Phase 6 — Media and stickers

- Local sticker scanner
- First remote GIF provider behind `MediaProvider`
- Disk cache with retention and privacy rules (no GIF query logging by default)

## Phase 7 — Ecosystem

- Community pack format
- Theme documentation
- Plugin-ready provider traits stabilized (no dynamic loading required yet)

## Ownership of later work

Phase 2 is complete. Do not start global shortcuts, overlay, or the Tauri
production UI until the history daemon is the agreed IPC source. Guessing
Wayland clipboard behavior from X11 code remains a defect, not a shortcut.
