//! DTOs sent to the Svelte UI. Full clipboard payloads never enter the webview.

use clipl_core::{ClipboardItem, ClipboardSource};
use clipl_protocol::{DaemonStatus, MonitoringStatus};
use serde::{Deserialize, Serialize};

/// Compact history row for the palette list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryRow {
    /// History id (UUID string).
    pub id: String,
    /// Privacy-conscious preview. Empty when [`Self::hidden`].
    pub preview: String,
    /// Content variant name (`text`, `html`, …).
    pub content_type: String,
    /// Created-at, milliseconds since epoch.
    pub created_at: i64,
    /// Pinned items survive eviction and sort first.
    pub pinned: bool,
    /// True when the daemon redacted the payload.
    pub hidden: bool,
    /// Origin label (`local`, `clipl`, …).
    pub source: String,
}

/// Result of writing the clipboard and attempting restore-focus + Ctrl+V.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InsertOutcome {
    /// True when a backend sent Ctrl+V into the previously focused app.
    pub inserted: bool,
    /// Safe-to-show explanation when not inserted. Empty on success.
    pub reason: String,
}

impl HistoryRow {
    /// Map a protocol item (already sanitized by the daemon) into a UI row.
    pub fn from_item(item: &ClipboardItem) -> Self {
        let hidden = !item.sensitive.is_empty();
        Self {
            id: item.id.to_string(),
            preview: if hidden {
                format!("[{} hidden]", item.content.type_name())
            } else {
                collapse_preview(&item.content.preview(160))
            },
            content_type: item.content.type_name().to_string(),
            created_at: item.created_at.as_millis(),
            pinned: item.pinned,
            hidden,
            source: source_label(&item.source).to_string(),
        }
    }
}

fn collapse_preview(text: &str) -> String {
    let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
}

fn source_label(source: &ClipboardSource) -> &'static str {
    match source {
        ClipboardSource::LocalSession => "local",
        ClipboardSource::ClipLinux => "clipl",
        ClipboardSource::Import => "import",
        ClipboardSource::Unknown => "unknown",
        _ => "unknown",
    }
}

/// Connection snapshot for the status chip.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionView {
    /// Unix socket connected.
    Connected {
        /// Clipboard watch availability.
        monitoring: MonitoringStatus,
        /// Safe-to-show reason when watch is not native.
        reason: String,
        /// Daemon version string.
        version: String,
    },
    /// Daemon is not listening.
    Disconnected {
        /// User-facing explanation.
        message: String,
        /// Suggested start command.
        start_command: String,
    },
    /// Connected but the protocol reply was unusable.
    Error {
        /// User-facing explanation.
        message: String,
    },
}

impl ConnectionView {
    /// Build from a successful status reply.
    pub fn from_status(status: &DaemonStatus) -> Self {
        Self::Connected {
            monitoring: status.monitoring,
            reason: status.monitoring_reason.clone(),
            version: status.version.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::{ClipboardItem, SensitiveContentType};

    #[test]
    fn hides_labelled_secrets() {
        let mut item = ClipboardItem::text("hunter2");
        item.sensitive.push(SensitiveContentType::Password);
        let row = HistoryRow::from_item(&item);
        assert!(row.hidden);
        assert!(!row.preview.contains("hunter2"));
    }

    #[test]
    fn collapses_whitespace() {
        let item = ClipboardItem::text("hello\n\n  world");
        let row = HistoryRow::from_item(&item);
        assert_eq!(row.preview, "hello world");
    }
}
