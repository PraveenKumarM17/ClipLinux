# Privacy notes

Canonical document: [/PRIVACY_MODEL.md](../../PRIVACY_MODEL.md)

Implementation: `crates/clipl-privacy`. History integration:
`crates/clipl-clipboard` (`record()` applies `evaluate()` before storage).
Reasons are labels such as `PEM/OpenSSH private key header`; payload bytes
are never logged.
