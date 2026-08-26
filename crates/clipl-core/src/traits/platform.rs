//! Platform adapter interface.

use crate::capabilities::PlatformCapabilities;
use crate::platform::PlatformIdentity;

/// Discovers session identity and capabilities.
///
/// One adapter exists per integration surface (generic Wayland, GNOME, KDE,
/// X11, future Hyprland/Sway). Adapters must report [`crate::SupportLevel`]
/// honestly rather than pretending protocols are interchangeable.
pub trait PlatformAdapter: Send + Sync {
    /// Adapter identifier, e.g. `linux-generic`, `gnome`, `kde`.
    fn name(&self) -> &'static str;

    /// Identity of the current session as seen by this adapter.
    fn identity(&self) -> PlatformIdentity;

    /// Capability matrix. Unknown is preferred over a guessed Native.
    fn capabilities(&self) -> PlatformCapabilities;
}
