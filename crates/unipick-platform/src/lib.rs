//! Linux session probing and adapter selection.
//!
//! This crate reads standard desktop environment variables and reports
//! capabilities honestly. It does **not** implement compositor-specific
//! clipboard protocols or input grabs.

#![forbid(unsafe_code)]

use std::env;

use unipick_core::{
    Capability, DesktopEnvironment, Platform, PlatformAdapter, PlatformCapabilities,
    PlatformIdentity, SessionType, SupportLevel,
};

/// Probe `XDG_*` variables. This is capability detection, not a protocol hack.
pub fn probe_identity_from_env() -> PlatformIdentity {
    let xdg_session_type = env::var("XDG_SESSION_TYPE").ok();
    let xdg_current_desktop = env::var("XDG_CURRENT_DESKTOP").ok();

    let session = xdg_session_type
        .as_deref()
        .map(SessionType::from_xdg)
        .unwrap_or(SessionType::Unknown);

    let desktop = xdg_current_desktop
        .as_deref()
        .map(DesktopEnvironment::from_xdg_current_desktop)
        .unwrap_or(DesktopEnvironment::Unknown);

    PlatformIdentity {
        platform: Platform::Linux,
        session,
        desktop,
        xdg_current_desktop,
        xdg_session_type,
    }
}

/// Generic Linux adapter used until a specialized adapter is selected.
#[derive(Debug, Clone)]
pub struct LinuxGenericAdapter {
    identity: PlatformIdentity,
}

impl LinuxGenericAdapter {
    /// Probe the current process environment.
    pub fn probe() -> Self {
        Self {
            identity: probe_identity_from_env(),
        }
    }

    /// Wrap an already-known identity (tests).
    pub fn with_identity(identity: PlatformIdentity) -> Self {
        Self { identity }
    }
}

impl Default for LinuxGenericAdapter {
    fn default() -> Self {
        Self::probe()
    }
}

impl PlatformAdapter for LinuxGenericAdapter {
    fn name(&self) -> &'static str {
        "linux-generic"
    }

    fn identity(&self) -> PlatformIdentity {
        self.identity.clone()
    }

    fn capabilities(&self) -> PlatformCapabilities {
        let mut caps = PlatformCapabilities::unknown(self.identity.clone());
        caps.set(Capability::LocalStorage, SupportLevel::Native);
        // Network is a host capability, not a compositor one.
        caps.set(Capability::Network, SupportLevel::Native);
        // Clipboard and overlay are unknown until a real adapter probes them.
        caps.set(Capability::ClipboardRead, SupportLevel::Unknown);
        caps.set(Capability::ClipboardWrite, SupportLevel::Unknown);
        caps.set(Capability::ClipboardWatch, SupportLevel::Unknown);
        caps.set(Capability::GlobalHotkey, SupportLevel::Unknown);
        caps.set(Capability::OverlayPopup, SupportLevel::Unknown);
        caps.set(Capability::ImagePaste, SupportLevel::Unknown);
        caps.set(Capability::FilePaste, SupportLevel::Unknown);
        caps.set(Capability::PortalIntegration, SupportLevel::Unknown);

        match self.identity.desktop {
            DesktopEnvironment::Gnome => {
                caps.set(Capability::GnomeExtension, SupportLevel::Unknown);
                caps.set(Capability::KdeIntegration, SupportLevel::Unsupported);
            }
            DesktopEnvironment::KdePlasma => {
                caps.set(Capability::KdeIntegration, SupportLevel::Unknown);
                caps.set(Capability::GnomeExtension, SupportLevel::Unsupported);
            }
            _ => {
                caps.set(Capability::GnomeExtension, SupportLevel::Unsupported);
                caps.set(Capability::KdeIntegration, SupportLevel::Unsupported);
            }
        }
        caps
    }
}

/// Named adapter slots. Implementations are placeholders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterKind {
    /// Generic Linux (XDG probe only).
    LinuxGeneric,
    /// X11 session adapter (not implemented).
    X11,
    /// Generic Wayland adapter (not implemented).
    WaylandGeneric,
    /// GNOME Shell adapter (not implemented).
    Gnome,
    /// KDE Plasma adapter (not implemented).
    Kde,
    /// wlroots family adapter (not implemented).
    Wlroots,
    /// Future Hyprland adapter.
    Hyprland,
    /// Future Sway adapter.
    Sway,
}

impl AdapterKind {
    /// Choose an adapter kind from identity without claiming it is implemented.
    pub fn preferred(identity: &PlatformIdentity) -> Self {
        match (&identity.desktop, identity.session) {
            (DesktopEnvironment::Gnome, _) => Self::Gnome,
            (DesktopEnvironment::KdePlasma, _) => Self::Kde,
            (DesktopEnvironment::Hyprland, _) => Self::Hyprland,
            (DesktopEnvironment::Sway, _) => Self::Sway,
            (DesktopEnvironment::WlrootsGeneric, SessionType::Wayland) => Self::Wlroots,
            (_, SessionType::X11) => Self::X11,
            (_, SessionType::Wayland) => Self::WaylandGeneric,
            _ => Self::LinuxGeneric,
        }
    }

    /// Whether a real adapter exists. Foundation: only generic Linux.
    pub fn is_implemented(self) -> bool {
        matches!(self, Self::LinuxGeneric)
    }

    /// Identifier used in CLI output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LinuxGeneric => "linux-generic",
            Self::X11 => "x11",
            Self::WaylandGeneric => "wayland-generic",
            Self::Gnome => "gnome",
            Self::Kde => "kde",
            Self::Wlroots => "wlroots",
            Self::Hyprland => "hyprland",
            Self::Sway => "sway",
        }
    }
}

/// Select the adapter that is safe to construct today.
pub fn select_adapter() -> LinuxGenericAdapter {
    LinuxGenericAdapter::probe()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnome_on_wayland_prefers_gnome_adapter() {
        let identity = PlatformIdentity {
            platform: Platform::Linux,
            session: SessionType::Wayland,
            desktop: DesktopEnvironment::Gnome,
            xdg_current_desktop: Some("GNOME".into()),
            xdg_session_type: Some("wayland".into()),
        };
        assert_eq!(AdapterKind::preferred(&identity), AdapterKind::Gnome);
        assert!(!AdapterKind::Gnome.is_implemented());
    }

    #[test]
    fn x11_without_known_de_prefers_x11() {
        let identity = PlatformIdentity {
            platform: Platform::Linux,
            session: SessionType::X11,
            desktop: DesktopEnvironment::Unknown,
            xdg_current_desktop: None,
            xdg_session_type: Some("x11".into()),
        };
        assert_eq!(AdapterKind::preferred(&identity), AdapterKind::X11);
    }

    #[test]
    fn generic_adapter_marks_storage_native() {
        let adapter = LinuxGenericAdapter::with_identity(PlatformIdentity::unknown_linux());
        let caps = adapter.capabilities();
        assert_eq!(caps.level(Capability::LocalStorage), SupportLevel::Native);
        assert_eq!(
            caps.level(Capability::ClipboardWatch),
            SupportLevel::Unknown
        );
    }
}
