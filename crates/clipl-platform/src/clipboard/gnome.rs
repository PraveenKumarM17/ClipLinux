//! GNOME Wayland clipboard boundary.
//!
//! Mutter does not offer wlr-data-control to regular clients. Clipboard watch
//! requires a GNOME Shell extension (see `extensions/gnome`) or a portal that
//! does not exist for clipboard history. This backend does not inject keys or
//! scrape the Shell.

use clipl_core::{ClipboardBackend, ClipboardContent, Error, Result};

/// GNOME session adapter: watch is unsupported until the extension exists.
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
            "GNOME Wayland clipboard read requires a Shell extension",
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
