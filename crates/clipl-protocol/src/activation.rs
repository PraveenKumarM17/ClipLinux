//! Activation IPC types. No keystroke payloads — only explicit picker commands.

use clipl_core::{
    ActivationBackendKind, ActivationCapability, ActivationSnapshot, ActivationStatus,
    DesktopEnvironment, SessionType,
};
use serde::{Deserialize, Serialize};

/// Status block returned by `GetStatus` / `GetActivationStatus`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationReport {
    /// Session type.
    pub session: SessionType,
    /// Desktop environment.
    pub desktop: DesktopEnvironment,
    /// Backend slot id.
    pub backend: ActivationBackendKind,
    /// Capability class.
    pub capability: ActivationCapability,
    /// Runtime status.
    pub status: ActivationStatus,
    /// Display form of the configured shortcut.
    pub shortcut: String,
    /// Whether a desktop process is subscribed for show/hide events.
    pub desktop_connected: bool,
    /// Safe-to-print explanation.
    pub reason: String,
}

impl Default for ActivationReport {
    fn default() -> Self {
        let snapshot = ActivationSnapshot::default();
        Self {
            session: SessionType::Unknown,
            desktop: DesktopEnvironment::Unknown,
            backend: snapshot.backend,
            capability: snapshot.capability,
            status: snapshot.status,
            shortcut: snapshot.shortcut,
            desktop_connected: false,
            reason: snapshot.reason,
        }
    }
}

impl ActivationReport {
    /// Build from a probe snapshot plus subscriber state.
    pub fn from_snapshot(
        session: SessionType,
        desktop: DesktopEnvironment,
        snapshot: &ActivationSnapshot,
        desktop_connected: bool,
    ) -> Self {
        Self {
            session,
            desktop,
            backend: snapshot.backend,
            capability: snapshot.capability,
            status: snapshot.status,
            shortcut: snapshot.shortcut.clone(),
            desktop_connected,
            reason: snapshot.reason.clone(),
        }
    }
}
