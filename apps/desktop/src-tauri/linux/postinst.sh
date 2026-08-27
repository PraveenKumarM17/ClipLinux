#!/bin/sh
# Appended to the Debian postinst. Runs as root after unpack.
# Do not talk to a user GNOME session from here — Wayland cannot load a new
# Shell extension until that user logs out and back in.
set -e

EXT="/usr/share/gnome-shell/extensions/clipl@io.clipl"
if [ -d "$EXT/schemas" ] && command -v glib-compile-schemas >/dev/null 2>&1; then
    glib-compile-schemas "$EXT/schemas" || true
fi

echo "ClipLinux: the daemon and picker autostart on login."
echo "ClipLinux: on GNOME Wayland, log out and back in so Shell loads the ClipLinux extension."
echo "ClipLinux: until then, copies are not recorded and picks stay on the clipboard (Ctrl+V)."
