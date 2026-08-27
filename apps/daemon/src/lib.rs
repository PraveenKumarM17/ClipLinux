//! ClipLinux daemon library: history recording, IPC, diagnostics.

#![forbid(unsafe_code)]

mod activation;
mod picker;

use std::fs;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clipl_clipboard::{content_hash, for_client, HistoryEngine, RecordOutcome, SqliteStore};
use clipl_core::placeholders::MemoryClipboard;
use clipl_core::{
    paths, ActivationRequest, ClipLinuxConfig, ClipLinuxPaths, ClipboardBackend, ClipboardContent,
    ClipboardItem, Error, PlatformAdapter, Result,
};
use clipl_platform::{
    capabilities_for, probe_identity_from_env, restore_focus_and_ctrl_v, select_activation_backend,
    select_adapter, select_clipboard_backend, AdapterKind, SelectedClipboard,
};
use clipl_privacy::default_rules;
use clipl_protocol::{
    cleanup_stale_socket, read_frame, set_socket_mode, write_frame, ActivationReport, DaemonStatus,
    Envelope, Message, MonitoringStatus, Request, Response, PROTOCOL_VERSION,
};

use activation::{copy_only_reason, DesktopHub, InsertHub};

/// Load `config.toml` or defaults. Invalid files are errors.
pub fn load_config() -> Result<ClipLinuxConfig> {
    let path = paths::config_path();
    if path.exists() {
        let text = fs::read_to_string(&path)
            .map_err(|err| Error::Config(format!("{}: {err}", path.display())))?;
        ClipLinuxConfig::from_toml_str(&text)
    } else {
        let cfg = ClipLinuxConfig::default();
        cfg.validate()?;
        Ok(cfg)
    }
}

/// Print-only diagnostic (no clipboard payloads).
pub fn diagnostic_report(config: &ClipLinuxConfig) -> String {
    diagnostic_report_with(config, &ClipLinuxPaths::from_env())
}

/// Diagnostic text using explicit paths.
pub fn diagnostic_report_with(config: &ClipLinuxConfig, dirs: &ClipLinuxPaths) -> String {
    let identity = probe_identity_from_env();
    let selected = select_clipboard_backend(&identity, &config.clipboard);
    let preferred = AdapterKind::preferred(&identity);
    let activation = select_activation_backend(&identity, &config.activation);
    format!(
        "ClipLinux Daemon Diagnostic\n\
         \n\
         Session: {:?}\n\
         Desktop: {:?}\n\
         Preferred adapter: {} (implemented: {})\n\
         Clipboard backend: {}\n\
         Monitoring: {:?}\n\
         Reason: {}\n\
         Database: {}\n\
         Socket: {}\n\
         Privacy engine: {}\n\
         History enabled: {}\n\
         History limit: {}\n\
         \n\
         {}\n",
        identity.session,
        identity.desktop,
        preferred.as_str(),
        preferred.is_implemented(),
        selected.name,
        monitoring_of(selected.watch),
        selected.reason,
        dirs.database_file().display(),
        dirs.socket_file().display(),
        if config.privacy.enabled {
            "enabled"
        } else {
            "disabled"
        },
        config.history.enabled,
        config.history.max_items,
        clipl_platform::format_activation_report(&identity, &activation, false),
    )
}

fn monitoring_of(level: clipl_core::SupportLevel) -> MonitoringStatus {
    match level {
        clipl_core::SupportLevel::Native | clipl_core::SupportLevel::Portal => {
            MonitoringStatus::Supported
        }
        clipl_core::SupportLevel::Fallback => MonitoringStatus::Partial,
        _ => MonitoringStatus::Unsupported,
    }
}

/// Shared daemon state.
pub struct DaemonState {
    engine: Mutex<HistoryEngine<SqliteStore>>,
    status: DaemonStatus,
    shutdown: Arc<AtomicBool>,
    dirs: ClipLinuxPaths,
    /// Hashes of items the desktop just copied; ingest skips one matching event.
    skip_copy: Mutex<Option<(String, std::time::Instant)>>,
    hub: Arc<DesktopHub>,
    insert_hub: Arc<InsertHub>,
    insert_enabled: bool,
    x11_focus: Option<Arc<AtomicU32>>,
    activation: Mutex<ActivationReport>,
}

