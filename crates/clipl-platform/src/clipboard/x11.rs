//! X11 CLIPBOARD monitoring via XFixes.
//!
//! **CLIPBOARD** is Ctrl+C / Ctrl+V. **PRIMARY** is the mouse-selection buffer.
//! ClipLinux defaults to CLIPBOARD only so selecting text does not flood history.
//!
//! Watch uses `XFixesSelectionNotify` (not `xclip` polling).

use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ConnectionExt, CreateWindowAux, EventMask, GetPropertyType, Window, WindowClass,
};
use x11rb::protocol::{xfixes, Event};
use x11rb::rust_connection::RustConnection;
use x11rb::COPY_DEPTH_FROM_PARENT;

use clipl_core::{ClipboardBackend, ClipboardContent, ClipboardWatcher, Error, Result};

/// X11 backend. Connections are opened per read/watch (the struct is `Sync`).
pub struct X11Clipboard {
    selection: SelectionSet,
}

#[derive(Clone, Copy)]
enum SelectionSet {
    Clipboard,
    Primary,
    Both,
}

impl X11Clipboard {
    /// Connect to `$DISPLAY` and require XFixes.
    pub fn connect(selection: &str) -> Result<Self> {
        let set = parse_selection(selection)?;
        let (conn, _) = x11rb::connect(None)
            .map_err(|err| Error::Clipboard(format!("X11 connect failed: {err}")))?;
        xfixes::query_version(&conn, 5, 0)
            .map_err(|err| Error::Clipboard(format!("XFixes query: {err}")))?
            .reply()
            .map_err(|err| Error::Clipboard(format!("XFixes unavailable: {err}")))?;
        drop(conn);
        Ok(Self { selection: set })
    }
}

impl ClipboardBackend for X11Clipboard {
    fn name(&self) -> &'static str {
        "x11"
    }

    fn read(&self) -> Result<Option<ClipboardContent>> {
        let mut session = X11Session::open()?;
        let atom = match self.selection {
            SelectionSet::Primary => session.primary,
            _ => session.clipboard,
        };
        session.read_utf8(atom)
    }

    fn write(&self, content: &ClipboardContent) -> Result<()> {
        let _ = content;
        Err(Error::unsupported(
            "X11 clipboard write is not implemented in this phase",
        ))
    }

    fn supports_watch(&self) -> bool {
        true
    }

    fn supports_images(&self) -> bool {
        false
    }

    fn watch(&self) -> Result<Box<dyn ClipboardWatcher>> {
        let session = X11Session::open()?;
        session.select_fixes(self.selection)?;
        Ok(Box::new(X11Watcher {
            session,
            set: self.selection,
        }))
    }
}

struct X11Session {
    conn: RustConnection,
    window: Window,
    clipboard: Atom,
    primary: Atom,
    utf8: Atom,
    property: Atom,
}

impl X11Session {
    fn open() -> Result<Self> {
        let (conn, screen_num) = x11rb::connect(None)
            .map_err(|err| Error::Clipboard(format!("X11 connect failed: {err}")))?;
        xfixes::query_version(&conn, 5, 0)
            .map_err(|err| Error::Clipboard(format!("XFixes query: {err}")))?
            .reply()
            .map_err(|err| Error::Clipboard(format!("XFixes unavailable: {err}")))?;
        let screen = &conn.setup().roots[screen_num];
        let window = conn
            .generate_id()
            .map_err(|err| Error::Clipboard(err.to_string()))?;
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
        )
        .map_err(|err| Error::Clipboard(err.to_string()))?;
        conn.flush()
            .map_err(|err| Error::Clipboard(err.to_string()))?;

        let intern = |name: &str| -> Result<Atom> {
            Ok(conn
                .intern_atom(false, name.as_bytes())
                .map_err(|err| Error::Clipboard(err.to_string()))?
                .reply()
                .map_err(|err| Error::Clipboard(err.to_string()))?
                .atom)
        };

