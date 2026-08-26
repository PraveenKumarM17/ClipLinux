//! Capability-based picker activation.
//!
//! ClipLinux does not pretend a single global hotkey API exists on Linux.
//! Each session reports an honest [`ActivationCapability`] and status.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// GNOME Shell extension UUID (must match `extensions/gnome/metadata.json`).
pub const GNOME_EXTENSION_UUID: &str = "clipl@io.clipl";

/// Default activation shortcut (may conflict with desktop bindings).
pub const DEFAULT_SHORTCUT: &str = "Super+V";

/// How this session can activate the picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivationCapability {
    /// The application can register a real global key grab (X11).
    NativeGlobalShortcut,
    /// The desktop environment owns the shortcut (GNOME Shell extension).
    DesktopManagedShortcut,
    /// The compositor config file is the correct binding mechanism.
    CompositorBinding,
    /// No shortcut path; CLI / manual launch only.
    ManualOnly,
    /// Probed and unavailable.
    Unsupported,
}

impl ActivationCapability {
    /// Stable identifier for CLI and docs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeGlobalShortcut => "native-global-shortcut",
            Self::DesktopManagedShortcut => "desktop-managed-shortcut",
            Self::CompositorBinding => "compositor-binding",
            Self::ManualOnly => "manual-only",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Named activation backend slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivationBackendKind {
    /// X11 `XGrabKey` on the root window.
    X11,
    /// GNOME Shell extension.
    GnomeShell,
    /// KDE Plasma (not implemented in this phase).
    KdePlasma,
    /// Sway compositor bind (not implemented; user config).
    Sway,
    /// Hyprland compositor bind (not implemented; user config).
    Hyprland,
    /// Generic wlroots compositor.
    WlrootsGeneric,
    /// Generic Wayland with no adapter.
    GenericWayland,
    /// Disabled or unknown session.
    Null,
}

impl ActivationBackendKind {
    /// Stable identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::X11 => "x11",
            Self::GnomeShell => "gnome-shell",
            Self::KdePlasma => "kde-plasma",
            Self::Sway => "sway",
            Self::Hyprland => "hyprland",
            Self::WlrootsGeneric => "wlroots",
            Self::GenericWayland => "wayland-generic",
            Self::Null => "none",
        }
    }

    /// Whether this slot has a real implementation in this phase.
    pub fn implementation(self) -> ActivationSlotState {
        match self {
            Self::X11 | Self::GnomeShell | Self::Null => ActivationSlotState::Implemented,
            Self::KdePlasma | Self::Sway | Self::Hyprland | Self::WlrootsGeneric => {
                ActivationSlotState::Planned
            }
            Self::GenericWayland => ActivationSlotState::Unsupported,
        }
    }
}

/// Honesty marker for a backend slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivationSlotState {
    /// Code exists and is used when selected.
    Implemented,
    /// Named slot only; do not claim it works.
    Planned,
    /// Will not be implemented as an in-process grab.
    Unsupported,
}

/// Runtime status of the selected activation path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ActivationStatus {
    /// Native grab is registered and receiving the shortcut.
    Active,
    /// Shortcut is owned outside ClipLinux (extension or compositor).
    ConfiguredExternally,
    /// The path exists but is not set up (extension missing, grab not taken).
    NotConfigured,
    /// This session has no activation backend.
    Unsupported,
    /// Registration was attempted and failed.
    Error,
}

impl ActivationStatus {
    /// Stable identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::ConfiguredExternally => "configured-externally",
            Self::NotConfigured => "not-configured",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        }
    }
}

/// What should happen when the shortcut (or CLI) fires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivationRequest {
    /// Show the picker.
    ShowPicker,
    /// Hide the picker without quitting the desktop process.
    HidePicker,
    /// Show if hidden, hide if shown.
    TogglePicker,
}

impl ActivationRequest {
    /// Stable identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShowPicker => "show",
            Self::HidePicker => "hide",
            Self::TogglePicker => "toggle",
        }
    }
}

/// Configured shortcut behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivationBehavior {
    /// Always show.
    Show,
    /// Toggle visibility.
    Toggle,
}