impl DaemonState {
    fn handle(&self, request: Request) -> Response {
        match request {
            Request::Ping => Response::Pong,
            Request::GetStatus => Response::Status(self.status_with_activation()),
            Request::GetActivationStatus => Response::Activation(self.activation_report()),
            Request::ShowDesktop => self.route_desktop(ActivationRequest::ShowPicker),
            Request::HideDesktop => self.route_desktop(ActivationRequest::HidePicker),
            Request::ToggleDesktop => self.route_desktop(ActivationRequest::TogglePicker),
            Request::SubscribeDesktop => Response::Error {
                message: "SubscribeDesktop must use the dedicated connection path".into(),
            },
            Request::SubscribeInsert => Response::Error {
                message: "SubscribeInsert must use the dedicated connection path".into(),
            },
            Request::InsertIntoApp => self.insert_into_app(),
            Request::GetCapabilities => {
                let adapter = select_adapter();
                Response::Capabilities(adapter.capabilities())
            }
            Request::GetHistory { limit } => match self.engine.lock() {
                Ok(engine) => match engine.list(limit.max(1) as usize) {
                    Ok(items) => Response::History(items.into_iter().map(for_client).collect()),
                    Err(err) => Response::Error {
                        message: err.to_string(),
                    },
                },
                Err(_) => Response::Error {
                    message: "history lock poisoned".into(),
                },
            },
            Request::SearchHistory { query, limit } => match self.engine.lock() {
                Ok(engine) => match engine.search(&query, limit.max(1) as usize) {
                    Ok(items) => Response::History(items.into_iter().map(for_client).collect()),
                    Err(err) => Response::Error {
                        message: err.to_string(),
                    },
                },
                Err(_) => Response::Error {
                    message: "history lock poisoned".into(),
                },
            },
            Request::DeleteItem { item_id } => match self.engine.lock() {
                Ok(engine) => match engine.delete(item_id) {
                    Ok(existed) => Response::Deleted { existed },
                    Err(err) => Response::Error {
                        message: err.to_string(),
                    },
                },
                Err(_) => Response::Error {
                    message: "history lock poisoned".into(),
                },
            },
            Request::ClearHistory => match self.engine.lock() {
                Ok(engine) => match engine.clear() {
                    Ok(count) => Response::Cleared { count },
                    Err(err) => Response::Error {
                        message: err.to_string(),
                    },
                },
                Err(_) => Response::Error {
                    message: "history lock poisoned".into(),
                },
            },
            Request::PinItem { item_id } => self.set_pin(item_id, true),
            Request::UnpinItem { item_id } => self.set_pin(item_id, false),
            Request::CopyItem { item_id } => self.copy_item(item_id),
            Request::ListSnippets => Response::Snippets(Vec::new()),
            Request::ListPrivacyRules => Response::PrivacyRules(default_rules()),
            Request::Paste { .. } => Response::Error {
                message: "paste is not implemented".into(),
            },
            other => self.handle_picker(other).unwrap_or(Response::Error {
                message: "unknown request".into(),
            }),
        }
    }

    fn set_pin(&self, item_id: clipl_core::ClipboardItemId, pinned: bool) -> Response {
        match self.engine.lock() {
            Ok(engine) => match engine.set_pinned(item_id, pinned) {
                Ok(()) => Response::Pinned { item_id, pinned },
                Err(err) => Response::Error {
                    message: err.to_string(),
                },
            },
            Err(_) => Response::Error {
                message: "history lock poisoned".into(),
            },
        }
    }

