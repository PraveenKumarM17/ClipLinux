//! Emoji / symbol / kaomoji IPC handlers.

use std::collections::HashSet;

use clipl_clipboard::StorePickerKind;
use clipl_core::SkinTone;
use clipl_emoji::{EmojiCatalog, EmojiRecord};
use clipl_protocol::{PickerItem, PickerKind, Request, Response, SkinTonePref};
use clipl_symbols::SymbolCatalog;

use super::DaemonState;

const DEFAULT_LIMIT: usize = 80;
const CATEGORY_LIMIT: usize = 400;

impl DaemonState {
    pub(crate) fn handle_picker(&self, request: Request) -> Option<Response> {
        Some(match request {
            Request::SearchEmoji { query, limit } => self.search_emoji(&query, limit),
            Request::ListEmojiCategory { category, limit } => {
                self.list_emoji_category(&category, limit)
            }
            Request::GetFrequentlyUsedEmoji { limit } => {
                self.list_emoji_category("Frequently Used", limit)
            }
            Request::RecordEmojiUsage { glyph } => {
                self.record_usage(StorePickerKind::Emoji, &glyph)
            }
            Request::GetFavoriteEmoji => self.favorite_list(PickerKind::Emoji),
            Request::FavoriteEmoji { glyph } => self.set_favorite(PickerKind::Emoji, &glyph, true),
            Request::UnfavoriteEmoji { glyph } => {
                self.set_favorite(PickerKind::Emoji, &glyph, false)
            }
            Request::GetSkinTonePref => self.get_skin_pref(),
            Request::SetSkinTonePref { tone } => self.set_skin_pref(&tone),
            Request::SearchSymbols { query, limit } => {
                self.search_symbols(PickerKind::Symbol, &query, limit)
            }
            Request::ListSymbolCategory { category } => {
                self.list_symbols(PickerKind::Symbol, &category)
            }
            Request::SearchKaomoji { query, limit } => {
                self.search_symbols(PickerKind::Kaomoji, &query, limit)
            }
            Request::ListKaomojiCategory { category } => {
                self.list_symbols(PickerKind::Kaomoji, &category)
            }
            Request::GetFavoritePicker { kind } => self.favorite_list(kind),
            Request::FavoritePicker { kind, glyph } => self.set_favorite(kind, &glyph, true),
            Request::UnfavoritePicker { kind, glyph } => self.set_favorite(kind, &glyph, false),
            _ => return None,
        })
    }

    fn search_emoji(&self, query: &str, limit: u32) -> Response {
        let catalog = EmojiCatalog::shared();
        let tone = self.skin_tone();
        let favs = self.fav_set(StorePickerKind::Emoji);
        let hits = catalog.search(query, clamp_limit(limit, DEFAULT_LIMIT));
        let items = hits
            .into_iter()
            .map(|hit| emoji_item(hit.record, tone, &favs))
            .collect();
        Response::PickerList(items)
    }

    fn list_emoji_category(&self, category: &str, limit: u32) -> Response {
        let catalog = EmojiCatalog::shared();
        let tone = self.skin_tone();
        let favs = self.fav_set(StorePickerKind::Emoji);
        let cap = clamp_limit(limit, CATEGORY_LIMIT);
        let items = if category.eq_ignore_ascii_case("Frequently Used")
            || category.eq_ignore_ascii_case("frequent")
        {
            let Ok(engine) = self.engine.lock() else {
                return lock_err();
            };
            match engine.store().frequent_picker(StorePickerKind::Emoji, cap) {
                Ok(rows) => rows
                    .into_iter()
                    .filter_map(|row| {
                        catalog
                            .by_glyph(&row.glyph)
                            .map(|rec| emoji_item(rec, tone, &favs))
                    })
                    .collect(),
                Err(err) => {
                    return Response::Error {
                        message: err.to_string(),
                    }
                }
            }
        } else {
            catalog
                .list_group(category)
                .into_iter()
                .take(cap)
                .map(|rec| emoji_item(rec, tone, &favs))
                .collect()
        };
        Response::PickerList(items)
    }

    fn search_symbols(&self, kind: PickerKind, query: &str, limit: u32) -> Response {
        let catalog = catalog_for(kind);
        let favs = self.fav_set(store_kind(kind));
        let items = catalog
            .search(query, clamp_limit(limit, DEFAULT_LIMIT))
            .into_iter()
            .map(|sym| symbol_item(sym, kind, &favs))
            .collect();
        Response::PickerList(items)
    }

    fn list_symbols(&self, kind: PickerKind, category: &str) -> Response {
        let catalog = catalog_for(kind);
        let favs = self.fav_set(store_kind(kind));
        let items = if category.eq_ignore_ascii_case("Favorites") {
            match self.engine.lock() {
                Ok(engine) => match engine.store().picker_favorites(store_kind(kind)) {
                    Ok(glyphs) => glyphs
                        .into_iter()
                        .filter_map(|g| {
                            catalog
                                .by_glyph(&g)
                                .map(|sym| symbol_item(sym, kind, &favs))
                        })
                        .collect(),
                    Err(err) => {
                        return Response::Error {
                            message: err.to_string(),
                        }
                    }
                },
                Err(_) => return lock_err(),
            }
        } else {
            catalog
                .list_group(category)
                .into_iter()
                .map(|sym| symbol_item(sym, kind, &favs))
                .collect()
        };
        Response::PickerList(items)
    }

    fn record_usage(&self, kind: StorePickerKind, glyph: &str) -> Response {
        let Ok(engine) = self.engine.lock() else {
            return lock_err();
        };
        match engine.store().record_picker_usage(kind, glyph) {
            Ok(count) => Response::PickerUsage {
                glyph: glyph.into(),
                count,
            },
            Err(err) => Response::Error {
                message: err.to_string(),
            },
        }
    }