impl ActivationBehavior {
    /// Parse `show` / `toggle`.
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "show" => Ok(Self::Show),
            "toggle" => Ok(Self::Toggle),
            other => Err(Error::Config(format!(
                "activation.behavior must be show or toggle (got {other})"
            ))),
        }
    }

    /// Request sent when the shortcut fires.
    pub fn request(self) -> ActivationRequest {
        match self {
            Self::Show => ActivationRequest::ShowPicker,
            Self::Toggle => ActivationRequest::TogglePicker,
        }
    }

    /// Config spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Show => "show",
            Self::Toggle => "toggle",
        }
    }
}

/// Parsed activation shortcut. Not a keylogger: one explicit chord only.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    /// Super / Win / Mod4.
    pub super_key: bool,
    /// Control.
    pub ctrl: bool,
    /// Alt / Mod1.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Normalized key name (`v`, `space`, `f1`).
    pub key: String,
}

impl Shortcut {
    /// Parse `Super+V`, `Ctrl+Alt+Space`, or GNOME `<Super>v`.
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(Error::Config("activation.shortcut is empty".into()));
        }
        if trimmed.starts_with('<') {
            return parse_gnome_accel(trimmed);
        }
        parse_plus_accel(trimmed)
    }

    /// Canonical display form, e.g. `Super+V`.
    pub fn display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if self.super_key {
            parts.push("Super".into());
        }
        if self.ctrl {
            parts.push("Ctrl".into());
        }
        if self.alt {
            parts.push("Alt".into());
        }
        if self.shift {
            parts.push("Shift".into());
        }
        parts.push(display_key(&self.key));
        parts.join("+")
    }

    /// GNOME `as` accelerator, e.g. `<Super>v`.
    pub fn to_gnome_binding(&self) -> String {
        let mut out = String::new();
        if self.super_key {
            out.push_str("<Super>");
        }
        if self.ctrl {
            out.push_str("<Ctrl>");
        }
        if self.alt {
            out.push_str("<Alt>");
        }
        if self.shift {
            out.push_str("<Shift>");
        }
        out.push_str(&self.key);
        out
    }

    /// At least one modifier is required so ClipLinux never grabs a bare key.
    pub fn has_modifier(&self) -> bool {
        self.super_key || self.ctrl || self.alt || self.shift
    }
}

impl Default for Shortcut {
    fn default() -> Self {
        Self::parse(DEFAULT_SHORTCUT).expect("default shortcut is valid")
    }
}

/// Probe snapshot used by doctor/status. No raw X11 state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationSnapshot {
    /// Selected backend slot.
    pub backend: ActivationBackendKind,
    /// Capability class.
    pub capability: ActivationCapability,
    /// Runtime status.
    pub status: ActivationStatus,
    /// Display form of the configured shortcut.
    pub shortcut: String,
    /// Safe-to-print explanation.
    pub reason: String,
}

impl Default for ActivationSnapshot {
    fn default() -> Self {
        Self {
            backend: ActivationBackendKind::Null,
            capability: ActivationCapability::ManualOnly,
            status: ActivationStatus::Unsupported,
            shortcut: DEFAULT_SHORTCUT.into(),
            reason: "activation not probed".into(),
        }
    }
}

fn parse_plus_accel(input: &str) -> Result<Shortcut> {
    let parts: Vec<&str> = input
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() < 2 {
        return Err(Error::Config(
            "activation shortcut must include a modifier and a key (e.g. Super+V)".into(),
        ));
    }
    let (key_raw, mods) = parts.split_last().expect("len >= 2");
    let mut shortcut = Shortcut {
        super_key: false,
        ctrl: false,
        alt: false,
        shift: false,
        key: normalize_key(key_raw)?,
    };
    for modifier in mods {
        apply_modifier(&mut shortcut, modifier)?;
    }
    if !shortcut.has_modifier() {
        return Err(Error::Config(
            "activation shortcut must include Super, Ctrl, Alt, or Shift".into(),
        ));
    }
    Ok(shortcut)
}

