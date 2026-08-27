//! Packaged-app helpers: find the sidecar daemon and copy the GNOME extension.
//! Independent of Tauri so workspace tests do not need WebKitGTK.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use clipl_core::paths;
use clipl_protocol::IpcClient;

/// Binary name shipped next to `clipl-desktop` in Linux bundles.
pub const DAEMON_SIDECAR: &str = "clipl-daemon";

/// PATH / package name when the sidecar is not beside the executable.
pub const DAEMON_ON_PATH: &str = "clipl-daemon";

/// GNOME Shell UUID. Must match `extensions/gnome/metadata.json`.
pub const GNOME_EXTENSION_UUID: &str = "clipl@io.clipl";

/// Suggested start command shown in the disconnected UI.
pub fn start_command() -> &'static str {
    if cfg!(debug_assertions) {
        "cargo run -p clipl-daemon"
    } else {
        DAEMON_ON_PATH
    }
}

/// `dir/clipl-daemon` when that file exists (deb/rpm/AppImage `/usr/bin` layout).
pub fn sidecar_next_to_exe(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    let candidate = dir.join(DAEMON_SIDECAR);
    candidate.is_file().then_some(candidate)
}

/// True when this process is running from an AppImage (FUSE mount or env).
pub fn running_from_appimage(exe: &Path) -> bool {
    std::env::var_os("APPIMAGE").is_some()
        || std::env::var_os("APPDIR").is_some()
        || exe_on_appimage_mount(exe)
}

/// True when `exe` lives on an AppImage FUSE mount (`/tmp/.mount_*`).
pub fn exe_on_appimage_mount(exe: &Path) -> bool {
    exe.components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with(".mount_"))
}

/// `$XDG_DATA_HOME/clipl/bin/clipl-daemon` — survives AppImage unmount.
pub fn persistent_daemon_bin() -> PathBuf {
    persistent_daemon_bin_in(&paths::data_dir())
}

/// `data_dir/bin/clipl-daemon`.
pub fn persistent_daemon_bin_in(data_dir: &Path) -> PathBuf {
    data_dir.join("bin").join(DAEMON_SIDECAR)
}

/// Copy `bundled` to `dest` when missing or stale. `dest` is left executable.
pub fn sync_persistent_daemon_to(bundled: &Path, dest: &Path) -> io::Result<PathBuf> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
        let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
    }
    if !daemon_copy_needed(bundled, dest)? {
        return Ok(dest.to_path_buf());
    }
    let tmp = dest.with_extension("new");
    fs::copy(bundled, &tmp)?;
    fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))?;
    fs::rename(&tmp, dest)?;
    Ok(dest.to_path_buf())
}

fn daemon_copy_needed(bundled: &Path, dest: &Path) -> io::Result<bool> {
    let Ok(dest_meta) = fs::metadata(dest) else {
        return Ok(true);
    };
    let src_meta = fs::metadata(bundled)?;
    if src_meta.len() != dest_meta.len() {
        return Ok(true);
    }
    match (src_meta.modified(), dest_meta.modified()) {
        (Ok(src), Ok(dst)) => Ok(src > dst),
        _ => Ok(false),
    }
}

/// Sidecar on disk, AppImage copy under the user data dir, or PATH.
pub fn daemon_binary_for_exe(exe: &Path) -> PathBuf {
    match sidecar_next_to_exe(exe) {
        Some(sidecar) if running_from_appimage(exe) => {
            let dest = persistent_daemon_bin();
            match sync_persistent_daemon_to(&sidecar, &dest) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!(
                        "clipl-desktop: could not extract clipl-daemon to {}: {err}",
                        dest.display()
                    );
                    sidecar
                }
            }
        }
        Some(sidecar) => sidecar,
        None => PathBuf::from(DAEMON_ON_PATH),
    }
}

/// Spawn `clipl-daemon` detached from this process's stdio and process group.
pub fn spawn_daemon(bin: &Path) -> io::Result<()> {
    let mut cmd = Command::new(bin);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.process_group(0);
    cmd.spawn()?;
    Ok(())
}

