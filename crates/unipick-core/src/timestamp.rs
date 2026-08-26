//! Unix-epoch timestamps used across UniPick records.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Milliseconds since the Unix epoch.
///
/// Stored as an integer so crates do not need a date-time library in the core
/// graph. Display formatting belongs in UI and CLI layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Current wall-clock time.
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(duration.as_millis() as i64)
    }

    /// Construct from milliseconds since epoch.
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Return milliseconds since epoch.
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}
