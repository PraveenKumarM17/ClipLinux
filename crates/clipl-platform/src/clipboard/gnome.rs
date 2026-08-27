//! GNOME Wayland clipboard boundary.
//!
//! Mutter does not offer wlr-data-control to regular clients. The Shell
//! extension watches CLIPBOARD (`Meta.Selection` owner-changed) and pushes
//! text to the daemon with `RecordClipboard`. This backend does not inject
//! keys or scrape the Shell from the daemon process.

use clipl_core::{ClipboardBackend, ClipboardContent, Error, Result};

/// GNOME session adapter: in-process watch stays off; the Shell extension pushes text.
#[derive(Debug, Default, Clone)]
pub struct GnomeClipboard;

impl GnomeClipboard {
    /// Construct the GNOME boundary backend.
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for GnomeClipboard {
    fn name(&self) -> &'static str {
        "gnome"
    }

    fn read(&self) -> Result<Option<ClipboardContent>> {
        Err(Error::unsupported(
            "GNOME Wayland clipboard read is the Shell extension; query history over IPC",
        ))
    }

    fn write(&self, _content: &ClipboardContent) -> Result<()> {
        Err(Error::unsupported(
            "GNOME Wayland clipboard write requires a Shell extension",
        ))
    }

    fn supports_watch(&self) -> bool {
        false
    }
}
