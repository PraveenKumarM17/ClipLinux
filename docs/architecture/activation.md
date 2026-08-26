# Activation architecture

Status: **IMPLEMENTED** for Phase 4A (X11 native grab + GNOME Shell boundary).
KDE, Sway, and Hyprland are **named slots only**.

ClipLinux does not implement a fake universal `global_shortcut()`. Wayland
sessions never use `XGrabKey`, even when `DISPLAY` is set (XWayland).

## Flow

```
User presses the configured shortcut
        ↓
Platform-specific backend (X11 grab or GNOME extension)
        ↓
clipl-daemon  (ShowDesktop / HideDesktop / ToggleDesktop)
        ↓
Subscribed clipl-desktop receives Event::ActivatePicker
        ↓
Window is shown or hidden; search is focused
```

CLI fallback (same IPC, no key grab):

```
clipl open
clipl hide
clipl toggle
```

If the desktop process is not subscribed, those commands report
`desktop picker is not running` and do **not** spawn a second WebView.

## Capability classes

| Capability | Meaning |
| --- | --- |
| `NativeGlobalShortcut` | Application `XGrabKey` (X11 only) |
| `DesktopManagedShortcut` | DE owns the key (GNOME extension) |
| `CompositorBinding` | User binds a key in Sway/Hyprland config |
| `ManualOnly` | CLI / launch only |
| `Unsupported` | No activation path |

## Backend matrix

| Session | Backend | Phase 4A | Shortcut owner |
| --- | --- | --- | --- |
| X11 | `x11` | **Implemented** `XGrabKey` | `clipl-daemon` |
| GNOME Wayland | `gnome-shell` | **Implemented** extension | GNOME GSettings |
| GNOME X11 | `x11` (preferred if x11 enabled) | **Implemented** | daemon grab; extension optional |
| KDE Plasma Wayland | `kde-plasma` | **Planned** | — |
| Sway | `sway` | **Unsupported** in-process | compositor config |
| Hyprland | `hyprland` | **Unsupported** in-process | compositor config |
| Generic Wayland | `wayland-generic` | **Unsupported** | CLI only |

Statuses: `Active`, `ConfiguredExternally`, `NotConfigured`, `Unsupported`, `Error`.

GNOME does **not** report `Active` from the daemon. The Shell owns the grab.
If the extension directory exists, status is `ConfiguredExternally`.

## Lifecycle

| Process | Role |
| --- | --- |
| `clipl-daemon` | Persistent. IPC, history, optional X11 grab. Does not spawn the desktop. |
| `clipl-desktop` | Persistent WebView. Escape and the window close button **hide** the window. The process stays alive. |
| GNOME extension | Optional. Registers the Shell shortcut. Sends `ToggleDesktop`. |

The daemon does not launch the desktop. The extension does not launch a
shell command; it may call `Shell.AppSystem.activate()` if
`io.clipl.ClipLinux.desktop` is installed (Wayland-supported focus).

Do not start a new WebView on every shortcut press.

## Configuration

```toml
[activation]
enabled = true
shortcut = "Super+V"
behavior = "toggle"   # or show

[activation.x11]
enabled = true

[activation.gnome]
enabled = true
```

**Ownership:** X11 reads `activation.shortcut` from `config.toml`. GNOME
reads `org.gnome.shell.extensions.clipl activate-shortcut`. They are not
synced automatically. Super+V may conflict with the window manager
(including GNOME's notification list).

Bare keys (`v`) are rejected so ClipLinux never grabs typing keys.

## Security

- Only the configured chord is registered (X11) or bound (GNOME)
- No general keyboard capture, no keystroke logs
- Activation IPC is the existing Unix socket (`0600`), not TCP
- Requests are an enum: show / hide / toggle — no command execution
- The GNOME extension does not spawn user-controlled shell commands

## Sway / Hyprland (user config, not implemented backends)

Sway:

```
bindsym $mod+v exec clipl toggle
```

Hyprland:

```
bind = SUPER, V, exec, clipl toggle
```

These snippets assume `clipl` is on `PATH` and the daemon + desktop are
already running.

## Wayland window focus

Hidden → shown on GNOME Wayland may still fail to take keyboard focus
without a compositor activation token. The extension's `app.activate()`
path is the supported mechanism when a `.desktop` file exists. ClipLinux
does not use `ydotool`, fake clicks, or input injection.
