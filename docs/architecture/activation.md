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

After a pick:

```
clipl-desktop writes CLIPBOARD, hides the picker
        ↓
InsertIntoApp
        ↓
X11: daemon XSetInputFocus + XTest Ctrl+V
GNOME Wayland: extension activates the saved window + Clutter Ctrl+V
```
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
| GNOME extension | Optional. Registers the Shell shortcut. Sends `ToggleDesktop`. Remembers the focused window and, after a pick, restores it and sends Ctrl+V. |

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

[insert]
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
- Requests are an enum: show / hide / toggle / insert — no command execution
- The GNOME extension does not spawn user-controlled shell commands
- Insert sends **only** Ctrl+V. It never types the clipboard payload and never logs keys.

## Insert into the previous app

Intended flow: you are typing in another app, open ClipLinux, pick emoji or
history, and the text appears in the message you were already writing.

1. On shortcut fire, snapshot the focused window (skip ClipLinux itself)
2. On pick, write CLIPBOARD, **then hide the picker**
3. Restore that window and send Ctrl+V

| Session | How insert is delivered | Validation |
| --- | --- | --- |
| X11 | Daemon `XSetInputFocus` + XTest Ctrl+V | Statically validated. Needs a live X11 session to runtime-test. |
| GNOME Wayland | Extension `SubscribeInsert` → activate saved window → Clutter virtual keyboard Ctrl+V | Statically validated. Super+V insert was **not** runtime-tested. |
| Generic Wayland / `clipl open` with no snapshot | Copy only | Expected |

`ydotool`, shell spawn, and payload key injection are forbidden.

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
does not use `ydotool`, fake clicks, or payload key injection. Restore-focus
insert (Ctrl+V only) is documented above.
