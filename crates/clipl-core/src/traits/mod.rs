//! Extension traits. Implementations belong in sibling crates, not here.

mod activation;
mod clipboard;
mod media;
mod platform;
mod storage;

pub use activation::ActivationBackend;
pub use clipboard::{ClipboardBackend, ClipboardWatcher};
pub use media::{MediaProvider, MediaQuery, StickerPackProvider};
pub use platform::PlatformAdapter;
pub use storage::StorageBackend;
