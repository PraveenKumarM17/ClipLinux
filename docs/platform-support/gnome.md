# GNOME

ClipLinux treats **GNOME + Wayland** as a different platform from GNOME + X11
and from generic Wayland.

## Clipboard

Mutter does not offer `wlr-data-control` to regular clients. The Shell
extension watches CLIPBOARD (`Meta.Selection` `owner-changed`) and sends
`RecordClipboard` to the daemon. Privacy, dedup, and SQLite still run in
the daemon. Generic polling of `wl-paste` is not used.

## Activation (Phase 4A)

On GNOME Wayland, ClipLinux **must not** grab global keys from the daemon or
Tauri process.

The supported path is `extensions/gnome`:

1. GNOME Shell registers the shortcut (`Main.wm.addKeybinding` + GSettings)
2. The extension sends `ToggleDesktop` to `$XDG_RUNTIME_DIR/clipl/daemon.sock`
3. `clipl-daemon` notifies the subscribed desktop
4. The picker window is shown or hidden

After a pick, the same extension restores the window that had focus when
the shortcut fired and sends Ctrl+V (clipboard already written by the
desktop). See [extensions/gnome/README.md](../../extensions/gnome/README.md).

**GNOME Shell versions listed in `metadata.json` (46–50) use the ESM
layout. None of them were runtime-tested for Super+Alt+V or insert.** This
development host reports GNOME Shell 50 on Wayland; `clipl doctor` correctly
selects `gnome-shell` and does **not** take an X11 grab despite `DISPLAY`
being set.

## Super+Alt+V

The GNOME extension default is Super+Alt+V. Ubuntu GNOME already binds
Super+V to the notification list (`toggle-message-tray`). Super+Period is
the emoji panel.

A custom GNOME shortcut that runs `clipl toggle` is also supported (see
[extensions/gnome/README.md](../../extensions/gnome/README.md)) and works
before a session restart. It does **not** snapshot the focused window;
the Shell extension still does that for insert. Copies also do not appear
in history until the extension is loaded.

X11 GNOME sessions prefer the daemon `XGrabKey` path when
`[activation.x11] enabled = true`. The extension is not required there
(XFixes watches CLIPBOARD).
