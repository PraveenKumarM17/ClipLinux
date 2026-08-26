//! Generic wlroots slot.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// wlroots family placeholder.
pub struct WlrootsActivation {
    shortcut: Shortcut,
}

impl WlrootsActivation {
    /// Planned slot.
    pub fn new(shortcut: Shortcut) -> Self {
        Self { shortcut }
    }
}

impl ActivationBackend for WlrootsActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::WlrootsGeneric
    }

    fn capability(&self) -> ActivationCapability {
        ActivationCapability::CompositorBinding
    }

    fn snapshot(&self) -> ActivationSnapshot {
        ActivationSnapshot {
            backend: self.kind(),
            capability: self.capability(),
            status: ActivationStatus::Unsupported,
            shortcut: self.shortcut.display(),
            reason: "Generic wlroots activation is planned. Use the compositor's keybind to `clipl toggle`.".into(),
        }
    }
}

impl NativeActivation for WlrootsActivation {}
