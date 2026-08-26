This directory holds the ClipLinux GNOME Shell extension **boundary**.

GNOME on Wayland does not expose a generic clipboard-watch protocol to the
daemon. The Rust backend `gnome` reports `ClipboardWatch = Unsupported` and
does not inject keys or scrape the Shell.

**PLANNED:** a Shell extension that:

- observes clipboard changes using Shell-owned APIs
- sends **text** to `clipl-daemon` over the local Unix socket
- never receives overlay/hotkey hacks from the daemon

Until that extension exists, history still works if another backend (for
example X11) is in use; on GNOME Wayland the daemon serves IPC with an empty
watch.
