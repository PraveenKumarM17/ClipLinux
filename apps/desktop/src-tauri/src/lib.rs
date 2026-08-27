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
mod launch;
mod picker;
mod window;

use clipl_core::PlatformCapabilities;
use clipl_protocol::{Envelope, Message, Request, Response};

pub use commands::{
    clear_history, copy_history_item, delete_history_item, get_daemon_status, get_history,
    insert_into_app, search_history, set_pinned,
};
pub use dto::{ConnectionView, HistoryRow, InsertOutcome};
pub use ipc::{disconnected_message, DaemonClient};
pub use launch::{
    daemon_binary_for_exe, ensure_daemon_running, gnome_enabled_extensions_with_uuid,
    gnome_extension_on_disk, install_user_gnome_extension, persistent_daemon_bin,
    running_from_appimage, start_command, try_enable_user_gnome_extension, DAEMON_ON_PATH,
    GNOME_EXTENSION_UUID,
};
pub use picker::{
    copy_picker_item, list_emoji_category, list_picker_category, picker_favorites, search_emoji,
    search_picker, set_picker_favorite, set_skin_tone_pref, skin_tone_pref,
};
pub use window::{apply_activation, PickerVisibility};

#[cfg(feature = "tauri-app")]
use std::sync::Mutex;

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
    use std::thread;
    use std::time::Duration;

    use clipl_core::paths;
    use clipl_protocol::ActivationSubscriber;
    use tauri::Manager;

    tauri::Builder::default()
        .manage(PickerState {
            visibility: Mutex::new(PickerVisibility::Hidden),
            shown_at: Mutex::new(None),
        })
        .setup(|app| {
            if let Some(window) = app.get_webview_window("palette") {
                style_palette_window(&window);
            }
            if !tauri::is_dev() {
                if let Ok(exe) = std::env::current_exe() {
                    let _ = crate::launch::ensure_daemon_running(&exe);
                }
                install_bundled_gnome_extension(app.handle());
                crate::launch::try_enable_user_gnome_extension();
            }
            let handle = app.handle().clone();
            thread::spawn(move || loop {
                match ActivationSubscriber::connect_path(&paths::socket_path()) {
                    Ok(mut sub) => loop {
                        match sub.recv() {
                            Ok(action) => {
                                let handle_inner = handle.clone();
                                let _ = handle.run_on_main_thread(move || {
                                    apply_window_action(&handle_inner, action);
                                });
                            }
                            Err(_) => break,
                        }
                    },
                    Err(_) => thread::sleep(Duration::from_millis(750)),
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                hide_picker_window(window);
            }
            tauri::WindowEvent::Focused(false) => {
                if should_hide_on_blur(window) {
                    hide_picker_window(window);
                }
            }
            _ => {}
        })
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
            cmd_hide_picker,
            cmd_show_picker,
            cmd_toggle_picker,
            cmd_search_emoji,
            cmd_list_emoji_category,
            cmd_search_picker,
            cmd_list_picker_category,
            cmd_picker_favorites,
            cmd_set_picker_favorite,
            cmd_skin_tone_pref,
            cmd_set_skin_tone_pref,
            cmd_copy_picker_item,
        ])
        .run(tauri::generate_context!())?;
    Ok(())
}

#[cfg(feature = "tauri-app")]
struct PickerState {
    visibility: Mutex<PickerVisibility>,
    shown_at: Mutex<Option<std::time::Instant>>,
}

#[cfg(feature = "tauri-app")]
fn style_palette_window(window: &tauri::WebviewWindow) {
    let _ = window.set_minimizable(false);
    let _ = window.set_maximizable(false);
    let _ = window.set_skip_taskbar(true);
    if let Ok(gtk_win) = window.gtk_window() {
        use gtk::prelude::GtkWindowExt;
        gtk_win.set_decorated(false);
        gtk_win.set_skip_taskbar_hint(true);
        gtk_win.set_type_hint(gdk::WindowTypeHint::Dialog);
    }
    let _ = window.hide();
}

#[cfg(feature = "tauri-app")]
fn install_bundled_gnome_extension(app: &tauri::AppHandle) {
    use tauri::Manager;
    if crate::launch::gnome_extension_on_disk() {
        return;
    }
    let Ok(resource_root) = app.path().resource_dir() else {
        return;
    };
    let bundled = resource_root.join("gnome-extension");
    if !bundled.join("metadata.json").is_file() {
        return;
    }
    if let Err(err) = crate::launch::install_user_gnome_extension(&bundled) {
        eprintln!("clipl-desktop: could not install GNOME extension files: {err}");
    }
}

#[cfg(feature = "tauri-app")]
fn hide_picker_window(window: &tauri::Window) {
    use tauri::Manager;
    let _ = window.hide();
    if let Some(state) = window.try_state::<PickerState>() {
        if let Ok(mut vis) = state.visibility.lock() {
            *vis = PickerVisibility::Hidden;
        }
    }
}

#[cfg(feature = "tauri-app")]
fn should_hide_on_blur(window: &tauri::Window) -> bool {
    use tauri::Manager;
    let Some(state) = window.try_state::<PickerState>() else {
        return false;
    };
    let Ok(vis) = state.visibility.lock() else {
        return false;
    };
    if *vis != PickerVisibility::Shown {
        return false;
    }
    drop(vis);
    let Ok(shown_at) = state.shown_at.lock() else {
        return true;
    };
    match *shown_at {
        Some(at) if at.elapsed() < std::time::Duration::from_millis(350) => false,
        _ => true,
    }
}

