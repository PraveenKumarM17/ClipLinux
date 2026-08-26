//! Capability-based picker activation. Never treats Wayland as X11.

mod gnome;
mod hyprland;
mod kde;
mod null;
mod sway;
mod wayland;
mod wlroots;

#[cfg(feature = "x11")]
mod x11;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use clipl_core::{
    ActivationBackend, ActivationBehavior, ActivationCapability, ActivationConfig,
    ActivationSnapshot, DesktopEnvironment, PlatformIdentity, SessionType, Shortcut,
    GNOME_EXTENSION_UUID,
};

pub use gnome::GnomeActivation;
pub use hyprland::HyprlandActivation;
pub use kde::KdeActivation;
pub use null::NullActivation;
pub use sway::SwayActivation;
pub use wayland::GenericWaylandActivation;
pub use wlroots::WlrootsActivation;
#[cfg(feature = "x11")]
pub use x11::X11Activation;

/// Selected activation backend plus routing metadata.
pub struct SelectedActivation {
    /// Live backend (may hold an X11 connection).
    pub backend: Box<dyn NativeActivation>,
    /// Backend id for status output.
    pub name: &'static str,
    /// Configured chord.
    pub shortcut: Shortcut,
    /// Show vs toggle.
    pub behavior: ActivationBehavior,
    /// Probe snapshot (updated after native listen attempts).
    pub snapshot: ActivationSnapshot,
}

/// [`ActivationBackend`] plus optional native listen.
pub trait NativeActivation: ActivationBackend + Send {
    /// Take the native grab if this backend owns one. Default: no-op.
    fn arm(&mut self) -> clipl_core::Result<()> {
        Ok(())
    }

    /// Block until `shutdown`, invoking `on_fire` only for the armed shortcut.
    fn listen(&mut self, shutdown: &AtomicBool, on_fire: &dyn Fn()) -> clipl_core::Result<()> {
        let _ = (shutdown, on_fire);
        Ok(())
    }
}

/// Choose an activation backend from session identity.
///
/// **Never** uses an X11 grab on a Wayland session, even if `DISPLAY` is set.
pub fn select_activation_backend(
    identity: &PlatformIdentity,
    config: &ActivationConfig,
) -> SelectedActivation {
    select_activation_backend_with(identity, config, gnome_extension_installed())
}

/// Same as [`select_activation_backend`] with an explicit extension probe (tests).
pub fn select_activation_backend_with(
    identity: &PlatformIdentity,
    config: &ActivationConfig,
    gnome_extension_present: bool,
) -> SelectedActivation {
    let shortcut = config.shortcut().unwrap_or_default();
    let behavior = config.behavior().unwrap_or(ActivationBehavior::Toggle);

    if !config.enabled {
        return selected(
            Box::new(NullActivation::manual(
                shortcut.clone(),
                "activation.enabled is false; use clipl open / clipl toggle",
            )),
            shortcut,
            behavior,
        );
    }

    match identity.session {
        SessionType::X11 => select_x11_session(
            identity,
            config,
            shortcut,
            behavior,
            gnome_extension_present,
        ),
        SessionType::Wayland => select_wayland_session(
            identity,
            config,
            shortcut,
            behavior,
            gnome_extension_present,
        ),
        SessionType::Unknown => selected(
            Box::new(NullActivation::unknown(shortcut.clone())),
            shortcut,
            behavior,
        ),
        _ => selected(
            Box::new(NullActivation::unsupported(shortcut.clone())),
            shortcut,
            behavior,
        ),
    }
}

fn select_x11_session(
    identity: &PlatformIdentity,
    config: &ActivationConfig,
    shortcut: Shortcut,
    behavior: ActivationBehavior,
    gnome_extension_present: bool,
) -> SelectedActivation {
    match identity.desktop {
        DesktopEnvironment::Gnome if config.gnome.enabled => {
            // GNOME on X11 can use the extension; native X11 grab is also valid.
            // Prefer native X11 grab when enabled so the extension is not required.
            if config.x11.enabled {
                return selected_x11(shortcut, behavior);
            }
            selected(
                Box::new(GnomeActivation::new(
                    shortcut.clone(),
                    gnome_extension_present,
                )),
                shortcut,
                behavior,
            )
        }
        _ if config.x11.enabled => selected_x11(shortcut, behavior),
        _ => selected(
            Box::new(NullActivation::manual(
                shortcut.clone(),
                "X11 activation is disabled in config; use clipl open / clipl toggle",
            )),
            shortcut,
            behavior,
        ),
    }
}

