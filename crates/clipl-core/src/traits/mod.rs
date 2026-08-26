//! Extension traits. Implementations belong in sibling crates, not here.

mod clipboard;
mod media;
mod platform;
mod storage;

pub use clipboard::{ClipboardBackend, ClipboardWatcher};
pub use media::{MediaProvider, MediaQuery, StickerPackProvider};
pub use platform::PlatformAdapter;
pub use storage::StorageBackend;
