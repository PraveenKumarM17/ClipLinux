//! Picker usage, favorites, and skin-tone preference (same SQLite file).

use clipl_core::{Error, Result, Timestamp};
use rusqlite::params;

use crate::sqlite::SqliteStore;

/// Maximum usage rows retained per picker kind.
pub const MAX_USAGE_ROWS: usize = 200;

/// Catalog kind stored in picker tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerKind {
    /// Unicode emoji.
    Emoji,
    /// Curated symbol.
    Symbol,
    /// Kaomoji.
    Kaomoji,
}

impl PickerKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Emoji => "emoji",
            Self::Symbol => "symbol",
            Self::Kaomoji => "kaomoji",
        }
    }
}

/// One usage row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageRow {
    /// Glyph key.
    pub glyph: String,
    /// Copy count.
    pub count: u64,
    /// Last copy, milliseconds.
    pub last_used_at: i64,
}

impl SqliteStore {
    /// Increment usage for `glyph`. Bounded per kind.
    pub fn record_picker_usage(&self, kind: PickerKind, glyph: &str) -> Result<u64> {
        if glyph.is_empty() {
            return Err(Error::invalid("empty picker glyph"));
        }
        let now = Timestamp::now().as_millis();
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO picker_usage (kind, glyph, count, last_used_at)
             VALUES (?1, ?2, 1, ?3)
             ON CONFLICT(kind, glyph) DO UPDATE SET
                count = picker_usage.count + 1,
                last_used_at = excluded.last_used_at",
            params![kind.as_str(), glyph, now],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT count FROM picker_usage WHERE kind = ?1 AND glyph = ?2",
                params![kind.as_str(), glyph],
                |row| row.get(0),
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        conn.execute(
            "DELETE FROM picker_usage WHERE kind = ?1 AND rowid IN (
                SELECT rowid FROM picker_usage WHERE kind = ?1
                ORDER BY count ASC, last_used_at ASC
                LIMIT MAX(0, (SELECT COUNT(*) FROM picker_usage WHERE kind = ?1) - ?2)
            )",
            params![kind.as_str(), MAX_USAGE_ROWS as i64],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(count as u64)
    }

    /// Usage-ranked glyphs.
    pub fn frequent_picker(&self, kind: PickerKind, limit: usize) -> Result<Vec<UsageRow>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare(
                "SELECT glyph, count, last_used_at FROM picker_usage
                 WHERE kind = ?1
                 ORDER BY count DESC, last_used_at DESC
                 LIMIT ?2",
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![kind.as_str(), limit.max(1) as i64], |row| {
                Ok(UsageRow {
                    glyph: row.get(0)?,
                    count: row.get::<_, i64>(1)? as u64,
                    last_used_at: row.get(2)?,
                })
            })
            .map_err(|err| Error::Storage(err.to_string()))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| Error::Storage(err.to_string()))
    }

    /// Set favorite flag.
    pub fn set_picker_favorite(
        &self,
        kind: PickerKind,
        glyph: &str,
        favorite: bool,
    ) -> Result<bool> {
        if glyph.is_empty() {
            return Err(Error::invalid("empty picker glyph"));
        }
        let conn = self.lock()?;
        if favorite {
            conn.execute(
                "INSERT INTO picker_favorites (kind, glyph, created_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(kind, glyph) DO NOTHING",
                params![kind.as_str(), glyph, Timestamp::now().as_millis()],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        } else {
            conn.execute(
                "DELETE FROM picker_favorites WHERE kind = ?1 AND glyph = ?2",
                params![kind.as_str(), glyph],
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        }
        Ok(favorite)
    }

    /// Favorite glyphs, newest first.
    pub fn picker_favorites(&self, kind: PickerKind) -> Result<Vec<String>> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT glyph FROM picker_favorites WHERE kind = ?1 ORDER BY created_at DESC")
            .map_err(|err| Error::Storage(err.to_string()))?;
        let rows = stmt
            .query_map(params![kind.as_str()], |row| row.get(0))
            .map_err(|err| Error::Storage(err.to_string()))?;
        rows.collect::<std::result::Result<Vec<String>, _>>()
            .map_err(|err| Error::Storage(err.to_string()))
    }

    /// Whether `glyph` is favorited.
    pub fn is_picker_favorite(&self, kind: PickerKind, glyph: &str) -> Result<bool> {
        let conn = self.lock()?;
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM picker_favorites WHERE kind = ?1 AND glyph = ?2",
                params![kind.as_str(), glyph],
                |row| row.get(0),
            )
            .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(n > 0)
    }

    /// Favorite set as a lookup table.
    pub fn picker_favorite_set(
        &self,
        kind: PickerKind,
    ) -> Result<std::collections::HashSet<String>> {
        Ok(self.picker_favorites(kind)?.into_iter().collect())
    }

    /// Skin-tone preference (`default` if unset).
    pub fn skin_tone_pref(&self) -> Result<String> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT value FROM picker_prefs WHERE key = 'emoji_skin_tone'",
            [],
            |row| row.get(0),
        )
        .optional_pref()
    }

    /// Persist skin-tone preference.
    pub fn set_skin_tone_pref(&self, tone: &str) -> Result<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT INTO picker_prefs (key, value) VALUES ('emoji_skin_tone', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![tone],
        )
        .map_err(|err| Error::Storage(err.to_string()))?;
        Ok(())
    }
}

trait OptionalPref {
    fn optional_pref(self) -> Result<String>;
}

impl OptionalPref for rusqlite::Result<String> {
    fn optional_pref(self) -> Result<String> {
        match self {
            Ok(value) => Ok(value),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok("default".into()),
            Err(err) => Err(Error::Storage(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::SCHEMA_VERSION;

    #[test]
    fn usage_ranks_and_is_bounded() {
        let store = SqliteStore::memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        store.record_picker_usage(PickerKind::Emoji, "🔥").unwrap();
        store.record_picker_usage(PickerKind::Emoji, "🚀").unwrap();
        store.record_picker_usage(PickerKind::Emoji, "🚀").unwrap();
        let top = store.frequent_picker(PickerKind::Emoji, 10).unwrap();
        assert_eq!(top[0].glyph, "🚀");
        assert_eq!(top[0].count, 2);
        for i in 0..MAX_USAGE_ROWS + 20 {
            store
                .record_picker_usage(PickerKind::Emoji, &format!("g{i}"))
                .unwrap();
        }
        let all = store.frequent_picker(PickerKind::Emoji, 500).unwrap();
        assert!(all.len() <= MAX_USAGE_ROWS);
    }

    #[test]
    fn favorites_and_skin_pref() {
        let store = SqliteStore::memory().unwrap();
        store
            .set_picker_favorite(PickerKind::Emoji, "🐧", true)
            .unwrap();
        assert!(store.is_picker_favorite(PickerKind::Emoji, "🐧").unwrap());
        store
            .set_picker_favorite(PickerKind::Symbol, "—", true)
            .unwrap();
        assert_eq!(
            store.picker_favorites(PickerKind::Symbol).unwrap(),
            vec!["—"]
        );
        store.set_skin_tone_pref("medium").unwrap();
        assert_eq!(store.skin_tone_pref().unwrap(), "medium");
        store
            .set_picker_favorite(PickerKind::Emoji, "🐧", false)
            .unwrap();
        assert!(!store.is_picker_favorite(PickerKind::Emoji, "🐧").unwrap());
    }
}
