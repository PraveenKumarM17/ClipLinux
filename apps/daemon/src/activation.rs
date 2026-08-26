//! Desktop activation mailbox. The daemon routes show/hide; it does not own the window.

use std::os::unix::net::UnixStream;
use std::sync::Mutex;

use clipl_core::ActivationRequest;
use clipl_protocol::{write_frame, Envelope, Event, Message};

/// At most one desktop subscriber. A new subscribe replaces the previous connection.
#[derive(Default)]
pub(crate) struct DesktopHub {
    subscriber: Mutex<Option<UnixStream>>,
}

impl DesktopHub {
    pub(crate) fn subscribe(&self, stream: UnixStream) -> bool {
        let mut slot = self
            .subscriber
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let replaced = slot.is_some();
        *slot = Some(stream);
        replaced
    }

    pub(crate) fn route(&self, action: ActivationRequest) -> bool {
        let mut slot = self
            .subscriber
            .lock()
            .unwrap_or_else(|err| err.into_inner());
        let Some(stream) = slot.as_mut() else {
            return false;
        };
        let envelope = Envelope::new(Message::Event(Event::ActivatePicker { action }));
        match write_frame(&mut *stream, &envelope) {
            Ok(()) => true,
            Err(_) => {
                *slot = None;
                false
            }
        }
    }

    pub(crate) fn connected(&self) -> bool {
        self.subscriber
            .lock()
            .map(|slot| slot.is_some())
            .unwrap_or(false)
    }
}
