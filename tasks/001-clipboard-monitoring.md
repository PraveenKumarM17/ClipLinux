# Task 001 — Clipboard monitoring

Status: **not started**

Do not implement this as part of the foundation.

## When this opens

After the foundation is accepted and a written capability probe exists for
the first target session.

## Required before code

1. Name the session (e.g. “X11 with XFixes”, “GNOME 47 portal”, “wlroots
   wlr-data-control-unstable-v1”).
2. Fill `SupportLevel` for `clipboard-read`, `clipboard-write`, and
   `clipboard-watch` from a real probe — not from compositor folklore.
3. Route every captured `ClipboardItem` through `unipick-privacy::decide`
   before SQLite.

## Explicitly forbidden until designed

- Polling `xclip` / `wl-paste` in a tight loop as the default backend
- Using X11 APIs on a Wayland session because “it compiled”
- Logging clipboard payloads
