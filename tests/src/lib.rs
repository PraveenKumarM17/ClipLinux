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
fn memory_clipboard_is_a_valid_backend() {
    let backend = clipl_core::placeholders::MemoryClipboard::default();
    assert_eq!(backend.name(), "memory");
    assert!(backend.supports_watch());
}
