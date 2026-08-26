//! Hyprland compositor-binding slot. ClipLinux does not capture keys.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// Hyprland user-config binding.
pub struct HyprlandActivation {
    shortcut: Shortcut,
}

impl HyprlandActivation {
    /// Planned compositor-config integration.
    pub fn new(shortcut: Shortcut) -> Self {
        Self { shortcut }
    }
}

impl ActivationBackend for HyprlandActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::Hyprland
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
            reason: "Bind a Hyprland key to `clipl toggle` (see docs/architecture/activation.md). ClipLinux does not grab keys inside Hyprland.".into(),
        }
    }
}

impl NativeActivation for HyprlandActivation {}
