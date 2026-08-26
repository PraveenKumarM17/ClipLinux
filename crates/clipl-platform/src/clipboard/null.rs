//! Backend used when monitoring is unavailable.

use clipl_core::{ClipboardBackend, ClipboardContent, Error, Result};

/// Always-empty clipboard. Watch is unsupported.
#[derive(Debug, Clone)]
pub struct NullClipboard {
    name: &'static str,
}

impl NullClipboard {
    /// Named stub (shown in diagnostics).
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

impl ClipboardBackend for NullClipboard {
    fn name(&self) -> &'static str {
        self.name
    }

    fn read(&self) -> Result<Option<ClipboardContent>> {
        Ok(None)
    }

    fn write(&self, _content: &ClipboardContent) -> Result<()> {
        Err(Error::unsupported("clipboard write is not available"))
    }

    fn supports_watch(&self) -> bool {
        false
    }
}
