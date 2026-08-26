//! Typed desktop operations. Independent of Tauri so unit tests need no WebView.

use clipl_core::{ClipboardItemId, Result};
use clipl_protocol::{Request, Response};

use crate::clipboard::ClipboardWriter;
use crate::dto::{ConnectionView, HistoryRow};
use crate::ipc::{disconnected_message, DaemonClient, START_COMMAND};

const HISTORY_LIMIT: u32 = 200;

/// Fetch status and map it for the UI.
pub fn get_daemon_status(client: &DaemonClient) -> ConnectionView {
    match client.request(Request::GetStatus) {
        Ok(Response::Status(status)) => {
            if status.protocol_version != clipl_protocol::PROTOCOL_VERSION {
                return ConnectionView::Error {
                    message: format!(
                        "incompatible protocol (daemon {}, desktop {})",
                        status.protocol_version,
                        clipl_protocol::PROTOCOL_VERSION
                    ),
                };
            }
            ConnectionView::from_status(&status)
        }
        Ok(Response::Error { message }) => ConnectionView::Error { message },
        Ok(other) => ConnectionView::Error {
            message: format!("unexpected status response: {other:?}"),
        },
        Err(err) => ConnectionView::Disconnected {
            message: disconnected_message(&err),
            start_command: START_COMMAND.into(),
        },
    }
}

/// Recent history as UI rows.
pub fn get_history(client: &DaemonClient) -> Result<Vec<HistoryRow>> {
    map_history(client.request(Request::GetHistory {
        limit: HISTORY_LIMIT,
    }))
}

/// Daemon search. Empty query is recent history.
pub fn search_history(client: &DaemonClient, query: &str) -> Result<Vec<HistoryRow>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return get_history(client);
    }
    map_history(client.request(Request::SearchHistory {
        query: trimmed.to_string(),
        limit: HISTORY_LIMIT,
    }))
}

fn map_history(response: Result<Response>) -> Result<Vec<HistoryRow>> {
    match response? {
        Response::History(items) => Ok(items.iter().map(HistoryRow::from_item).collect()),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected history response: {other:?}"
        ))),
    }
}

/// Delete an unpinned item.
pub fn delete_history_item(client: &DaemonClient, id: &str) -> Result<bool> {
    let item_id: ClipboardItemId = id.parse()?;
    match client.request(Request::DeleteItem { item_id })? {
        Response::Deleted { existed } => Ok(existed),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected delete response: {other:?}"
        ))),
    }
}

/// Clear unpinned history.
pub fn clear_history(client: &DaemonClient) -> Result<u64> {
    match client.request(Request::ClearHistory)? {
        Response::Cleared { count } => Ok(count),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected clear response: {other:?}"
        ))),
    }
}

/// Pin or unpin.
pub fn set_pinned(client: &DaemonClient, id: &str, pinned: bool) -> Result<bool> {
    let item_id: ClipboardItemId = id.parse()?;
    let request = if pinned {
        Request::PinItem { item_id }
    } else {
        Request::UnpinItem { item_id }
    };
    match client.request(request)? {
        Response::Pinned { pinned, .. } => Ok(pinned),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected pin response: {other:?}"
        ))),
    }
}

