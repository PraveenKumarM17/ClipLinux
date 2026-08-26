//! Clipboard backend interface.

use crate::clipboard::ClipboardContent;
use crate::error::Result;

/// Reads and writes the system clipboard.
///
/// Implementations are platform-specific and live in `unipick-platform` or
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
}
