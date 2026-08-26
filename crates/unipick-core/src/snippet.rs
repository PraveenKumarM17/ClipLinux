//! Snippet domain type.

use serde::{Deserialize, Serialize};

use crate::id::SnippetId;
use crate::timestamp::Timestamp;

/// A reusable text fragment that can be pasted or expanded.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snippet {
    /// Stable identity.
    pub id: SnippetId,
    /// Display name.
    pub name: String,
    /// Body pasted when the snippet is selected.
    pub body: String,
    /// Optional trigger token, e.g. `;sig`.
    pub trigger: Option<String>,
    /// Optional language or format hint (`markdown`, `html`, `plain`).
    pub language: Option<String>,
    /// Optional grouping label.
    pub folder: Option<String>,
    /// When the snippet was created.
    pub created_at: Timestamp,
    /// When the snippet was last updated.
    pub updated_at: Timestamp,
}

impl Snippet {
    /// Create a plain-text snippet.
    pub fn new(name: impl Into<String>, body: impl Into<String>) -> Self {
        let now = Timestamp::now();
        Self {
            id: SnippetId::new(),
            name: name.into(),
            body: body.into(),
            trigger: None,
            language: Some("plain".to_string()),
            folder: None,
            created_at: now,
            updated_at: now,
        }
    }
}
