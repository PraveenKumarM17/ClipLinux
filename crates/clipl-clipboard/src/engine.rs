//! Privacy → dedup → persistence pipeline.

use clipl_core::{ClipLinuxConfig, ClipboardItem, ClipboardItemId, PrivacyRule, Result, Timestamp};
use clipl_privacy::{evaluate, PrivacyDecision, PrivacyVerdict};

use crate::hash::content_hash;
use crate::store::{HistoryQuery, HistoryStore};

/// Result of attempting to persist a clipboard item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordOutcome {
    /// Item was stored as a new row.
    Stored,
    /// Consecutive duplicate: existing row was reused.
    Reused,
    /// Privacy policy excluded the item. Nothing was written.
    Excluded,
    /// Caller must confirm before storing.
    NeedsConfirmation,
    /// Not persisted (history disabled, or non-text in this phase).
    Skipped,
}

/// History recording with privacy-before-write.
pub struct HistoryEngine<S: HistoryStore> {
    store: S,
    rules: Vec<PrivacyRule>,
    config: ClipLinuxConfig,
}

impl<S: HistoryStore> HistoryEngine<S> {
    /// Create an engine.
    pub fn new(store: S, rules: Vec<PrivacyRule>, config: ClipLinuxConfig) -> Self {
        Self {
            store,
            rules,
            config,
        }
    }

    /// Underlying store (tests and IPC handlers).
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Record `item` after privacy and dedup. Mutates hash/labels on a copy.
    pub fn record(&self, item: &ClipboardItem) -> Result<Recorded> {
        if !self.config.history.enabled {
            return Ok(Recorded {
                outcome: RecordOutcome::Skipped,
                item_id: item.id,
                verdict: None,
            });
        }
        if !persistable(item) {
            return Ok(Recorded {
                outcome: RecordOutcome::Skipped,
                item_id: item.id,
                verdict: None,
            });
        }

        let mut item = item.clone();
        item.content_hash = content_hash(&item.content);
        let verdict = evaluate(&item, &self.rules, &self.config.privacy);
        item.sensitive = verdict.labels.clone();

        match verdict.decision {
            PrivacyDecision::Exclude => {
                return Ok(Recorded {
                    outcome: RecordOutcome::Excluded,
                    item_id: item.id,
                    verdict: Some(verdict),
                });
            }
            PrivacyDecision::Confirm => {
                return Ok(Recorded {
                    outcome: RecordOutcome::NeedsConfirmation,
                    item_id: item.id,
                    verdict: Some(verdict),
                });
            }
            PrivacyDecision::Redact => {
                redact_payload(&mut item);
            }
            PrivacyDecision::Expire { ttl_ms } => {
                item.expires_at = Some(Timestamp::now().saturating_add_millis(ttl_ms));
            }
            PrivacyDecision::Allow => {}
        }

        if self.config.clipboard.deduplication_policy == "consecutive" {
            if let Some(latest) = self.store.latest()? {
                if latest.content_hash == item.content_hash {
                    let id = latest.id;
                    let mut reused = latest;
                    reused.last_used_at = Some(Timestamp::now());
                    reused.updated_at = Timestamp::now();
                    self.store.update(&reused)?;
                    return Ok(Recorded {
                        outcome: RecordOutcome::Reused,
                        item_id: id,
                        verdict: Some(verdict),
                    });
                }
            }
        }

        if item.updated_at.as_millis() == 0 {
            item.updated_at = item.created_at;
        }
        self.store.insert(&item)?;
        self.apply_retention()?;
        Ok(Recorded {
            outcome: RecordOutcome::Stored,
            item_id: item.id,
            verdict: Some(verdict),
        })
    }

    /// Apply max_items and max_age. Pinned rows are kept.
    pub fn apply_retention(&self) -> Result<()> {
        let max = self.config.history.max_items as usize;
        self.store.enforce_limit(max)?;
        let cutoff = if self.config.history.max_age_days == 0 {
            None
        } else {
            let millis = i64::from(self.config.history.max_age_days) * 86_400_000;
            Some(Timestamp::from_millis(
                Timestamp::now().as_millis().saturating_sub(millis),
            ))
        };
        self.store.expire(Timestamp::now(), cutoff)?;
        Ok(())
    }

    /// List recent items.
    pub fn list(&self, limit: usize) -> Result<Vec<ClipboardItem>> {
        self.store.list(&HistoryQuery::latest(limit))
    }

    /// Search text items.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<ClipboardItem>> {
        self.store.search(query, limit)
    }

    /// Fetch one item.
    pub fn get(&self, id: ClipboardItemId) -> Result<Option<ClipboardItem>> {
        self.store.get(id)
    }

    /// Delete one item.
    pub fn delete(&self, id: ClipboardItemId) -> Result<bool> {
        self.store.delete(id)
    }

    /// Clear unpinned history.
    pub fn clear(&self) -> Result<u64> {
        self.store.clear_unpinned()
    }

    /// Pin or unpin.
    pub fn set_pinned(&self, id: ClipboardItemId, pinned: bool) -> Result<()> {
        let mut item = self
            .store
            .get(id)?
            .ok_or_else(|| clipl_core::Error::not_found(id.to_string()))?;
        item.pinned = pinned;
        item.updated_at = Timestamp::now();
        self.store.update(&item)
    }
}

/// Outcome plus identifiers. `verdict` is omitted for skip paths.
pub struct Recorded {
    /// What happened.
    pub outcome: RecordOutcome,
    /// Item id (new, reused, or the excluded candidate).
    pub item_id: ClipboardItemId,
    /// Privacy explanation when evaluation ran.
    pub verdict: Option<PrivacyVerdict>,
}

fn persistable(item: &ClipboardItem) -> bool {
    matches!(
        item.content,
        clipl_core::ClipboardContent::Text { .. }
            | clipl_core::ClipboardContent::Html { .. }
            | clipl_core::ClipboardContent::Uri { .. }
    )
}

fn redact_payload(item: &mut ClipboardItem) {
    match &mut item.content {
        clipl_core::ClipboardContent::Text { text, .. } => {
            *text = String::new();
        }
        clipl_core::ClipboardContent::Html { html, plain, .. } => {
            *html = String::new();
            *plain = Some(String::new());
        }
        clipl_core::ClipboardContent::Uri { uri } => {
            *uri = String::new();
        }
        _ => {}
    }
}
