//! Privacy filtering for clipboard history.
//!
//! Detectors are conservative and explainable. They never log payload bytes.

#![forbid(unsafe_code)]

mod detect;

use clipl_core::{
    ClipboardContent, ClipboardItem, PrivacyAction, PrivacyConfig, PrivacyMatcher, PrivacyRule,
    SensitiveContentType,
};

pub use clipl_core::{PrivacyMatcher as Matcher, PrivacyRule as Rule};
pub use detect::{classify_text, Detection, DetectorKind};

/// Decision produced by the privacy engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// Explainable verdict. Reasons must never include secret values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrivacyVerdict {
    /// Store / exclude / redact / expire / confirm.
    pub decision: PrivacyDecision,
    /// Labels attached to the item.
    pub labels: Vec<SensitiveContentType>,
    /// Human-readable reasons (detector names, rule names).
    pub reasons: Vec<String>,
}

impl PrivacyVerdict {
    /// Persist?
    pub fn allows_store(&self) -> bool {
        matches!(
            self.decision,
            PrivacyDecision::Allow | PrivacyDecision::Redact | PrivacyDecision::Expire { .. }
        )
    }
}

/// Evaluate detectors then rules. First matching rule wins.
pub fn evaluate(
    item: &ClipboardItem,
    rules: &[PrivacyRule],
    privacy: &PrivacyConfig,
) -> PrivacyVerdict {
    if !privacy.enabled {
        return PrivacyVerdict {
            decision: PrivacyDecision::Allow,
            labels: item.sensitive.clone(),
            reasons: vec!["privacy engine disabled".into()],
        };
    }

    let mut labels = item.sensitive.clone();
    let mut reasons = Vec::new();

    if let Some(text) = item.content.text_for_scan() {
        for hit in classify_text(text, privacy) {
            if !labels.contains(&hit.label) {
                labels.push(hit.label.clone());
            }
            reasons.push(hit.reason);
        }
    }
    if let Some(mime) = item.content.mime() {
        if mime_looks_like_secret(mime) {
            if !labels.contains(&SensitiveContentType::Password) {
                labels.push(SensitiveContentType::Password);
            }
            reasons.push("clipboard MIME indicates a password-manager secret".into());
        }
    }

    let mut tagged = item.clone();
    tagged.sensitive = labels.clone();

    for rule in rules.iter().filter(|rule| rule.enabled) {
        if rule_matches(&tagged, rule) {
            let decision = match &rule.action {
                PrivacyAction::ExcludeFromHistory => PrivacyDecision::Exclude,
                PrivacyAction::RedactPayload => PrivacyDecision::Redact,
                PrivacyAction::ExpireAfter { ttl_ms } => {
                    PrivacyDecision::Expire { ttl_ms: *ttl_ms }
                }
                PrivacyAction::RequireConfirmation => PrivacyDecision::Confirm,
                _ => PrivacyDecision::Exclude,
            };
            reasons.push(format!("rule `{}` matched", rule.name));
            return PrivacyVerdict {
                decision,
                labels,
                reasons,
            };
        }
    }

    PrivacyVerdict {
        decision: PrivacyDecision::Allow,
        labels,
        reasons,
    }
}

/// Evaluate using default privacy toggles.
pub fn decide(item: &ClipboardItem, rules: &[PrivacyRule]) -> PrivacyDecision {
    evaluate(item, rules, &PrivacyConfig::default()).decision
}

/// Default rules shipped with ClipLinux.
pub fn default_rules() -> Vec<PrivacyRule> {
    vec![
        PrivacyRule::exclude("Exclude passwords", SensitiveContentType::Password),
        PrivacyRule::exclude("Exclude private keys", SensitiveContentType::PrivateKey),
        PrivacyRule::exclude("Exclude tokens", SensitiveContentType::Token),
        PrivacyRule::exclude("Exclude payment cards", SensitiveContentType::CreditCard),
        PrivacyRule::exclude("Exclude one-time codes", SensitiveContentType::OneTimeCode),
    ]
}

/// Classify using default privacy toggles and merge existing labels.
pub fn classify(item: &ClipboardItem) -> Vec<SensitiveContentType> {
    evaluate(item, &[], &PrivacyConfig::default()).labels
}

fn mime_looks_like_secret(mime: &str) -> bool {
    let lower = mime.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("secret")
        || lower.contains("passwordmanager")
        || lower.contains("x-kde-passwordmanagerhint")
}

fn rule_matches(item: &ClipboardItem, rule: &PrivacyRule) -> bool {
    rule.matchers.iter().any(|matcher| match matcher {
        PrivacyMatcher::Sensitive(kind) => item.sensitive.contains(kind),
        PrivacyMatcher::MimePrefix(prefix) => mime_matches(&item.content, prefix),
        PrivacyMatcher::ApplicationId(app) => item.source_app.as_deref() == Some(app.as_str()),
        PrivacyMatcher::TextContains(needle) => item
            .content
            .text_for_scan()
            .is_some_and(|text| text.contains(needle)),
        _ => false,
    })
}

fn mime_matches(content: &ClipboardContent, prefix: &str) -> bool {
    content.mime().is_some_and(|mime| mime.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clipl_core::ClipboardItem;

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
    fn pem_is_excluded_with_reason() {
        let item = ClipboardItem::text(
            "-----BEGIN OPENSSH PRIVATE KEY-----\nAAAA\n-----END OPENSSH PRIVATE KEY-----",
        );
        let verdict = evaluate(&item, &default_rules(), &PrivacyConfig::default());
        assert_eq!(verdict.decision, PrivacyDecision::Exclude);
        assert!(verdict.labels.contains(&SensitiveContentType::PrivateKey));
        assert!(verdict.reasons.iter().any(|r| r.contains("private key")));
        assert!(!verdict.reasons.iter().any(|r| r.contains("BEGIN OPENSSH")));
    }

    #[test]
    fn disabled_engine_allows_pem() {
        let item = ClipboardItem::text(
            "-----BEGIN RSA PRIVATE KEY-----\nMII\n-----END RSA PRIVATE KEY-----",
        );
        let cfg = PrivacyConfig {
            enabled: false,
            ..PrivacyConfig::default()
        };
        let verdict = evaluate(&item, &default_rules(), &cfg);
        assert_eq!(verdict.decision, PrivacyDecision::Allow);
    }

    #[test]
    fn application_id_matcher() {
        let mut item = ClipboardItem::text("whatever");
        item.source_app = Some("org.keepassxc.KeePassXC".into());
        let rule = PrivacyRule {
            id: clipl_core::PrivacyRuleId::new(),
            name: "keepass".into(),
            enabled: true,
            matchers: vec![PrivacyMatcher::ApplicationId(
                "org.keepassxc.KeePassXC".into(),
            )],
            action: PrivacyAction::ExcludeFromHistory,
        };
        assert_eq!(decide(&item, &[rule]), PrivacyDecision::Exclude);
    }
}
