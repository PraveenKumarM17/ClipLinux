//! Conservative content detectors. Reasons never include payload bytes.

use clipl_core::{PrivacyConfig, SensitiveContentType};

/// Which detector produced a hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectorKind {
    /// PEM / OpenSSH private key markers.
    PrivateKey,
    /// JWT-shaped token with a JSON `alg` header prefix.
    Jwt,
    /// Documented high-confidence API key prefix.
    ApiKey,
    /// Luhn-valid payment card candidate.
    CreditCard,
    /// Whole-clipboard OTP-shaped number.
    Otp,
}

/// A detector hit. `reason` is safe to log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detection {
    /// Detector identity.
    pub kind: DetectorKind,
    /// Label to attach.
    pub label: SensitiveContentType,
    /// Explainable reason without secret material.
    pub reason: String,
}

/// Scan `text` with the enabled detectors.
pub fn classify_text(text: &str, privacy: &PrivacyConfig) -> Vec<Detection> {
    let mut out = Vec::new();
    if privacy.block_private_keys {
        if let Some(hit) = detect_private_key(text) {
            out.push(hit);
        }
    }
    if privacy.block_high_confidence_tokens {
        if let Some(hit) = detect_jwt(text) {
            out.push(hit);
        }
        out.extend(detect_api_keys(text));
    }
    if privacy.block_credit_cards {
        if let Some(hit) = detect_credit_card(text) {
            out.push(hit);
        }
    }
    if privacy.block_otp {
        if let Some(hit) = detect_otp(text) {
            out.push(hit);
        }
    }
    out
}

fn detect_private_key(text: &str) -> Option<Detection> {
    const MARKERS: &[&str] = &[
        "BEGIN PRIVATE KEY",
        "BEGIN RSA PRIVATE KEY",
        "BEGIN DSA PRIVATE KEY",
        "BEGIN EC PRIVATE KEY",
        "BEGIN OPENSSH PRIVATE KEY",
        "BEGIN ENCRYPTED PRIVATE KEY",
        "BEGIN SSH2 ENCRYPTED PRIVATE KEY",
        "BEGIN PGP PRIVATE KEY BLOCK",
    ];
    let upper = text.to_ascii_uppercase();
    for marker in MARKERS {
        if upper.contains(marker) {
            return Some(Detection {
                kind: DetectorKind::PrivateKey,
                label: SensitiveContentType::PrivateKey,
                reason: "PEM/OpenSSH private key header".into(),
            });
        }
    }
    None
}

/// JWT: exactly three base64url segments, header starts with `eyJ` (`{"`).
fn detect_jwt(text: &str) -> Option<Detection> {
    let trimmed = text.trim();
    if trimmed.len() < 40 || trimmed.contains(char::is_whitespace) {
        return None;
    }
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    if !parts.iter().all(|p| is_base64url_token(p) && p.len() >= 8) {
        return None;
    }
    // `eyJ` is base64url for `{"` — required so `a.b.c` and `file.tar.gz` do not match.
    if !parts[0].starts_with("eyJ") {
        return None;
    }
    Some(Detection {
        kind: DetectorKind::Jwt,
        label: SensitiveContentType::Token,
        reason: "JWT-shaped token (three base64url segments, JSON header)".into(),
    })
}

