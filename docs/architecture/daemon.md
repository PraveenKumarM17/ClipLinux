# Daemon

Status: **IMPLEMENTED** as a resident process with IPC, SQLite, and optional watch.

`clipl-daemon` does not contain UI logic.

## Startup

1. Load `$XDG_CONFIG_HOME/clipl/config.toml` or defaults
2. Probe `XDG_SESSION_TYPE` / `XDG_CURRENT_DESKTOP`
3. Select a clipboard backend (never X11-on-Wayland)
4. Open `$XDG_DATA_HOME/clipl/history.sqlite3` (mode `0600`, dir `0700`)
5. If `supports_watch()`, start a watch thread
6. Listen on `$XDG_RUNTIME_DIR/clipl/daemon.sock` (mode `0600`)

If watch is unsupported, the daemon **still serves history** over IPC. That is a graceful degradation, not a crash.

## Diagnose

```bash
cargo run -p clipl-daemon -- --diagnose
```

Prints session, backend, monitoring level, database path, socket, privacy, and history limit. Never prints clipboard payloads.

## Shutdown

SIGINT/SIGTERM set an atomic flag. The watch loop wakes on timeout; IPC accept is non-blocking. The socket file is removed.

## Logging

`tracing` at `info` by default (`RUST_LOG` overrides). Events log item **ids** and privacy **reasons**, never payload text.