        let clipboard = intern("CLIPBOARD")?;
        let utf8 = intern("UTF8_STRING")?;
        let property = intern("CLIPL_CLIPBOARD")?;
        Ok(Self {
            clipboard,
            primary: AtomEnum::PRIMARY.into(),
            utf8,
            property,
            window,
            conn,
        })
    }

    fn select_fixes(&self, set: SelectionSet) -> Result<()> {
        let mask = xfixes::SelectionEventMask::SET_SELECTION_OWNER
            | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
            | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE;
        let mut atoms = Vec::new();
        match set {
            SelectionSet::Clipboard => atoms.push(self.clipboard),
            SelectionSet::Primary => atoms.push(self.primary),
            SelectionSet::Both => {
                atoms.push(self.clipboard);
                atoms.push(self.primary);
            }
        }
        for atom in atoms {
            xfixes::select_selection_input(&self.conn, self.window, atom, mask)
                .map_err(|err| Error::Clipboard(err.to_string()))?;
        }
        self.conn
            .flush()
            .map_err(|err| Error::Clipboard(err.to_string()))?;
        Ok(())
    }

    fn read_utf8(&mut self, selection: Atom) -> Result<Option<ClipboardContent>> {
        self.conn
            .delete_property(self.window, self.property)
            .map_err(|err| Error::Clipboard(err.to_string()))?;
        self.conn
            .convert_selection(
                self.window,
                selection,
                self.utf8,
                self.property,
                x11rb::CURRENT_TIME,
            )
            .map_err(|err| Error::Clipboard(err.to_string()))?;
        self.conn
            .flush()
            .map_err(|err| Error::Clipboard(err.to_string()))?;

        let deadline = Instant::now() + Duration::from_millis(250);
        loop {
            if Instant::now() > deadline {
                return Ok(None);
            }
            match self
                .conn
                .poll_for_event()
                .map_err(|err| Error::Clipboard(err.to_string()))?
            {
                Some(Event::SelectionNotify(ev)) if ev.requestor == self.window => {
                    if ev.property == Atom::from(AtomEnum::NONE) {
                        return Ok(None);
                    }
                    return self.read_property();
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    fn read_property(&self) -> Result<Option<ClipboardContent>> {
        let reply = self
            .conn
            .get_property(
                false,
                self.window,
                self.property,
                GetPropertyType::ANY,
                0,
                1024 * 1024 / 4,
            )
            .map_err(|err| Error::Clipboard(err.to_string()))?
            .reply()
            .map_err(|err| Error::Clipboard(err.to_string()))?;
        if reply.bytes_after > 0 {
            return Err(Error::unsupported(
                "X11 INCR / large clipboard transfers are not implemented",
            ));
        }
        if reply.value.is_empty() {
            return Ok(None);
        }
        let text = String::from_utf8_lossy(&reply.value).into_owned();
        Ok(Some(ClipboardContent::Text {
            text,
            mime: "text/plain;charset=utf-8".into(),
        }))
    }
}

struct X11Watcher {
    session: X11Session,
    set: SelectionSet,
}

impl ClipboardWatcher for X11Watcher {
    fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ClipboardContent>> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Ok(None);
            }
            match self
                .session
                .conn
                .poll_for_event()
                .map_err(|err| Error::Clipboard(err.to_string()))?
            {
                Some(Event::XfixesSelectionNotify(ev)) => {
                    let wanted = match self.set {
                        SelectionSet::Clipboard => ev.selection == self.session.clipboard,
                        SelectionSet::Primary => ev.selection == self.session.primary,
                        SelectionSet::Both => {
                            ev.selection == self.session.clipboard
                                || ev.selection == self.session.primary
                        }
                    };
                    if wanted {
                        return self.session.read_utf8(ev.selection);
                    }
                }
                Some(_) => continue,
                None => std::thread::sleep(Duration::from_millis(25)),
            }
        }
    }
}

fn parse_selection(value: &str) -> Result<SelectionSet> {
    match value {
        "clipboard" => Ok(SelectionSet::Clipboard),
        "primary" => Ok(SelectionSet::Primary),
        "both" => Ok(SelectionSet::Both),
        other => Err(Error::Config(format!(
            "unknown X11 selection `{other}` (clipboard, primary, both)"
        ))),
    }
}
