//! Generic Wayland clipboard.
//!
//! There is no portable Wayland clipboard-watch protocol. This backend reports
//! `Unsupported` and does not poll `wl-paste`.

use clipl_core::{ClipboardBackend, ClipboardContent, Error, Result};

/// Honest generic Wayland stub.
#[derive(Debug, Default, Clone)]
pub struct WaylandGenericClipboard;

impl WaylandGenericClipboard {
    /// Construct the stub backend.
    pub fn new() -> Self {
        Self
    }
}

impl ClipboardBackend for WaylandGenericClipboard {
    fn name(&self) -> &'static str {
        "wayland-generic"
    }

    fn read(&self) -> Result<Option<ClipboardContent>> {
        Err(Error::unsupported(
            "generic Wayland cannot read the clipboard without a compositor protocol",
        ))
    }

    fn write(&self, _content: &ClipboardContent) -> Result<()> {
        Err(Error::unsupported(
            "generic Wayland cannot write the clipboard without a compositor protocol",
        ))
    }

    fn supports_watch(&self) -> bool {
        false
    }
}