#[cfg(feature = "tauri-app")]
fn mark_picker_shown(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(state) = app.try_state::<PickerState>() {
        if let Ok(mut shown_at) = state.shown_at.lock() {
            *shown_at = Some(std::time::Instant::now());
        }
    }
}

#[cfg(feature = "tauri-app")]
fn apply_window_action(app: &tauri::AppHandle, action: clipl_core::ActivationRequest) {
    use tauri::{Emitter, Manager};

    let next = {
        let Some(state) = app.try_state::<PickerState>() else {
            return;
        };
        let Ok(mut vis) = state.visibility.lock() else {
            return;
        };
        let next = apply_activation(*vis, action);
        *vis = next;
        next
    };
    let Some(window) = app.get_webview_window("palette") else {
        return;
    };
    match next {
        PickerVisibility::Shown => {
            mark_picker_shown(app);
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_skip_taskbar(true);
            let _ = window.set_focus();
            let _ = app.emit("picker-activated", ());
        }
        PickerVisibility::Hidden => {
            let _ = window.hide();
        }
    }
}

#[cfg(feature = "tauri-app")]
fn set_picker_visible(window: &tauri::WebviewWindow, app: &tauri::AppHandle, shown: bool) {
    use tauri::{Emitter, Manager};

    if let Some(state) = app.try_state::<PickerState>() {
        if let Ok(mut vis) = state.visibility.lock() {
            *vis = if shown {
                PickerVisibility::Shown
            } else {
                PickerVisibility::Hidden
            };
        }
    }
    if shown {
        mark_picker_shown(app);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_skip_taskbar(true);
        let _ = window.set_focus();
        let _ = app.emit("picker-activated", ());
    } else {
        let _ = window.hide();
    }
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
fn cmd_copy_history_item(
    id: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<InsertOutcome, String> {
    copy_then_insert(&window, &app, || {
        copy_history_item(&DaemonClient::from_env(), &clipboard::SystemClipboard, &id)
            .map_err(|err| err.to_string())
    })
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_close_window(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    set_picker_visible(&window, &app, false);
    Ok(())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_hide_picker(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    set_picker_visible(&window, &app, false);
    Ok(())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_show_picker(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    set_picker_visible(&window, &app, true);
    Ok(())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_toggle_picker(window: tauri::WebviewWindow, app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let shown = app
        .try_state::<PickerState>()
        .and_then(|state| {
            state
                .visibility
                .lock()
                .ok()
                .map(|vis| *vis == PickerVisibility::Shown)
        })
        .unwrap_or(true);
    set_picker_visible(&window, &app, !shown);
    Ok(())
}

#[cfg(feature = "tauri-app")]
use clipl_protocol::{PickerItem, PickerKind};

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_search_emoji(query: String, limit: u32) -> Result<Vec<PickerItem>, String> {
    picker::search_emoji(&DaemonClient::from_env(), &query, limit).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_list_emoji_category(category: String, limit: u32) -> Result<Vec<PickerItem>, String> {
    picker::list_emoji_category(&DaemonClient::from_env(), &category, limit)
        .map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_search_picker(
    kind: PickerKind,
    query: String,
    limit: u32,
) -> Result<Vec<PickerItem>, String> {
    picker::search_picker(&DaemonClient::from_env(), kind, &query, limit)
        .map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_list_picker_category(kind: PickerKind, category: String) -> Result<Vec<PickerItem>, String> {
    picker::list_picker_category(&DaemonClient::from_env(), kind, &category)
        .map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_picker_favorites(kind: PickerKind) -> Result<Vec<PickerItem>, String> {
    picker::picker_favorites(&DaemonClient::from_env(), kind).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_set_picker_favorite(
    kind: PickerKind,
    glyph: String,
    favorite: bool,
) -> Result<bool, String> {
    picker::set_picker_favorite(&DaemonClient::from_env(), kind, &glyph, favorite)
        .map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_skin_tone_pref() -> Result<String, String> {
    picker::skin_tone_pref(&DaemonClient::from_env()).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_set_skin_tone_pref(tone: String) -> Result<String, String> {
    picker::set_skin_tone_pref(&DaemonClient::from_env(), &tone).map_err(|err| err.to_string())
}

#[cfg(feature = "tauri-app")]
#[tauri::command]
fn cmd_copy_picker_item(
    kind: PickerKind,
    glyph: String,
    base: String,
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
) -> Result<InsertOutcome, String> {
    copy_then_insert(&window, &app, || {
        picker::copy_picker_item(
            &DaemonClient::from_env(),
            &clipboard::SystemClipboard,
            kind,
            &glyph,
            &base,
        )
        .map_err(|err| err.to_string())
    })
}

#[cfg(feature = "tauri-app")]
fn copy_then_insert(
    window: &tauri::WebviewWindow,
    app: &tauri::AppHandle,
    write: impl FnOnce() -> Result<(), String>,
) -> Result<InsertOutcome, String> {
    write()?;
    set_picker_visible(window, app, false);
    std::thread::sleep(std::time::Duration::from_millis(120));
    match insert_into_app(&DaemonClient::from_env()) {
        Ok(outcome) => Ok(outcome),
        Err(_) => Ok(InsertOutcome {
            inserted: false,
            reason: "copied; press Ctrl+V in the other app".into(),
        }),
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
