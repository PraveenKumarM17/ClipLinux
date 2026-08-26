# UniPick master plan

UniPick is a **universal paste, clipboard, and expression platform for Linux**.
The product is an ecosystem, not a single window:

1. **UniPick Desktop** — compact, keyboard-first palette (clipboard history,
   emoji, GIF search, stickers, symbols, snippets).
2. **UniPick Daemon** — background clipboard monitoring, history, privacy
   filtering, capability detection, media cache.
3. **UniPick CLI** — terminal access, diagnostics, capability checks.
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
- `unipick-core` must not depend on Tauri, Svelte, GTK, Qt, or compositor
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

## Phase 0 — Foundation (this repository state)

**Goal:** a compiling workspace, a documented architecture, and domain types
that later milestones can depend on without rewriting.

**In:**

- Cargo workspace and crate graph
- Domain types and traits
- CLI `doctor` / daemon stub / desktop stub
- Privacy rule types and a conservative engine
- Capability model with `SupportLevel::{Native,Portal,Fallback,Unsupported,Unknown}`
- Documentation listed in the README

**Out:**

- OS clipboard watching
- Global hotkeys
- Tauri window runtime dependency
- SQLite schema
- Remote GIF APIs
- GNOME / KDE extension code

**Exit:** `cargo test --workspace` passes; docs describe what is *not* built.

## Phase 1 — Persistence and daemon process

- SQLite `StorageBackend`
- Unix-socket IPC using `unipick-protocol`
- Daemon stays resident, loads privacy rules, exposes ping/history
- Still no clipboard watching

## Phase 2 — Clipboard history on one honest path

- Implement **one** `ClipboardBackend` chosen by capability detection
- Prefer documented APIs (X11 `CLIPBOARD`, Wayland data-control *or* portal)
- If watch is `Unsupported`, the UI still allows manual capture; it does not
  silently poll unless a documented fallback is accepted in a design review
- Privacy engine runs **before** bytes hit SQLite

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

Do not start Phase 2 work (clipboard monitoring) until Phase 0 is merged and
the capability matrix for the target session is written down. Guessing
Wayland behavior from X11 code is a defect, not a shortcut.
