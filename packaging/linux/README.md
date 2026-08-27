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

**glibc, measured on the Ubuntu 26.04 build host**

| Artifact | Uses host libc? | Highest `GLIBC_*` in *our* binaries | 24.04 (glibc 2.39) |
| --- | --- | --- | --- |
| `.deb` / `.rpm` | Yes (system WebKit/GTK) | `clipl-desktop` **2.39**; daemon/CLI **2.34** | `apt install` of a 26.04-built `.deb` succeeded on `ubuntu:24.04`; daemon started |
| AppImage | Yes — **does not bundle `libc.so.6`** | same as above for `usr/bin/*` | **Not a 24.04 candidate.** Bundled `libglib-2.0` / `libwebkit2gtk-4.1` from 26.04 require **GLIBC_2.43** (`version 'GLIBC_2.43' not found` on 24.04) |

CI (`runs-on: ubuntu-24.04`) produces AppImages whose bundled GTK stack is linked against glibc 2.39. Use those AppImages as release candidates, not `target/release/bundle/` from this host.

AppImage does **not** install system XDG autostart. On first launch it copies
`clipl-daemon` to `$XDG_DATA_HOME/clipl/bin/clipl-daemon` and starts that
copy, so capture can outlive the FUSE mount.

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

Packages built on Ubuntu 26.04: the `.deb`/`.rpm` binaries need at most
glibc 2.39 (24.04-compatible). The **local AppImage** bundles 26.04 WebKit
and needs glibc 2.43. Treat CI `ubuntu-24.04` AppImages as the AppImage
release candidates.

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
