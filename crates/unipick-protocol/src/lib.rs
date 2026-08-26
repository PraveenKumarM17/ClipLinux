//! IPC protocol between UniPick desktop, daemon, and CLI.
//!
//! Transport (Unix socket, length prefixes) is intentionally not implemented
//! in this foundation. Only the message vocabulary is stable enough to share.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use unipick_core::{
    ClipboardItem, ClipboardItemId, PlatformCapabilities, PrivacyRule, Snippet, SnippetId,
};
use uuid::Uuid;

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
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, unipick_core::Error> {
        serde_json::to_vec(self).map_err(|err| unipick_core::Error::Protocol(err.to_string()))
    }

    /// Deserialize from JSON bytes.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, unipick_core::Error> {
        serde_json::from_slice(bytes).map_err(|err| unipick_core::Error::Protocol(err.to_string()))
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

/// Requests the UI and CLI may send once the daemon is listening.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Request {
    /// Liveness check.
    Ping,
    /// Capability matrix for the current session.
    GetCapabilities,
    /// Recent clipboard history.
    GetHistory {
        /// Maximum items to return.
        limit: u32,
    },
    /// Ask the daemon to paste an item (implementation comes later).
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
    /// Capability snapshot.
    Capabilities(PlatformCapabilities),
    /// History page.
    History(Vec<ClipboardItem>),
    /// Paste accepted by the daemon (not yet executed in the foundation).
    PasteAccepted,
    /// Snippet list.
    Snippets(Vec<Snippet>),
    /// Privacy rule list.
    PrivacyRules(Vec<PrivacyRule>),
    /// Request failed.
    Error {
        /// Error message.
        message: String,
    },
}

/// Events the daemon may emit later.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Event {
    /// Clipboard contents changed. Not emitted in the foundation.
    ClipboardChanged {
        /// New history item.
        item: ClipboardItem,
    },
    /// Session capabilities changed (e.g. after login or DE switch).
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
}
