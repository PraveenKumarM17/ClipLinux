//! X11 restore-focus + Ctrl+V. Does not synthesize the clipboard payload.

use std::sync::atomic::AtomicU32;
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, InputFocus, Keycode, Window};
use x11rb::protocol::xtest::ConnectionExt as XTestExt;
use x11rb::rust_connection::RustConnection;

use clipl_core::{Error, Result};

const XK_CONTROL_L: u32 = 0xffe3;
const XK_V: u32 = 0x0076;
const KEY_PRESS: u8 = 2;
const KEY_RELEASE: u8 = 3;

/// Snapshot the X11 input focus into `slot` (0 means none / root).
pub fn snapshot_input_focus(conn: &RustConnection, root: Window, slot: &AtomicU32) {
    if let Ok(cookie) = conn.get_input_focus() {
        if let Ok(reply) = cookie.reply() {
            let focus = reply.focus;
            if focus > 1 && focus != root && !window_looks_like_clipl(conn, focus) {
                slot.store(focus, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}

/// Focus `window` and send Control+V via XTEST. Never types the payload.
pub fn restore_focus_and_ctrl_v(window: u32) -> Result<()> {
    if window <= 1 {
        return Err(Error::unsupported("no previous X11 window to insert into"));
    }
    let (conn, screen_num) = x11rb::connect(None)
        .map_err(|err| Error::Activation(format!("X11 insert connect failed: {err}")))?;
    let root = conn.setup().roots[screen_num].root;
    conn.set_input_focus(InputFocus::PARENT, window, 0u32)
        .map_err(|err| Error::Activation(format!("XSetInputFocus: {err}")))?
        .check()
        .map_err(|err| Error::Activation(format!("XSetInputFocus: {err}")))?;
    conn.flush()
        .map_err(|err| Error::Activation(err.to_string()))?;
    thread::sleep(Duration::from_millis(80));

    let ctrl = keycode_for_keysym(&conn, XK_CONTROL_L)?;
    let v = keycode_for_keysym(&conn, XK_V)?;
    fake_key(&conn, KEY_PRESS, ctrl, root)?;
    fake_key(&conn, KEY_PRESS, v, root)?;
    fake_key(&conn, KEY_RELEASE, v, root)?;
    fake_key(&conn, KEY_RELEASE, ctrl, root)?;
    conn.flush()
        .map_err(|err| Error::Activation(err.to_string()))?;
    Ok(())
}

fn window_looks_like_clipl(conn: &RustConnection, window: Window) -> bool {
    let Ok(atom) = conn.intern_atom(false, b"WM_CLASS") else {
        return false;
    };
    let Ok(atom) = atom.reply() else {
        return false;
    };
    let Ok(prop) = conn.get_property(false, window, atom.atom, AtomEnum::STRING, 0, 256) else {
        return false;
    };
    let Ok(prop) = prop.reply() else {
        return false;
    };
    String::from_utf8_lossy(&prop.value)
        .to_ascii_lowercase()
        .contains("clipl")
}

fn fake_key(conn: &RustConnection, event: u8, keycode: Keycode, root: Window) -> Result<()> {
    conn.xtest_fake_input(event, keycode, 0, root, 0, 0, 0)
        .map_err(|err| Error::Activation(format!("XTest: {err}")))?
        .check()
        .map_err(|err| Error::Activation(format!("XTest: {err}")))?;
    Ok(())
}

fn keycode_for_keysym(conn: &RustConnection, keysym: u32) -> Result<Keycode> {
    let setup = conn.setup();
    let min = setup.min_keycode;
    let max = setup.max_keycode;
    let count = max.saturating_sub(min).saturating_add(1);
    let reply = conn
        .get_keyboard_mapping(min, count)
        .map_err(|err| Error::Activation(err.to_string()))?
        .reply()
        .map_err(|err| Error::Activation(err.to_string()))?;
    let per = reply.keysyms_per_keycode as usize;
    if per == 0 {
        return Err(Error::Activation("X11 keyboard mapping is empty".into()));
    }
    for (index, chunk) in reply.keysyms.chunks(per).enumerate() {
        if chunk.contains(&keysym) {
            return Ok(min + index as u8);
        }
    }
    Err(Error::Activation(format!(
        "no keycode for keysym {keysym:#x}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_none_and_pointer_root() {
        let err = restore_focus_and_ctrl_v(0).unwrap_err();
        assert!(err.to_string().contains("no previous"));
        let err = restore_focus_and_ctrl_v(1).unwrap_err();
        assert!(err.to_string().contains("no previous"));
    }
}
