//! Clipboard backend selection. Wayland is not treated as X11.

mod gnome;
mod null;
mod wayland;

#[cfg(feature = "x11")]
mod x11;

use clipl_core::{
    ClipboardBackend, ClipboardConfig, DesktopEnvironment, PlatformIdentity, SessionType,
    SupportLevel,
};

pub use gnome::GnomeClipboard;
pub use null::NullClipboard;
pub use wayland::WaylandGenericClipboard;

#[cfg(feature = "x11")]
pub use x11::X11Clipboard;

/// A backend plus the honest watch support level.
pub struct SelectedClipboard {
    /// Live backend.
    pub backend: Box<dyn ClipboardBackend>,
    /// Backend id.
    pub name: &'static str,
    /// Clipboard-watch support.
    pub watch: SupportLevel,
    /// Clipboard-read support.
    pub read: SupportLevel,
    /// Human-readable reason (safe to print).
    pub reason: String,
}

/// Pick a clipboard backend from session identity.
///
/// Never uses an X11 connection on a Wayland session, even if `DISPLAY` is set
/// (XWayland).
pub fn select_clipboard_backend(
    identity: &PlatformIdentity,
    clipboard: &ClipboardConfig,
) -> SelectedClipboard {
    match identity.session {
        SessionType::X11 => select_x11(clipboard),
        SessionType::Wayland => match identity.desktop {
            DesktopEnvironment::Gnome => gnome_selection(),
            _ => wayland_generic_selection(),
        },
        SessionType::Unknown => SelectedClipboard {
            backend: Box::new(NullClipboard::new("unknown-session")),
            name: "none",
            watch: SupportLevel::Unknown,
            read: SupportLevel::Unknown,
            reason: "session type is unknown; clipboard watch not started".into(),
        },
        _ => SelectedClipboard {
            backend: Box::new(NullClipboard::new("unsupported-session")),
            name: "none",
            watch: SupportLevel::Unsupported,
            read: SupportLevel::Unsupported,
            reason: "unrecognized session type".into(),
        },
    }
}

fn select_x11(clipboard: &ClipboardConfig) -> SelectedClipboard {
    #[cfg(feature = "x11")]
    {
        match X11Clipboard::connect(&clipboard.selection) {
            Ok(backend) => SelectedClipboard {
                backend: Box::new(backend),
                name: "x11",
                watch: SupportLevel::Native,
                read: SupportLevel::Native,
                reason: "X11 CLIPBOARD via XFixes SelectionNotify".into(),
            },
            Err(err) => SelectedClipboard {
                backend: Box::new(NullClipboard::new("x11-unavailable")),
                name: "x11-unavailable",
                watch: SupportLevel::Unsupported,
                read: SupportLevel::Unsupported,
                reason: format!("X11 clipboard backend failed to connect: {err}"),
            },
        }
    }
    #[cfg(not(feature = "x11"))]
    {
        let _ = clipboard;
        SelectedClipboard {
            backend: Box::new(NullClipboard::new("x11-disabled")),
            name: "x11-disabled",
            watch: SupportLevel::Unsupported,
            read: SupportLevel::Unsupported,
            reason: "compiled without the `x11` feature".into(),
        }
    }
}

fn gnome_selection() -> SelectedClipboard {
    SelectedClipboard {
        backend: Box::new(GnomeClipboard::new()),
        name: "gnome",
        watch: SupportLevel::Unsupported,
        read: SupportLevel::Unsupported,
        reason: "GNOME on Wayland does not expose a generic clipboard watch API; a Shell extension is required (see extensions/gnome)".into(),
    }
}

fn wayland_generic_selection() -> SelectedClipboard {
    SelectedClipboard {
        backend: Box::new(WaylandGenericClipboard::new()),
        name: "wayland-generic",
        watch: SupportLevel::Unsupported,
        read: SupportLevel::Unsupported,
        reason: "generic Wayland has no clipboard-watch protocol; wlroots data-control and portals are future adapters".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::{DesktopEnvironment, Platform, SessionType};

    fn identity(session: SessionType, desktop: DesktopEnvironment) -> PlatformIdentity {
        PlatformIdentity {
            platform: Platform::Linux,
            session,
            desktop,
            xdg_current_desktop: None,
            xdg_session_type: None,
        }
    }

    #[test]
    fn wayland_is_not_x11() {
        let cfg = ClipboardConfig::default();
        let selected = select_clipboard_backend(
            &identity(SessionType::Wayland, DesktopEnvironment::Unknown),
            &cfg,
        );
        assert_eq!(selected.name, "wayland-generic");
        assert_eq!(selected.watch, SupportLevel::Unsupported);
        assert!(!selected.backend.supports_watch());
    }

    #[test]
    fn gnome_wayland_is_unsupported_watch() {
        let cfg = ClipboardConfig::default();
        let selected = select_clipboard_backend(
            &identity(SessionType::Wayland, DesktopEnvironment::Gnome),
            &cfg,
        );
        assert_eq!(selected.name, "gnome");
        assert_eq!(selected.watch, SupportLevel::Unsupported);
    }
}
