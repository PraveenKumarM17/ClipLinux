# Contributing to ClipLinux

Thank you for helping build a Linux paste ecosystem. This project is designed
so a first-time contributor can own one crate or one adapter without learning
every compositor.

## Before you write code

1. Read [ARCHITECTURE.md](ARCHITECTURE.md) — especially the forbidden
   dependency table.
2. Read [PLATFORM_CAPABILITIES.md](PLATFORM_CAPABILITIES.md) if you touch
   sessions, clipboards, or hotkeys.
3. Read [PRIVACY_MODEL.md](PRIVACY_MODEL.md) if you touch history or media.
4. Check [ROADMAP.md](ROADMAP.md). Clipboard monitoring is **not** open for
   drive-by implementation until the foundation milestone is accepted.

## Setup

See [DEVELOPMENT.md](DEVELOPMENT.md). In short:

```bash
cargo test --workspace
cargo fmt --all
```

## Patch rules

- **One concern per PR.** A GNOME adapter and a GIF provider do not share a
  pull request.
- **No Electron. No new GUI toolkit** in core. Desktop UI is Svelte; shell is
  Tauri.
- **Do not depend on Tauri or Svelte from `crates/`.**
- **Do not copy X11 logic into Wayland backends.** Share types and tests, not
  protocol assumptions.
- **Unknown is better than a fake Native.** If you did not probe it, leave
  `SupportLevel::Unknown`.
- **No undocumented polling** of the clipboard unless a design review accepts
  a named `Fallback` and this is written into `PLATFORM_CAPABILITIES.md`.
- **Do not log clipboard contents.** Doctor output is identity + capabilities.
- **Placeholders stay placeholders** until their milestone. Do not “just add
  xclip” in the daemon stub.

## Crate ownership cheat sheet

| You want to… | Work in |
| --- | --- |
| Change domain types | `crates/clipl-core` (discuss first; everything serializes this) |
| Add IPC messages | `crates/clipl-protocol` |
| Filter secrets | `crates/clipl-privacy` |
| History record/query | `crates/clipl-clipboard` |
| Emoji search | `crates/clipl-emoji` + `packages/emoji-data` |
| GIF vendor | `crates/clipl-media` implementing `MediaProvider` |
| Session probe / adapter | `crates/clipl-platform` |
| GNOME/KDE UI glue | `extensions/` |
| Palette UI | `apps/desktop` |
| CLI diagnostics | `apps/cli` |

## Tests

Every new matcher, adapter, or provider needs tests that run **without** a
graphical session. Display-server tests must be ignored by default
(`#[ignore]` or a feature flag).

## License

Contributions are dual-licensed MIT OR Apache-2.0, matching the repository.