    fn copy_item(&self, item_id: clipl_core::ClipboardItemId) -> Response {
        let Ok(engine) = self.engine.lock() else {
            return Response::Error {
                message: "history lock poisoned".into(),
            };
        };
        let item = match engine.get(item_id) {
            Ok(Some(item)) => item,
            Ok(None) => {
                return Response::Error {
                    message: "item not found".into(),
                };
            }
            Err(err) => {
                return Response::Error {
                    message: err.to_string(),
                };
            }
        };
        if !item.sensitive.is_empty() {
            return Response::Error {
                message: "hidden items cannot be copied".into(),
            };
        }
        let Some(text) = item.content.text_for_scan().map(str::to_string) else {
            return Response::Error {
                message: "only text items can be copied in this phase".into(),
            };
        };
        let hash = if item.content_hash.is_empty() {
            content_hash(&item.content)
        } else {
            item.content_hash.clone()
        };
        if let Err(err) = engine.touch(item_id) {
            return Response::Error {
                message: err.to_string(),
            };
        }
        drop(engine);
        if let Ok(mut skip) = self.skip_copy.lock() {
            *skip = Some((hash, std::time::Instant::now()));
        }
        Response::Copied { item_id, text }
    }

    fn should_skip_copy(&self, hash: &str) -> bool {
        let Ok(mut skip) = self.skip_copy.lock() else {
            return false;
        };
        let Some((stored, at)) = skip.as_ref() else {
            return false;
        };
        if at.elapsed() > Duration::from_secs(3) {
            *skip = None;
            return false;
        }
        if stored == hash {
            *skip = None;
            true
        } else {
            false
        }
    }

    fn activation_report(&self) -> ActivationReport {
        let mut report = self
            .activation
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        report.desktop_connected = self.hub.connected();
        report
    }

    fn status_with_activation(&self) -> DaemonStatus {
        let mut status = self.status.clone();
        status.activation = self.activation_report();
        status
    }

    fn route_desktop(&self, action: ActivationRequest) -> Response {
        if action != ActivationRequest::HidePicker {
            let _ = self.insert_hub.prepare();
        }
        Response::DesktopRouted {
            delivered: self.hub.route(action),
        }
    }

    fn insert_into_app(&self) -> Response {
        if !self.insert_enabled {
            return Response::Inserted {
                delivered: false,
                reason: "insert is disabled; copied to the clipboard".into(),
            };
        }
        if self.insert_hub.route() {
            return Response::Inserted {
                delivered: true,
                reason: String::new(),
            };
        }
        if let Some(slot) = &self.x11_focus {
            match restore_focus_and_ctrl_v(slot.load(Ordering::SeqCst)) {
                Ok(()) => {
                    return Response::Inserted {
                        delivered: true,
                        reason: String::new(),
                    };
                }
                Err(_) => {
                    return Response::Inserted {
                        delivered: false,
                        reason: copy_only_reason(),
                    };
                }
            }
        }
        Response::Inserted {
            delivered: false,
            reason: copy_only_reason(),
        }
    }
}

/// Run the daemon until `shutdown` is set.
pub fn run(config: ClipLinuxConfig, shutdown: Arc<AtomicBool>) -> Result<()> {
    let identity = probe_identity_from_env();
    let selected = select_clipboard_backend(&identity, &config.clipboard);
    run_with_backend(
        config,
        identity,
        selected,
        ClipLinuxPaths::from_env(),
        shutdown,
    )
}

