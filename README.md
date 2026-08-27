# ClipLinux

Universal paste, clipboard, and expression for Linux.

ClipLinux is a keyboard-first Linux ecosystem: clipboard history, emoji, GIFs,
stickers, symbols, and snippets — with a daemon, a CLI, and first-class
desktop integrations. It is **not** an Electron app and **not** merely an
emoji picker.

**Latest completed milestone:** GNOME clipboard bridge + Linux packaging
(`.deb` / `.rpm` / AppImage via Tauri). GNOME Wayland needs the Shell
extension and a log out after install. GIFs, stickers, snippets UI, and
cloud sync are not implemented.

## Who this works for

| Session | History + insert | How to open |
| --- | --- | --- |
| GNOME Wayland | After the ClipLinux extension loads (log out/in) | Super+Alt+V |
| X11 | Yes | Super+V (configurable) |
| Other Wayland compositors | Not yet (copy + Ctrl+V only) | `clipl toggle` |

This is **not** “runs on all Linux desktops.” KDE, Sway, and Hyprland still
need compositor-specific work.

## Quick start (from source)

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

On GNOME Wayland, install `extensions/gnome` then **log out and back in**.
See [extensions/gnome/README.md](extensions/gnome/README.md).

## Packages

Build installers (needs WebKitGTK; produces files under `target/release/bundle/`):

```bash
cd apps/desktop
npm install
npm run tauri build
```

`.deb` / `.rpm` autostart the daemon and picker on login and install the GNOME
extension files. **Log out after installing on GNOME Wayland.** Other clipboard
extensions do not need to be disabled first; if copies still do not appear,
`gnome-extensions info clipl@io.clipl` should show the ClipLinux extension as
enabled in this session.

Release tags `v*.*.*` build those artifacts in GitHub Actions. Details:
[packaging/linux/README.md](packaging/linux/README.md).

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
