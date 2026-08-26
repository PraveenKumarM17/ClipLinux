//! KDE Plasma activation slot (not implemented).

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// Plasma backend placeholder.
pub struct KdeActivation {
    shortcut: Shortcut,
}

impl KdeActivation {
    /// Planned slot.
    pub fn new(shortcut: Shortcut) -> Self {
        Self { shortcut }
    }
}

impl ActivationBackend for KdeActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::KdePlasma
    }

    fn capability(&self) -> ActivationCapability {
        ActivationCapability::DesktopManagedShortcut
    }

    fn snapshot(&self) -> ActivationSnapshot {
        ActivationSnapshot {
            backend: self.kind(),
            capability: self.capability(),
            status: ActivationStatus::Unsupported,
            shortcut: self.shortcut.display(),
            reason:
                "KDE Plasma activation is planned, not implemented. Use clipl open / clipl toggle."
                    .into(),
        }
    }
}

impl NativeActivation for KdeActivation {}