/// Test/production entry with an explicit backend and directories.
pub fn run_with_backend(
    config: ClipLinuxConfig,
    identity: clipl_core::PlatformIdentity,
    selected: SelectedClipboard,
    dirs: ClipLinuxPaths,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    dirs.ensure()?;

    let db_path = dirs.database_file();
    let store = SqliteStore::open(&db_path)?;
    let engine = HistoryEngine::new(store, default_rules(), config.clone());

    let mut activation_selected = select_activation_backend(&identity, &config.activation);
    let x11_focus = activation_selected.backend.focus_window_slot();
    let insert_enabled = config.insert.enabled;
    let mut activation_snapshot = activation_selected.snapshot.clone();
    let pending_listen = if activation_selected.backend.supports_native_listen() {
        match activation_selected.backend.arm() {
            Ok(()) => {
                activation_snapshot = activation_selected.backend.snapshot();
                Some((
                    activation_selected.backend,
                    activation_selected.behavior.request(),
                ))
            }
            Err(err) => {
                tracing::warn!("native activation grab failed: {err}");
                activation_snapshot.status = clipl_core::ActivationStatus::Error;
                activation_snapshot.reason = err.to_string();
                None
            }
        }
    } else {
        None
    };

    let activation_report = ActivationReport::from_snapshot(
        identity.session,
        identity.desktop.clone(),
        &activation_snapshot,
        false,
    );
    let hub = Arc::new(DesktopHub::default());
    let insert_hub = Arc::new(InsertHub::default());

    let status = DaemonStatus {
        version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: PROTOCOL_VERSION,
        session: identity.session,
        desktop: identity.desktop.clone(),
        backend: selected.name.into(),
        monitoring: monitoring_of(selected.watch),
        monitoring_reason: selected.reason.clone(),
        database: db_path.display().to_string(),
        socket_path: dirs.socket_file().display().to_string(),
        privacy_enabled: config.privacy.enabled,
        history_enabled: config.history.enabled,
        history_limit: config.history.max_items,
        activation: activation_report.clone(),
    };

    tracing::info!(
        backend = selected.name,
        monitoring = ?status.monitoring,
        database = %status.database,
        "clipl-daemon starting"
    );

    let state = Arc::new(DaemonState {
        engine: Mutex::new(engine),
        status,
        shutdown: Arc::clone(&shutdown),
        dirs,
        skip_copy: Mutex::new(None),
        hub: Arc::clone(&hub),
        insert_hub: Arc::clone(&insert_hub),
        insert_enabled,
        x11_focus,
        activation: Mutex::new(activation_report),
    });

    let activation_thread = if let Some((mut backend, fire)) = pending_listen {
        let stop = Arc::clone(&shutdown);
        let hub_thread = Arc::clone(&hub);
        tracing::info!(
            shortcut = %activation_snapshot.shortcut,
            "X11 activation grab armed"
        );
        Some(
            thread::Builder::new()
                .name("clipl-activation".into())
                .spawn(move || {
                    if let Err(err) = backend.listen(stop.as_ref(), &|| {
                        hub_thread.route(fire);
                    }) {
                        tracing::warn!("activation listen ended: {err}");
                    }
                })
                .map_err(|err| Error::Io(err.to_string()))?,
        )
    } else {
        tracing::info!(
            backend = %activation_snapshot.backend.as_str(),
            status = %activation_snapshot.status.as_str(),
            "native activation grab not started"
        );
        None
    };

    let watch_thread = if selected.backend.supports_watch() {
        let watch_state = Arc::clone(&state);
        let backend = selected.backend;
        Some(
            thread::Builder::new()
                .name("clipl-clipboard".into())
                .spawn(move || watch_loop(backend, watch_state))
                .map_err(|err| Error::Io(err.to_string()))?,
        )
    } else {
        tracing::info!(
            reason = %selected.reason,
            "clipboard monitoring not started"
        );
        drop(selected.backend);
        None
    };

    let result = serve_ipc(state);
    if let Some(handle) = watch_thread {
        let _ = handle.join();
    }
    if let Some(handle) = activation_thread {
        let _ = handle.join();
    }
    result
}

fn watch_loop(backend: Box<dyn ClipboardBackend>, state: Arc<DaemonState>) {
    let mut watcher = match backend.watch() {
        Ok(w) => w,
        Err(err) => {
            tracing::warn!("clipboard watch failed to start: {err}");
            return;
        }
    };
    while !state.shutdown.load(Ordering::SeqCst) {
        match watcher.recv_timeout(Duration::from_millis(400)) {
            Ok(Some(content)) => ingest(&state, content),
            Ok(None) => {}
            Err(err) => {
                tracing::warn!("clipboard watch error: {err}");
                if state.shutdown.load(Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_millis(500));
            }
        }
    }
}

