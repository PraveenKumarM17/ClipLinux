# ClipLinux

Universal paste, clipboard, and expression for Linux.

ClipLinux is a keyboard-first Linux ecosystem: clipboard history, emoji, GIFs,
stickers, symbols, and snippets — with a daemon, a CLI, and first-class
desktop integrations. It is **not** an Electron app and **not** merely an
emoji picker.

**Latest completed milestone:** capability-based picker activation (Phase 4A).
X11 registers Super+V (configurable) via `XGrabKey`. GNOME Wayland uses the
Shell extension in `extensions/gnome` (default Super+Alt+V; Super+V is
GNOME's notification list). That extension is **statically validated**, not
GNOME-runtime-tested. Wayland clipboard monitoring is still **unsupported**.
GIFs, stickers, snippets UI, cloud sync, and packaging are not implemented.

## Quick start

```bash
cargo test --workspace
cargo run -p clipl-daemon -- --diagnose
cargo run -p clipl-daemon          # start (Ctrl+C to stop)
cargo run -p clipl -- doctor
cargo run -p clipl -- status
cargo run -p clipl -- open
cargo run -p clipl -- toggle
cargo run -p clipl -- hide
cargo run -p clipl -- history --limit 20
```

Desktop picker (requires the daemon and Tauri system libraries):

```bash
# Terminal 1
cargo run -p clipl-daemon

# Terminal 2
cd apps/desktop
npm install
npm run tauri dev
```

See [DEVELOPMENT.md](DEVELOPMENT.md) for the full contributor workflow.

## Documentation

| Document | Purpose |
| --- | --- |
| [MASTER_PLAN.md](MASTER_PLAN.md) | Product vision and delivery sequence |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crate graph, boundaries, and rules |
| [ROADMAP.md](ROADMAP.md) | Milestones and explicit non-goals |
| [PLATFORM_CAPABILITIES.md](PLATFORM_CAPABILITIES.md) | X11 / Wayland / GNOME / KDE matrix |
| [PRIVACY_MODEL.md](PRIVACY_MODEL.md) | What is stored, filtered, and never logged |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to land changes |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Tooling and local commands |
| [docs/architecture/clipboard-engine.md](docs/architecture/clipboard-engine.md) | History pipeline |
| [docs/architecture/daemon.md](docs/architecture/daemon.md) | Daemon behaviour |
| [docs/architecture/ipc.md](docs/architecture/ipc.md) | Unix socket protocol |
| [docs/architecture/storage.md](docs/architecture/storage.md) | SQLite schema and paths |
| [docs/architecture/desktop.md](docs/architecture/desktop.md) | Tauri + Svelte picker |
| [docs/architecture/desktop-daemon-boundary.md](docs/architecture/desktop-daemon-boundary.md) | Why the UI never opens SQLite |
| [docs/architecture/emoji-engine.md](docs/architecture/emoji-engine.md) | Unicode 17.0 emoji search |
| [docs/architecture/symbols-engine.md](docs/architecture/symbols-engine.md) | Curated symbols and kaomoji |
| [docs/architecture/activation.md](docs/architecture/activation.md) | X11 grab, GNOME extension, CLI open/toggle |
| [docs/platform-support/x11.md](docs/platform-support/x11.md) | X11 clipboard + native shortcut |
| [docs/platform-support/gnome.md](docs/platform-support/gnome.md) | GNOME Wayland activation |

## Layout

```
apps/          desktop (Tauri + Svelte), daemon, CLI
crates/        Rust libraries; core has no UI toolkit dependency
extensions/    GNOME Shell activation extension; KDE planned slot
packages/      emoji data; sticker packs and themes (planned)
docs/          additional architecture notes
tasks/         completed historical tasks
scripts/       workspace quality checks
```

Licensed under MIT OR Apache-2.0.
