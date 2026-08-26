#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.cargo/bin:${PATH}"

echo "==> cargo fmt --check"
cargo fmt --all -- --check

echo "==> cargo check"
cargo check --workspace --all-targets

echo "==> cargo test"
cargo test --workspace

echo "==> cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> markdown links"
python3 scripts/check-md-links.py

if command -v glib-compile-schemas >/dev/null 2>&1; then
  echo "==> GNOME schemas"
  glib-compile-schemas --dry-run extensions/gnome/schemas
else
  echo "==> GNOME schemas (skipped: glib-compile-schemas not installed)"
fi

echo "All checks passed."
