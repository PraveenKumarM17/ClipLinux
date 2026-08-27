#!/usr/bin/env bash
# Install the ClipLinux .desktop file and a GNOME custom shortcut that runs
# `clipl toggle`. Does not spawn a shell from the GNOME extension.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DESKTOP_SRC="$ROOT/packaging/linux/io.clipl.ClipLinux.desktop"
ICON_SRC="$ROOT/apps/desktop/src-tauri/icons/128x128.png"
CLIPL_BIN="${CLIPL_BIN:-$ROOT/target/debug/clipl}"
DESKTOP_BIN="${CLIPL_DESKTOP_BIN:-$ROOT/target/debug/clipl-desktop}"
BINDING="${CLIPL_SHORTCUT:-<Super><Alt>v}"
SCHEMA="org.gnome.settings-daemon.plugins.media-keys"
RELOC="org.gnome.settings-daemon.plugins.media-keys.custom-keybinding"
CLIPL_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clipl/"

if [[ ! -f "$DESKTOP_SRC" ]]; then
  echo "missing $DESKTOP_SRC" >&2
  exit 1
fi
if [[ ! -x "$CLIPL_BIN" ]]; then
  echo "clipl binary not found at $CLIPL_BIN (build with: cargo build -p clipl)" >&2
  exit 1
fi
if [[ ! -x "$DESKTOP_BIN" ]]; then
  echo "clipl-desktop binary not found at $DESKTOP_BIN (build with: cargo build -p clipl-desktop --features tauri-app)" >&2
  exit 1
fi

mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" \
  "$HOME/.local/share/icons/hicolor/128x128/apps"

ln -sfn "$CLIPL_BIN" "$HOME/.local/bin/clipl"
ln -sfn "$DESKTOP_BIN" "$HOME/.local/bin/clipl-desktop"
cp "$DESKTOP_SRC" "$HOME/.local/share/applications/io.clipl.ClipLinux.desktop"
if [[ -f "$ICON_SRC" ]]; then
  cp "$ICON_SRC" "$HOME/.local/share/icons/hicolor/128x128/apps/io.clipl.ClipLinux.png"
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$HOME/.local/share/applications" >/dev/null 2>&1 || true
fi

python3 - "$SCHEMA" "$RELOC" "$CLIPL_PATH" "$BINDING" <<'PY'
import ast
import subprocess
import sys

schema, reloc, path, binding = sys.argv[1:5]
raw = subprocess.check_output(["gsettings", "get", schema, "custom-keybindings"], text=True)
paths = list(ast.literal_eval(raw.strip()))
if path not in paths:
    paths.append(path)
    value = "[" + ", ".join("'" + item + "'" for item in paths) + "]"
    subprocess.check_call(["gsettings", "set", schema, "custom-keybindings", value])

prefix = f"{reloc}:{path}"
subprocess.check_call(["gsettings", "set", prefix, "name", "ClipLinux"])
subprocess.check_call(["gsettings", "set", prefix, "command", "clipl toggle"])
subprocess.check_call(["gsettings", "set", prefix, "binding", binding])
PY

echo "Installed ~/.local/share/applications/io.clipl.ClipLinux.desktop"
echo "Bound ClipLinux to $BINDING → clipl toggle"
echo "Keep clipl-daemon and clipl-desktop running. Super+V stays GNOME's notification list."
