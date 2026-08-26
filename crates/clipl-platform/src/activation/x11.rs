//! X11 native global shortcut via `XGrabKey`.
//!
//! Registers **only** the configured chord. Does not select `KeyPress` on the
//! root window (that would receive every key). Events arrive solely because of
//! the grab.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, GrabMode, Keycode, ModMask, Window};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use clipl_core::{
    ActivationBackend, ActivationBackendKind, ActivationCapability, ActivationSnapshot,
    ActivationStatus, Error, Result, Shortcut,
};

use super::{should_fire, NativeActivation};

const SUPER_L: u32 = 0xffeb;
const SUPER_R: u32 = 0xffec;
const META_L: u32 = 0xffe7;
const META_R: u32 = 0xffe8;
const HYPER_L: u32 = 0xffed;
const HYPER_R: u32 = 0xffee;

/// X11 `XGrabKey` backend. Constructed unbound; [`NativeActivation::listen`] grabs.
pub struct X11Activation {
    shortcut: Shortcut,
    grab: Option<X11Grab>,
    last_error: Option<String>,
}

impl X11Activation {
    /// Do not connect to X11 yet (selection / tests).
    pub fn unbound(shortcut: Shortcut) -> Self {
        Self {
            shortcut,
            grab: None,
            last_error: None,
        }
    }
}

impl ActivationBackend for X11Activation {
    fn kind(&self) -> ActivationBackendKind {
        ActivationBackendKind::X11
    }

    fn capability(&self) -> ActivationCapability {
        ActivationCapability::NativeGlobalShortcut
    }

    fn snapshot(&self) -> ActivationSnapshot {
        if let Some(reason) = &self.last_error {
            return ActivationSnapshot {
                backend: self.kind(),
                capability: self.capability(),
                status: ActivationStatus::Error,
                shortcut: self.shortcut.display(),
                reason: reason.clone(),
            };
        }
        if self.grab.is_some() {
            return ActivationSnapshot {
                backend: self.kind(),
                capability: self.capability(),
                status: ActivationStatus::Active,
                shortcut: self.shortcut.display(),
                reason: format!(
                    "X11 XGrabKey registered for {}. Super+V may conflict with the window manager.",
                    self.shortcut.display()
                ),
            };
        }
        ActivationSnapshot {
            backend: self.kind(),
            capability: self.capability(),
            status: ActivationStatus::NotConfigured,
            shortcut: self.shortcut.display(),
            reason: "X11 native shortcut will be registered when the daemon starts listening"
                .into(),
        }
    }

    fn supports_native_listen(&self) -> bool {
        true
    }
}

impl NativeActivation for X11Activation {
    fn arm(&mut self) -> Result<()> {
        match X11Grab::register(&self.shortcut) {
            Ok(grab) => {
                self.grab = Some(grab);
                self.last_error = None;
                Ok(())
            }
            Err(err) => {
                self.last_error = Some(err.to_string());
                Err(err)
            }
        }
    }

    fn listen(&mut self, shutdown: &AtomicBool, on_fire: &dyn Fn()) -> Result<()> {
        let grab = self
            .grab
            .as_mut()
            .ok_or_else(|| Error::Activation("X11 shortcut was not armed".into()))?;
        grab.run(shutdown, on_fire)
    }
}

struct X11Grab {
    conn: RustConnection,
    root: Window,
    keycode: Keycode,
    base_mask: u16,
    grabbed_masks: Vec<u16>,
}

impl X11Grab {
    fn register(shortcut: &Shortcut) -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None)
            .map_err(|err| Error::Activation(format!("X11 connect failed: {err}")))?;
        let root = conn.setup().roots[screen_num].root;
        let keysym = keysym_for_key(&shortcut.key).ok_or_else(|| {
            Error::Activation(format!("unsupported activation key `{}`", shortcut.key))
        })?;
        let keycode = keycode_for_keysym(&conn, keysym)?;
        let base_mask = modifier_mask(&conn, shortcut)?;

        let mut grabbed_masks = Vec::new();
        for extra in lock_variants() {
            let modifiers = base_mask | extra;
            conn.grab_key(
                false,
                root,
                modifiers.into(),
                keycode,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(|err| Error::Activation(format!("XGrabKey: {err}")))?
            .check()
            .map_err(|err| {
                Error::Activation(format!(
                    "XGrabKey failed for {} (modifiers={modifiers:#x}): {err}",
                    shortcut.display()
                ))
            })?;
            grabbed_masks.push(modifiers);
        }
        conn.flush()
            .map_err(|err| Error::Activation(err.to_string()))?;

        Ok(Self {
            conn,
            root,
            keycode,
            base_mask,
            grabbed_masks,
        })
    }

    fn run(&mut self, shutdown: &AtomicBool, on_fire: &dyn Fn()) -> Result<()> {
        let mut last = None;
        while !shutdown.load(Ordering::SeqCst) {
            match self
                .conn
                .poll_for_event()
                .map_err(|err| Error::Activation(err.to_string()))?
            {
                Some(Event::KeyPress(event)) => {
                    if event.detail != self.keycode {
                        continue;
                    }
                    let effective = u16::from(event.state) & 0x00FF & !ignorable_locks();
                    if effective != self.base_mask {
                        continue;
                    }
                    if should_fire(&mut last, Instant::now(), Duration::from_millis(250)) {
                        on_fire();
                    }
                }
                Some(_) => {}
                None => std::thread::sleep(Duration::from_millis(40)),
            }
        }
        Ok(())
    }
}

