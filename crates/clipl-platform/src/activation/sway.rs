//! Sway compositor-binding slot. ClipLinux does not capture keys.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// Sway user-config binding.
pub struct SwayActivation {
    shortcut: Shortcut,
}

impl SwayActivation {
    /// Planned compositor-config integration.
    pub fn new(shortcut: Shortcut) -> Self {
        Self { shortcut }
    }
}

impl ActivationBackend for SwayActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::Sway
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
            reason: "Bind a Sway key to `clipl toggle` (see docs/architecture/activation.md). ClipLinux does not grab keys inside Sway.".into(),
        }
    }
}

impl NativeActivation for SwayActivation {}