/// Ask the daemon for the payload, then write the OS clipboard.
///
/// The daemon records a skip-hash so the watch thread does not insert a new
/// row for this echo.
pub fn copy_history_item(
    client: &DaemonClient,
    writer: &dyn ClipboardWriter,
    id: &str,
) -> Result<()> {
    let item_id: ClipboardItemId = id.parse()?;
    match client.request(Request::CopyItem { item_id })? {
        Response::Copied { text, .. } => writer.write_text(&text),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected copy response: {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use clipl_core::{ClipboardItem, ClipboardItemId};
    use clipl_protocol::{Envelope, Message};
    use std::os::unix::net::UnixListener;
    use std::thread;

    use crate::clipboard::RecordingClipboard;

    fn serve_one(path: &std::path::Path, reply: Response) {
        let listener = UnixListener::bind(path).unwrap();
        let expected = reply;
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let env = clipl_protocol::read_frame(&mut stream).unwrap();
            let Envelope { id, .. } = env;
            clipl_protocol::write_frame(
                &mut stream,
                &Envelope {
                    id,
                    payload: Message::Response(expected),
                },
            )
            .unwrap();
        });
    }

    #[test]
    fn disconnected_when_socket_missing() {
        let client = DaemonClient::with_socket("/tmp/clipl-desktop-missing.sock");
        match get_daemon_status(&client) {
            ConnectionView::Disconnected { message, .. } => {
                assert!(message.contains("not running"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn maps_history_without_payloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("h.sock");
        let item = ClipboardItem::text("alpha beta");
        serve_one(&path, Response::History(vec![item.clone()]));
        let client = DaemonClient::with_socket(&path);
        let rows = get_history(&client).unwrap();
        assert_eq!(rows[0].preview, "alpha beta");
        assert_eq!(rows[0].id, item.id.to_string());
    }

    #[test]
    fn pin_maps_response() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p.sock");
        let id = ClipboardItemId::new();
        serve_one(
            &path,
            Response::Pinned {
                item_id: id,
                pinned: true,
            },
        );
        let client = DaemonClient::with_socket(&path);
        assert!(set_pinned(&client, &id.to_string(), true).unwrap());
    }

    #[test]
    fn copy_writes_through_sink() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.sock");
        let id = ClipboardItemId::new();
        serve_one(
            &path,
            Response::Copied {
                item_id: id,
                text: "payload".into(),
            },
        );
        let client = DaemonClient::with_socket(&path);
        let sink = RecordingClipboard {
            writes: Mutex::new(Vec::new()),
        };
        copy_history_item(&client, &sink, &id.to_string()).unwrap();
        assert_eq!(*sink.writes.lock().unwrap(), vec!["payload".to_string()]);
    }

    fn dummy_status(protocol_version: u32) -> clipl_protocol::DaemonStatus {
        clipl_protocol::DaemonStatus {
            version: "0.1.0".into(),
            protocol_version,
            session: clipl_core::SessionType::Wayland,
            desktop: clipl_core::DesktopEnvironment::Gnome,
            backend: "none".into(),
            monitoring: clipl_protocol::MonitoringStatus::Unsupported,
            monitoring_reason: "test".into(),
            database: "/tmp/x".into(),
            socket_path: "/tmp/y".into(),
            privacy_enabled: true,
            history_enabled: true,
            history_limit: 500,
        }
    }

    #[test]
    fn protocol_mismatch_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v.sock");
        serve_one(
            &path,
            Response::Status(dummy_status(clipl_protocol::PROTOCOL_VERSION + 7)),
        );
        let client = DaemonClient::with_socket(&path);
        match get_daemon_status(&client) {
            ConnectionView::Error { message } => {
                assert!(message.contains("incompatible protocol"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unexpected_history_response_is_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.sock");
        serve_one(&path, Response::Pong);
        let client = DaemonClient::with_socket(&path);
        let err = get_history(&client).unwrap_err();
        assert!(err.to_string().contains("unexpected history"));
    }

    #[test]
    fn empty_search_uses_recent_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sock");
        serve_one(
            &path,
            Response::History(vec![ClipboardItem::text("recent")]),
        );
        let client = DaemonClient::with_socket(&path);
        let rows = search_history(&client, "   ").unwrap();
        assert_eq!(rows[0].preview, "recent");
    }

    #[test]
    fn delete_error_is_surfaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("d.sock");
        serve_one(
            &path,
            Response::Error {
                message: "unpin this item before deleting it".into(),
            },
        );
        let client = DaemonClient::with_socket(&path);
        let err = delete_history_item(&client, &ClipboardItemId::new().to_string()).unwrap_err();
        assert!(err.to_string().contains("unpin"));
    }
}