impl Drop for X11Grab {
    fn drop(&mut self) {
        for mask in &self.grabbed_masks {
            let _ = self
                .conn
                .ungrab_key(self.keycode, self.root, (*mask).into());
        }
        let _ = self.conn.flush();
    }
}

fn ignorable_locks() -> u16 {
    u16::from(ModMask::LOCK) | u16::from(ModMask::M2) | u16::from(ModMask::M5)
}

fn lock_variants() -> [u16; 8] {
    let lock = u16::from(ModMask::LOCK);
    let num = u16::from(ModMask::M2);
    let scroll = u16::from(ModMask::M5);
    [
        0,
        lock,
        num,
        scroll,
        lock | num,
        lock | scroll,
        num | scroll,
        lock | num | scroll,
    ]
}

fn modifier_mask(conn: &RustConnection, shortcut: &Shortcut) -> Result<u16> {
    let mut mask = 0u16;
    if shortcut.shift {
        mask |= u16::from(ModMask::SHIFT);
    }
    if shortcut.ctrl {
        mask |= u16::from(ModMask::CONTROL);
    }
    if shortcut.alt {
        mask |= u16::from(ModMask::M1);
    }
    if shortcut.super_key {
        mask |= detect_super_mask(conn)?;
    }
    Ok(mask)
}

fn detect_super_mask(conn: &RustConnection) -> Result<u16> {
    let mapping = conn
        .get_modifier_mapping()
        .map_err(|err| Error::Activation(err.to_string()))?
        .reply()
        .map_err(|err| Error::Activation(err.to_string()))?;
    let per = mapping.keycodes_per_modifier() as usize;
    let wanted = [SUPER_L, SUPER_R, META_L, META_R, HYPER_L, HYPER_R];
    for (mod_index, chunk) in mapping.keycodes.chunks(per).enumerate() {
        for &keycode in chunk {
            if keycode == 0 {
                continue;
            }
            if let Ok(syms) = keysyms_for_keycode(conn, keycode) {
                if syms.iter().any(|sym| wanted.contains(sym)) {
                    return Ok(1u16 << mod_index);
                }
            }
        }
    }
    Ok(u16::from(ModMask::M4))
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

fn keysyms_for_keycode(conn: &RustConnection, keycode: Keycode) -> Result<Vec<u32>> {
    let reply = conn
        .get_keyboard_mapping(keycode, 1)
        .map_err(|err| Error::Activation(err.to_string()))?
        .reply()
        .map_err(|err| Error::Activation(err.to_string()))?;
    Ok(reply.keysyms)
}

pub(crate) fn keysym_for_key(name: &str) -> Option<u32> {
    match name {
        "space" => Some(0x0020),
        "tab" => Some(0xff09),
        "return" => Some(0xff0d),
        "escape" => Some(0xff1b),
        "backspace" => Some(0xff08),
        "delete" => Some(0xffff),
        "home" => Some(0xff50),
        "end" => Some(0xff57),
        "left" => Some(0xff51),
        "up" => Some(0xff52),
        "right" => Some(0xff53),
        "down" => Some(0xff54),
        "page_up" => Some(0xff55),
        "page_down" => Some(0xff56),
        s if s.len() == 1 => {
            let byte = s.as_bytes()[0];
            if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
                Some(u32::from(byte))
            } else {
                None
            }
        }
        s if s.starts_with('f') => {
            let n: u32 = s[1..].parse().ok()?;
            if (1..=12).contains(&n) {
                Some(0xffbe + (n - 1))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keysym_v_is_ascii() {
        assert_eq!(keysym_for_key("v"), Some(0x0076));
        assert_eq!(keysym_for_key("1"), Some(0x0031));
        assert_eq!(keysym_for_key("f12"), Some(0xffc9));
        assert_eq!(keysym_for_key("space"), Some(0x0020));
        assert!(keysym_for_key("fn").is_none());
    }

    #[test]
    fn unbound_does_not_claim_active() {
        let backend = X11Activation::unbound(Shortcut::default());
        assert_eq!(backend.snapshot().status, ActivationStatus::NotConfigured);
        assert!(backend.supports_native_listen());
    }
}
