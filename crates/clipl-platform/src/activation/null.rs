//! Disabled / unknown activation backend.

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Shortcut,
};

use super::NativeActivation;

/// No in-process shortcut registration.
pub struct NullActivation {
    shortcut: Shortcut,
    capability: ActivationCapability,
    status: ActivationStatus,
    reason: String,
}

impl NullActivation {
    /// CLI / manual only.
    pub fn manual(shortcut: Shortcut, reason: impl Into<String>) -> Self {
        Self {
            shortcut,
            capability: ActivationCapability::ManualOnly,
            status: ActivationStatus::NotConfigured,
            reason: reason.into(),
        }
    }

    /// Session could not be identified.
    pub fn unknown(shortcut: Shortcut) -> Self {
        Self {
            shortcut,
            capability: ActivationCapability::ManualOnly,
            status: ActivationStatus::Unsupported,
            reason: "session type is unknown; activation not started".into(),
        }
    }

    /// Confirmed unavailable.
    pub fn unsupported(shortcut: Shortcut) -> Self {
        Self {
            shortcut,
            capability: ActivationCapability::Unsupported,
            status: ActivationStatus::Unsupported,
            reason: "no activation backend for this session".into(),
        }
    }

    /// Registration failed or feature missing.
    pub fn error(shortcut: Shortcut, reason: impl Into<String>) -> Self {
        Self {
            shortcut,
            capability: ActivationCapability::NativeGlobalShortcut,
            status: ActivationStatus::Error,
            reason: reason.into(),
        }
    }
}

impl ActivationBackend for NullActivation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::Null
    }

    fn capability(&self) -> ActivationCapability {
        self.capability
    }

    fn snapshot(&self) -> ActivationSnapshot {
        ActivationSnapshot {
            backend: self.kind(),
            capability: self.capability,
            status: self.status,
            shortcut: self.shortcut.display(),
            reason: self.reason.clone(),
        }
    }
}

impl NativeActivation for NullActivation {}
