# Emoji engine

Status: **IMPLEMENTED** (Unicode 17.0, offline).

Search and ranking live in `clipl-emoji`. Persistence (usage, favorites, skin
tone) lives in `clipl-daemon` SQLite. The desktop UI never opens the database
and never receives the full catalog.

## Data

See [packages/emoji-data/README.md](../../packages/emoji-data/README.md).

- Authoritative list: Unicode `emoji-test.txt` fully-qualified sequences
- Keywords: CLDR 48.2 English annotations + a small `aliases.json`
- Skin tones: official sequences from `emoji-test.txt` only (never concatenated)

## Search

`EmojiCatalog::search(query, limit)`:

1. Tokenize on whitespace, case-fold ASCII
2. Score exact glyph / name / slug / keyword highest
3. Then all-token name-word matches, prefixes, substrings
4. Sort by score desc, name, glyph (deterministic)
5. Truncate to `limit` (max 400)

Empty query returns no hits; the UI lists a category instead.

## Skin tones

`SkinTone::{Default, Light, MediumLight, Medium, MediumDark, Dark}`.

If a record has five official variants, the daemon applies the stored preference
to `PickerItem.glyph`. `PickerItem.variants` lists the official sequences for
the popover. Emoji without variants are unchanged.

## IPC (daemon)

`SearchEmoji`, `ListEmojiCategory`, `GetFrequentlyUsedEmoji`, `RecordEmojiUsage`,
`GetFavoriteEmoji`, `FavoriteEmoji`, `UnfavoriteEmoji`, `GetSkinTonePref`,
`SetSkinTonePref`.

Responses are `PickerList` rows (`glyph`, `base`, `name`, `category`,
`has_skin_tones`, `variants`, `favorite`) — no keyword dumps.
