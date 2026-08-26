//! Host platform identity. Detection must not assume Wayland equals X11.

use serde::{Deserialize, Serialize};

/// Operating system ClipLinux is running on.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Platform {
    /// Linux and Linux-based systems, including most *nix desktops ClipLinux targets.
    Linux,
    /// Recognized but not a supported ClipLinux host.
    Unsupported { name: String },
    /// Could not be determined.
    Unknown,
}

/// Display server / session protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SessionType {
    /// X11 session.
    X11,
    /// Wayland session.
    Wayland,
    /// Could not be determined.
    Unknown,
}

/// Desktop environment or compositor family.
///
/// Values are coarse on purpose. Hyprland and Sway are listed separately from
/// generic wlroots so adapters can opt in without pretending they are identical.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DesktopEnvironment {
    /// GNOME Shell.
    Gnome,
    /// KDE Plasma.
    KdePlasma,
    /// COSMIC.
    Cosmic,
    /// Hyprland (future first-class adapter).
    Hyprland,
    /// Sway (future first-class adapter).
    Sway,
    /// Other wlroots-based compositor.
    WlrootsGeneric,
    /// Other named environment.
    Other(String),
    /// Could not be determined.
    Unknown,
}

impl DesktopEnvironment {
    /// Parse a value from `XDG_CURRENT_DESKTOP`-style tokens.
    pub fn from_xdg_token(token: &str) -> Self {
        match token.trim().to_ascii_lowercase().as_str() {
            "gnome" => Self::Gnome,
            "kde" | "plasma" => Self::KdePlasma,
            "cosmic" => Self::Cosmic,
            "hyprland" => Self::Hyprland,
            "sway" => Self::Sway,
            "wlroots" => Self::WlrootsGeneric,
            "" => Self::Unknown,
            other => Self::Other(other.to_string()),
        }
    }

    /// Parse the full `XDG_CURRENT_DESKTOP` value (colon-separated).
    ///
    /// Distro prefixes such as `ubuntu` or `pop` are ignored when a known
    /// environment appears later in the list (`ubuntu:GNOME` → GNOME).
    pub fn from_xdg_current_desktop(value: &str) -> Self {
        let tokens: Vec<Self> = value
            .split(':')
            .map(Self::from_xdg_token)
            .filter(|de| !matches!(de, Self::Unknown))
            .collect();
        tokens
            .iter()
            .find(|de| !matches!(de, Self::Other(_)))
            .cloned()
            .or_else(|| tokens.into_iter().next())
            .unwrap_or(Self::Unknown)
    }
}

impl SessionType {
    /// Parse `XDG_SESSION_TYPE`.
    pub fn from_xdg(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "x11" => Self::X11,
            "wayland" => Self::Wayland,
            _ => Self::Unknown,
        }
    }
}

/// Snapshot of the current desktop session.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformIdentity {
    /// Host OS.
    pub platform: Platform,
    /// Display protocol.
    pub session: SessionType,
    /// Desktop environment or compositor.
    pub desktop: DesktopEnvironment,
    /// Raw `XDG_CURRENT_DESKTOP` value, if present.
    pub xdg_current_desktop: Option<String>,
    /// Raw `XDG_SESSION_TYPE` value, if present.
    pub xdg_session_type: Option<String>,
}

impl PlatformIdentity {
    /// Unknown Linux identity used when probing is unavailable.
    pub fn unknown_linux() -> Self {
        Self {
            platform: Platform::Linux,
            session: SessionType::Unknown,
            desktop: DesktopEnvironment::Unknown,
            xdg_current_desktop: None,
            xdg_session_type: None,
        }
    }
}

impl Default for PlatformIdentity {
    fn default() -> Self {
        Self::unknown_linux()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ubuntu_gnome_is_gnome() {
        assert_eq!(
            DesktopEnvironment::from_xdg_current_desktop("ubuntu:GNOME"),
            DesktopEnvironment::Gnome
        );
    }

    #[test]
    fn pop_gnome_is_gnome() {
        assert_eq!(
            DesktopEnvironment::from_xdg_current_desktop("pop:GNOME"),
            DesktopEnvironment::Gnome
        );
    }
}
