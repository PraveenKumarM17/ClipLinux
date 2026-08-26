//! Desktop mapping for picker IPC.

use clipl_core::Result;
use clipl_protocol::{PickerItem, PickerKind, Request, Response, SkinTonePref};

use crate::clipboard::ClipboardWriter;
use crate::ipc::DaemonClient;

fn picker_list(response: Result<Response>) -> Result<Vec<PickerItem>> {
    match response? {
        Response::PickerList(items) => Ok(items),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected picker response: {other:?}"
        ))),
    }
}

/// Search emoji via the daemon.
pub fn search_emoji(client: &DaemonClient, query: &str, limit: u32) -> Result<Vec<PickerItem>> {
    picker_list(client.request(Request::SearchEmoji {
        query: query.to_string(),
        limit,
    }))
}

/// List an emoji category (`Frequently Used` included).
pub fn list_emoji_category(
    client: &DaemonClient,
    category: &str,
    limit: u32,
) -> Result<Vec<PickerItem>> {
    picker_list(client.request(Request::ListEmojiCategory {
        category: category.to_string(),
        limit,
    }))
}

/// Search symbols or kaomoji.
pub fn search_picker(
    client: &DaemonClient,
    kind: PickerKind,
    query: &str,
    limit: u32,
) -> Result<Vec<PickerItem>> {
    let request = match kind {
        PickerKind::Symbol => Request::SearchSymbols {
            query: query.to_string(),
            limit,
        },
        PickerKind::Kaomoji => Request::SearchKaomoji {
            query: query.to_string(),
            limit,
        },
        PickerKind::Emoji => Request::SearchEmoji {
            query: query.to_string(),
            limit,
        },
    };
    picker_list(client.request(request))
}

/// List a symbol/kaomoji category.
pub fn list_picker_category(
    client: &DaemonClient,
    kind: PickerKind,
    category: &str,
) -> Result<Vec<PickerItem>> {
    let request = match kind {
        PickerKind::Symbol => Request::ListSymbolCategory {
            category: category.to_string(),
        },
        PickerKind::Kaomoji => Request::ListKaomojiCategory {
            category: category.to_string(),
        },
        PickerKind::Emoji => Request::ListEmojiCategory {
            category: category.to_string(),
            limit: 400,
        },
    };
    picker_list(client.request(request))
}

/// Favorites for a catalog.
pub fn picker_favorites(client: &DaemonClient, kind: PickerKind) -> Result<Vec<PickerItem>> {
    let request = match kind {
        PickerKind::Emoji => Request::GetFavoriteEmoji,
        other => Request::GetFavoritePicker { kind: other },
    };
    picker_list(client.request(request))
}

/// Toggle favorite.
pub fn set_picker_favorite(
    client: &DaemonClient,
    kind: PickerKind,
    glyph: &str,
    favorite: bool,
) -> Result<bool> {
    let request = match (kind, favorite) {
        (PickerKind::Emoji, true) => Request::FavoriteEmoji {
            glyph: glyph.into(),
        },
        (PickerKind::Emoji, false) => Request::UnfavoriteEmoji {
            glyph: glyph.into(),
        },
        (kind, true) => Request::FavoritePicker {
            kind,
            glyph: glyph.into(),
        },
        (kind, false) => Request::UnfavoritePicker {
            kind,
            glyph: glyph.into(),
        },
    };
    match client.request(request)? {
        Response::PickerFavorite { favorite, .. } => Ok(favorite),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected favorite response: {other:?}"
        ))),
    }
}

/// Skin-tone preference.
pub fn skin_tone_pref(client: &DaemonClient) -> Result<String> {
    match client.request(Request::GetSkinTonePref)? {
        Response::SkinTone(SkinTonePref { tone }) => Ok(tone),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected skin response: {other:?}"
        ))),
    }
}

/// Persist skin-tone preference.
pub fn set_skin_tone_pref(client: &DaemonClient, tone: &str) -> Result<String> {
    match client.request(Request::SetSkinTonePref {
        tone: tone.to_string(),
    })? {
        Response::SkinTone(SkinTonePref { tone }) => Ok(tone),
        Response::Error { message } => Err(clipl_core::Error::Protocol(message)),
        other => Err(clipl_core::Error::Protocol(format!(
            "unexpected skin response: {other:?}"
        ))),
    }
}

/// Write `glyph` to the OS clipboard, then record emoji usage on the daemon.
pub fn copy_picker_item(
    client: &DaemonClient,
    writer: &dyn ClipboardWriter,
    kind: PickerKind,
    glyph: &str,
    base: &str,
) -> Result<()> {
    writer.write_text(glyph)?;
    if kind == PickerKind::Emoji {
        match client.request(Request::RecordEmojiUsage {
            glyph: base.to_string(),
        }) {
            Ok(Response::PickerUsage { .. }) | Ok(Response::Error { .. }) => {}
            Ok(other) => {
                return Err(clipl_core::Error::Protocol(format!(
                    "unexpected usage response: {other:?}"
                )))
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::thread;

    use clipl_protocol::{Envelope, Message};
    use std::os::unix::net::UnixListener;

    use crate::clipboard::RecordingClipboard;

    fn serve_one(path: &std::path::Path, reply: Response) {
        let listener = UnixListener::bind(path).unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let env = clipl_protocol::read_frame(&mut stream).unwrap();
            let Envelope { id, .. } = env;
            clipl_protocol::write_frame(
                &mut stream,
                &Envelope {
                    id,
                    payload: Message::Response(reply),
                },
            )
            .unwrap();
        });
    }

    #[test]
    fn maps_picker_list() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("e.sock");
        serve_one(
            &path,
            Response::PickerList(vec![PickerItem {
                glyph: "🐧".into(),
                base: "🐧".into(),
                name: "penguin".into(),
                category: "Animals & Nature".into(),
                has_skin_tones: false,
                variants: Vec::new(),
                favorite: false,
            }]),
        );
        let client = DaemonClient::with_socket(&path);
        let rows = search_emoji(&client, "linux", 10).unwrap();
        assert_eq!(rows[0].glyph, "🐧");
    }

    #[test]
    fn copy_writes_then_records() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("c.sock");
        serve_one(
            &path,
            Response::PickerUsage {
                glyph: "🔥".into(),
                count: 3,
            },
        );
        let client = DaemonClient::with_socket(&path);
        let sink = RecordingClipboard {
            writes: Mutex::new(Vec::new()),
        };
        copy_picker_item(&client, &sink, PickerKind::Emoji, "🔥", "🔥").unwrap();
        assert_eq!(*sink.writes.lock().unwrap(), vec!["🔥".to_string()]);
    }
}
