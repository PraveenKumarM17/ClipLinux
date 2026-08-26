//! Capability detection model.
//!
//! ClipLinux must query what a session can actually do instead of branching on
//! compositor names with implicit assumptions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::platform::{DesktopEnvironment, Platform, PlatformIdentity, SessionType};

/// A discrete desktop capability ClipLinux may need.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Capability {
    /// Read the current clipboard contents.
    ClipboardRead,
    /// Write the clipboard contents.
    ClipboardWrite,
    /// Observe clipboard changes without polling hacks.
    ClipboardWatch,
    /// Register a global hotkey to open the palette.
    GlobalHotkey,
    /// Show a compact overlay popup over other windows.
    OverlayPopup,
    /// Paste images, not only text.
    ImagePaste,
    /// Paste file URIs.
    FilePaste,
    /// Use xdg-desktop-portal for clipboard or screenshots.
    PortalIntegration,
    /// GNOME Shell extension integration.
    GnomeExtension,
    /// KDE Plasma widget / runner integration.
    KdeIntegration,
    /// Access to a persistent local data directory.
    LocalStorage,
    /// Network access for remote media providers.
    Network,
}

impl Capability {
    /// All capabilities known to this version of ClipLinux.
    pub fn all() -> &'static [Capability] {
        &[
            Self::ClipboardRead,
            Self::ClipboardWrite,
            Self::ClipboardWatch,
            Self::GlobalHotkey,
            Self::OverlayPopup,
            Self::ImagePaste,
            Self::FilePaste,
            Self::PortalIntegration,
            Self::GnomeExtension,
            Self::KdeIntegration,
            Self::LocalStorage,
            Self::Network,
        ]
    }

    /// Stable identifier used in docs and CLI output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClipboardRead => "clipboard-read",
            Self::ClipboardWrite => "clipboard-write",
            Self::ClipboardWatch => "clipboard-watch",
            Self::GlobalHotkey => "global-hotkey",
            Self::OverlayPopup => "overlay-popup",
            Self::ImagePaste => "image-paste",
            Self::FilePaste => "file-paste",
            Self::PortalIntegration => "portal-integration",
            Self::GnomeExtension => "gnome-extension",
            Self::KdeIntegration => "kde-integration",
            Self::LocalStorage => "local-storage",
            Self::Network => "network",
        }
    }
}

/// How well a capability is supported in the current session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SupportLevel {
    /// First-class protocol or toolkit support.
    Native,
    /// Available through a desktop portal or extension.
    Portal,
    /// Degraded path that is documented and tested, not a hidden hack.
    Fallback,
    /// Confirmed unavailable.
    Unsupported,
    /// Not probed yet, or the probe is inconclusive.
    Unknown,
}

impl SupportLevel {
    /// Whether ClipLinux may attempt the capability.
    pub fn is_usable(self) -> bool {
        matches!(self, Self::Native | Self::Portal | Self::Fallback)
    }
}

/// Full capability matrix for a session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// Identity of the probed session.
    pub identity: PlatformIdentity,
    /// Per-capability support levels.
    pub levels: BTreeMap<Capability, SupportLevel>,
}

impl PlatformCapabilities {
    /// Matrix filled with [`SupportLevel::Unknown`] for every known capability.
    pub fn unknown(identity: PlatformIdentity) -> Self {
        let levels = Capability::all()
            .iter()
            .copied()
            .map(|cap| (cap, SupportLevel::Unknown))
            .collect();
        Self { identity, levels }
    }

    /// Conservative Linux defaults: local storage is native; everything else unknown.
    pub fn conservative_linux() -> Self {
        let mut caps = Self::unknown(PlatformIdentity::unknown_linux());
        caps.set(Capability::LocalStorage, SupportLevel::Native);
        caps
    }

    /// Set a capability level.
    pub fn set(&mut self, capability: Capability, level: SupportLevel) {
        self.levels.insert(capability, level);
    }

    /// Read a capability level, defaulting to unknown.
    pub fn level(&self, capability: Capability) -> SupportLevel {
        self.levels
            .get(&capability)
            .copied()
            .unwrap_or(SupportLevel::Unknown)
    }

    /// Whether the capability may be used.
    pub fn is_usable(&self, capability: Capability) -> bool {
        self.level(capability).is_usable()
    }

    /// Convenience accessors used by docs and CLI.
    pub fn platform(&self) -> &Platform {
        &self.identity.platform
    }

    /// Session type accessor.
    pub fn session(&self) -> SessionType {
        self.identity.session
    }

    /// Desktop environment accessor.
    pub fn desktop(&self) -> &DesktopEnvironment {
        &self.identity.desktop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_matrix_covers_every_capability() {
        let caps = PlatformCapabilities::unknown(PlatformIdentity::unknown_linux());
        for cap in Capability::all() {
            assert_eq!(caps.level(*cap), SupportLevel::Unknown);
        }
    }

    #[test]
    fn native_is_usable_unsupported_is_not() {
        assert!(SupportLevel::Native.is_usable());
        assert!(SupportLevel::Portal.is_usable());
        assert!(SupportLevel::Fallback.is_usable());
        assert!(!SupportLevel::Unsupported.is_usable());
        assert!(!SupportLevel::Unknown.is_usable());
    }
}
