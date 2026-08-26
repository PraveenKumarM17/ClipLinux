//! Cross-crate smoke tests for the UniPick foundation.
#![cfg(test)]

use unipick_clipboard::{ClipboardHistory, RecordOutcome};
use unipick_core::{
    ClipboardBackend, ClipboardItem, MediaQuery, PlatformAdapter, StickerPackProvider,
};
use unipick_emoji::EmojiCatalog;
use unipick_media::{LocalStickerLibrary, MediaRegistry};
use unipick_platform::{select_adapter, AdapterKind};
use unipick_privacy::{decide, default_rules, PrivacyDecision};
use unipick_protocol::{Envelope, Message, Request};
use unipick_snippets::SnippetLibrary;
use unipick_symbols::SymbolCatalog;

#[test]
fn workspace_crates_compose() {
    let adapter = select_adapter();
    let _caps = adapter.capabilities();
    let _kind = AdapterKind::preferred(&adapter.identity());

    let history = ClipboardHistory::default();
    let item = ClipboardItem::text("workspace smoke");
    assert_eq!(history.record(&item).unwrap(), RecordOutcome::Stored);

    let catalog = EmojiCatalog::builtin();
    assert!(!catalog.search("penguin").is_empty());

    let symbols = SymbolCatalog::builtin();
    assert!(!symbols.search("euro").is_empty());

    let snippets = SnippetLibrary::default();
    snippets
        .upsert(&unipick_core::Snippet::new("hello", "world"))
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
    let backend = unipick_core::placeholders::MemoryClipboard::default();
    assert_eq!(backend.name(), "memory");
    assert!(!backend.supports_watch());
}
