//! Insert picked text into the previously focused application.
//!
//! ClipLinux never types the payload. The desktop writes CLIPBOARD, then a
//! backend restores focus and sends **Ctrl+V** only.

#[cfg(feature = "x11")]
mod x11;

#[cfg(feature = "x11")]
pub use x11::{restore_focus_and_ctrl_v, snapshot_input_focus};