fn ingest(state: &DaemonState, content: ClipboardContent) {
    if !matches!(
        content,
        ClipboardContent::Text { .. }
            | ClipboardContent::Html { .. }
            | ClipboardContent::Uri { .. }
    ) {
        tracing::debug!("ignored non-text clipboard event");
        return;
    }
    let hash = content_hash(&content);
    if state.should_skip_copy(&hash) {
        tracing::debug!("skipped clipboard echo from palette copy");
        return;
    }
    let item = ClipboardItem {
        id: clipl_core::ClipboardItemId::new(),
        content,
        source: clipl_core::ClipboardSource::LocalSession,
        created_at: clipl_core::Timestamp::now(),
        last_used_at: None,
        pinned: false,
        tags: Vec::new(),
        sensitive: Vec::new(),
        content_hash: String::new(),
        updated_at: clipl_core::Timestamp::now(),
        expires_at: None,
        source_app: None,
    };
    let Ok(engine) = state.engine.lock() else {
        tracing::warn!("history lock poisoned");
        return;
    };
    match engine.record(&item) {
        Ok(recorded) => match recorded.outcome {
            RecordOutcome::Stored => {
                tracing::info!(id = %recorded.item_id, "recorded clipboard item");
            }
            RecordOutcome::Reused => {
                tracing::debug!(id = %recorded.item_id, "reused consecutive duplicate");
            }
            RecordOutcome::Excluded => {
                let reason = recorded
                    .verdict
                    .as_ref()
                    .map(|v| v.reasons.join("; "))
                    .unwrap_or_default();
                tracing::info!(reason = %reason, "excluded clipboard item");
            }
            RecordOutcome::NeedsConfirmation => {
                tracing::info!("clipboard item requires confirmation; not stored");
            }
            RecordOutcome::Skipped => {
                tracing::debug!("clipboard item skipped");
            }
        },
        Err(err) => tracing::warn!("failed to record clipboard item: {err}"),
    }
}

