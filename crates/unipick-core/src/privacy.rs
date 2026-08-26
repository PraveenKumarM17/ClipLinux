//! Privacy policy types. Filtering logic lives in `unipick-privacy`.

use serde::{Deserialize, Serialize};

use crate::id::PrivacyRuleId;

/// Categories of content that must not be stored casually.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SensitiveContentType {
    /// Password or passphrase material.
    Password,
    /// Payment card number.
    CreditCard,
    /// Cryptographic private key or seed.
    PrivateKey,
    /// API token, session cookie, or bearer credential.
    Token,
    /// Government or civil identifier.
    PersonalIdentifier,
    /// One-time code.
    OneTimeCode,
    /// User-defined category.
    Custom(String),
}

impl SensitiveContentType {
    /// Stable identifier for docs, rules, and CLI output.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Password => "password",
            Self::CreditCard => "credit-card",
            Self::PrivateKey => "private-key",
            Self::Token => "token",
            Self::PersonalIdentifier => "personal-identifier",
            Self::OneTimeCode => "otp",
            Self::Custom(name) => name,
        }
    }
}

/// A user or system rule applied before clipboard history persistence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivacyRule {
    /// Stable identity.
    pub id: PrivacyRuleId,
    /// Display name.
    pub name: String,
    /// Whether the rule is active.
    pub enabled: bool,
    /// What the rule matches.
    pub matchers: Vec<PrivacyMatcher>,
    /// What happens on a match.
    pub action: PrivacyAction,
}

impl PrivacyRule {
    /// Exclude a sensitive category from history.
    pub fn exclude(name: impl Into<String>, kind: SensitiveContentType) -> Self {
        Self {
            id: PrivacyRuleId::new(),
            name: name.into(),
            enabled: true,
            matchers: vec![PrivacyMatcher::Sensitive(kind)],
            action: PrivacyAction::ExcludeFromHistory,
        }
    }
}

/// Matcher for a [`PrivacyRule`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PrivacyMatcher {
    /// Matches a classified sensitive category.
    Sensitive(SensitiveContentType),
    /// Matches a MIME type prefix, e.g. `image/`.
    MimePrefix(String),
    /// Matches an application identifier when the platform provides one.
    ApplicationId(String),
    /// Matches when the text contains a literal needle.
    TextContains(String),
}

/// Action taken when a rule matches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PrivacyAction {
    /// Do not persist the item.
    ExcludeFromHistory,
    /// Persist metadata only, drop payload bytes.
    RedactPayload,
    /// Persist, then delete after this many milliseconds.
    ExpireAfter { ttl_ms: u64 },
    /// Persist only after the user confirms.
    RequireConfirmation,
}
