//! Shared error type for ClipLinux crates.

use serde::{Deserialize, Serialize};

/// Result alias using the ClipLinux core error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Recoverable failure originating in domain logic or a backend implementation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Error {
    /// The requested operation is not available on this backend or platform.
    #[error("unsupported: {0}")]
    Unsupported(String),
    /// A referenced entity does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// Stored or provided data could not be interpreted.
    #[error("invalid data: {0}")]
    Invalid(String),
    /// Persistence backend failure.
    #[error("storage: {0}")]
    Storage(String),
    /// Clipboard backend failure.
    #[error("clipboard: {0}")]
    Clipboard(String),
    /// Media provider failure.
    #[error("media: {0}")]
    Media(String),
    /// Privacy policy rejected the operation.
    #[error("privacy: {0}")]
    Privacy(String),
    /// Protocol or IPC failure.
    #[error("protocol: {0}")]
    Protocol(String),
    /// Configuration file or value is invalid.
    #[error("config: {0}")]
    Config(String),
    /// Local I/O failure (socket, files, directories).
    #[error("io: {0}")]
    Io(String),
    /// Catch-all for backend-specific messages that have no richer mapping yet.
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// Convenience constructor for [`Error::Unsupported`].
    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    /// Convenience constructor for [`Error::NotFound`].
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// Convenience constructor for [`Error::Invalid`].
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }
}
