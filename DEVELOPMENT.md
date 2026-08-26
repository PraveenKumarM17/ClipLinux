# Development

## Requirements

- Rust **1.80+** (developers should use stable via `rust-toolchain.toml`)
- `rustfmt` and `clippy`
- Linux. ClipLinux does not target macOS or Windows.
- A C compiler (bundled SQLite). X11 watch also needs a working `$DISPLAY`
  at **runtime**, not at compile time.

Optional, for the desktop UI later:

- Node.js 20+
- WebKitGTK / Tauri system libraries

## Workspace commands

```bash
cargo fmt --all
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
bash scripts/check.sh
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

### Desktop frontend (optional)

The Svelte app is a layout placeholder. It is not required for `cargo test`.

```bash
cd apps/desktop
npm install
npm run dev
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

`tasks/000-foundation.md` is done. `tasks/001-clipboard-monitoring.md` is this
phase.
