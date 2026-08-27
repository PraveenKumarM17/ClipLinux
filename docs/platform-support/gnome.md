# GNOME

ClipLinux treats **GNOME + Wayland** as a different platform from GNOME + X11
and from generic Wayland.

## Clipboard

Mutter does not offer `wlr-data-control` to regular clients. Clipboard
**watch** remains `Unsupported` until a future extension pushes text to the
daemon. Phase 4A does **not** add clipboard bridging.

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
layout. None of them were runtime-tested for Super+V or insert.** This
development host reports GNOME Shell 50 on Wayland; `clipl doctor` correctly
selects `gnome-shell` and does **not** take an X11 grab despite `DISPLAY`
being set.

## Super+V

Default ClipLinux shortcut is Super+V. GNOME may already use Super+V for
notifications. Change one of the bindings if they collide.

X11 GNOME sessions prefer the daemon `XGrabKey` path when
`[activation.x11] enabled = true`. The extension is not required there.
