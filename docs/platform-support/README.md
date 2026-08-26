# Platform support notes

Canonical document: [/PLATFORM_CAPABILITIES.md](../../PLATFORM_CAPABILITIES.md)

- [x11.md](x11.md) — clipboard watch + native shortcut
- [gnome.md](gnome.md) — Wayland activation via Shell extension

Adapters are selected by identity, then must **probe**. Generic Wayland does
not gain a global hotkey because X11 APIs exist on the same machine.