fn select_wayland_session(
    identity: &PlatformIdentity,
    config: &ActivationConfig,
    shortcut: Shortcut,
    behavior: ActivationBehavior,
    gnome_extension_present: bool,
) -> SelectedActivation {
    match identity.desktop {
        DesktopEnvironment::Gnome if config.gnome.enabled => selected(
            Box::new(GnomeActivation::new(
                shortcut.clone(),
                gnome_extension_present,
            )),
            shortcut,
            behavior,
        ),
        DesktopEnvironment::Gnome => selected(
            Box::new(NullActivation::manual(
                shortcut.clone(),
                "GNOME activation is disabled in config; use clipl open / clipl toggle",
            )),
            shortcut,
            behavior,
        ),
        DesktopEnvironment::KdePlasma => selected(
            Box::new(KdeActivation::new(shortcut.clone())),
            shortcut,
            behavior,
        ),
        DesktopEnvironment::Sway => selected(
            Box::new(SwayActivation::new(shortcut.clone())),
            shortcut,
            behavior,
        ),
        DesktopEnvironment::Hyprland => selected(
            Box::new(HyprlandActivation::new(shortcut.clone())),
            shortcut,
            behavior,
        ),
        DesktopEnvironment::WlrootsGeneric => selected(
            Box::new(WlrootsActivation::new(shortcut.clone())),
            shortcut,
            behavior,
        ),
        _ => selected(
            Box::new(GenericWaylandActivation::new(shortcut.clone())),
            shortcut,
            behavior,
        ),
    }
}

fn selected_x11(shortcut: Shortcut, behavior: ActivationBehavior) -> SelectedActivation {
    #[cfg(feature = "x11")]
    {
        selected(
            Box::new(X11Activation::unbound(shortcut.clone())),
            shortcut,
            behavior,
        )
    }
    #[cfg(not(feature = "x11"))]
    {
        selected(
            Box::new(NullActivation::error(
                shortcut.clone(),
                "X11 activation was not compiled (clipl-platform built without the x11 feature)",
            )),
            shortcut,
            behavior,
        )
    }
}

fn selected(
    backend: Box<dyn NativeActivation>,
    shortcut: Shortcut,
    behavior: ActivationBehavior,
) -> SelectedActivation {
    let snapshot = backend.snapshot();
    SelectedActivation {
        name: backend.kind().as_str(),
        backend,
        shortcut,
        behavior,
        snapshot,
    }
}

/// Whether the ClipLinux GNOME extension appears installed for this user.
pub fn gnome_extension_installed() -> bool {
    gnome_extension_search_roots()
        .iter()
        .any(|root| gnome_extension_present_in(root))
}

/// Test helper: `XDG_DATA_HOME`-style root contains the extension.
pub fn gnome_extension_present_in(data_home: &Path) -> bool {
    data_home
        .join("gnome-shell/extensions")
        .join(GNOME_EXTENSION_UUID)
        .join("metadata.json")
        .is_file()
}

fn gnome_extension_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(data) = std::env::var("XDG_DATA_HOME") {
        roots.push(PathBuf::from(data));
    } else if let Ok(home) = std::env::var("HOME") {
        roots.push(PathBuf::from(home).join(".local/share"));
    }
    roots.push(PathBuf::from("/usr/share"));
    roots
}

/// Debounce helper so X11 auto-repeat does not toggle the picker rapidly.
pub fn should_fire(last: &mut Option<Instant>, now: Instant, window: Duration) -> bool {
    match *last {
        Some(prev) if now.duration_since(prev) < window => false,
        _ => {
            *last = Some(now);
            true
        }
    }
}

