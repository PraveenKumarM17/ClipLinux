//! Generic Wayland: no portable global shortcut API.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// Honest unsupported Wayland grab.
pub struct GenericWaylandActivation {
    shortcut: Shortcut,
}

impl GenericWaylandActivation {
    /// Manual/CLI activation only.
    pub fn new(shortcut: Shortcut) -> Self {
        Self { shortcut }
    }
}

impl ActivationBackend for GenericWaylandActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::GenericWayland
    }

    fn capability(&self) -> ActivationCapability {
        ActivationCapability::Unsupported
    }

    fn snapshot(&self) -> ActivationSnapshot {
        ActivationSnapshot {
            backend: self.kind(),
            capability: self.capability(),
            status: ActivationStatus::Unsupported,
            shortcut: self.shortcut.display(),
            reason: "Generic Wayland has no portable global shortcut API. Use clipl open / clipl toggle.".into(),
        }
    }
}

impl NativeActivation for GenericWaylandActivation {}
