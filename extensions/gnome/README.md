# ClipLinux GNOME Shell extension

This extension is the **GNOME Wayland activation backend**. It does not
monitor the clipboard, open SQLite, or inject keystrokes.

```
GNOME Shell shortcut
        ↓
ClipLinux extension (this directory)
        ↓
Unix socket ToggleDesktop  ($XDG_RUNTIME_DIR/clipl/daemon.sock)
        ↓
clipl-daemon
        ↓
subscribed clipl-desktop shows the picker
```

On GNOME Wayland, `clipl-daemon` **must not** call `XGrabKey`. The Shell
owns Super+V (default) through GSettings.

## Requirements

- GNOME Shell **46–50** listed in `metadata.json` so the extension can load on
  current Ubuntu GNOME. The layout is ESM (GNOME 45+). **No Shell version was
  runtime-tested in Phase 4A** (no live Super+V session test).
- `clipl-daemon` running in the same user session
- `clipl-desktop` running (hidden is fine) so the daemon has a subscriber

This machine's GNOME Shell version was **not** used as a test target during
Phase 4A unless noted in the engineering report.

## Install (user session)

```bash
EXT="$HOME/.local/share/gnome-shell/extensions/clipl@io.clipl"
mkdir -p "$EXT"
cp -a extensions/gnome/. "$EXT/"
glib-compile-schemas "$EXT/schemas"
gnome-extensions enable clipl@io.clipl
```

On Wayland, restart the session (log out/in) after installing. `Alt+F2` `r`
only restarts the Shell on X11.

Confirm:

```bash
gnome-extensions info clipl@io.clipl
gsettings get org.gnome.shell.extensions.clipl activate-shortcut
```

Default binding: `['<Super>v']`.

**Conflict:** recent GNOME versions also use Super+V for the notification
list. Change either binding if they collide.

```bash
gsettings set org.gnome.shell.extensions.clipl activate-shortcut "['<Super>period']"
```

X11 sessions do **not** need this extension. `clipl-daemon` registers
`XGrabKey` there when `[activation.x11] enabled = true`.

## Security

- Local Unix socket only (`0600`)
- The action is hardcoded `ToggleDesktop` — no user-controlled command string
- No `GLib.spawn_*`, no `xdg-open` of arbitrary URLs, no clipboard access
- If `io.clipl.ClipLinux.desktop` is installed, the extension may call
  GNOME's `Shell.AppSystem.activate()` so Wayland focus is user-initiated

## Ownership of the shortcut

| Store | Owner |
| --- | --- |
| `config.toml` `[activation].shortcut` | ClipLinux (X11 grab + doctor text) |
| GSettings `activate-shortcut` | This extension on GNOME |

They are not automatically synchronized in Phase 4A. Document both.

## Uninstall

```bash
gnome-extensions disable clipl@io.clipl
rm -rf "$HOME/.local/share/gnome-shell/extensions/clipl@io.clipl"
```
