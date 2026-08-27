# ClipLinux GNOME Shell extension

This extension is the **GNOME Wayland activation and insert backend**. It does
not monitor the clipboard or open SQLite. After a pick it may send **Ctrl+V**
only — never the clipboard payload as typed keys.

```
GNOME Shell shortcut
        ↓
ClipLinux extension remembers global.display.focus_window
        ↓
Unix socket ToggleDesktop  ($XDG_RUNTIME_DIR/clipl/daemon.sock)
        ↓
clipl-daemon
        ↓
subscribed clipl-desktop shows the picker
        ↓
pick → desktop writes CLIPBOARD, hides, InsertIntoApp
        ↓
extension activates the saved window and sends Ctrl+V
```

On GNOME Wayland, `clipl-daemon` **must not** call `XGrabKey`. The Shell
owns Super+V (default) through GSettings.

## Requirements

- GNOME Shell **46–50** listed in `metadata.json` so the extension can load on
  current Ubuntu GNOME. The layout is ESM (GNOME 45+). **No Shell version was
  runtime-tested** for Super+V or insert (static review only).
- `clipl-daemon` running in the same user session
- `clipl-desktop` running (hidden is fine) so the daemon has a subscriber
- This extension enabled so insert can restore the previous window

Re-copy the extension into
`~/.local/share/gnome-shell/extensions/clipl@io.clipl` and log out/in on
Wayland after updating.

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

X11 sessions do **not** need this extension for the shortcut (`XGrabKey`).
They also do not need it for insert (daemon XTest). On GNOME Wayland this
extension is required for both.

## Security

- Local Unix socket only (`0600`)
- Actions are hardcoded `ToggleDesktop` / `SubscribeInsert` / Ctrl+V — no
  user-controlled command string
- No `GLib.spawn_*`, no `xdg-open`, no `ydotool`, no typing of clipboard text
- `SubscribeInsert` uses `read_bytes_async` so the Shell is not blocked
- If `io.clipl.ClipLinux.desktop` is installed, the extension may call
  GNOME's `Shell.AppSystem.activate()` so Wayland focus is user-initiated
- Windows whose `wm_class` contains `clipl` are never insert targets

## Ownership of the shortcut

| Store | Owner |
| --- | --- |
| `config.toml` `[activation].shortcut` | ClipLinux (X11 grab + doctor text) |
| GSettings `activate-shortcut` | This extension on GNOME |

They are not automatically synchronized. Document both.

## Uninstall

```bash
gnome-extensions disable clipl@io.clipl
rm -rf "$HOME/.local/share/gnome-shell/extensions/clipl@io.clipl"
```
