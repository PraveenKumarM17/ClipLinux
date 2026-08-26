# KDE Plasma integration

Status: **Planned / Not implemented**

This directory is the architectural slot for Plasma-facing pieces:

- Global shortcut via KGlobalAccel
- Optional widget / runner
- Clipboard integration through the APIs Plasma documents

Nothing here is a working integration. `clipl.desktop` is a stub, not an
installed service. The daemon must not assume KWin window rules or
undocumented shortcuts. Integration is selected only when
`Capability::KdeIntegration` probes as usable.

Do not call KDE “supported”.
