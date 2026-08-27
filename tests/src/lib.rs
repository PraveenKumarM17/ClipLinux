//! Cross-crate smoke tests for the ClipLinux foundation.
#![cfg(test)]

use clipl_clipboard::{ClipboardHistory, RecordOutcome};
use clipl_core::{
    ClipboardBackend, ClipboardItem, MediaQuery, PlatformAdapter, StickerPackProvider,
};
use clipl_emoji::EmojiCatalog;
use clipl_media::{LocalStickerLibrary, MediaRegistry};
use clipl_platform::{select_adapter, AdapterKind};
use clipl_privacy::{decide, default_rules, PrivacyDecision};
use clipl_protocol::{Envelope, Message, Request};
use clipl_snippets::SnippetLibrary;
use clipl_symbols::SymbolCatalog;

#[test]
fn workspace_crates_compose() {
    let adapter = select_adapter();
    let _caps = adapter.capabilities();
    let _kind = AdapterKind::preferred(&adapter.identity());

    let history = ClipboardHistory::default();
    let item = ClipboardItem::text("workspace smoke");
    assert_eq!(history.record(&item).unwrap(), RecordOutcome::Stored);

    let catalog = EmojiCatalog::load_packed();
    assert!(!catalog.search("penguin", 8).is_empty());

    let symbols = SymbolCatalog::builtin();
    assert!(!symbols.search("euro", 8).is_empty());

    let snippets = SnippetLibrary::default();
    snippets
        .upsert(&clipl_core::Snippet::new("hello", "world"))
        .unwrap();
    assert_eq!(snippets.list().unwrap().len(), 1);

    let media = MediaRegistry::default();
    assert!(media.search(&MediaQuery::new("gif")).unwrap().is_empty());
    assert!(LocalStickerLibrary::default()
        .list_packs()
        .unwrap()
        .is_empty());

    let decision = decide(&item, &default_rules());
    assert_eq!(decision, PrivacyDecision::Allow);

    let ping = Envelope::new(Message::Request(Request::Ping));
    assert!(!ping.to_json_bytes().unwrap().is_empty());
}

#[test]
fn gnome_extension_metadata_is_well_formed() {
    let meta = include_str!("../../extensions/gnome/metadata.json");
    assert!(meta.contains("\"uuid\": \"clipl@io.clipl\""));
    assert!(meta.contains("org.gnome.shell.extensions.clipl"));
    assert!(meta.contains("\"46\""));
    assert!(meta.contains("\"47\""));
    assert!(meta.contains("\"48\""));
    assert!(meta.contains("\"50\""));
}

#[test]
fn gnome_extension_does_not_spawn_shell() {
    let js = include_str!("../../extensions/gnome/extension.js");
    assert!(js.contains("ToggleDesktop"));
    assert!(js.contains("SubscribeInsert"));
    assert!(js.contains("InsertIntoApp"));
    assert!(js.contains("create_virtual_device"));
    assert!(js.contains("KEY_Control_L"));
    assert!(js.contains("addKeybinding"));
    assert!(js.contains("removeKeybinding"));
    assert!(!js.contains("spawn_command_line"));
    assert!(!js.contains("GLib.spawn"));
    assert!(!js.contains("xdg-open"));
    assert!(!js.contains("ydotool"));
}

#[test]
fn linux_desktop_file_is_well_formed() {
    let desktop = include_str!("../../packaging/linux/io.clipl.ClipLinux.desktop");
    assert!(desktop.contains("Name=ClipLinux"));
    assert!(desktop.contains("Exec=clipl-desktop"));
    assert!(desktop.contains("io.clipl.ClipLinux"));
    assert!(!desktop.contains("xdg-open"));
}

#[test]
fn gnome_schema_declares_shortcut() {
    let xml =
        include_str!("../../extensions/gnome/schemas/org.gnome.shell.extensions.clipl.gschema.xml");
    assert!(xml.contains("activate-shortcut"));
    assert!(
        xml.contains("&lt;Super&gt;&lt;Alt&gt;v")
            || xml.contains("<![CDATA[['<Super><Alt>v']]]>")
    );
}

#[test]
fn memory_clipboard_is_a_valid_backend() {
    let backend = clipl_core::placeholders::MemoryClipboard::default();
    assert_eq!(backend.name(), "memory");
    assert!(backend.supports_watch());
}
