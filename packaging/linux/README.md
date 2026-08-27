# Linux packages

ClipLinux can be installed from source (see the repository README) or from
Tauri bundles: `.deb`, `.rpm`, and AppImage.

## What a package contains

- `clipl-desktop` (picker)
- `clipl-daemon` (history, privacy, IPC)
- `clipl` (CLI)
- XDG autostart entries so the daemon and picker start on login
- GNOME Shell extension files (`.deb` / `.rpm` install them under
  `/usr/share/gnome-shell/extensions/clipl@io.clipl`)

The picker still never opens SQLite. The daemon remains the source of truth.

AppImage is portable: it does **not** install XDG autostart or the system
GNOME extension. Launching it starts the daemon if needed and copies the
extension into the user data dir.

`.deb` / `.rpm` do **not** run `gnome-extensions enable` as root. The picker,
once started as your user, appends `clipl@io.clipl` to GNOME's enabled
extensions list. Shell still only loads that code after a log out.

## Desktop environment support

| Session | Clipboard history | Insert into the previous app | Shortcut |
| --- | --- | --- | --- |
| GNOME Wayland + extension loaded | Yes (Shell push) | Yes (restore focus + Ctrl+V) | Super+Alt+V |
| X11 (any desktop) | Yes (XFixes) | Yes (XTest Ctrl+V) | Super+V by default |
| Other Wayland (KDE, Sway, Hyprland, …) | Not yet | Copy only; press Ctrl+V | `clipl toggle` / compositor bind |

GNOME on Wayland **must log out and back in** after install. The installer
cannot load a new Shell extension into a live Wayland session.

Empty clipboard history after that restart usually means Shell still has not
loaded `clipl@io.clipl`. Check with `gnome-extensions info clipl@io.clipl`.
Other clipboard extensions (including Clipboard Indicator) can watch the same
copy event; they do not block ClipLinux by occupying a signal.

Packages built on Ubuntu 26.04 need glibc 2.43 and will not install on
24.04. GitHub Actions builds on Ubuntu 24.04 for wider `.deb` compatibility.

## Build locally

```bash
cd apps/desktop
npm install
npm run tauri build
```

Artifacts land in `target/release/bundle/` at the workspace root.

`apps/desktop/scripts/prepare-bundle.sh` (Tauri `beforeBuildCommand`) builds
`clipl-daemon` and `clipl` into `target/release/` so the bundle file map can
copy them. Do not point Tauri at `externalBin` sidecars — that breaks
`tauri dev` until a release binary exists.
