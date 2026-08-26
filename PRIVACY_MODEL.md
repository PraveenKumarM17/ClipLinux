# Privacy model

Clipboard history is **private user data**. ClipLinux stores it on the local
machine, filters it **before** persistence, and does not send it to ClipLinux
maintainers or to media vendors as a side effect of copying.

Types live in `clipl-core`; the engine lives in `clipl-privacy`.

## Goals

1. Secrets must be easy to keep **out of history**.
2. Offline features must not require an account or a network.
3. Remote GIF search, if enabled, must not upload clipboard contents.
4. Defaults should be safe for a shared Linux desktop.
5. Decisions must be **explainable** (reasons never include payload bytes).

## What may be stored

| Data | Where | Default |
| --- | --- | --- |
| Text/HTML/URI that pass policy | SQLite `clipboard_items` | On, with exclusion rules |
| Snippets the user created | SQLite `kv` (later) | On |
| Emoji / symbol catalogs | Packaged files | On (not personal data) |
| Capability matrix | Memory / diagnostics | Not sensitive |
| Crash logs | Local | Must not include clipboard payloads |

Images and files are **not** persisted yet. Only text/HTML/URI that pass
policy are stored.

## Detectors (IMPLEMENTED, conservative)

All detectors can be toggled in `config.toml` under `[privacy]`. Reasons logged
to tracing are labels such as `PEM/OpenSSH private key header`, never the
secret itself.

### A. Private keys — `block_private_keys`

Markers (case-insensitive):

- `BEGIN PRIVATE KEY`
- `BEGIN RSA PRIVATE KEY` / `DSA` / `EC`
- `BEGIN OPENSSH PRIVATE KEY`
- `BEGIN ENCRYPTED PRIVATE KEY`
- `BEGIN SSH2 ENCRYPTED PRIVATE KEY`
- `BEGIN PGP PRIVATE KEY BLOCK`

Public keys (`BEGIN PUBLIC KEY`) are **not** flagged.

### B. JWT-like tokens — `block_high_confidence_tokens`

Requires **all** of:

- no whitespace
- exactly three `.`-separated segments
- each segment base64url (`A–Z a–z 0–9 - _`) length ≥ 8
- total length ≥ 40
- header starts with `eyJ` (base64url of `{"`)

`file.tar.gz`, `v1.2.3`, and `a.b.c` do not match.

### C. API tokens — `block_high_confidence_tokens`

High-confidence prefixes only:

| Prefix | Notes |
| --- | --- |
| `ghp_` / `gho_` | GitHub PATs (min length enforced) |
| `github_pat_` | Fine-grained GitHub PAT |
| `glpat-` | GitLab |
| `sk_live_` / `sk_test_` / `rk_live_` / `rk_test_` | Stripe |
| `xoxb-` / `xoxp-` | Slack |
| `AIza` + 35 charset chars | Google API key (39 total) |

Short `sk-` strings are **not** flagged.

### D. Credit cards — `block_credit_cards`

- 13–19 digits, Luhn-valid
- AND either the whole clipboard is only digits/spaces/dashes, **or** the
  number is grouped with spaces/dashes (`4111-1111-1111-1111`)
- A 16-digit run inside a sentence **without** grouping is ignored

### E. OTP — `block_otp`

Whole clipboard is 6 or 8 digits (optional spaces). `your code is 847291` is
**not** flagged. 4-digit years are not flagged.

### F. Password-manager sources

- MIME containing `password`, `secret`, `passwordmanager`, or
  `x-kde-passwordmanagerhint`
- `PrivacyMatcher::ApplicationId` when `ClipboardItem.source_app` is set
  (origin detection itself is **PLANNED**)

## Decision pipeline

```
ClipboardContent
    → classify_text / MIME  → labels + reasons
    → PrivacyRule list (first match wins)
    → PrivacyVerdict { Allow, Exclude, Redact, Expire, Confirm }
    → SQLite  (only if allowed)
```

`Exclude` is success: the secret was handled, not stored.

## Defaults

| Type | Default action |
| --- | --- |
| Password | Exclude |
| PrivateKey | Exclude |
| Token | Exclude |
| CreditCard | Exclude |
| OneTimeCode | Exclude |

`privacy.enabled = false` skips detectors and rules (user choice).

## Access control

History files: directory `0700`, database `0600`. No multi-user sync.

## Diagnostics

`clipl doctor` / `clipl-daemon --diagnose` print identity and support
levels. They must not print clipboard payloads.
