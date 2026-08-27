# ClipLinux GNOME Shell extension

This extension is the **GNOME Wayland activation, insert, and clipboard
bridge**. It does not open SQLite. After a pick it may send **Ctrl+V** only —
never the clipboard payload as typed keys. On copy it reads CLIPBOARD text in
the Shell and sends `RecordClipboard` to the daemon.

```
Copy in another app
        ↓
Shell owner-changed → St.Clipboard.get_text → RecordClipboard → daemon history

GNOME Shell shortcut or `clipl toggle`
        ↓
daemon sends PrepareInsert (extension snapshots the focused app)
        ↓
daemon ToggleDesktop → picker shows
        ↓
pick → desktop writes CLIPBOARD, hides, InsertIntoApp
        ↓
extension restores that app and sends Ctrl+V
```

On GNOME Wayland, `clipl-daemon` **must not** call `XGrabKey`. The Shell
owns Super+Alt+V (default) through GSettings. Super+V is left for GNOME's
notification list.

## Requirements

- GNOME Shell **46–50** listed in `metadata.json` so the extension can load on
  current Ubuntu GNOME. The layout is ESM (GNOME 45+). **No Shell version was
  runtime-tested** for Super+Alt+V or insert (static review only).
- `clipl-daemon` running in the same user session
- `clipl-desktop` running (hidden is fine) so the daemon has a subscriber
- This extension enabled so insert can restore the previous window

Re-copy the extension into
`~/.local/share/gnome-shell/extensions/clipl@io.clipl` and log out/in on
Wayland after updating.

## Install (user session)

From source:

```bash
EXT="$HOME/.local/share/gnome-shell/extensions/clipl@io.clipl"
mkdir -p "$EXT"
cp -a extensions/gnome/. "$EXT/"
glib-compile-schemas "$EXT/schemas"
python3 - <<'PY'
import gi
gi.require_version("Gio", "2.0")
from gi.repository import Gio
s = Gio.Settings.new("org.gnome.shell")
exts = list(s.get_strv("enabled-extensions"))
uuid = "clipl@io.clipl"
if uuid not in exts:
    exts.append(uuid)
    s.set_strv("enabled-extensions", exts)
print("enabled-extensions:", exts)
PY
```

`.deb` / `.rpm` install the same files under
`/usr/share/gnome-shell/extensions/clipl@io.clipl`. You still must **log out
and back in**. The packaged picker then appends the UUID to
`enabled-extensions` (it does not run `gnome-extensions enable` as root).

On Wayland, GNOME Shell **does not see a newly copied extension until you log
out and back in**. `gnome-extensions enable` and `Alt+F2` `r` will fail until
that session restart. Adding the UUID to `enabled-extensions` means it should
turn on after login.

Confirm:

```bash
gnome-extensions info clipl@io.clipl
gsettings get org.gnome.shell.extensions.clipl activate-shortcut
```

Default binding: `['<Super><Alt>v']`.

**Conflict:** Ubuntu GNOME uses Super+V for the notification list, so
ClipLinux does not take that chord. Super+Period is GNOME's emoji panel.
Change the ClipLinux binding if it collides with something else:

```bash
gsettings set org.gnome.shell.extensions.clipl activate-shortcut "['<Super>period']"
```

Until the Shell extension is loaded (Wayland needs a log out/in), a GNOME
**custom keyboard shortcut** can call `clipl toggle`:

```bash
bash packaging/linux/install-gnome-shortcut.sh
```

That binds Super+Alt+V without replacing your other custom shortcuts. Keep
`clipl-daemon` and `clipl-desktop` running. After the extension is enabled,
remove the "ClipLinux" custom shortcut in Settings → Keyboard so both do not
fire on the same key (double-toggle).

X11 sessions do **not** need this extension for the shortcut (`XGrabKey`).
They also do not need it for insert (daemon XTest) or history (XFixes).
On GNOME Wayland this extension is required for insert **and** for copies
to appear in history. The custom shortcut can open the picker before a
session restart, but copies will not be recorded until the extension loads.

## Security

- Local Unix socket only (`0600`)
- Actions are hardcoded `ToggleDesktop` / `SubscribeInsert` / `RecordClipboard`
  / Ctrl+V — no user-controlled command string
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