    fn set_favorite(&self, kind: PickerKind, glyph: &str, favorite: bool) -> Response {
        let Ok(engine) = self.engine.lock() else {
            return lock_err();
        };
        match engine
            .store()
            .set_picker_favorite(store_kind(kind), glyph, favorite)
        {
            Ok(favorite) => Response::PickerFavorite {
                glyph: glyph.into(),
                favorite,
            },
            Err(err) => Response::Error {
                message: err.to_string(),
            },
        }
    }

    fn favorite_list(&self, kind: PickerKind) -> Response {
        let favs = match self.engine.lock() {
            Ok(engine) => match engine.store().picker_favorites(store_kind(kind)) {
                Ok(v) => v,
                Err(err) => {
                    return Response::Error {
                        message: err.to_string(),
                    }
                }
            },
            Err(_) => return lock_err(),
        };
        let fav_set: HashSet<String> = favs.iter().cloned().collect();
        let items = match kind {
            PickerKind::Emoji => {
                let catalog = EmojiCatalog::shared();
                let tone = self.skin_tone();
                favs.into_iter()
                    .filter_map(|g| {
                        catalog
                            .by_glyph(&g)
                            .map(|rec| emoji_item(rec, tone, &fav_set))
                    })
                    .collect()
            }
            PickerKind::Symbol | PickerKind::Kaomoji => {
                let catalog = catalog_for(kind);
                favs.into_iter()
                    .filter_map(|g| {
                        catalog
                            .by_glyph(&g)
                            .map(|sym| symbol_item(sym, kind, &fav_set))
                    })
                    .collect()
            }
        };
        Response::PickerList(items)
    }

    fn get_skin_pref(&self) -> Response {
        Response::SkinTone(SkinTonePref {
            tone: skin_name(self.skin_tone()).into(),
        })
    }

    fn set_skin_pref(&self, tone: &str) -> Response {
        if parse_skin(tone).is_none() {
            return Response::Error {
                message: format!("unknown skin tone '{tone}'"),
            };
        }
        let Ok(engine) = self.engine.lock() else {
            return lock_err();
        };
        match engine.store().set_skin_tone_pref(tone) {
            Ok(()) => Response::SkinTone(SkinTonePref {
                tone: tone.to_ascii_lowercase(),
            }),
            Err(err) => Response::Error {
                message: err.to_string(),
            },
        }
    }

    fn skin_tone(&self) -> SkinTone {
        let Ok(engine) = self.engine.lock() else {
            return SkinTone::Default;
        };
        engine
            .store()
            .skin_tone_pref()
            .ok()
            .and_then(|s| parse_skin(&s))
            .unwrap_or(SkinTone::Default)
    }

    fn fav_set(&self, kind: StorePickerKind) -> HashSet<String> {
        self.engine
            .lock()
            .ok()
            .and_then(|engine| engine.store().picker_favorite_set(kind).ok())
            .unwrap_or_default()
    }
}

fn catalog_for(kind: PickerKind) -> &'static SymbolCatalog {
    match kind {
        PickerKind::Symbol => SymbolCatalog::symbols(),
        PickerKind::Kaomoji => SymbolCatalog::kaomoji(),
        PickerKind::Emoji => SymbolCatalog::symbols(),
    }
}

fn store_kind(kind: PickerKind) -> StorePickerKind {
    match kind {
        PickerKind::Emoji => StorePickerKind::Emoji,
        PickerKind::Symbol => StorePickerKind::Symbol,
        PickerKind::Kaomoji => StorePickerKind::Kaomoji,
    }
}

fn emoji_item(record: &EmojiRecord, tone: SkinTone, favs: &HashSet<String>) -> PickerItem {
    PickerItem {
        glyph: record.glyph_for(tone).to_string(),
        base: record.emoji.glyph.clone(),
        name: record.emoji.name.clone(),
        category: record.emoji.group.clone(),
        has_skin_tones: record.skin_tones.is_some(),
        variants: record
            .skin_tones
            .as_ref()
            .map(|tones| tones.to_vec())
            .unwrap_or_default(),
        favorite: favs.contains(&record.emoji.glyph),
    }
}

fn symbol_item(
    symbol: &clipl_symbols::Symbol,
    _kind: PickerKind,
    favs: &HashSet<String>,
) -> PickerItem {
    PickerItem {
        glyph: symbol.glyph.clone(),
        base: symbol.glyph.clone(),
        name: symbol.name.clone(),
        category: symbol.group.clone(),
        has_skin_tones: false,
        variants: Vec::new(),
        favorite: favs.contains(&symbol.glyph),
    }
}

fn clamp_limit(limit: u32, fallback: usize) -> usize {
    let n = limit as usize;
    if n == 0 {
        fallback
    } else {
        n.min(400)
    }
}

fn parse_skin(tone: &str) -> Option<SkinTone> {
    Some(match tone.trim().to_ascii_lowercase().as_str() {
        "default" | "yellow" | "none" => SkinTone::Default,
        "light" => SkinTone::Light,
        "medium_light" | "medium-light" => SkinTone::MediumLight,
        "medium" => SkinTone::Medium,
        "medium_dark" | "medium-dark" => SkinTone::MediumDark,
        "dark" => SkinTone::Dark,
        _ => return None,
    })
}

fn skin_name(tone: SkinTone) -> &'static str {
    match tone {
        SkinTone::Default => "default",
        SkinTone::Light => "light",
        SkinTone::MediumLight => "medium_light",
        SkinTone::Medium => "medium",
        SkinTone::MediumDark => "medium_dark",
        SkinTone::Dark => "dark",
    }
}

fn lock_err() -> Response {
    Response::Error {
        message: "history lock poisoned".into(),
    }
}
