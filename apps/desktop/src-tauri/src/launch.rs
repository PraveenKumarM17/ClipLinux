//! Packaged-app helpers: find the sidecar daemon and copy the GNOME extension.
//! Independent of Tauri so workspace tests do not need WebKitGTK.

use std::fs;
use std::io;
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

/// Spawn `clipl-daemon` detached from this process's stdio.
pub fn spawn_daemon(bin: &Path) -> io::Result<()> {
    Command::new(bin)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
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
    let bin = sidecar_next_to_exe(exe).unwrap_or_else(|| PathBuf::from(DAEMON_ON_PATH));
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
}
