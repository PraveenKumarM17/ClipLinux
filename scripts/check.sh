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

echo "All checks passed."
