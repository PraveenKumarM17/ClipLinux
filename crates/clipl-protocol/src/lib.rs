//! IPC protocol between ClipLinux desktop, daemon, and CLI.

#![forbid(unsafe_code)]

mod transport;

use clipl_core::{
    ClipboardItem, ClipboardItemId, DesktopEnvironment, PlatformCapabilities, PrivacyRule,
    SessionType, Snippet, SnippetId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use transport::{
    cleanup_stale_socket, read_frame, set_socket_mode, write_frame, IpcClient, PROTOCOL_VERSION,
};

/// Envelope wrapping a request, response, or event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Correlation id shared by a request and its response.
    pub id: Uuid,
    /// Message body.
    pub payload: Message,
}

impl Envelope {
    /// Wrap a message with a fresh correlation id.
    pub fn new(payload: Message) -> Self {
        Self {
            id: Uuid::new_v4(),
            payload,
        }
    }

    /// Serialize to JSON bytes.
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, clipl_core::Error> {
        serde_json::to_vec(self).map_err(|err| clipl_core::Error::Protocol(err.to_string()))
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, clipl_core::Error> {
        serde_json::from_slice(bytes).map_err(|err| clipl_core::Error::Protocol(err.to_string()))
    }
}

/// Top-level protocol message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Message {
    /// Client → daemon.
    Request(Request),
    /// Daemon → client.
    Response(Response),
    /// Unsolicited daemon → client.
    Event(Event),
}

/// Requests the UI and CLI may send.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Request {
    /// Liveness check.
    Ping,
    /// Daemon status (no clipboard payloads).
    GetStatus,
    /// Capability matrix for the current session.
    GetCapabilities,
    /// Recent clipboard history.
    GetHistory {
        /// Maximum items to return.
        limit: u32,
    },
    /// Substring search over stored text items.
    SearchHistory {
        /// Search needle.
        query: String,
        /// Maximum items to return.
        limit: u32,
    },
    /// Delete one history item.
    DeleteItem {
        /// History item to delete.
        item_id: ClipboardItemId,
    },
    /// Clear unpinned history.
    ClearHistory,
    /// Pin a history item.
    PinItem {
        /// History item to pin.
        item_id: ClipboardItemId,
    },
    /// Unpin a history item.
    UnpinItem {
        /// History item to unpin.
        item_id: ClipboardItemId,
    },
    /// Copy an item back to the system clipboard without recording a new row.
    CopyItem {
        /// History item whose payload should be copied.
        item_id: ClipboardItemId,
    },
    /// Ask the daemon to paste an item (not implemented in this phase).
    Paste {
        /// History item to paste.
        item_id: ClipboardItemId,
    },
    /// List snippets.
    ListSnippets,
    /// Fetch privacy rules.
    ListPrivacyRules,
}

/// Responses matching [`Request`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Response {
    /// Successful ping.
    Pong,
    /// Daemon status snapshot.
    Status(DaemonStatus),
    /// Capability snapshot.
    Capabilities(PlatformCapabilities),
    /// History page.
    History(Vec<ClipboardItem>),
    /// Delete result.
    Deleted {
        /// Whether a row was present.
        existed: bool,
    },
    /// Clear result.
    Cleared {
        /// Rows removed.
        count: u64,
    },
    /// Pin/unpin result.
    Pinned {
        /// History item.
        item_id: ClipboardItemId,
        /// Resulting pin state.
        pinned: bool,
    },
    /// Payload the client should write to the OS clipboard.
    Copied {
        /// Item that was copied.
        item_id: ClipboardItemId,
        /// Text body. Never populated for hidden/sensitive items.
        text: String,
    },
    /// Paste accepted by the daemon (not yet executed).
    PasteAccepted,
    /// Snippet list.
    Snippets(Vec<Snippet>),
    /// Privacy rule list.
    PrivacyRules(Vec<PrivacyRule>),
    /// Request failed.
    Error {
        /// Error message. Must not contain clipboard secrets.
        message: String,
    },
}

/// Daemon status. Never includes clipboard payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Daemon crate version.
    pub version: String,
    /// IPC protocol version.
    pub protocol_version: u32,
    /// Session type.
    pub session: SessionType,
    /// Desktop environment.
    pub desktop: DesktopEnvironment,
    /// Selected clipboard backend id.
    pub backend: String,
    /// Monitoring availability.
    pub monitoring: MonitoringStatus,
    /// Why monitoring is partial/unsupported, if applicable.
    pub monitoring_reason: String,
    /// SQLite path.
    pub database: String,
    /// Unix socket path.
    pub socket_path: String,
    /// Privacy engine enabled.
    pub privacy_enabled: bool,
    /// History persistence enabled.
    pub history_enabled: bool,
    /// Configured unpinned history cap.
    pub history_limit: u32,
}

/// Clipboard watch availability as reported to clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringStatus {
    /// Native watch is running.
    Supported,
    /// Backend exists but watch is degraded (not used for silent polling).
    Partial,
    /// No watch; history is still queryable.
    Unsupported,
}

/// Events the daemon may emit later.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    /// Clipboard contents changed.
    ClipboardChanged {
        /// New history item.
        item: Box<ClipboardItem>,
    },
    /// Session capabilities changed.
    CapabilitiesChanged {
        /// Updated matrix.
        capabilities: PlatformCapabilities,
    },
    /// A snippet was created or updated.
    SnippetUpserted {
        /// Snippet identity.
        id: SnippetId,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_json_round_trip() {
        let env = Envelope::new(Message::Request(Request::Ping));
        let bytes = env.to_json_bytes().expect("ser");
        let back = Envelope::from_json_bytes(&bytes).expect("de");
        assert!(matches!(back.payload, Message::Request(Request::Ping)));
    }

    #[test]
    fn status_round_trip() {
        let status = DaemonStatus {
            version: "0.1.0".into(),
            protocol_version: PROTOCOL_VERSION,
            session: SessionType::X11,
            desktop: DesktopEnvironment::Unknown,
            backend: "x11".into(),
            monitoring: MonitoringStatus::Supported,
            monitoring_reason: String::new(),
            database: "/tmp/x.sqlite3".into(),
            socket_path: "/tmp/clipl.sock".into(),
            privacy_enabled: true,
            history_enabled: true,
            history_limit: 500,
        };
        let env = Envelope::new(Message::Response(Response::Status(status.clone())));
        let back = Envelope::from_json_bytes(&env.to_json_bytes().unwrap()).unwrap();
        assert!(matches!(
            back.payload,
            Message::Response(Response::Status(_))
        ));
    }
}
