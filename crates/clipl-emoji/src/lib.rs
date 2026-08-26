//! Emoji catalog and ranked search.
//!
//! Runtime data is the compact file generated from Unicode 17.0 `emoji-test.txt`
//! plus CLDR 48.2 English annotations. See `packages/emoji-data/README.md`.

#![forbid(unsafe_code)]

mod catalog;
mod search;
mod skin;

pub use catalog::{EmojiCatalog, EmojiRecord, UNICODE_VERSION};
pub use search::SearchHit;
pub use skin::apply_skin;

pub use clipl_core::SkinTone;