/// Wait until the daemon socket accepts a connection.
pub fn wait_for_daemon(socket: &Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if IpcClient::connect_path(socket).is_ok() {
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// Start the sidecar or PATH daemon when nothing is listening.
///
/// No-op when the socket already works. Does not spawn when `bin` is missing.
pub fn ensure_daemon_running(exe: &Path) -> bool {
    let socket = paths::socket_path();
    if IpcClient::connect_path(&socket).is_ok() {
        return true;
    }
    let bin = daemon_binary_for_exe(exe);
    if let Err(err) = spawn_daemon(&bin) {
        eprintln!(
            "clipl-desktop: could not start clipl-daemon from {}: {err}",
            bin.display()
        );
        return false;
    }
    wait_for_daemon(&socket, Duration::from_secs(3))
}

/// User and system install locations for the GNOME Shell extension.
pub fn gnome_extension_destinations() -> (PathBuf, PathBuf) {
    let user = user_data_home()
        .join("gnome-shell/extensions")
        .join(GNOME_EXTENSION_UUID);
    let system = PathBuf::from("/usr/share/gnome-shell/extensions").join(GNOME_EXTENSION_UUID);
    (user, system)
}

fn user_data_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share")
}

/// True when metadata.json exists in the user or system extension dir.
pub fn gnome_extension_on_disk() -> bool {
    let (user, system) = gnome_extension_destinations();
    user.join("metadata.json").is_file() || system.join("metadata.json").is_file()
}

/// Copy bundled extension files into the user data dir. Compiles schemas when
/// `glib-compile-schemas` is on PATH. Idempotent if already installed.
pub fn install_user_gnome_extension(src: &Path) -> io::Result<PathBuf> {
    let (user, _system) = gnome_extension_destinations();
    install_gnome_extension_into(src, &user)
}

/// Merge `uuid` into a `gsettings get org.gnome.shell enabled-extensions` value.
/// Returns `None` when the UUID is already listed.
pub fn gnome_enabled_extensions_with_uuid(raw: &str, uuid: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.contains(uuid) {
        return None;
    }
    if trimmed == "[]" || trimmed == "@as []" {
        return Some(format!("['{uuid}']"));
    }
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(format!("['{uuid}']"));
    }
    Some(format!("[{inner}, '{uuid}']"))
}

/// Best-effort: append the ClipLinux UUID to the user GNOME enabled-extensions
/// list. Shell still will not load new code until the user logs out.
pub fn try_enable_user_gnome_extension() {
    let output = Command::new("gsettings")
        .args(["get", "org.gnome.shell", "enabled-extensions"])
        .output();
    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let current = String::from_utf8_lossy(&output.stdout);
    let Some(next) = gnome_enabled_extensions_with_uuid(&current, GNOME_EXTENSION_UUID) else {
        return;
    };
    let _ = Command::new("gsettings")
        .args(["set", "org.gnome.shell", "enabled-extensions", &next])
        .status();
}

