# Symbols engine

Status: **IMPLEMENTED** (curated offline catalogs).

`clipl-symbols` loads packed JSON from `packages/symbols-data/`:

- `symbols.json` — practical punctuation, arrows, math, currency, technical,
  Greek, Latin Extended, shapes, stars, weather, units
- `kaomoji.json` — Happy, Sad, Angry, Shrug, Table Flip, Cute, Surprised,
  Actions, Other

This is **not** the full Unicode character set. Glyphs are stored as Unicode
strings so kaomoji combining marks survive copy.

## Search

Same ranking idea as emoji (exact glyph/name/keyword, then substring). Empty
query lists a category. Results are bounded.

## Persistence

Favorites share the daemon `picker_favorites` table with `kind=symbol|kaomoji`.
Usage ranking is emoji-only in this phase.

## IPC

`SearchSymbols`, `ListSymbolCategory`, `SearchKaomoji`, `ListKaomojiCategory`,
`GetFavoritePicker`, `FavoritePicker`, `UnfavoritePicker`.
