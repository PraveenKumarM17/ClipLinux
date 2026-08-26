//! ClipLinux desktop library.
//!
//! Default `cargo test --workspace` builds this crate **without** Tauri so
//! WebKitGTK is not required. The production window is compiled with
//! `--features tauri-app`.

#![forbid(unsafe_code)]

mod clipboard;
mod commands;
mod dto;
mod ipc;

use clipl_core::PlatformCapabilities;
use clipl_protocol::{Envelope, Message, Request, Response};

pub use commands::{
    clear_history, copy_history_item, delete_history_item, get_daemon_status, get_history,
    search_history, set_pinned,
};
pub use dto::{ConnectionView, HistoryRow};
pub use ipc::{DaemonClient, START_COMMAND};

/// Application identifier used by Tauri and desktop files.
pub const APP_ID: &str = "io.clipl.ClipLinux";

/// Human-readable product name.
pub const APP_NAME: &str = "ClipLinux";

/// Binary entry used by `main`.
pub fn entry() {
    #[cfg(feature = "tauri-app")]
    {
        if let Err(err) = run_tauri() {
            eprintln!("clipl-desktop: {err}");
            std::process::exit(1);
        }
    }
    #[cfg(not(feature = "tauri-app"))]
    {
        println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
        println!("app id: {APP_ID}");
        println!();
        println!("This workspace build excludes the Tauri WebView.");
        println!("Run the production shell with:");
        println!("  cd apps/desktop && npm install && npm run tauri dev");
    }
}

/// Handle a protocol request locally (tests / fallback). Production UI talks
/// to `clipl-daemon` instead.
pub fn handle_local(request: Request) -> Response {
    match request {
        Request::Ping => Response::Pong,
        Request::GetCapabilities => {
            Response::Capabilities(PlatformCapabilities::conservative_linux())
        }
        Request::GetHistory { .. } => Response::History(Vec::new()),
        Request::SearchHistory { .. } => Response::History(Vec::new()),
        Request::GetStatus => Response::Error {
            message: "desktop stub has no daemon status".into(),
        },
        Request::PinItem { .. } | Request::UnpinItem { .. } | Request::CopyItem { .. } => {
            Response::Error {
                message: "use the daemon IPC path from the Tauri shell".into(),
            }
        }
        Request::ListSnippets => Response::Snippets(Vec::new()),
        Request::ListPrivacyRules => Response::PrivacyRules(Vec::new()),
        Request::Paste { .. } => Response::Error {
            message: "paste is not implemented in this phase".into(),
        },
        _ => Response::Error {
            message: "request is not implemented without the daemon".into(),
        },
    }
}

/// Decode a JSON envelope and produce a response envelope.
pub fn dispatch_json(bytes: &[u8]) -> Result<Envelope, clipl_core::Error> {
    let incoming = Envelope::from_json_bytes(bytes)?;
    match incoming.payload {
        Message::Request(request) => Ok(Envelope {
            id: incoming.id,
            payload: Message::Response(handle_local(request)),
        }),
        _ => Err(clipl_core::Error::Protocol(
            "desktop stub only accepts requests".into(),
        )),
    }
}

#[cfg(feature = "tauri-app")]
fn run_tauri() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            cmd_get_daemon_status,
            cmd_get_history,
            cmd_search_history,
            cmd_delete_history_item,
            cmd_clear_history,
            cmd_pin_history_item,
            cmd_unpin_history_item,
            cmd_copy_history_item,
            cmd_close_window,
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_get_daemon_status() -> ConnectionView {
    get_daemon_status(&DaemonClient::from_env())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_get_history() -> Result<Vec<HistoryRow>, String> {
    get_history(&DaemonClient::from_env()).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_search_history(query: String) -> Result<Vec<HistoryRow>, String> {
    search_history(&DaemonClient::from_env(), &query).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_delete_history_item(id: String) -> Result<bool, String> {
    delete_history_item(&DaemonClient::from_env(), &id).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_clear_history() -> Result<u64, String> {
    clear_history(&DaemonClient::from_env()).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_pin_history_item(id: String) -> Result<bool, String> {
    set_pinned(&DaemonClient::from_env(), &id, true).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_unpin_history_item(id: String) -> Result<bool, String> {
    set_pinned(&DaemonClient::from_env(), &id, false).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_copy_history_item(id: String) -> Result<(), String> {
    copy_history_item(&DaemonClient::from_env(), &clipboard::SystemClipboard, &id)
        .map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_close_window(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|err| err.to_string())
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
