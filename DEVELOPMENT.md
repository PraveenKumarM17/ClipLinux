# Development

## Requirements

- Rust **1.80+** (CI/workspace `rust-version` is 1.80; developers should use
  stable via `rust-toolchain.toml`)
- `rustfmt` and `clippy` (`rustup component add rustfmt clippy`)
- Linux. UniPick does not target macOS or Windows.

Optional, for the desktop UI later:

- Node.js 20+
- WebKitGTK / Tauri system libraries (only when the `tauri` crate is enabled)

## Workspace commands

```bash
# Format
cargo fmt --all

# Type-check every crate and binary
cargo check --workspace --all-targets

# Tests (no display server required)
cargo test --workspace

# Clippy
cargo clippy --workspace --all-targets -- -D warnings

# All of the above
bash scripts/check.sh
```

### Binaries

```bash
cargo run -p unipick -- doctor
cargo run -p unipick -- doctor --json
cargo run -p unipick -- ping
cargo run -p unipick -- version

cargo run -p unipick-daemon
cargo run -p unipick-daemon -- --diagnose

cargo run -p unipick-desktop
```

`unipick doctor` reads `XDG_SESSION_TYPE` and `XDG_CURRENT_DESKTOP`. It does
not open the clipboard.

### Desktop frontend (optional)

The Svelte app is a layout placeholder. It is not required for `cargo test`.

```bash
cd apps/desktop
npm install
npm run dev
```

Tauri’s CLI (`npm run tauri dev`) is **not** wired until the desktop-shell
milestone. `apps/desktop/src-tauri` compiles as a plain Rust binary so the
workspace does not require WebKitGTK for foundation checks.

## Crate layout

See [ARCHITECTURE.md](ARCHITECTURE.md). Path dependencies are declared once in
the workspace `Cargo.toml` `[workspace.dependencies]` table.

## Formatting and lints

- `rustfmt.toml` is the Rust style source of truth
- `unsafe_code` is **forbidden** at the workspace lint level
- Prefer `thiserror` over ad-hoc strings at crate boundaries; map into
  `unipick_core::Error` for IPC

## Data you should not commit

`.gitignore` excludes `.env`, SQLite files, and local cache directories.
Never commit clipboard dumps.

## Tasks

Numbered work items live in `tasks/`. `tasks/000-foundation.md` is done.
`tasks/001-clipboard-monitoring.md` is the next milestone and must not start
from this foundation PR.
