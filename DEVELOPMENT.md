# Development

## Requirements

- Rust **1.80+** (developers should use stable via `rust-toolchain.toml`)
- `rustfmt` and `clippy`
- Linux. ClipLinux does not target macOS or Windows.
- A C compiler (bundled SQLite). X11 watch also needs a working `$DISPLAY`
  at **runtime**, not at compile time.

Optional, for the desktop UI:

- Node.js 20+
- Tauri v2 system libraries (WebKitGTK 4.1). Ubuntu/Debian:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  pkg-config \
  libdbus-1-dev \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Workspace `cargo test` does **not** require WebKitGTK. The desktop crate’s
default features exclude Tauri. `npm run tauri dev` enables `--features tauri-app`.

## Workspace commands

```bash
cargo fmt --all
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check.sh   # fmt, check, test, clippy, markdown links, GNOME schemas
```

Default tests never open the host clipboard. They use `MemoryClipboard` and
temporary SQLite files.

### Daemon and CLI

```bash
cargo run -p clipl-daemon -- --diagnose
cargo run -p clipl-daemon

cargo run -p clipl -- doctor
cargo run -p clipl -- doctor --json
cargo run -p clipl -- ping
cargo run -p clipl -- status
cargo run -p clipl -- status --json
cargo run -p clipl -- open
cargo run -p clipl -- toggle
cargo run -p clipl -- hide
cargo run -p clipl -- history --limit 20
cargo run -p clipl -- history search "query"
cargo run -p clipl -- history delete <id>
cargo run -p clipl -- history clear --yes
```

Config (optional): copy `config.example.toml` to
`$XDG_CONFIG_HOME/clipl/config.toml`.

Isolated runs for debugging:

```bash
CLIPL_DATA_DIR=/tmp/clipl-data \
CLIPL_RUNTIME_DIR=/tmp/clipl-run \
CLIPL_CONFIG_DIR=/tmp/clipl-config \
cargo run -p clipl-daemon -- --diagnose
```

Disable the X11 backend at compile time with `--no-default-features` on
`clipl-platform` if needed; the workspace default enables `x11`.

### Desktop + daemon

`cargo test --workspace` never starts a display server or `clipl-daemon`.

Run the picker against a live daemon:

```bash
# Terminal 1 — source of truth
cargo run -p clipl-daemon

# Terminal 2 — Tauri WebView (requires WebKitGTK)
cd apps/desktop
npm install
npm run test
npm run build
npm run tauri dev
```

`npm run tauri dev` is `tauri dev --features tauri-app`. Vite alone
(`npm run dev`) cannot talk to the daemon; the UI shows a disconnected state
instead of crashing.

Frontend checks:

```bash
cd apps/desktop
npm test
npm run check
npm run build
```

## Crate layout

See [ARCHITECTURE.md](ARCHITECTURE.md).

## Formatting and lints

- `rustfmt.toml` is the Rust style source of truth
- `unsafe_code` is **forbidden** at the workspace lint level

## Data you should not commit

`.gitignore` excludes `.env`, SQLite files, and local cache directories.
Never commit clipboard dumps.

## Tasks

`tasks/000-foundation.md` and `tasks/001-clipboard-monitoring.md` are
historical and complete. See [tasks/README.md](tasks/README.md). Current
status is [ROADMAP.md](ROADMAP.md). Phase 3A is the production desktop
shell. Phase 3B is the offline emoji and symbols engine. Phase 4A is
capability-based activation (`clipl open` / GNOME extension / X11
`XGrabKey`). The GNOME extension is statically validated, not
runtime-tested. See [extensions/gnome/README.md](extensions/gnome/README.md).