fn serve_ipc(state: Arc<DaemonState>) -> Result<()> {
    let sock = state.dirs.socket_file();
    cleanup_stale_socket(&sock)?;
    let listener =
        UnixListener::bind(&sock).map_err(|err| Error::Io(format!("{}: {err}", sock.display())))?;
    set_socket_mode(&sock)?;
    listener
        .set_nonblocking(true)
        .map_err(|err| Error::Io(err.to_string()))?;
    tracing::info!(socket = %sock.display(), "IPC listening");

    while !state.shutdown.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
                if let Err(err) = handle_connection(&state, stream) {
                    tracing::warn!("ipc client error: {err}");
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(err) => {
                tracing::warn!("ipc accept: {err}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    let _ = fs::remove_file(&sock);
    tracing::info!("clipl-daemon stopped");
    Ok(())
}

fn handle_connection(
    state: &DaemonState,
    mut stream: std::os::unix::net::UnixStream,
) -> Result<()> {
    let incoming = read_frame(&mut stream)?;
    let Envelope { id, payload } = incoming;
    match payload {
        Message::Request(Request::SubscribeDesktop) => {
            let replaced = state.hub.connected();
            let _ = stream.set_read_timeout(None);
            let _ = stream.set_write_timeout(None);
            write_frame(
                &mut stream,
                &Envelope {
                    id,
                    payload: Message::Response(Response::DesktopSubscribed { replaced }),
                },
            )?;
            let _ = state.hub.subscribe(stream);
            Ok(())
        }
        Message::Request(Request::SubscribeInsert) => {
            let replaced = state.insert_hub.connected();
            let _ = stream.set_read_timeout(None);
            let _ = stream.set_write_timeout(None);
            write_frame(
                &mut stream,
                &Envelope {
                    id,
                    payload: Message::Response(Response::InsertSubscribed { replaced }),
                },
            )?;
            let _ = state.insert_hub.subscribe(stream);
            Ok(())
        }
        Message::Request(request) => {
            let response = state.handle(request);
            write_frame(
                &mut stream,
                &Envelope {
                    id,
                    payload: Message::Response(response),
                },
            )
        }
        _ => write_frame(
            &mut stream,
            &Envelope {
                id,
                payload: Message::Response(Response::Error {
                    message: "expected a request".into(),
                }),
            },
        ),
    }
}

/// Build a [`SelectedClipboard`] around a mock backend for tests.
pub fn mock_selection(backend: MemoryClipboard) -> SelectedClipboard {
    SelectedClipboard {
        backend: Box::new(backend),
        name: "memory",
        watch: clipl_core::SupportLevel::Native,
        read: clipl_core::SupportLevel::Native,
        reason: "in-process mock clipboard".into(),
    }
}

/// Re-export for diagnostics that want the adapter probe.
pub fn session_capabilities(config: &ClipLinuxConfig) -> clipl_core::PlatformCapabilities {
    let identity = probe_identity_from_env();
    capabilities_for(&identity, &config.clipboard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_protocol::{IpcClient, Request};

    fn wait_history(
        sock: &std::path::Path,
        pred: impl Fn(&[clipl_core::ClipboardItem]) -> bool,
    ) -> Vec<clipl_core::ClipboardItem> {
        for _ in 0..40 {
            if let Ok(mut client) = IpcClient::connect_path(sock) {
                match client.request(Request::GetHistory { limit: 10 }) {
                    Ok(Response::History(items)) if pred(&items) => return items,
                    Ok(Response::History(_)) | Err(_) => {}
                    Ok(other) => panic!("unexpected {other:?}"),
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("timed out waiting for expected history");
    }

    fn test_config() -> ClipLinuxConfig {
        ClipLinuxConfig {
            activation: clipl_core::ActivationConfig {
                enabled: false,
                ..clipl_core::ActivationConfig::default()
            },
            ..ClipLinuxConfig::default()
        }
    }

    fn wait_socket(sock: &std::path::Path) {
        for _ in 0..100 {
            if sock.exists() {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("daemon socket was not created");
    }

    fn wayland_identity() -> clipl_core::PlatformIdentity {
        clipl_core::PlatformIdentity {
            platform: clipl_core::Platform::Linux,
            session: clipl_core::SessionType::Wayland,
            desktop: clipl_core::DesktopEnvironment::Gnome,
            xdg_current_desktop: Some("GNOME".into()),
            xdg_session_type: Some("wayland".into()),
        }
    }

    #[test]
    fn mock_backend_records_and_serves_history() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = ClipLinuxPaths::isolated(tmp.path());
        let clipboard = MemoryClipboard::default();
        let producer = clipboard.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let identity = wayland_identity();
        let selected = mock_selection(clipboard);
        let dirs_thread = dirs.clone();
        let thread = thread::spawn(move || {
            run_with_backend(test_config(), identity, selected, dirs_thread, stop).unwrap();
        });
        let sock = dirs.socket_file();
        wait_socket(&sock);
        producer
            .write(&ClipboardItem::text("from-mock").content)
            .unwrap();
        let items = wait_history(&sock, |items| {
            items.len() == 1 && items[0].content.text_for_scan() == Some("from-mock")
        });
        assert_eq!(items.len(), 1);
        let pem = ClipboardItem::text("-----BEGIN OPENSSH PRIVATE KEY-----\nx");
        producer.write(&pem.content).unwrap();
        thread::sleep(Duration::from_millis(200));
        let items = wait_history(&sock, |items| {
            items.len() == 1 && items[0].content.text_for_scan() == Some("from-mock")
        });
        assert_eq!(items[0].content.text_for_scan(), Some("from-mock"));
        let item_id = items[0].id;
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::PinItem { item_id }).unwrap() {
            Response::Pinned { pinned, .. } => assert!(pinned),
            other => panic!("unexpected {other:?}"),
        }
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::CopyItem { item_id }).unwrap() {
            Response::Copied { text, .. } => assert_eq!(text, "from-mock"),
            other => panic!("unexpected {other:?}"),
        }
        producer
            .write(&ClipboardItem::text("from-mock").content)
            .unwrap();
        thread::sleep(Duration::from_millis(250));
        let listed = wait_history(&sock, |rows| rows.len() == 1);
        assert_eq!(listed.len(), 1);
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::DeleteItem { item_id }).unwrap() {
            Response::Error { message } => assert!(message.contains("unpin")),
            other => panic!("unexpected {other:?}"),
        }
        shutdown.store(true, Ordering::SeqCst);
        thread.join().unwrap();
    }

    #[test]
    fn toggle_without_desktop_is_not_delivered() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = ClipLinuxPaths::isolated(tmp.path());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let dirs_thread = dirs.clone();
        let thread = thread::spawn(move || {
            run_with_backend(
                test_config(),
                wayland_identity(),
                mock_selection(MemoryClipboard::default()),
                dirs_thread,
                stop,
            )
            .unwrap();
        });
        let sock = dirs.socket_file();
        wait_socket(&sock);
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::ToggleDesktop).unwrap() {
            Response::DesktopRouted { delivered } => assert!(!delivered),
            other => panic!("unexpected {other:?}"),
        }
        shutdown.store(true, Ordering::SeqCst);
        thread.join().unwrap();
    }

    #[test]
    fn insert_without_helper_is_copy_only() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = ClipLinuxPaths::isolated(tmp.path());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let dirs_thread = dirs.clone();
        let thread = thread::spawn(move || {
            run_with_backend(
                test_config(),
                wayland_identity(),
                mock_selection(MemoryClipboard::default()),
                dirs_thread,
                stop,
            )
            .unwrap();
        });
        let sock = dirs.socket_file();
        wait_socket(&sock);
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::InsertIntoApp).unwrap() {
            Response::Inserted { delivered, reason } => {
                assert!(!delivered);
                assert!(reason.contains("Ctrl+V"));
            }
            other => panic!("unexpected {other:?}"),
        }
        shutdown.store(true, Ordering::SeqCst);
        thread.join().unwrap();
    }

    #[test]
    fn subscribe_then_toggle_delivers_event() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = ClipLinuxPaths::isolated(tmp.path());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let dirs_thread = dirs.clone();
        let thread = thread::spawn(move || {
            run_with_backend(
                test_config(),
                wayland_identity(),
                mock_selection(MemoryClipboard::default()),
                dirs_thread,
                stop,
            )
            .unwrap();
        });
        let sock = dirs.socket_file();
        wait_socket(&sock);
        let sock_sub = sock.clone();
        let subscriber = thread::spawn(move || {
            let mut sub = clipl_protocol::ActivationSubscriber::connect_path(&sock_sub).unwrap();
            sub.recv().unwrap()
        });
        thread::sleep(Duration::from_millis(80));
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::ToggleDesktop).unwrap() {
            Response::DesktopRouted { delivered } => assert!(delivered),
            other => panic!("unexpected {other:?}"),
        }
        let action = subscriber.join().unwrap();
        assert_eq!(action, ActivationRequest::TogglePicker);
        shutdown.store(true, Ordering::SeqCst);
        thread.join().unwrap();
    }

    #[test]
    fn second_subscribe_replaces_the_first() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = ClipLinuxPaths::isolated(tmp.path());
        let shutdown = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&shutdown);
        let dirs_thread = dirs.clone();
        let thread = thread::spawn(move || {
            run_with_backend(
                test_config(),
                wayland_identity(),
                mock_selection(MemoryClipboard::default()),
                dirs_thread,
                stop,
            )
            .unwrap();
        });
        let sock = dirs.socket_file();
        wait_socket(&sock);
        let first = clipl_protocol::ActivationSubscriber::connect_path(&sock).unwrap();
        let mut second = clipl_protocol::ActivationSubscriber::connect_path(&sock).unwrap();
        let mut client = IpcClient::connect_path(&sock).unwrap();
        match client.request(Request::ShowDesktop).unwrap() {
            Response::DesktopRouted { delivered } => assert!(delivered),
            other => panic!("unexpected {other:?}"),
        }
        assert_eq!(second.recv().unwrap(), ActivationRequest::ShowPicker);
        let _ = first;
        shutdown.store(true, Ordering::SeqCst);
        thread.join().unwrap();
    }
}