/// Copy `src` into `dest` unless `dest/metadata.json` already exists.
pub fn install_gnome_extension_into(src: &Path, dest: &Path) -> io::Result<PathBuf> {
    if dest.join("metadata.json").is_file() {
        return Ok(dest.to_path_buf());
    }
    copy_dir_all(src, dest)?;
    let schemas = dest.join("schemas");
    if schemas.is_dir() {
        let _ = Command::new("glib-compile-schemas").arg(&schemas).status();
    }
    Ok(dest.to_path_buf())
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if ty.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_requires_a_real_file() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("clipl-desktop");
        fs::write(&exe, []).unwrap();
        assert!(sidecar_next_to_exe(&exe).is_none());
        fs::write(tmp.path().join(DAEMON_SIDECAR), []).unwrap();
        let found = sidecar_next_to_exe(&exe).unwrap();
        assert_eq!(found.file_name().unwrap(), DAEMON_SIDECAR);
    }

    #[test]
    fn copies_extension_once() {
        let src = tempfile::tempdir().unwrap();
        fs::write(
            src.path().join("metadata.json"),
            "{\"uuid\":\"clipl@io.clipl\"}",
        )
        .unwrap();
        let dest_root = tempfile::tempdir().unwrap();
        let dest = dest_root.path().join(GNOME_EXTENSION_UUID);
        let first = install_gnome_extension_into(src.path(), &dest).unwrap();
        assert!(first.join("metadata.json").is_file());
        fs::write(src.path().join("metadata.json"), "changed").unwrap();
        let second = install_gnome_extension_into(src.path(), &dest).unwrap();
        let body = fs::read_to_string(second.join("metadata.json")).unwrap();
        assert_eq!(body, "{\"uuid\":\"clipl@io.clipl\"}");
    }

    #[test]
    fn start_command_is_never_empty() {
        assert!(!start_command().is_empty());
    }

    #[test]
    fn wait_times_out_when_socket_missing() {
        assert!(!wait_for_daemon(
            Path::new("/tmp/clipl-no-such-launch.sock"),
            Duration::from_millis(30),
        ));
    }

    #[test]
    fn spawn_missing_binary_fails() {
        assert!(spawn_daemon(Path::new("/no/such/clipl-daemon-binary")).is_err());
    }

    #[test]
    fn gnome_destinations_use_the_uuid() {
        let (user, system) = gnome_extension_destinations();
        assert_eq!(
            user.file_name().unwrap().to_str().unwrap(),
            GNOME_EXTENSION_UUID
        );
        assert_eq!(
            system.file_name().unwrap().to_str().unwrap(),
            GNOME_EXTENSION_UUID
        );
        let _ = gnome_extension_on_disk();
    }

    #[test]
    fn gsettings_list_appends_uuid_once() {
        assert_eq!(
            gnome_enabled_extensions_with_uuid("@as []", GNOME_EXTENSION_UUID).unwrap(),
            format!("['{GNOME_EXTENSION_UUID}']")
        );
        assert_eq!(
            gnome_enabled_extensions_with_uuid("['other@ext']", GNOME_EXTENSION_UUID).unwrap(),
            format!("['other@ext', '{GNOME_EXTENSION_UUID}']")
        );
        assert!(gnome_enabled_extensions_with_uuid(
            &format!("['{GNOME_EXTENSION_UUID}']"),
            GNOME_EXTENSION_UUID
        )
        .is_none());
    }

    #[test]
    fn detects_appimage_mount_in_exe_path() {
        let mount = Path::new("/tmp/.mount_ClipLinABC/usr/bin/clipl-desktop");
        assert!(exe_on_appimage_mount(mount));
        let deb = Path::new("/usr/bin/clipl-desktop");
        assert!(!exe_on_appimage_mount(deb));
    }

    #[test]
    fn copies_daemon_into_persistent_bin() {
        let tmp = tempfile::tempdir().unwrap();
        let bundled = tmp.path().join("bundled");
        fs::write(&bundled, b"daemon-v1").unwrap();
        let dest = persistent_daemon_bin_in(&tmp.path().join("data"));
        let first = sync_persistent_daemon_to(&bundled, &dest).unwrap();
        assert_eq!(first, dest);
        assert_eq!(fs::read(&dest).unwrap(), b"daemon-v1");
        let mode = fs::metadata(&dest).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111);
        fs::write(&bundled, b"daemon-v2-longer").unwrap();
        sync_persistent_daemon_to(&bundled, &dest).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"daemon-v2-longer");
    }

    #[test]
    fn deb_layout_uses_sidecar_not_data_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("clipl-desktop");
        fs::write(&exe, []).unwrap();
        fs::write(tmp.path().join(DAEMON_SIDECAR), b"from-deb").unwrap();
        let bin = daemon_binary_for_exe(&exe);
        assert_eq!(bin, tmp.path().join(DAEMON_SIDECAR));
    }
}
