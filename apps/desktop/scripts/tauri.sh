#!/usr/bin/env bash
# Enable the `tauri-app` Cargo feature for real WebView builds.
# Workspace `cargo test` keeps that feature off so WebKitGTK is not required.
set -euo pipefail
cd "$(dirname "$0")/.."
cmd="${1:-help}"
shift || true
case "$cmd" in
  dev|build)
    exec npx tauri "$cmd" --features tauri-app "$@"
    ;;
  *)
    exec npx tauri "$cmd" "$@"
    ;;
esac
