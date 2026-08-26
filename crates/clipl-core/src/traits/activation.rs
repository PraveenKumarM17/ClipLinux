//! Activation backend interface.
//!
//! Implementations live in `clipl-platform`. Core only defines the contract.

use crate::activation::{ActivationBackendKind, ActivationCapability, ActivationSnapshot};

/// Discovers how the picker can be shown in this session.
///
/// Implementations must not capture keys they were not asked to register,
/// and must not claim Wayland sessions can use X11 grabs.
pub trait ActivationBackend: Send {
    /// Backend slot identifier.
    fn kind(&self) -> ActivationBackendKind;

    /// Capability class for this backend.
    fn capability(&self) -> ActivationCapability;

    /// Honest status snapshot (no keystrokes, no secrets).
    fn snapshot(&self) -> ActivationSnapshot;

    /// Whether [`ActivationBackend::listen`] registers a native grab.
    fn supports_native_listen(&self) -> bool {
        false
    }
}
