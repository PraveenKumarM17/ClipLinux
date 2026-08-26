//! Clipboard backend interface.

use std::time::Duration;

use crate::clipboard::ClipboardContent;
use crate::error::{Error, Result};

/// Blocking observer for clipboard changes.
///
/// Implementations must wait on a documented event source (XFixes, a test
/// condvar, …). They must not busy-poll `xclip` or `wl-paste`.
pub trait ClipboardWatcher: Send {
    /// Wait up to `timeout` for a new clipboard value.
    ///
    /// Returns `Ok(None)` on timeout so the caller can check shutdown flags.
    fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ClipboardContent>>;
}

/// Reads and writes the system clipboard.
///
/// Implementations are platform-specific and live in `clipl-platform` or
/// compositor adapters. This trait must not mention X11, Wayland, or GTK.
pub trait ClipboardBackend: Send + Sync {
    /// Backend identifier, e.g. `x11`, `wayland-wlr`, `portal`.
    fn name(&self) -> &'static str;

    /// Read the current clipboard, if anything is available.
    fn read(&self) -> Result<Option<ClipboardContent>>;

    /// Replace the clipboard contents.
    fn write(&self, content: &ClipboardContent) -> Result<()>;

    /// Whether this backend can observe changes without undocumented polling.
    fn supports_watch(&self) -> bool {
        false
    }

    /// Whether image payloads can be written.
    fn supports_images(&self) -> bool {
        false
    }

    /// Subscribe to clipboard changes. Default: unsupported.
    fn watch(&self) -> Result<Box<dyn ClipboardWatcher>> {
        Err(Error::unsupported(format!(
            "{} does not support clipboard watch",
            self.name()
        )))
    }
}
