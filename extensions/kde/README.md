# KDE Plasma integration (placeholder)

This directory will hold Plasma-facing pieces:

- Global shortcut via KGlobalAccel
- Optional widget / runner
- Clipboard integration through the APIs Plasma documents

Nothing is implemented in the foundation. The daemon must not assume KWin
window rules or undocumented shortcuts. Integration is selected only when
`Capability::KdeIntegration` probes as usable.
