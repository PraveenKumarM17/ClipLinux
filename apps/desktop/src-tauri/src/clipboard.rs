//! OS clipboard write used after a successful `CopyItem` IPC round-trip.

use clipl_core::Result;

/// Sink for text the user asked to copy from history.
pub trait ClipboardWriter: Send + Sync {
    /// Replace the current CLIPBOARD selection with `text`.
    fn write_text(&self, text: &str) -> Result<()>;
}

/// Test sink that records writes and never touches the host clipboard.
#[allow(dead_code)]
#[derive(Default)]
pub struct RecordingClipboard {
    /// Written payloads, newest last.
    pub writes: std::sync::Mutex<Vec<String>>,
}

impl ClipboardWriter for RecordingClipboard {
    fn write_text(&self, text: &str) -> Result<()> {
        self.writes
            .lock()
            .map_err(|_| clipl_core::Error::Clipboard("lock poisoned".into()))?
            .push(text.to_string());
        Ok(())
    }
}

/// No-op writer used when the Tauri shell is not linked.
#[allow(dead_code)]
pub struct NullClipboard;

impl ClipboardWriter for NullClipboard {
    fn write_text(&self, _text: &str) -> Result<()> {
        Err(clipl_core::Error::unsupported(
            "clipboard write requires the Tauri desktop shell",
        ))
    }
}

/// Host clipboard via `arboard` (X11 / wlroots data-control). GNOME Wayland
/// may fail; the UI surfaces that error instead of injecting keys.
#[cfg(feature = "tauri-app")]
pub struct SystemClipboard;

#[cfg(feature = "tauri-app")]
impl ClipboardWriter for SystemClipboard {
    fn write_text(&self, text: &str) -> Result<()> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|err| clipl_core::Error::Clipboard(err.to_string()))?;
        clipboard
            .set_text(text)
            .map_err(|err| clipl_core::Error::Clipboard(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_writer_does_not_touch_host() {
        let err = NullClipboard.write_text("secret").unwrap_err();
        assert!(err.to_string().contains("Tauri"));
    }
}
