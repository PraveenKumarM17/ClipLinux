//! IPC protocol between ClipLinux desktop, daemon, and CLI.

#![forbid(unsafe_code)]

mod activation;
mod picker;
mod transport;

use clipl_core::{
    ActivationRequest, ClipboardItem, ClipboardItemId, DesktopEnvironment, PlatformCapabilities,
    PrivacyRule, SessionType, Snippet, SnippetId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use activation::ActivationReport;
pub use picker::{PickerItem, PickerKind, SkinTonePref};
pub use transport::{
    cleanup_stale_socket, read_frame, set_socket_mode, write_frame, ActivationSubscriber,
    IpcClient, PROTOCOL_VERSION,
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
    /// GNOME Shell extension (or tests) pushes captured CLIPBOARD text.
    RecordClipboard {
        /// Plain text. Must not be used to log secrets; privacy still runs in the daemon.
        text: String,
    },
    /// Ask the daemon to paste an item (legacy; use InsertIntoApp after a clipboard write).
    Paste {
        /// History item to paste.
        item_id: ClipboardItemId,
    },
    /// List snippets.
    ListSnippets,
    /// Fetch privacy rules.
    ListPrivacyRules,
    /// Ranked emoji search. Empty query returns no rows.
    SearchEmoji {
        /// Search needle.
        query: String,
        /// Maximum items to return.
        limit: u32,
    },
    /// Emoji in a Unicode group, or `Frequently Used`.
    ListEmojiCategory {
        /// Group name.
        category: String,
        /// Maximum items to return.
        limit: u32,
    },
    /// Usage-ranked emoji.
    GetFrequentlyUsedEmoji {
        /// Maximum items to return.
        limit: u32,
    },
    /// Record that an emoji was copied.
    RecordEmojiUsage {
        /// Catalog base glyph.
        glyph: String,
    },
    /// Favorite emoji glyphs.
    GetFavoriteEmoji,
    /// Mark an emoji as favorite.
    FavoriteEmoji {
        /// Catalog base glyph.
        glyph: String,
    },
    /// Remove an emoji favorite.
    UnfavoriteEmoji {
        /// Catalog base glyph.
        glyph: String,
    },
    /// Stored default skin tone.
    GetSkinTonePref,
    /// Persist default skin tone.
    SetSkinTonePref {
        /// Preference name.
        tone: String,
    },
    /// Ranked symbol search.
    SearchSymbols {
        /// Search needle.
        query: String,
        /// Maximum items to return.
        limit: u32,
    },
    /// Symbols in a curated group.
    ListSymbolCategory {
        /// Group name.
        category: String,
    },
    /// Ranked kaomoji search.
    SearchKaomoji {
        /// Search needle.
        query: String,
        /// Maximum items to return.
        limit: u32,
    },
    /// Kaomoji in a curated group.
    ListKaomojiCategory {
        /// Group name.
        category: String,
    },
    /// Favorite symbols or kaomoji.
    GetFavoritePicker {
        /// Catalog kind.
        kind: PickerKind,
    },
    /// Mark a symbol or kaomoji as favorite.
    FavoritePicker {
        /// Catalog kind.
        kind: PickerKind,
        /// Glyph.
        glyph: String,
    },
    /// Remove a symbol or kaomoji favorite.
    UnfavoritePicker {
        /// Catalog kind.
        kind: PickerKind,
        /// Glyph.
        glyph: String,
    },
    /// Show the desktop picker if a subscriber is connected.
    ShowDesktop,
    /// Hide the desktop picker if a subscriber is connected.
    HideDesktop,
    /// Toggle the desktop picker if a subscriber is connected.
    ToggleDesktop,
    /// Desktop process keeps this connection open to receive activation events.
    SubscribeDesktop,
    /// GNOME Shell extension keeps this connection open to receive insert events.
    SubscribeInsert,
    /// Restore the previously focused app and send Ctrl+V (clipboard already written).
    InsertIntoApp,
    /// Activation capability/status (no keystroke data).
    GetActivationStatus,
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
    /// Whether captured clipboard text was stored (false if empty, skipped, or excluded).
    ClipboardRecorded {
        /// True when a history row was stored or reused.
        stored: bool,
    },
    /// Paste accepted by the daemon (not yet executed).
    PasteAccepted,
    /// Snippet list.
    Snippets(Vec<Snippet>),
    /// Privacy rule list.
    PrivacyRules(Vec<PrivacyRule>),
    /// Compact picker rows (emoji, symbols, or kaomoji).
    PickerList(Vec<PickerItem>),
    /// Favorite mutation result.
    PickerFavorite {
        /// Glyph.
        glyph: String,
        /// Resulting favorite state.
        favorite: bool,
    },
    /// Usage recorded.
    PickerUsage {
        /// Glyph.
        glyph: String,
        /// New count.
        count: u64,
    },
    /// Skin-tone preference.
    SkinTone(SkinTonePref),
    /// Desktop was subscribed for activation events.
    DesktopSubscribed {
        /// True when a previous subscriber was replaced.
        replaced: bool,
    },
    /// GNOME extension (or other insert helper) is subscribed.
    InsertSubscribed {
        /// True when a previous subscriber was replaced.
        replaced: bool,
    },
    /// Whether restore-focus + Ctrl+V was delivered.
    Inserted {
        /// True when a backend actually sent the paste chord.
        delivered: bool,
        /// Safe-to-show explanation when not delivered.
        reason: String,
    },
    /// Show/hide/toggle was accepted. `delivered` is false when no desktop is connected.
    DesktopRouted {
        /// Whether a desktop subscriber received the event.
        delivered: bool,
    },
    /// Activation snapshot.
    Activation(ActivationReport),
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
    /// Activation path for this session.
    #[serde(default)]
    pub activation: ActivationReport,
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
    /// Ask the subscribed desktop process to show, hide, or toggle the picker.
    ActivatePicker {
        /// Show, hide, or toggle.
        action: ActivationRequest,
    },
    /// Ask the GNOME extension to remember the currently focused window.
    PrepareInsert,
    /// Ask the GNOME extension to restore the previous window and send Ctrl+V.
    InsertIntoApp,
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
    fn toggle_desktop_json_tag() {
        let env = Envelope::new(Message::Request(Request::ToggleDesktop));
        let json = String::from_utf8(env.to_json_bytes().unwrap()).unwrap();
        assert!(json.contains("ToggleDesktop"));
        assert!(!json.contains("shell"));
        assert!(!json.contains("InsertIntoApp"));
        let insert = Envelope::new(Message::Request(Request::InsertIntoApp));
        let insert_json = String::from_utf8(insert.to_json_bytes().unwrap()).unwrap();
        assert!(insert_json.contains("InsertIntoApp"));
        let prepare = Envelope::new(Message::Event(Event::PrepareInsert));
        let prepare_json = String::from_utf8(prepare.to_json_bytes().unwrap()).unwrap();
        assert!(prepare_json.contains("PrepareInsert"));
        let record = Envelope::new(Message::Request(Request::RecordClipboard {
            text: "copied".into(),
        }));
        let record_json = String::from_utf8(record.to_json_bytes().unwrap()).unwrap();
        assert!(record_json.contains("RecordClipboard"));
        assert!(record_json.contains("copied"));
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
            activation: ActivationReport::default(),
        };
        let env = Envelope::new(Message::Response(Response::Status(status.clone())));
        let back = Envelope::from_json_bytes(&env.to_json_bytes().unwrap()).unwrap();
        assert!(matches!(
            back.payload,
            Message::Response(Response::Status(_))
        ));
    }
}
