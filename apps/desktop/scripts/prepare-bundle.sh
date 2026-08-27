#!/usr/bin/env bash
# Build the Svelte UI and release daemon/CLI binaries for `tauri build`.
# Invoked as `beforeBuildCommand` from apps/desktop.
set -euo pipefail

DESKTOP="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROOT="$(cd "$DESKTOP/../.." && pwd)"
cd "$DESKTOP"

npm run build

# Always the workspace target so tauri.conf.json extra-file paths stay valid
# even when CARGO_TARGET_DIR is set (CI caches, local sandboxes).
cargo build --release -p clipl-daemon -p clipl \
  --manifest-path "$ROOT/Cargo.toml" \
  --target-dir "$ROOT/target"

# Ship compiled GSettings so GNOME can load the extension even when the
# install target does not have `glib-compile-schemas` on PATH.
if command -v glib-compile-schemas >/dev/null 2>&1; then
  glib-compile-schemas "$ROOT/extensions/gnome/schemas"
fi
