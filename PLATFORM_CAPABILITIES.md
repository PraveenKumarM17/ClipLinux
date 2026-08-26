# Platform capabilities

ClipLinux treats **X11, generic Wayland, GNOME, KDE Plasma, and wlroots** as
different integration surfaces. A feature that works on one is not assumed to
work on another.

The source of truth at runtime is `PlatformCapabilities` plus
`select_clipboard_backend()`.

## Identity vs capability

| Input | Maps to |
| --- | --- |
| `XDG_SESSION_TYPE` | `SessionType` (`x11` / `wayland`) |
| `XDG_CURRENT_DESKTOP` | `DesktopEnvironment` |

**Never use X11 APIs on a Wayland session**, even if `DISPLAY` is set (XWayland).

## Clipboard backend matrix (Phase 2)

| Session | Backend id | Read | Watch | Notes |
| --- | --- | --- | --- | --- |
| X11 + XFixes | `x11` | **IMPLEMENTED** Native (text) | **IMPLEMENTED** Native (`XFixesSelectionNotify`) | CLIPBOARD by default; PRIMARY optional via config |
| X11 connect fail | `x11-unavailable` | Unsupported | Unsupported | Diagnose prints the error |
| Generic Wayland | `wayland-generic` | **UNSUPPORTED** | **UNSUPPORTED** | No portable protocol; does **not** poll `wl-paste` |
| GNOME Wayland | `gnome` | **UNSUPPORTED** | **UNSUPPORTED** | Clipboard watch needs a **future** clipboard-bridge extension. `extensions/gnome` currently provides **activation only**. |
| KDE / Hyprland / Sway / wlroots | slot only | **PLANNED** | **PLANNED** | Named adapters, not implemented |
| Unknown session | `none` | Unknown | Unknown | Watch not started |

Write/image paste: **PLANNED** (X11 write is not implemented in this phase).

### X11 PRIMARY vs CLIPBOARD

- **CLIPBOARD** — Ctrl+C / Ctrl+V. Default (`clipboard.selection = "clipboard"`).
- **PRIMARY** — mouse selection buffer. Enable with `primary` or `both`.
- Monitoring PRIMARY will record many selection events; it is opt-in.

Watch waits on XFixes events. Idle wait uses `poll_for_event` with a short
sleep so shutdown is prompt. It does **not** spawn `xclip` in a loop.

### GNOME Wayland

Mutter does not offer `wlr-data-control` to regular clients. The daemon reports
Unsupported and keeps serving IPC. A future GNOME Shell extension should push
text to the daemon over the Unix socket — not key injection from the daemon.

## Adapter selection

`AdapterKind::preferred` still picks a slot from identity. Implemented slots:

- `linux-generic` — XDG probe + capability fill
- `x11` — XFixes backend
- `wayland-generic` — honest stub
- `gnome` — honest stub + extension boundary (activation implemented)

## Activation backend matrix (Phase 4A)

| Session | Backend | Shortcut | Status |
| --- | --- | --- | --- |
| X11 | `x11` | `XGrabKey` of `activation.shortcut` | **IMPLEMENTED** Native. Never used on Wayland. |
| GNOME Wayland | `gnome-shell` | GNOME extension GSettings | **IMPLEMENTED** Desktop-managed. Extension install is user-side. |
| KDE Plasma Wayland | `kde-plasma` | — | **PLANNED** |
| Sway / Hyprland | compositor bind | user config → `clipl toggle` | **UNSUPPORTED** in-process (correct mechanism is compositor config) |
| Generic Wayland | `wayland-generic` | — | **UNSUPPORTED** |

`clipl doctor` and `clipl status` print an Activation block. Super+V is the
default and **may conflict** with the desktop (GNOME notifications, WM binds).

See [docs/architecture/activation.md](docs/architecture/activation.md).

## Contributor rule

If you add a backend, you must:

1. Name the session it supports
2. Set each relevant `SupportLevel` explicitly
3. Add tests that do not require other compositors
4. Update this document