fn parse_gnome_accel(input: &str) -> Result<Shortcut> {
    let mut shortcut = Shortcut {
        super_key: false,
        ctrl: false,
        alt: false,
        shift: false,
        key: String::new(),
    };
    let mut rest = input;
    while rest.starts_with('<') {
        let Some(end) = rest.find('>') else {
            return Err(Error::Config(
                "activation shortcut has an unclosed GNOME modifier".into(),
            ));
        };
        let token = &rest[1..end];
        apply_modifier(&mut shortcut, token)?;
        rest = &rest[end + 1..];
    }
    shortcut.key = normalize_key(rest.trim())?;
    if !shortcut.has_modifier() {
        return Err(Error::Config(
            "activation shortcut must include Super, Ctrl, Alt, or Shift".into(),
        ));
    }
    Ok(shortcut)
}

fn apply_modifier(shortcut: &mut Shortcut, raw: &str) -> Result<()> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "super" | "win" | "meta" | "mod4" => shortcut.super_key = true,
        "ctrl" | "control" => shortcut.ctrl = true,
        "alt" | "mod1" | "option" => shortcut.alt = true,
        "shift" => shortcut.shift = true,
        other => {
            return Err(Error::Config(format!(
                "unknown activation modifier `{other}`"
            )));
        }
    }
    Ok(())
}

fn normalize_key(raw: &str) -> Result<String> {
    let key = raw.trim().to_ascii_lowercase();
    match key.as_str() {
        "" => Err(Error::Config("activation shortcut is missing a key".into())),
        "return" | "enter" => Ok("return".into()),
        "esc" => Ok("escape".into()),
        "pgup" => Ok("page_up".into()),
        "pgdn" | "pgdown" => Ok("page_down".into()),
        other if other.len() == 1 && other.as_bytes()[0].is_ascii_alphanumeric() => {
            Ok(other.to_string())
        }
        "space" | "tab" | "escape" | "backspace" | "delete" | "home" | "end" | "left" | "right"
        | "up" | "down" | "page_up" | "page_down" => Ok(key.clone()),
        other
            if other.len() >= 2
                && other.starts_with('f')
                && other[1..]
                    .parse::<u8>()
                    .is_ok_and(|n| (1..=12).contains(&n)) =>
        {
            Ok(other.to_string())
        }
        other => Err(Error::Config(format!(
            "unsupported activation key `{other}`"
        ))),
    }
}

fn display_key(key: &str) -> String {
    if key.len() == 1 {
        return key.to_ascii_uppercase();
    }
    match key {
        "return" => "Enter".into(),
        "escape" => "Escape".into(),
        "space" => "Space".into(),
        "page_up" => "PageUp".into(),
        "page_down" => "PageDown".into(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_super_v() {
        let shortcut = Shortcut::parse("Super+V").unwrap();
        assert!(shortcut.super_key);
        assert_eq!(shortcut.key, "v");
        assert_eq!(shortcut.display(), "Super+V");
        assert_eq!(shortcut.to_gnome_binding(), "<Super>v");
    }

    #[test]
    fn parses_gnome_form() {
        let shortcut = Shortcut::parse("<Super>v").unwrap();
        assert_eq!(shortcut.display(), "Super+V");
    }

    #[test]
    fn parses_ctrl_shift() {
        let shortcut = Shortcut::parse("Ctrl+Shift+space").unwrap();
        assert!(shortcut.ctrl && shortcut.shift);
        assert_eq!(shortcut.key, "space");
    }

    #[test]
    fn normalizes_case() {
        assert_eq!(
            Shortcut::parse("super+v").unwrap(),
            Shortcut::parse("SUPER+V").unwrap()
        );
    }

    #[test]
    fn rejects_bare_key() {
        assert!(Shortcut::parse("v").is_err());
        assert!(Shortcut::parse("Space").is_err());
    }

    #[test]
    fn rejects_unknown_modifier() {
        assert!(Shortcut::parse("Foo+V").is_err());
    }

    #[test]
    fn behavior_parse() {
        assert_eq!(
            ActivationBehavior::parse("toggle").unwrap().request(),
            ActivationRequest::TogglePicker
        );
        assert_eq!(
            ActivationBehavior::parse("show").unwrap().request(),
            ActivationRequest::ShowPicker
        );
        assert!(ActivationBehavior::parse("paste").is_err());
    }
}
