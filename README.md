# UniPick

Universal paste, clipboard, and expression for Linux.

UniPick is a keyboard-first Linux ecosystem: clipboard history, emoji, GIFs,
stickers, symbols, and snippets — with a daemon, a CLI, and first-class
desktop integrations. It is **not** an Electron app and **not** merely an
emoji picker.

This repository currently contains the **foundation**: crate layout, domain
types, traits, documentation, and compiling placeholders. Clipboard monitoring
is intentionally not implemented yet.

## Quick start

```bash
# Rust workspace
cargo check --workspace
cargo test --workspace
cargo run -p unipick -- doctor
cargo run -p unipick-daemon -- --diagnose
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

## Layout

```
apps/          desktop (Tauri + Svelte), daemon, CLI
crates/        Rust libraries; core has no UI toolkit dependency
extensions/    GNOME and KDE integration (placeholders)
packages/      emoji data, sticker packs, themes
docs/          additional architecture notes
```

Licensed under MIT OR Apache-2.0.
