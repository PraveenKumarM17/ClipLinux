# ClipLinux

Universal paste, clipboard, and expression for Linux.

ClipLinux is a keyboard-first Linux ecosystem: clipboard history, emoji, GIFs,
stickers, symbols, and snippets — with a daemon, a CLI, and first-class
desktop integrations. It is **not** an Electron app and **not** merely an
emoji picker.

**Current milestone:** clipboard history daemon (text), SQLite, Unix IPC, X11
watch. Wayland clipboard monitoring is **unsupported** until a compositor
adapter or GNOME extension exists. Global shortcuts and the Tauri UI are not
in this phase.

## Quick start

```bash
cargo test --workspace
cargo run -p clipl-daemon -- --diagnose
cargo run -p clipl-daemon          # start (Ctrl+C to stop)
cargo run -p clipl -- doctor
cargo run -p clipl -- status
cargo run -p clipl -- history --limit 20
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

## Layout

```
apps/          desktop (Tauri + Svelte), daemon, CLI
crates/        Rust libraries; core has no UI toolkit dependency
extensions/    GNOME and KDE integration (placeholders)
packages/      emoji data, sticker packs, themes
docs/          additional architecture notes
```

Licensed under MIT OR Apache-2.0.