/// Format a doctor/status activation block (no secrets).
pub fn format_activation_report(
    identity: &PlatformIdentity,
    selected: &SelectedActivation,
    desktop_connected: bool,
) -> String {
    let alternatives = match selected.snapshot.capability {
        ActivationCapability::NativeGlobalShortcut => "clipl open / clipl toggle",
        ActivationCapability::DesktopManagedShortcut => {
            "Install the GNOME extension, or use clipl open / clipl toggle"
        }
        ActivationCapability::CompositorBinding => {
            "Bind the compositor key to `clipl toggle`, or use clipl open"
        }
        ActivationCapability::ManualOnly | ActivationCapability::Unsupported | _ => {
            "clipl open / clipl toggle / launch ClipLinux desktop"
        }
    };
    format!(
        "Activation\n\
         \n\
         Session: {:?}\n\
         Desktop: {:?}\n\
         \n\
         Global shortcut:\n\
           {}\n\
         \n\
         Backend:\n\
           {} ({})\n\
         \n\
         Status:\n\
           {}\n\
         \n\
         Desktop subscriber:\n\
           {}\n\
         \n\
         Reason:\n\
           {}\n\
         \n\
         Supported alternatives:\n\
           {}\n",
        identity.session,
        identity.desktop,
        selected.shortcut.display(),
        selected.snapshot.backend.as_str(),
        selected.snapshot.capability.as_str(),
        selected.snapshot.status.as_str(),
        if desktop_connected {
            "connected"
        } else {
            "not running"
        },
        selected.snapshot.reason,
        alternatives,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::{
        ActivationBackendKind, ActivationCapability, ActivationStatus, Platform, PlatformIdentity,
    };

    fn identity(session: SessionType, desktop: DesktopEnvironment) -> PlatformIdentity {
        PlatformIdentity {
            platform: Platform::Linux,
            session,
            desktop,
            xdg_current_desktop: None,
            xdg_session_type: Some(match session {
                SessionType::X11 => "x11".into(),
                SessionType::Wayland => "wayland".into(),
                SessionType::Unknown => "unknown".into(),
                _ => "unknown".into(),
            }),
        }
    }

    #[test]
    fn wayland_gnome_never_selects_x11() {
        let selected = select_activation_backend_with(
            &identity(SessionType::Wayland, DesktopEnvironment::Gnome),
            &ActivationConfig::default(),
            false,
        );
        assert_eq!(selected.snapshot.backend, ActivationBackendKind::GnomeShell);
        assert!(!selected.backend.supports_native_listen());
        assert_eq!(
            selected.snapshot.capability,
            ActivationCapability::DesktopManagedShortcut
        );
        assert_eq!(selected.snapshot.status, ActivationStatus::NotConfigured);
    }

    #[test]
    fn wayland_gnome_with_extension_is_external() {
        let selected = select_activation_backend_with(
            &identity(SessionType::Wayland, DesktopEnvironment::Gnome),
            &ActivationConfig::default(),
            true,
        );
        assert_eq!(
            selected.snapshot.status,
            ActivationStatus::ConfiguredExternally
        );
    }

    #[test]
    fn x11_selects_native_grab() {
        let selected = select_activation_backend_with(
            &identity(SessionType::X11, DesktopEnvironment::Unknown),
            &ActivationConfig::default(),
            false,
        );
        assert_eq!(selected.snapshot.backend, ActivationBackendKind::X11);
        assert_eq!(
            selected.snapshot.capability,
            ActivationCapability::NativeGlobalShortcut
        );
        assert!(selected.backend.supports_native_listen());
    }

    #[test]
    fn wayland_does_not_use_x11_when_x11_enabled() {
        let selected = select_activation_backend_with(
            &identity(SessionType::Wayland, DesktopEnvironment::Unknown),
            &ActivationConfig::default(),
            false,
        );
        assert_eq!(
            selected.snapshot.backend,
            ActivationBackendKind::GenericWayland
        );
        assert!(!selected.backend.supports_native_listen());
    }

    #[test]
    fn sway_is_compositor_binding_not_a_grab() {
        let selected = select_activation_backend_with(
            &identity(SessionType::Wayland, DesktopEnvironment::Sway),
            &ActivationConfig::default(),
            false,
        );
        assert_eq!(selected.snapshot.backend, ActivationBackendKind::Sway);
        assert_eq!(
            selected.snapshot.capability,
            ActivationCapability::CompositorBinding
        );
        assert!(!selected.backend.supports_native_listen());
    }

    #[test]
    fn kde_slot_is_planned() {
        let selected = select_activation_backend_with(
            &identity(SessionType::Wayland, DesktopEnvironment::KdePlasma),
            &ActivationConfig::default(),
            false,
        );
        assert_eq!(selected.snapshot.backend, ActivationBackendKind::KdePlasma);
        assert_eq!(selected.snapshot.status, ActivationStatus::Unsupported);
    }

    #[test]
    fn disabled_is_manual_only() {
        let config = ActivationConfig {
            enabled: false,
            ..ActivationConfig::default()
        };
        let selected = select_activation_backend_with(
            &identity(SessionType::X11, DesktopEnvironment::Unknown),
            &config,
            false,
        );
        assert_eq!(selected.snapshot.backend, ActivationBackendKind::Null);
        assert_eq!(
            selected.snapshot.capability,
            ActivationCapability::ManualOnly
        );
    }

    #[test]
    fn debounce_blocks_repeats() {
        let mut last = None;
        let t0 = Instant::now();
        assert!(should_fire(&mut last, t0, Duration::from_millis(200)));
        assert!(!should_fire(
            &mut last,
            t0 + Duration::from_millis(50),
            Duration::from_millis(200)
        ));
        assert!(should_fire(
            &mut last,
            t0 + Duration::from_millis(250),
            Duration::from_millis(200)
        ));
    }

    #[test]
    fn gnome_extension_probe_uses_data_home_layout() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!gnome_extension_present_in(tmp.path()));
        let meta = tmp
            .path()
            .join("gnome-shell/extensions")
            .join(GNOME_EXTENSION_UUID);
        std::fs::create_dir_all(&meta).unwrap();
        std::fs::write(meta.join("metadata.json"), "{}").unwrap();
        assert!(gnome_extension_present_in(tmp.path()));
    }
}
