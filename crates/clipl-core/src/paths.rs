//! XDG paths for ClipLinux data, config, and runtime files.

use std::env;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Directory name under XDG bases.
pub const APP_DIR_NAME: &str = "clipl";

/// Resolved directories for one ClipLinux instance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipLinuxPaths {
    /// `$XDG_DATA_HOME/clipl` (history database).
    pub data_dir: PathBuf,
    /// `$XDG_CONFIG_HOME/clipl`.
    pub config_dir: PathBuf,
    /// `$XDG_RUNTIME_DIR/clipl` (Unix socket).
    pub runtime_dir: PathBuf,
}

impl ClipLinuxPaths {
    /// Resolve from the process environment.
    pub fn from_env() -> Self {
        Self {
            data_dir: data_dir(),
            config_dir: config_dir(),
            runtime_dir: runtime_dir(),
        }
    }

    /// Isolated directories under `root` (tests).
    pub fn isolated(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        Self {
            data_dir: root.join("data"),
            config_dir: root.join("config"),
            runtime_dir: root.join("run"),
        }
    }

    /// `config.toml`.
    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    /// Unix-domain socket.
    pub fn socket_file(&self) -> PathBuf {
        self.runtime_dir.join("daemon.sock")
    }

    /// SQLite database.
    pub fn database_file(&self) -> PathBuf {
        self.data_dir.join("history.sqlite3")
    }

    /// Create data/config/runtime dirs with mode `0700`.
    pub fn ensure(&self) -> Result<()> {
        ensure_user_dir(&self.data_dir)?;
        ensure_user_dir(&self.config_dir)?;
        ensure_user_dir(&self.runtime_dir)?;
        Ok(())
    }
}

/// `$CLIPL_DATA_DIR` or `$XDG_DATA_HOME/clipl` or `~/.local/share/clipl`.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = env_path("CLIPL_DATA_DIR") {
        return dir;
    }
    xdg_home("XDG_DATA_HOME", ".local/share").join(APP_DIR_NAME)
}

/// `$CLIPL_CONFIG_DIR` or `$XDG_CONFIG_HOME/clipl` or `~/.config/clipl`.
pub fn config_dir() -> PathBuf {
    if let Some(dir) = env_path("CLIPL_CONFIG_DIR") {
        return dir;
    }
    xdg_home("XDG_CONFIG_HOME", ".config").join(APP_DIR_NAME)
}

/// Config file path. `$CLIPL_CONFIG_PATH` overrides.
pub fn config_path() -> PathBuf {
    if let Some(path) = env_path("CLIPL_CONFIG_PATH") {
        return path;
    }
    config_dir().join("config.toml")
}

/// `$CLIPL_RUNTIME_DIR` or `$XDG_RUNTIME_DIR/clipl` or `/tmp/clipl-<uid>`.
pub fn runtime_dir() -> PathBuf {
    if let Some(dir) = env_path("CLIPL_RUNTIME_DIR") {
        return dir;
    }
    if let Some(dir) = env_path("XDG_RUNTIME_DIR") {
        return dir.join(APP_DIR_NAME);
    }
    PathBuf::from(format!("/tmp/clipl-{}", proc_uid()))
}

/// Unix-domain socket used by the daemon.
pub fn socket_path() -> PathBuf {
    runtime_dir().join("daemon.sock")
}

/// SQLite database path.
pub fn database_path() -> PathBuf {
    data_dir().join("history.sqlite3")
}

/// Create `dir` with mode `0700` if it does not exist.
pub fn ensure_user_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(|err| Error::Io(format!("{}: {err}", dir.display())))?;
    fs::set_permissions(dir, Permissions::from_mode(0o700))
        .map_err(|err| Error::Io(format!("{}: {err}", dir.display())))?;
    Ok(())
}

fn env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key).map(PathBuf::from)
}

fn xdg_home(var: &str, fallback_under_home: &str) -> PathBuf {
    if let Some(dir) = env_path(var) {
        return dir;
    }
    home_dir().join(fallback_under_home)
}

fn home_dir() -> PathBuf {
    env_path("HOME").unwrap_or_else(|| PathBuf::from("/"))
}

fn proc_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("Uid:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|id| id.parse().ok())
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn socket_lives_under_runtime_dir() {
        let sock = socket_path();
        assert!(sock.ends_with("daemon.sock"));
    }

    #[test]
    fn isolated_layout() {
        let paths = ClipLinuxPaths::isolated("/tmp/clipl-test");
        assert!(paths.database_file().ends_with("history.sqlite3"));
        assert!(paths.socket_file().ends_with("daemon.sock"));
    }
}
