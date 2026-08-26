//! Ranked offline emoji search.

use crate::catalog::EmojiRecord;

/// One search hit with a deterministic rank.
#[derive(Clone, Copy, Debug)]
pub struct SearchHit<'a> {
    /// Matched record.
    pub record: &'a EmojiRecord,
    /// Higher is better.
    pub score: u32,
}

pub(crate) fn rank_records<'a>(
    records: &'a [EmojiRecord],
    query: &str,
    limit: usize,
) -> Vec<SearchHit<'a>> {
    let limit = limit.clamp(1, 400);
    let raw = query.trim();
    if raw.is_empty() {
        return Vec::new();
    }
    let lowered = raw.to_ascii_lowercase();
    let tokens: Vec<&str> = lowered
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut hits: Vec<SearchHit<'a>> = records
        .iter()
        .filter_map(|record| {
            let score = score_record(record, raw, &lowered, &tokens);
            (score > 0).then_some(SearchHit { record, score })
        })
        .collect();
    hits.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.record.emoji.name.cmp(&b.record.emoji.name))
            .then_with(|| a.record.emoji.glyph.cmp(&b.record.emoji.glyph))
    });
    hits.truncate(limit);
    hits
}

fn score_record(record: &EmojiRecord, raw: &str, lowered: &str, tokens: &[&str]) -> u32 {
    let emoji = &record.emoji;
    if emoji.glyph == raw {
        return 10_000;
    }
    let name = emoji.name.to_ascii_lowercase();
    let slug = emoji.slug.to_ascii_lowercase();
    if name == lowered {
        return 9_000;
    }
    if slug == lowered.replace(' ', "-") {
        return 8_800;
    }
    if emoji
        .keywords
        .iter()
        .chain(emoji.aliases.iter())
        .any(|kw| kw.eq_ignore_ascii_case(lowered))
    {
        return 8_500;
    }

    let name_words: Vec<&str> = name.split(|c: char| !c.is_ascii_alphanumeric()).collect();
    if tokens.iter().all(|tok| name_words.iter().any(|w| w == tok)) {
        return 8_000;
    }
    if tokens
        .iter()
        .all(|tok| name_words.iter().any(|w| w.starts_with(tok)))
    {
        return 7_200;
    }
    if tokens
        .iter()
        .all(|tok| name.contains(tok) || slug.contains(tok))
    {
        return 5_500;
    }

    let mut matched = 0u32;
    for tok in tokens {
        let in_kw = emoji
            .keywords
            .iter()
            .chain(emoji.aliases.iter())
            .any(|kw| kw == tok || kw.starts_with(tok) || kw.contains(tok));
        if in_kw || name.contains(tok) || slug.contains(tok) {
            matched += 1;
        }
    }
    if matched == tokens.len() as u32 {
        return 4_000 + matched * 10;
    }
    if matched > 0 {
        return 1_000 + matched * 50;
    }
    0
}

#[cfg(test)]
mod tests {
    use crate::catalog::EmojiCatalog;

    #[test]
    fn smile_ranks_grinning_faces() {
        let catalog = EmojiCatalog::load_packed();
        let hits = catalog.search("smile", 8);
        let glyphs: Vec<&str> = hits.iter().map(|h| h.record.emoji.glyph.as_str()).collect();
        assert!(glyphs.contains(&"😀"), "{glyphs:?}");
        assert!(hits.windows(2).all(|w| w[0].score >= w[1].score));
    }

    #[test]
    fn rocket_and_fire_and_thumb() {
        let catalog = EmojiCatalog::load_packed();
        assert_eq!(catalog.search("rocket", 5)[0].record.emoji.glyph, "🚀");
        assert_eq!(catalog.search("fire", 5)[0].record.emoji.glyph, "🔥");
        let thumbs: Vec<_> = catalog
            .search("thumb", 8)
            .iter()
            .map(|h| h.record.emoji.glyph.as_str())
            .collect();
        assert!(thumbs.contains(&"👍"), "{thumbs:?}");
        assert!(thumbs.contains(&"👎"), "{thumbs:?}");
    }

    #[test]
    fn india_and_computer_and_heart() {
        let catalog = EmojiCatalog::load_packed();
        assert_eq!(catalog.search("india", 5)[0].record.emoji.glyph, "🇮🇳");
        let computers: Vec<_> = catalog
            .search("computer", 12)
            .iter()
            .map(|h| h.record.emoji.glyph.as_str())
            .collect();
        assert!(
            computers.iter().any(|g| *g == "💻" || g.contains('💻')),
            "{computers:?}"
        );
        assert!(catalog.search("heart", 5).iter().any(|h| h
            .record
            .emoji
            .name
            .to_ascii_lowercase()
            .contains("heart")));
    }

    #[test]
    fn case_and_limit_and_empty() {
        let catalog = EmojiCatalog::load_packed();
        let a = catalog.search("PENGUIN", 5);
        let b = catalog.search("penguin", 5);
        assert_eq!(a[0].record.emoji.glyph, b[0].record.emoji.glyph);
        assert_eq!(catalog.search("face", 3).len(), 3);
        assert!(catalog.search("   ", 10).is_empty());
        assert!(catalog.search("linux", 5)[0].record.emoji.glyph == "🐧");
    }

    #[test]
    fn multi_word_and_alias() {
        let catalog = EmojiCatalog::load_packed();
        let hits = catalog.search("thumbs up", 5);
        assert_eq!(hits[0].record.emoji.glyph, "👍");
        assert_eq!(
            catalog.search("grinning face", 5)[0].record.emoji.slug,
            "grinning-face"
        );
    }

    #[test]
    fn deterministic() {
        let catalog = EmojiCatalog::load_packed();
        let a: Vec<_> = catalog
            .search("star", 20)
            .iter()
            .map(|h| h.record.emoji.glyph.clone())
            .collect();
        let b: Vec<_> = catalog
            .search("star", 20)
            .iter()
            .map(|h| h.record.emoji.glyph.clone())
            .collect();
        assert_eq!(a, b);
    }
}
