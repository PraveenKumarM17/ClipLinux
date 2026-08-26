# GNOME Shell extension (placeholder)

This directory will hold the UniPick GNOME Shell extension:

- palette toggle via a Shell-owned shortcut
- clipboard / paste integration that GNOME actually supports
- D-Bus or Unix-socket talk to `unipick-daemon`

Nothing is implemented in the foundation. Do not ship a hidden overlay hack
from the Rust daemon; GNOME integration belongs here and is selected only when
`Capability::GnomeExtension` probes as usable.
