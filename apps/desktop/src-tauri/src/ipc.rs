//! Unix-socket client used by the desktop process. The webview never sees the socket.

use std::path::{Path, PathBuf};

use clipl_core::{paths, Result};
use clipl_protocol::{IpcClient, Request, Response};

/// Talks to `clipl-daemon` over the local protocol.
#[derive(Clone, Debug)]
pub struct DaemonClient {
    socket: PathBuf,
}

impl DaemonClient {
    /// Socket from XDG / `CLIPL_RUNTIME_DIR`.
    pub fn from_env() -> Self {
        Self {
            socket: paths::socket_path(),
        }
    }

    /// Explicit socket (tests).
    pub fn with_socket(socket: impl Into<PathBuf>) -> Self {
        Self {
            socket: socket.into(),
        }
    }

    /// Socket path.
    pub fn socket(&self) -> &Path {
        &self.socket
    }

    /// One request/response on a fresh connection.
    pub fn request(&self, request: Request) -> Result<Response> {
        let mut client = IpcClient::connect_path(&self.socket)?;
        client.request(request)
    }

    /// Whether something accepts connections on the socket.
    pub fn reachable(&self) -> bool {
        IpcClient::connect_path(&self.socket).is_ok()
    }
}

/// Map a connect failure into the disconnected UI copy.
pub fn disconnected_message(err: &clipl_core::Error) -> String {
    let raw = err.to_string();
    if raw.contains("not running") || raw.contains("No such file") {
        "ClipLinux daemon is not running.".into()
    } else {
        format!("Cannot reach clipl-daemon: {raw}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_is_not_reachable() {
        let client = DaemonClient::with_socket("/tmp/clipl-desktop-no-such.sock");
        assert!(!client.reachable());
    }

    #[test]
    fn maps_missing_socket() {
        let err = clipl_core::Error::Io(
            "clipl-daemon is not running (socket /tmp/x): No such file".into(),
        );
        assert_eq!(
            disconnected_message(&err),
            "ClipLinux daemon is not running."
        );
    }
}
