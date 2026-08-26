# Privacy model

Clipboard history is **private user data**. UniPick stores it on the local
machine, filters it before persistence, and does not send it to UniPick
maintainers or to media vendors as a side effect of copying.

This document is the product contract. Types live in `unipick-core`; the
engine lives in `unipick-privacy`.

## Goals

1. Secrets must be easy to keep **out of history**.
2. Offline features must not require an account or a network.
3. Remote GIF search, if enabled, must not upload clipboard contents.
4. Defaults should be safe for a shared Linux desktop.

## What may be stored

| Data | Where (planned) | Default |
| --- | --- | --- |
| Clipboard text/images that pass policy | SQLite + blob store | On, with exclusion rules |
| Snippets the user created | SQLite | On |
| Emoji / symbol catalogs | Packaged files | On (not personal data) |
| Media cache (GIFs, stickers) | Disk cache | On when the user searches |
| Capability matrix | Memory / optional diagnostics | Not sensitive |
| Crash logs | Local | Must not include clipboard payloads |

## What must not be stored (defaults)

`SensitiveContentType` labels drive `PrivacyRule` actions:

| Type | Default action |
| --- | --- |
| Password | Exclude from history |
| PrivateKey | Exclude from history |
| Token | Exclude from history |
| CreditCard | Exclude from history |
| PersonalIdentifier | Exclude from history (when detected) |
| OneTimeCode | Exclude from history (when detected) |

In the foundation, the classifier **does not guess**. It only honors labels
already attached to a `ClipboardItem` and explicit `PrivacyMatcher`s (MIME
prefix, literal text). Heuristic detectors (Luhn, PEM, password-manager MIME)
need their own review before they ship, to avoid hiding legitimate text.

## Decision pipeline

```
ClipboardContent
    → optional classify()
    → PrivacyRule list (first match wins)
    → PrivacyDecision { Allow, Exclude, Redact, Expire, Confirm }
    → StorageBackend  (only if allowed)
```

`unipick-clipboard` calls this pipeline in `record()`. An `Exclude` result is
success from the caller’s point of view: the secret was handled, not stored.

## Network and providers

- Emoji, symbols, snippets, and history work **offline**.
- `MediaProvider::is_available` is false when a remote vendor cannot be
  reached; the registry still includes the offline provider.
- Search queries for GIFs are user-initiated. Clipboard text is **not** used
  as a search query unless the user explicitly searches using that text.
- Provider API keys, if any, are user-supplied configuration, not phoned-home
  telemetry.

## Access control

UniPick does not implement multi-user sync. History files must be created with
user-only permissions (`0600` / directory `0700`) when SQLite lands. Wayland
and X11 clipboard access is whatever the session already grants; UniPick must
not weaken that.

## Diagnostics

`unipick doctor` prints session identity and support levels. It must not print
clipboard payloads. Bug reports should attach doctor output, not history
dumps.

## Future work (not in foundation)

- Password-manager offered MIME (`x-kde-passwordManagerHint`, etc.)
- Optional “never persist images”
- Per-application exclusion (`PrivacyMatcher::ApplicationId`) when the
  platform provides an origin
- Encrypted-at-rest database (only if a real threat model demands it; file
  permissions come first)
