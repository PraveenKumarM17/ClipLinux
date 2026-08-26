//! Privacy filtering for clipboard history.
//!
//! Classification in this foundation is intentionally conservative: it never
//! guesses. Production detectors land in a later milestone.

#![forbid(unsafe_code)]

use unipick_core::{
    ClipboardContent, ClipboardItem, PrivacyAction, PrivacyMatcher, PrivacyRule,
    SensitiveContentType,
};

pub use unipick_core::{PrivacyMatcher as Matcher, PrivacyRule as Rule};

/// Decision produced by the privacy engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PrivacyDecision {
    /// Persist the item as-is.
    Allow,
    /// Do not persist.
    Exclude,
    /// Persist metadata, drop payload bytes.
    Redact,
    /// Persist with a time-to-live in milliseconds.
    Expire { ttl_ms: u64 },
    /// Ask the user before persisting.
    Confirm,
}

/// Evaluate enabled rules against an item. First matching action wins.
pub fn decide(item: &ClipboardItem, rules: &[PrivacyRule]) -> PrivacyDecision {
    for rule in rules.iter().filter(|rule| rule.enabled) {
        if rule_matches(item, rule) {
            return match &rule.action {
                PrivacyAction::ExcludeFromHistory => PrivacyDecision::Exclude,
                PrivacyAction::RedactPayload => PrivacyDecision::Redact,
                PrivacyAction::ExpireAfter { ttl_ms } => {
                    PrivacyDecision::Expire { ttl_ms: *ttl_ms }
                }
                PrivacyAction::RequireConfirmation => PrivacyDecision::Confirm,
                _ => PrivacyDecision::Exclude,
            };
        }
    }
    PrivacyDecision::Allow
}

/// Default rules shipped with UniPick. Detectors are not wired yet, so these
/// rules only fire when content is already labelled sensitive.
pub fn default_rules() -> Vec<PrivacyRule> {
    vec![
        PrivacyRule::exclude("Exclude passwords", SensitiveContentType::Password),
        PrivacyRule::exclude("Exclude private keys", SensitiveContentType::PrivateKey),
        PrivacyRule::exclude("Exclude tokens", SensitiveContentType::Token),
        PrivacyRule::exclude("Exclude payment cards", SensitiveContentType::CreditCard),
    ]
}

/// Placeholder classifier. Returns labels already attached to the item.
///
/// Real detectors (password-manager MIME, luhn, PEM headers) are deferred so
/// this crate does not ship false-positive heuristics without review.
pub fn classify(item: &ClipboardItem) -> Vec<SensitiveContentType> {
    item.sensitive.clone()
}

fn rule_matches(item: &ClipboardItem, rule: &PrivacyRule) -> bool {
    rule.matchers.iter().any(|matcher| match matcher {
        PrivacyMatcher::Sensitive(kind) => item.sensitive.contains(kind),
        PrivacyMatcher::MimePrefix(prefix) => mime_matches(&item.content, prefix),
        PrivacyMatcher::ApplicationId(_) => false,
        PrivacyMatcher::TextContains(needle) => match &item.content {
            ClipboardContent::Text { text, .. } => text.contains(needle),
            ClipboardContent::Html { html, plain, .. } => {
                html.contains(needle) || plain.as_ref().is_some_and(|p| p.contains(needle))
            }
            _ => false,
        },
        _ => false,
    })
}

fn mime_matches(content: &ClipboardContent, prefix: &str) -> bool {
    match content {
        ClipboardContent::Text { mime, .. } | ClipboardContent::Image { mime, .. } => {
            mime.starts_with(prefix)
        }
        ClipboardContent::Html { .. } => "text/html".starts_with(prefix) || prefix == "text/",
        ClipboardContent::Custom { mime, .. } => mime.starts_with(prefix),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unipick_core::ClipboardItem;

    #[test]
    fn unlabeled_text_is_allowed() {
        let item = ClipboardItem::text("hello");
        assert_eq!(decide(&item, &default_rules()), PrivacyDecision::Allow);
    }

    #[test]
    fn labelled_password_is_excluded() {
        let mut item = ClipboardItem::text("secret");
        item.sensitive.push(SensitiveContentType::Password);
        assert_eq!(decide(&item, &default_rules()), PrivacyDecision::Exclude);
    }

    #[test]
    fn text_contains_matcher() {
        let item = ClipboardItem::text("BEGIN OPENSSH PRIVATE KEY");
        let rule = PrivacyRule {
            id: unipick_core::PrivacyRuleId::new(),
            name: "pem".into(),
            enabled: true,
            matchers: vec![PrivacyMatcher::TextContains("PRIVATE KEY".into())],
            action: PrivacyAction::ExcludeFromHistory,
        };
        assert_eq!(decide(&item, &[rule]), PrivacyDecision::Exclude);
    }
}
