# Platform capabilities

UniPick treats **X11, generic Wayland, GNOME, KDE Plasma, and wlroots** as
different integration surfaces. A feature that works on one is not assumed to
work on another.

The source of truth at runtime is `PlatformCapabilities` in `unipick-core`.
This document is the human-readable contract for that type.

## Identity vs capability

Probe identity from documented environment:

| Input | Maps to |
| --- | --- |
| `XDG_SESSION_TYPE` | `SessionType` (`x11` / `wayland`) |
| `XDG_CURRENT_DESKTOP` | `DesktopEnvironment` |

Identity is **not** permission to use a feature. After identity is known, each
`Capability` is assigned a `SupportLevel` by an adapter that actually checks
(or honestly reports `Unknown`).

```
XDG_CURRENT_DESKTOP=Hyprland  ≠  ClipboardWatch = Native
```

Hyprland is a named `DesktopEnvironment` so a future adapter can exist. Until
that adapter is implemented, `AdapterKind::Hyprland.is_implemented()` is
`false` and the generic Linux adapter stays in use.

## Capabilities

| Capability | Meaning |
| --- | --- |
| `clipboard-read` | Read the current selection/clipboard |
| `clipboard-write` | Set the clipboard |
| `clipboard-watch` | Observe changes without an undocumented poll loop |
| `global-hotkey` | Register a shortcut while another app is focused |
| `overlay-popup` | Show a compact palette above other windows |
| `image-paste` | Write image bytes, not only text |
| `file-paste` | Write file URIs |
| `portal-integration` | xdg-desktop-portal (or equivalent) is usable |
| `gnome-extension` | GNOME Shell extension APIs are in play |
| `kde-integration` | Plasma APIs are in play |
| `local-storage` | App data directory (almost always native on Linux) |
| `network` | Remote media providers may run |

## Support levels

| Level | Meaning | May UniPick use it? |
| --- | --- | --- |
| `Native` | First-class protocol/toolkit | Yes |
| `Portal` | Desktop portal / DE API | Yes |
| `Fallback` | Documented degraded path, reviewed | Yes |
| `Unsupported` | Probed, unavailable | No |
| `Unknown` | Not probed | No (do not guess) |

`Fallback` is allowed only when written down (this file or a design doc) and
covered by tests. A busy-loop `xclip` poll is not a silent fallback.

## Expected matrix (design, not current probes)

Foundation adapters report `Unknown` for almost everything except
`local-storage` and `network`. The table below is the **intended** end state,
not a claim that code implements it.

| Capability | X11 | Generic Wayland | GNOME | KDE Plasma | wlroots | Hyprland / Sway |
| --- | --- | --- | --- | --- | --- | --- |
| clipboard-read | Native (ICCMM) | Unknown / compositor-specific | Portal or Shell | Native / portal | wlr-data-control where present | Same as owning compositor |
| clipboard-write | Native | Same as read | Same as read | Same as read | Same as read | Same as read |
| clipboard-watch | Native (XFixes) | Often **Unsupported** without a protocol | Prefer DE/portal | Prefer Klipper/portal | Native if data-control | Only with a dedicated adapter |
| global-hotkey | Possible (XGrabKey) | Often **Unsupported** globally | Shell extension | KGlobalAccel | compositor IPC | compositor IPC |
| overlay-popup | Layered window | Layer shell *if* offered | Shell-owned UI preferred | Plasma windowing | layer-shell | layer-shell |
| image-paste | Usually yes | Depends on mime offer | Depends | Depends | Depends | Depends |
| gnome-extension | Unsupported | Unsupported | Native when installed | Unsupported | Unsupported | Unsupported |
| kde-integration | Unsupported | Unsupported | Unsupported | Native when installed | Unsupported | Unsupported |

**Never copy X11 clipboard code into a Wayland backend.** Share tests and
types; do not share protocol assumptions.

## Adapter selection

`unipick-platform::AdapterKind::preferred` picks a *slot* from identity:

1. GNOME → `gnome`
2. Plasma → `kde`
3. Hyprland / Sway / wlroots → named slots
4. X11 session → `x11`
5. Wayland session → `wayland-generic`
6. Else → `linux-generic`

Only `linux-generic` is implemented in the foundation. Preferred-but-missing
adapters must not be silently replaced with X11 behavior on Wayland.

## Portals

When a compositor does not expose clipboard watch or global shortcuts, UniPick
should try **xdg-desktop-portal** (or the DE equivalent) and set
`SupportLevel::Portal`. If the portal is absent, the level is `Unsupported`,
and the UI explains the gap instead of polling.

## Contributor rule

If you add a backend, you must:

1. Name the session it supports
2. Set each relevant `SupportLevel` explicitly
3. Add tests that do not require other compositors
4. Update this document
