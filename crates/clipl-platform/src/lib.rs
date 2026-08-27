//! Linux session probing, adapter selection, and clipboard backends.

#![forbid(unsafe_code)]

mod activation;
mod clipboard;
mod insert;

use std::env;

use clipl_core::{
    Capability, ClipboardConfig, DesktopEnvironment, Platform, PlatformAdapter,
    PlatformCapabilities, PlatformIdentity, SessionType, SupportLevel,
};

#[cfg(feature = "x11")]
pub use clipboard::X11Clipboard;
pub use clipboard::{
    select_clipboard_backend, GnomeClipboard, NullClipboard, SelectedClipboard,
    WaylandGenericClipboard,
};

pub use activation::{
    format_activation_report, gnome_extension_installed, gnome_extension_present_in,
    select_activation_backend, select_activation_backend_with, GenericWaylandActivation,
    GnomeActivation, HyprlandActivation, KdeActivation, NativeActivation, NullActivation,
    SelectedActivation, SwayActivation, WlrootsActivation,
};
#[cfg(feature = "x11")]
pub use insert::{restore_focus_and_ctrl_v, snapshot_input_focus};

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
        capabilities_for(&self.identity, &ClipboardConfig::default())
    }
}

/// Fill a capability matrix using identity plus a real clipboard probe.
pub fn capabilities_for(
    identity: &PlatformIdentity,
    clipboard: &ClipboardConfig,
) -> PlatformCapabilities {
    let selected = select_clipboard_backend(identity, clipboard);
    let mut caps = PlatformCapabilities::unknown(identity.clone());
    caps.set(Capability::LocalStorage, SupportLevel::Native);
    caps.set(Capability::Network, SupportLevel::Native);
    caps.set(Capability::ClipboardRead, selected.read);
    caps.set(Capability::ClipboardWatch, selected.watch);
    match identity.session {
        SessionType::X11 => {
            caps.set(Capability::ClipboardWrite, SupportLevel::Native);
            caps.set(Capability::GlobalHotkey, SupportLevel::Native);
            caps.set(Capability::InsertIntoApp, SupportLevel::Native);
        }
        SessionType::Wayland => match identity.desktop {
            DesktopEnvironment::Gnome => {
                caps.set(Capability::ClipboardWrite, SupportLevel::Fallback);
                caps.set(Capability::GlobalHotkey, SupportLevel::Portal);
                caps.set(Capability::InsertIntoApp, SupportLevel::Portal);
            }
            DesktopEnvironment::Sway
            | DesktopEnvironment::Hyprland
            | DesktopEnvironment::WlrootsGeneric => {
                caps.set(Capability::ClipboardWrite, SupportLevel::Unsupported);
                caps.set(Capability::GlobalHotkey, SupportLevel::Fallback);
                caps.set(Capability::InsertIntoApp, SupportLevel::Unsupported);
            }
            _ => {
                caps.set(Capability::ClipboardWrite, SupportLevel::Unsupported);
                caps.set(Capability::GlobalHotkey, SupportLevel::Unsupported);
                caps.set(Capability::InsertIntoApp, SupportLevel::Unsupported);
            }
        },
        _ => {
            caps.set(Capability::ClipboardWrite, SupportLevel::Unknown);
            caps.set(Capability::GlobalHotkey, SupportLevel::Unknown);
            caps.set(Capability::InsertIntoApp, SupportLevel::Unknown);
        }
    }
    caps.set(Capability::OverlayPopup, SupportLevel::Unknown);
    caps.set(Capability::ImagePaste, SupportLevel::Unsupported);
    caps.set(Capability::FilePaste, SupportLevel::Unknown);
    caps.set(Capability::PortalIntegration, SupportLevel::Unknown);

    match identity.desktop {
        DesktopEnvironment::Gnome => {
            caps.set(Capability::GnomeExtension, SupportLevel::Portal);
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

/// Named adapter slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterKind {
    /// Generic Linux (XDG probe only).
    LinuxGeneric,
    /// X11 session adapter.
    X11,
    /// Generic Wayland adapter (honest unsupported watch).
    WaylandGeneric,
    /// GNOME Shell adapter (extension boundary).
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
    /// Choose an adapter kind from identity without claiming watch works.
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

    /// Whether a backend slot has code (including honest Unsupported stubs).
    pub fn is_implemented(self) -> bool {
        matches!(
            self,
            Self::LinuxGeneric | Self::X11 | Self::WaylandGeneric | Self::Gnome
        )
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
        assert!(AdapterKind::Gnome.is_implemented());
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
    }

    #[test]
    fn wayland_watch_is_unsupported() {
        let identity = PlatformIdentity {
            platform: Platform::Linux,
            session: SessionType::Wayland,
            desktop: DesktopEnvironment::Unknown,
            xdg_current_desktop: None,
            xdg_session_type: Some("wayland".into()),
        };
        let caps = capabilities_for(&identity, &ClipboardConfig::default());
        assert_eq!(
            caps.level(Capability::ClipboardWatch),
            SupportLevel::Unsupported
        );
        assert_eq!(
            caps.level(Capability::GlobalHotkey),
            SupportLevel::Unsupported
        );
        assert_eq!(
            caps.level(Capability::InsertIntoApp),
            SupportLevel::Unsupported
        );
    }

    #[test]
    fn gnome_wayland_hotkey_is_portal() {
        let identity = PlatformIdentity {
            platform: Platform::Linux,
            session: SessionType::Wayland,
            desktop: DesktopEnvironment::Gnome,
            xdg_current_desktop: Some("GNOME".into()),
            xdg_session_type: Some("wayland".into()),
        };
        let caps = capabilities_for(&identity, &ClipboardConfig::default());
        assert_eq!(caps.level(Capability::GlobalHotkey), SupportLevel::Portal);
        assert_eq!(caps.level(Capability::GnomeExtension), SupportLevel::Portal);
        assert_eq!(caps.level(Capability::InsertIntoApp), SupportLevel::Portal);
        assert_eq!(caps.level(Capability::ClipboardWatch), SupportLevel::Portal);
    }

    #[test]
    fn x11_hotkey_is_native() {
        let identity = PlatformIdentity {
            platform: Platform::Linux,
            session: SessionType::X11,
            desktop: DesktopEnvironment::Unknown,
            xdg_current_desktop: None,
            xdg_session_type: Some("x11".into()),
        };
        let caps = capabilities_for(&identity, &ClipboardConfig::default());
        assert_eq!(caps.level(Capability::GlobalHotkey), SupportLevel::Native);
        assert_eq!(caps.level(Capability::InsertIntoApp), SupportLevel::Native);
    }
}
