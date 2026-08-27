//! Local ClipLinux configuration. Loaded from TOML by the daemon; core only
//! defines the shape and defaults.

use serde::{Deserialize, Serialize};

use crate::activation::{ActivationBehavior, Shortcut};
use crate::error::{Error, Result};

/// Top-level configuration file (`config.toml`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ClipLinuxConfig {
    /// History persistence.
    #[serde(default)]
    pub history: HistoryConfig,
    /// Privacy engine.
    #[serde(default)]
    pub privacy: PrivacyConfig,
    /// Clipboard capture.
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    /// Picker activation (shortcuts / desktop integration).
    #[serde(default)]
    pub activation: ActivationConfig,
    /// Insert picked text into the previously focused application.
    #[serde(default)]
    pub insert: InsertConfig,
}

impl ClipLinuxConfig {
    /// Parse TOML, applying defaults for missing tables.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        let cfg: Self = toml::from_str(s).map_err(|err| Error::Config(err.to_string()))?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Reject values that would make the daemon misbehave.
    pub fn validate(&self) -> Result<()> {
        if self.history.max_items == 0 {
            return Err(Error::Config("history.max_items must be at least 1".into()));
        }
        if self.history.max_items > 100_000 {
            return Err(Error::Config(
                "history.max_items is unreasonably large (max 100000)".into(),
            ));
        }
        match self.clipboard.selection.as_str() {
            "clipboard" | "primary" | "both" => {}
            other => {
                return Err(Error::Config(format!(
                    "clipboard.selection must be clipboard, primary, or both (got {other})"
                )));
            }
        }
        match self.clipboard.deduplication_policy.as_str() {
            "consecutive" | "none" => {}
            other => {
                return Err(Error::Config(format!(
                    "clipboard.deduplication_policy must be consecutive or none (got {other})"
                )));
            }
        }
        Shortcut::parse(&self.activation.shortcut)?;
        ActivationBehavior::parse(&self.activation.behavior)?;
        Ok(())
    }
}

/// Clipboard history retention.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// When false, monitoring still runs but nothing is persisted.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Maximum unpinned items retained.
    #[serde(default = "default_max_items")]
    pub max_items: u32,
    /// Drop unpinned items older than this many days. `0` disables age expiry.
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u32,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_items: default_max_items(),
            max_age_days: default_max_age_days(),
        }
    }
}

/// Privacy engine toggles. Detectors stay conservative even when enabled.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Master switch. When false, detectors and rules are skipped.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Exclude PEM / OpenSSH private key material.
    #[serde(default = "default_true")]
    pub block_private_keys: bool,
    /// Exclude Luhn-valid card candidates.
    #[serde(default = "default_true")]
    pub block_credit_cards: bool,
    /// Exclude high-confidence API tokens and JWTs.
    #[serde(default = "default_true")]
    pub block_high_confidence_tokens: bool,
    /// Exclude whole-clipboard OTP-shaped numbers.
    #[serde(default = "default_true")]
    pub block_otp: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_private_keys: true,
            block_credit_cards: true,
            block_high_confidence_tokens: true,
            block_otp: true,
        }
    }
}

/// Clipboard capture behaviour.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClipboardConfig {
    /// X11 selection: `clipboard` (Ctrl+C), `primary` (mouse), or `both`.
    /// Ignored on Wayland. Default is `clipboard`.
    #[serde(default = "default_selection")]
    pub selection: String,
    /// `consecutive` reuses the latest row when the same text is copied again
    /// immediately. `none` always inserts (privacy still applies).
    #[serde(default = "default_dedup")]
    pub deduplication_policy: String,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            selection: default_selection(),
            deduplication_policy: default_dedup(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_max_items() -> u32 {
    500
}

fn default_max_age_days() -> u32 {
    30
}

fn default_selection() -> String {
    "clipboard".into()
}

fn default_dedup() -> String {
    "consecutive".into()
}

fn default_shortcut() -> String {
    crate::activation::DEFAULT_SHORTCUT.into()
}

fn default_behavior() -> String {
    "toggle".into()
}

/// Picker activation. Backends do not all own the same shortcut registration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivationConfig {
    /// Master switch. When false, no native grab is attempted.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Preferred chord (`Super+V`). GNOME may own a separate GSettings key.
    #[serde(default = "default_shortcut")]
    pub shortcut: String,
    /// `toggle` or `show`.
    #[serde(default = "default_behavior")]
    pub behavior: String,
    /// X11 native grab. Ignored on Wayland even if `DISPLAY` is set.
    #[serde(default)]
    pub x11: ActivationX11Config,
    /// GNOME Shell extension integration.
    #[serde(default)]
    pub gnome: ActivationGnomeConfig,
}

impl Default for ActivationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shortcut: default_shortcut(),
            behavior: default_behavior(),
            x11: ActivationX11Config::default(),
            gnome: ActivationGnomeConfig::default(),
        }
    }
}

impl ActivationConfig {
    /// Parsed shortcut.
    pub fn shortcut(&self) -> Result<Shortcut> {
        Shortcut::parse(&self.shortcut)
    }

    /// Parsed behaviour.
    pub fn behavior(&self) -> Result<ActivationBehavior> {
        ActivationBehavior::parse(&self.behavior)
    }
}

/// X11-specific activation switches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivationX11Config {
    /// Register `XGrabKey` when the session is X11.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ActivationX11Config {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// GNOME-specific activation switches.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActivationGnomeConfig {
    /// Use the Shell extension path on GNOME Wayland / GNOME sessions.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for ActivationGnomeConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// Insert picked text into the app that had focus before the palette opened.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InsertConfig {
    /// After a pick: restore that app and send Ctrl+V. Never types the payload.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for InsertConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_validate() {
        ClipLinuxConfig::default().validate().unwrap();
    }

    #[test]
    fn rejects_zero_max_items() {
        let toml = "[history]\nmax_items = 0\n";
        let err = ClipLinuxConfig::from_toml_str(toml).unwrap_err();
        assert!(err.to_string().contains("max_items"));
    }

    #[test]
    fn rejects_unknown_selection() {
        let toml = "[clipboard]\nselection = \"middle\"\n";
        assert!(ClipLinuxConfig::from_toml_str(toml).is_err());
    }

    #[test]
    fn rejects_bare_activation_shortcut() {
        let toml = "[activation]\nshortcut = \"v\"\n";
        assert!(ClipLinuxConfig::from_toml_str(toml).is_err());
    }

    #[test]
    fn accepts_activation_table() {
        let toml = "[activation]\nshortcut = \"Ctrl+Shift+space\"\nbehavior = \"show\"\n";
        let cfg = ClipLinuxConfig::from_toml_str(toml).unwrap();
        assert_eq!(cfg.activation.behavior, "show");
        assert_eq!(cfg.activation.shortcut().unwrap().key, "space");
    }
}
