# Task 001 — Clipboard monitoring

Status: **complete** (Phase 2)

## Named sessions

| Session | Watch mechanism | SupportLevel |
| --- | --- | --- |
| X11 | XFixes `SelectionNotify` on CLIPBOARD | Native when `$DISPLAY` + XFixes work |
| Generic Wayland | none | Unsupported |
| GNOME Wayland | none (extension boundary) | Unsupported |

Every captured item is classified and passed through `clipl_privacy::evaluate`
**before** SQLite.

## Done

- SQLite schema v1, migrations, XDG paths, `0600` files
- Privacy detectors + explainable verdicts
- Consecutive dedup (SHA-256)
- Daemon + Unix IPC
- CLI history commands
- Mock clipboard integration tests (no host clipboard)

## Still forbidden / not in this task

- Polling `xclip` / `wl-paste` as the default backend
- Using X11 APIs on a Wayland session
- Logging clipboard payloads
- Global hotkeys, overlay, auto-paste
