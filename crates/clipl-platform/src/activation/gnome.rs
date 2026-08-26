//! Honest GNOME Shell activation slot. Does not grab keys.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// GNOME Shell extension boundary.
pub struct GnomeActivation {
    shortcut: Shortcut,
    extension_present: bool,
}

impl GnomeActivation {
    /// `extension_present` is a filesystem probe, not a Shell enablement check.
    pub fn new(shortcut: Shortcut, extension_present: bool) -> Self {
        Self {
            shortcut,
            extension_present,
        }
    }
}

impl ActivationBackend for GnomeActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::GnomeShell
    }

    fn capability(&self) -> ActivationCapability {
        ActivationCapability::DesktopManagedShortcut
    }

    fn snapshot(&self) -> ActivationSnapshot {
        if self.extension_present {
            ActivationSnapshot {
                backend: self.kind(),
                capability: self.capability(),
                status: ActivationStatus::ConfiguredExternally,
                shortcut: self.shortcut.display(),
                reason: "GNOME Shell extension is installed. Shortcut registration is owned by the extension, not by clipl-daemon.".into(),
            }
        } else {
            ActivationSnapshot {
                backend: self.kind(),
                capability: self.capability(),
                status: ActivationStatus::NotConfigured,
                shortcut: self.shortcut.display(),
                reason: "GNOME Shell extension not installed. See extensions/gnome/README.md. Use clipl open / clipl toggle until then.".into(),
            }
        }
    }
}

impl NativeActivation for GnomeActivation {}
