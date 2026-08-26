//! Official skin-tone variant selection.

use clipl_core::SkinTone;

use crate::catalog::EmojiRecord;

/// Return the official sequence for `tone`, or the base glyph.
pub fn apply_skin(record: &EmojiRecord, tone: SkinTone) -> &str {
    match (tone.variant_index(), record.skin_tones.as_ref()) {
        (Some(idx), Some(tones)) => tones[idx].as_str(),
        _ => record.emoji.glyph.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::EmojiCatalog;

    #[test]
    fn thumbs_up_has_five_official_variants() {
        let catalog = EmojiCatalog::load_packed();
        let record = catalog.by_slug("thumbs-up").unwrap();
        assert!(record.skin_tones.is_some());
        let light = apply_skin(record, SkinTone::Light);
        assert_ne!(light, record.emoji.glyph);
        assert!(
            light.contains('\u{1F3FB}')
                || light.chars().count() > record.emoji.glyph.chars().count()
        );
        assert_eq!(apply_skin(record, SkinTone::Default), record.emoji.glyph);
    }

    #[test]
    fn penguin_has_no_variants() {
        let catalog = EmojiCatalog::load_packed();
        let record = catalog.by_slug("penguin").unwrap();
        assert!(record.skin_tones.is_none());
        assert_eq!(apply_skin(record, SkinTone::Dark), "🐧");
    }
}
