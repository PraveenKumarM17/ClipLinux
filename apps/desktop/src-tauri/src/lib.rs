//! UniPick desktop library.
//!
//! The Tauri v2 shell is the intended host. This crate compiles as a plain
//! binary until webview dependencies are introduced in the desktop-shell
//! milestone. Shared command handlers will live here so the UI never talks to
//! OS APIs directly.

#![forbid(unsafe_code)]

use unipick_core::PlatformCapabilities;
use unipick_protocol::{Envelope, Message, Request, Response};

/// Application identifier used by Tauri and desktop files.
pub const APP_ID: &str = "dev.unipick.UniPick";

/// Human-readable product name.
pub const APP_NAME: &str = "UniPick";

/// Handle a protocol request locally (no daemon yet).
pub fn handle_local(request: Request) -> Response {
    match request {
        Request::Ping => Response::Pong,
        Request::GetCapabilities => {
            Response::Capabilities(PlatformCapabilities::conservative_linux())
        }
        Request::GetHistory { .. } => Response::History(Vec::new()),
        Request::ListSnippets => Response::Snippets(Vec::new()),
        Request::ListPrivacyRules => Response::PrivacyRules(Vec::new()),
        Request::Paste { .. } => Response::Error {
            message: "paste is not implemented in the foundation".into(),
        },
        _ => Response::Error {
            message: "request is not implemented in the foundation".into(),
        },
    }
}

/// Decode a JSON envelope and produce a response envelope.
pub fn dispatch_json(bytes: &[u8]) -> Result<Envelope, unipick_core::Error> {
    let incoming = Envelope::from_json_bytes(bytes)?;
    match incoming.payload {
        Message::Request(request) => Ok(Envelope {
            id: incoming.id,
            payload: Message::Response(handle_local(request)),
        }),
        _ => Err(unipick_core::Error::Protocol(
            "desktop stub only accepts requests".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_round_trip() {
        let env = Envelope::new(Message::Request(Request::Ping));
        let out = dispatch_json(&env.to_json_bytes().unwrap()).unwrap();
        assert!(matches!(out.payload, Message::Response(Response::Pong)));
    }
}