fn is_base64url_token(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// High-confidence vendor prefixes only. Documented in PRIVACY_MODEL.md.
fn detect_api_keys(text: &str) -> Vec<Detection> {
    let mut hits = Vec::new();
    if contains_prefixed_token(text, "ghp_", 36)
        || contains_prefixed_token(text, "gho_", 36)
        || contains_prefixed_token(text, "github_pat_", 40)
        || contains_prefixed_token(text, "glpat-", 20)
        || contains_prefixed_token(text, "sk_live_", 24)
        || contains_prefixed_token(text, "sk_test_", 24)
        || contains_prefixed_token(text, "rk_live_", 24)
        || contains_prefixed_token(text, "rk_test_", 24)
        || contains_prefixed_token(text, "xoxb-", 24)
        || contains_prefixed_token(text, "xoxp-", 24)
        || contains_google_api_key(text)
    {
        hits.push(Detection {
            kind: DetectorKind::ApiKey,
            label: SensitiveContentType::Token,
            reason: "high-confidence API token prefix".into(),
        });
    }
    hits
}

fn contains_prefixed_token(text: &str, prefix: &str, min_len: usize) -> bool {
    for (idx, _) in text.match_indices(prefix) {
        let token_len = text[idx..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .count();
        if token_len >= min_len {
            return true;
        }
    }
    false
}

/// Google API keys: `AIza` + 35 ASCII alphanumerics / `_` / `-` (39 chars total).
fn contains_google_api_key(text: &str) -> bool {
    for (idx, _) in text.match_indices("AIza") {
        let body: String = text[idx..].chars().take(39).collect();
        if body.len() == 39
            && body
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return true;
        }
    }
    false
}

/// Card candidate: Luhn-valid 13–19 digits AND (whole clipboard is the number
/// with optional spaces/dashes, or the number is grouped with separators).
fn detect_credit_card(text: &str) -> Option<Detection> {
    let trimmed = text.trim();
    if let Some(grouped) = grouped_card(trimmed) {
        if luhn_valid(&grouped) {
            return Some(card_hit());
        }
    }
    if is_whole_clipboard_digits(trimmed) {
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if (13..=19).contains(&digits.len()) && luhn_valid(&digits) {
            return Some(card_hit());
        }
    }
    None
}

fn card_hit() -> Detection {
    Detection {
        kind: DetectorKind::CreditCard,
        label: SensitiveContentType::CreditCard,
        reason: "Luhn-valid payment card candidate".into(),
    }
}

fn grouped_card(text: &str) -> Option<String> {
    // Require *internal* spaces or dashes (4111-1111-…) so a 16-digit run
    // sitting in a sentence is not treated as grouped.
    for candidate in card_runs(text) {
        let trimmed = candidate.trim();
        if !trimmed.contains(' ') && !trimmed.contains('-') {
            continue;
        }
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if (13..=19).contains(&digits.len()) {
            return Some(digits);
        }
    }
    None
}

fn card_runs(text: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        if c.is_ascii_digit() || c == ' ' || c == '-' {
            current.push(c);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

fn is_whole_clipboard_digits(text: &str) -> bool {
    let mut digits = 0usize;
    for c in text.chars() {
        if c.is_ascii_digit() {
            digits += 1;
        } else if c != ' ' && c != '-' {
            return false;
        }
    }
    (13..=19).contains(&digits)
}

fn luhn_valid(digits: &str) -> bool {
    if !(13..=19).contains(&digits.len()) || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let mut sum = 0u32;
    let mut alt = false;
    for c in digits.chars().rev() {
        let mut n = c.to_digit(10).unwrap_or(0);
        if alt {
            n *= 2;
            if n > 9 {
                n -= 9;
            }
        }
        sum += n;
        alt = !alt;
    }
    sum % 10 == 0
}

/// Whole clipboard is 6 or 8 digits, optional single spaces. Nothing else.
fn detect_otp(text: &str) -> Option<Detection> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains('\n') {
        return None;
    }
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 6 && digits.len() != 8 {
        return None;
    }
    if trimmed.chars().any(|c| !c.is_ascii_digit() && c != ' ') {
        return None;
    }
    // Reject strings that are mostly spaces around a number with other words already filtered.
    if trimmed.chars().filter(|c| *c == ' ').count() > 2 {
        return None;
    }
    Some(Detection {
        kind: DetectorKind::Otp,
        label: SensitiveContentType::OneTimeCode,
        reason: "whole clipboard is a 6- or 8-digit OTP candidate".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> PrivacyConfig {
        PrivacyConfig::default()
    }

    #[test]
    fn pem_rsa() {
        let text = "-----BEGIN RSA PRIVATE KEY-----\nMIIFake\n-----END RSA PRIVATE KEY-----";
        let hits = classify_text(text, &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::PrivateKey));
    }

    #[test]
    fn pem_openssh() {
        let hits = classify_text("-----BEGIN OPENSSH PRIVATE KEY-----", &cfg());
        assert_eq!(hits[0].kind, DetectorKind::PrivateKey);
    }

    #[test]
    fn public_key_is_not_private() {
        let hits = classify_text(
            "-----BEGIN PUBLIC KEY-----\nMIIB\n-----END PUBLIC KEY-----",
            &cfg(),
        );
        assert!(hits.is_empty());
    }

    #[test]
    fn jwt_high_confidence() {
        // {"alg":"HS256"} base64url-prefixed eyJ. Payload and sig are dummy but long enough.
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4ifQ.dummySignatureValueHere";
        let hits = classify_text(jwt, &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::Jwt));
    }

    #[test]
    fn dotted_version_is_not_jwt() {
        assert!(classify_text("file.tar.gz", &cfg()).is_empty());
        assert!(classify_text("v1.2.3", &cfg()).is_empty());
        assert!(classify_text("a.b.c", &cfg()).is_empty());
    }

    /// Prefix and body stay in separate literals so secret scanners do not
    /// treat test fixtures as live credentials.
    fn fake_token(prefix: &str, body: &str) -> String {
        let mut token = String::from(prefix);
        token.push_str(body);
        token
    }

    #[test]
    fn github_pat() {
        let token = fake_token("ghp_", "1234567890abcdefghijklmnopqrstuvwx");
        assert!(token.len() >= 36);
        let hits = classify_text(&token, &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::ApiKey));
    }

    #[test]
    fn stripe_live_key() {
        let token = fake_token("sk_live_", "abcdefghijklmnopqrstuvwx");
        let hits = classify_text(&token, &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::ApiKey));
    }

    #[test]
    fn google_api_key() {
        let token = format!("AIza{}", "A".repeat(35));
        assert_eq!(token.len(), 39);
        let hits = classify_text(&token, &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::ApiKey));
    }

    #[test]
    fn random_sk_prefix_is_not_enough() {
        assert!(classify_text("sk-short", &cfg()).is_empty());
    }

    #[test]
    fn visa_test_number_whole_clipboard() {
        let hits = classify_text("4111111111111111", &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::CreditCard));
    }

    #[test]
    fn grouped_visa() {
        let hits = classify_text("4111-1111-1111-1111", &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::CreditCard));
    }

    #[test]
    fn luhn_fail_is_not_a_card() {
        assert!(classify_text("4111111111111112", &cfg())
            .iter()
            .all(|h| h.kind != DetectorKind::CreditCard));
    }

    #[test]
    fn digits_in_a_sentence_without_grouping_are_ignored() {
        let text = "invoice 4111111111111111 shipped";
        assert!(classify_text(text, &cfg())
            .iter()
            .all(|h| h.kind != DetectorKind::CreditCard));
    }

    #[test]
    fn otp_six_digits() {
        let hits = classify_text("847291", &cfg());
        assert!(hits.iter().any(|h| h.kind == DetectorKind::Otp));
    }

    #[test]
    fn otp_not_in_sentence() {
        assert!(classify_text("your code is 847291", &cfg())
            .iter()
            .all(|h| h.kind != DetectorKind::Otp));
    }

    #[test]
    fn four_digits_are_not_otp() {
        assert!(classify_text("2024", &cfg()).is_empty());
    }

    #[test]
    fn toggling_off_skips_cards() {
        let privacy = PrivacyConfig {
            block_credit_cards: false,
            ..cfg()
        };
        assert!(classify_text("4111111111111111", &privacy)
            .iter()
            .all(|h| h.kind != DetectorKind::CreditCard));
    }
}
